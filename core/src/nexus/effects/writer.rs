//! Writer Effect - Append-Only Logging
//!
//! The Writer effect provides append-only output that accumulates
//! through computations.
//!
//! # Handler Laws
//!
//! The Writer handler must satisfy these algebraic laws:
//!
//! ## Tell-Tell Law (Monoid Homomorphism)
//! ```text
//! tell(w1).and_then(|_| tell(w2)) = tell(w1.append(w2))
//! ```
//! Sequential tells combine via the monoid operation.
//!
//! ## Tell-Empty Law
//! ```text
//! tell(empty) = pure(())
//! ```
//! Telling the identity element is a no-op.
//!
//! ## Listen-Pure Law
//! ```text
//! listen(pure(x)) = pure((x, empty))
//! ```
//! Listening to a pure value produces an empty log.
//!
//! ## Censor-Tell Law
//! ```text
//! censor(f, tell(w)) = tell(f(w))
//! ```
//! Censoring a tell applies the transformation to the written value.
//!
//! # Verification Tier
//!
//! **Tier 1**: Laws are documented; tests in `nexus::laws`.
//!
//! # Monoid Requirement
//!
//! The log type `W` must be a monoid (have `empty` and `append`).
//! Common examples: `String`, `Vec<T>`, custom log types.

use alloc::boxed::Box;
use core::any::TypeId;
use core::marker::PhantomData;

use crate::nexus::effect::{Eff, EffectMarker};
use crate::nexus::row::{Row, WRITER_BIT};

// =============================================================================
// Writer Effect Type
// =============================================================================

/// The Writer effect marker type.
///
/// `WriterEffect<W>` represents computations that can append to
/// a log of type `W`.
#[derive(Copy, Clone, Debug)]
pub struct WriterEffect<W> {
    _marker: PhantomData<W>,
}

impl<W> EffectMarker for WriterEffect<W> {
    const BIT: u128 = WRITER_BIT;
    const NAME: &'static str = "Writer";
}

/// Type alias for a row containing only Writer.
pub type WriterRow = Row<WRITER_BIT>;

// =============================================================================
// Writer Operations
// =============================================================================

/// Operations that can be performed with the Writer effect.
pub enum WriterOp<W> {
    /// Write a value to the log.
    Tell(W),
    /// Listen to the log produced by a computation.
    Listen,
    /// Modify the log produced by a computation.
    Pass,
}

// =============================================================================
// Monoid Trait
// =============================================================================

/// A monoid has an identity element and an associative binary operation.
pub trait Monoid: Clone {
    /// The identity element.
    fn empty() -> Self;

    /// Combine two values.
    fn append(&self, other: &Self) -> Self;
}

impl Monoid for alloc::string::String {
    fn empty() -> Self {
        alloc::string::String::new()
    }

    fn append(&self, other: &Self) -> Self {
        let mut result = self.clone();
        result.push_str(other);
        result
    }
}

impl<T: Clone> Monoid for alloc::vec::Vec<T> {
    fn empty() -> Self {
        alloc::vec::Vec::new()
    }

    fn append(&self, other: &Self) -> Self {
        let mut result = self.clone();
        result.extend(other.iter().cloned());
        result
    }
}

// =============================================================================
// Writer Effect Functions
// =============================================================================

/// Write a value to the log.
///
/// # Example
///
/// ```rust
/// use ordofp_core::nexus::effects::writer::WriterComputation;
///
/// let comp = WriterComputation::<String, ()>::tell("Hello, ".to_string())
///     .and_then(|_| WriterComputation::<String, ()>::tell("World!".to_string()));
/// let ((), log) = comp.run();
/// assert_eq!(log, "Hello, World!");
/// ```
pub fn writer_tell<W: Monoid + 'static>(_value: W) -> Eff<WriterRow, ()> {
    Eff::lazy(move || crate::cold_panic!("writer_tell requires Writer handler"))
}

// =============================================================================
// Concrete Writer Implementation
// =============================================================================

/// A concrete writer computation that can be run.
///
/// Uses an enum representation to avoid heap allocation for pure values and tell.
#[must_use = "computations do nothing unless run"]
#[repr(u8)]
pub enum WriterComputation<W: Monoid, A> {
    /// Pure value with empty log - no allocation.
    Pure(A),
    /// Tell operation - no allocation for the closure.
    Tell(W),
    /// Boxed computation (for complex chains).
    Boxed(Box<dyn FnOnce() -> (A, W)>),
}

