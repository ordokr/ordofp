//! IO Effect - Input/Output Operations
//!
//! The IO effect represents computations that perform input/output
//! operations. Unlike pure effects like State and Reader, IO effects
//! cannot be eliminated without actually performing the I/O.
//!
//! # Design Philosophy
//!
//! IO effects are "terminal" - they can only be handled at the program
//! boundary by actually performing the I/O operations.

#[cfg(feature = "std")]
use alloc::borrow::ToOwned;
use alloc::boxed::Box;

use crate::nexus::effect::{Eff, EffectMarker};
use crate::nexus::row::{IO_BIT, Row};

// =============================================================================
// IO Effect Type
// =============================================================================

/// The IO effect marker type.
///
/// `IoEffect` represents computations that perform input/output.
#[derive(Copy, Clone, Debug)]
pub struct IoEffect;

impl EffectMarker for IoEffect {
    const BIT: u128 = IO_BIT;
    const NAME: &'static str = "IO";
}

/// Type alias for a row containing only IO.
pub type IoRow = Row<IO_BIT>;

// =============================================================================
// IO Operations
// =============================================================================

/// Operations that can be performed with the IO effect.
pub enum IoOp<A> {
    /// Read from input.
    Read,
    /// Write to output.
    Write(A),
    /// Perform a side effect.
    Perform(Box<dyn FnOnce() -> A>),
}

// =============================================================================
// IO Effect Functions
// =============================================================================

/// Perform an IO action.
///
/// # Example
///
/// ```rust
/// use ordofp_core::nexus::effects::io::IoComputation;
///
/// let comp = IoComputation::new(|| {
///     println!("Hello, World!");
///     42
/// });
/// let result = comp.run();
/// assert_eq!(result, 42);
/// ```
pub fn io_perform<A: 'static, F: FnOnce() -> A + 'static>(_f: F) -> Eff<IoRow, A> {
    Eff::lazy(|| crate::cold_panic!("io_perform requires IO handler"))
}

// =============================================================================
// Concrete IO Implementation
// =============================================================================

/// A concrete IO computation that can be run.
///
/// Uses an enum representation to avoid heap allocation for pure values.
#[must_use = "IO computations do nothing unless run"]
#[repr(u8)]
pub enum IoComputation<A> {
    /// Pure value - no I/O, no allocation.
    Pure(A),
    /// Boxed computation (for actual I/O).
    Perform(Box<dyn FnOnce() -> A>),
}

impl<A: 'static> IoComputation<A> {
    /// Create a new IO computation from a function.
    #[inline(always)]
    pub fn new<F: FnOnce() -> A + 'static>(f: F) -> Self {
        IoComputation::Perform(Box::new(f))
    }

    /// Run the IO computation, performing side effects.
    ///
    /// # Safety
    ///
    /// This actually performs I/O operations. Only call at program boundaries.
    #[inline(always)]
    pub fn run(self) -> A {
        match self {
            IoComputation::Pure(a) => a,
            IoComputation::Perform(f) => f(),
        }
    }

    /// Pure value in IO context - NO HEAP ALLOCATION.
    #[inline(always)]
    pub fn pure(value: A) -> Self {
        IoComputation::Pure(value)
    }

    /// Map over the result.
    #[inline(always)]
    pub fn map<B: 'static, F: FnOnce(A) -> B + 'static>(self, f: F) -> IoComputation<B> {
        match self {
            IoComputation::Pure(a) => IoComputation::Pure(f(a)),
            IoComputation::Perform(run_fn) => IoComputation::Perform(Box::new(move || f(run_fn()))),
        }
    }

    /// Chain two IO computations.
    #[inline(always)]
    pub fn and_then<B: 'static, F: FnOnce(A) -> IoComputation<B> + 'static>(
        self,
        f: F,
    ) -> IoComputation<B> {
        IoComputation::Perform(Box::new(move || {
            let a = self.run();
            f(a).run()
        }))
    }

    /// Sequence, discarding the first result.
    #[inline(always)]
    pub fn then<B: 'static>(self, next: IoComputation<B>) -> IoComputation<B> {
        self.and_then(move |_| next)
    }

    /// Sequence, discarding the second result.
    #[inline(always)]
    pub fn before<B: 'static>(self, next: IoComputation<B>) -> IoComputation<A> {
        self.and_then(move |a| next.map(move |_| a))
    }
}

// =============================================================================
// IO Utilities
// =============================================================================

/// Print to standard output (when std feature is available).
#[cfg(feature = "std")]
pub fn print_line(msg: alloc::string::String) -> IoComputation<()> {
    IoComputation::new(move || {
        std::println!("{msg}");
    })
}

/// Read a line from standard input.
///
/// # Error handling
///
/// `IoComputation<String>` has no error channel — this effect is infallible
/// by design. I/O errors read as `""` by contract of this infallible
/// effect: a failed read is not distinguishable from an empty line, so
/// callers that need to detect I/O failures must not use this function.
#[cfg(feature = "std")]
pub fn read_line() -> IoComputation<alloc::string::String> {
    IoComputation::new(|| {
        let mut input = alloc::string::String::new();
        match std::io::stdin().read_line(&mut input) {
            Ok(_) => input.trim().to_owned(),
            // I/O errors read as "" by contract of this infallible effect — documented above.
            Err(_) => alloc::string::String::new(),
        }
    })
}

/// Delay execution.
#[cfg(feature = "std")]
pub fn delay(millis: u64) -> IoComputation<()> {
    IoComputation::new(move || {
        std::thread::sleep(std::time::Duration::from_millis(millis));
    })
}

/// Get current time as milliseconds since epoch.
#[cfg(feature = "std")]
pub fn current_time_millis() -> IoComputation<u128> {
    IoComputation::new(|| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_millis())
    })
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_io_pure() {
        let comp = IoComputation::pure(42);
        assert_eq!(comp.run(), 42);
    }

    #[test]
    fn test_io_map() {
        let comp = IoComputation::pure(21).map(|x| x * 2);
        assert_eq!(comp.run(), 42);
    }

    #[test]
    fn test_io_and_then() {
        let comp = IoComputation::pure(20).and_then(|x| IoComputation::pure(x + 22));
        assert_eq!(comp.run(), 42);
    }

    #[test]
    fn test_io_then() {
        let comp = IoComputation::pure(1).then(IoComputation::pure(42));
        assert_eq!(comp.run(), 42);
    }

    #[test]
    fn test_io_before() {
        let comp = IoComputation::pure(42).before(IoComputation::pure(0));
        assert_eq!(comp.run(), 42);
    }

    #[test]
    fn test_io_side_effect() {
        use alloc::rc::Rc;
        use core::cell::Cell;

        let counter = Rc::new(Cell::new(0));
        let counter_clone = counter.clone();
        let comp = IoComputation::new(move || {
            counter_clone.set(counter_clone.get() + 1);
            42
        });

        assert_eq!(counter.get(), 0);
        let result = comp.run();
        assert_eq!(result, 42);
        assert_eq!(counter.get(), 1);
    }

    #[test]
    fn test_io_chain_side_effects() {
        use alloc::rc::Rc;
        use core::cell::Cell;

        let counter = Rc::new(Cell::new(0));
        let counter1 = counter.clone();
        let counter2 = counter.clone();

        let comp = IoComputation::new(move || {
            counter1.set(counter1.get() + 1);
            10
        })
        .and_then(move |x| {
            IoComputation::new(move || {
                counter2.set(counter2.get() + x);
                counter2.get()
            })
        });

        let result = comp.run();
        assert_eq!(result, 11); // 0 + 1 + 10
    }
}