impl<W: Monoid + 'static, A: 'static> WriterComputation<W, A> {
    /// Create a new writer computation from a function.
    #[inline(always)]
    pub fn new<F: FnOnce() -> (A, W) + 'static>(f: F) -> Self {
        WriterComputation::Boxed(Box::new(f))
    }

    /// Run the computation, returning result and log.
    ///
    /// `Pure` yields the value with `W::empty()`; `Tell` yields `()` with
    /// its log entry; `Boxed` invokes the deferred chain.
    ///
    /// # Panics
    ///
    /// Panics on a `Tell` variant whose result type `A` is not `()`
    /// (checked at runtime via `TypeId`). The `tell` constructor only
    /// builds `Tell` at `A = ()`, so this can only fire on a
    /// hand-constructed variant.
    #[inline(always)]
    pub fn run(self) -> (A, W) {
        match self {
            WriterComputation::Pure(a) => (a, W::empty()),
            WriterComputation::Tell(w) => {
                if TypeId::of::<A>() == TypeId::of::<()>() {
                    let mut opt: Option<()> = Some(());
                    let any_opt = &mut opt as &mut dyn core::any::Any;
                    let downcast: &mut Option<A> = any_opt.downcast_mut::<Option<A>>().unwrap();
                    let unit: A = downcast.take().unwrap();
                    (unit, w)
                } else {
                    panic!("WriterComputation::Tell invariant violated: A must be ()");
                }
            }
            WriterComputation::Boxed(f) => f(),
        }
    }

    /// Pure value with empty log - NO HEAP ALLOCATION.
    #[inline(always)]
    pub fn pure(value: A) -> Self {
        WriterComputation::Pure(value)
    }

    /// Map over the result.
    ///
    /// `f` is applied eagerly for `Pure`/`Tell` values and deferred until
    /// [`run`](Self::run) for `Boxed` chains; the log is untouched either
    /// way.
    ///
    /// # Panics
    ///
    /// Panics on a `Tell` variant whose result type `A` is not `()`
    /// (checked at runtime via `TypeId`). The `tell` constructor only
    /// builds `Tell` at `A = ()`, so this can only fire on a
    /// hand-constructed variant.
    #[inline(always)]
    pub fn map<B: 'static, F: FnOnce(A) -> B + 'static>(self, f: F) -> WriterComputation<W, B> {
        match self {
            WriterComputation::Pure(a) => WriterComputation::Pure(f(a)),
            WriterComputation::Tell(w) => {
                if TypeId::of::<A>() == TypeId::of::<()>() {
                    let mut opt: Option<()> = Some(());
                    let any_opt = &mut opt as &mut dyn core::any::Any;
                    let downcast: &mut Option<A> = any_opt.downcast_mut::<Option<A>>().unwrap();
                    let unit: A = downcast.take().unwrap();
                    let b = f(unit);
                    WriterComputation::Boxed(Box::new(move || (b, w)))
                } else {
                    panic!("WriterComputation::Tell invariant violated: A must be ()");
                }
            }
            WriterComputation::Boxed(run_fn) => WriterComputation::Boxed(Box::new(move || {
                let (a, w) = run_fn();
                (f(a), w)
            })),
        }
    }

    /// Chain two writer computations.
    #[inline(always)]
    pub fn and_then<B: 'static, F: FnOnce(A) -> WriterComputation<W, B> + 'static>(
        self,
        f: F,
    ) -> WriterComputation<W, B> {
        WriterComputation::Boxed(Box::new(move || {
            let (a, w1) = self.run();
            let (b, w2) = f(a).run();
            (b, w1.append(&w2))
        }))
    }

    /// Listen to the log produced.
    #[inline(always)]
    pub fn listen(self) -> WriterComputation<W, (A, W)> {
        WriterComputation::Boxed(Box::new(move || {
            let (a, w) = self.run();
            ((a, w.clone()), w)
        }))
    }

    /// Modify the log with a function.
    #[inline(always)]
    pub fn censor<F: FnOnce(W) -> W + 'static>(self, f: F) -> WriterComputation<W, A> {
        WriterComputation::Boxed(Box::new(move || {
            let (a, w) = self.run();
            (a, f(w))
        }))
    }
}

/// Specialized implementation for tell operation.
impl<W: Monoid + 'static> WriterComputation<W, ()> {
    /// Write a value to the log - NO HEAP ALLOCATION.
    #[inline(always)]
    pub fn tell(w: W) -> Self {
        WriterComputation::Tell(w)
    }

    /// Run the tell computation - specialized for A = ().
    #[inline(always)]
    pub fn run_tell(self) -> ((), W) {
        match self {
            WriterComputation::Pure(()) => ((), W::empty()),
            WriterComputation::Tell(w) => ((), w),
            WriterComputation::Boxed(f) => f(),
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::String;

    #[test]
    fn test_writer_pure() {
        let comp = WriterComputation::<String, i32>::pure(42);
        let (result, log) = comp.run();
        assert_eq!(result, 42);
        assert_eq!(log, "");
    }

    #[test]
    fn test_writer_tell() {
        let comp = WriterComputation::<String, ()>::tell("Hello".to_string());
        let ((), log) = comp.run();
        assert_eq!(log, "Hello");
    }

    #[test]
    fn test_writer_and_then() {
        let comp = WriterComputation::<String, ()>::tell("Hello, ".to_string())
            .and_then(|()| WriterComputation::<String, ()>::tell("World!".to_string()));
        let ((), log) = comp.run();
        assert_eq!(log, "Hello, World!");
    }

    #[test]
    fn test_writer_map() {
        let comp = WriterComputation::<String, i32>::pure(21).map(|x| x * 2);
        let (result, log) = comp.run();
        assert_eq!(result, 42);
        assert_eq!(log, "");
    }

    #[test]
    fn test_writer_listen() {
        let comp = WriterComputation::<String, ()>::tell("Log".to_string())
            .and_then(|()| WriterComputation::<String, i32>::pure(42))
            .listen();
        let ((result, inner_log), log) = comp.run();
        assert_eq!(result, 42);
        assert_eq!(inner_log, "Log");
        assert_eq!(log, "Log");
    }

    #[test]
    fn test_writer_censor() {
        let comp =
            WriterComputation::<String, ()>::tell("hello".to_string()).censor(|s| s.to_uppercase());
        let ((), log) = comp.run();
        assert_eq!(log, "HELLO");
    }

    #[test]
    fn test_writer_vec() {
        let comp = WriterComputation::<alloc::vec::Vec<i32>, ()>::tell(alloc::vec![1, 2])
            .and_then(|()| WriterComputation::<alloc::vec::Vec<i32>, ()>::tell(alloc::vec![3, 4]));
        let ((), log) = comp.run();
        assert_eq!(log, alloc::vec![1, 2, 3, 4]);
    }

    use alloc::string::ToString;
}
