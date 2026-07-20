//! Easy IO Operations
//!
//! Simplified IO patterns that hide the effect system complexity.
//!
//! # Example
//!
//! ```rust
//! use ordofp_core::easy::*;
//!
//! // Lazy IO computation
//! let computation = io(|| {
//!     println!("Hello, World!");
//!     42
//! });
//!
//! // Run the computation
//! let result = computation.run();
//! assert_eq!(result, 42);
//! ```

use alloc::boxed::Box;
use alloc::vec::Vec;

// =============================================================================
// IO Type
// =============================================================================

/// A lazy IO computation.
///
/// This represents a computation that performs IO when run.
/// It provides a pure interface for composing effectful operations.
pub struct IO<A> {
    run_fn: Box<dyn FnOnce() -> A>,
}

impl<A: 'static> IO<A> {
    /// Create a new IO computation.
    #[inline]
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce() -> A + 'static,
    {
        IO {
            run_fn: Box::new(f),
        }
    }

    /// Run the IO computation.
    #[inline]
    pub fn run(self) -> A {
        (self.run_fn)()
    }

    /// Map over the result.
    #[inline]
    pub fn map<B: 'static, F>(self, f: F) -> IO<B>
    where
        F: FnOnce(A) -> B + 'static,
    {
        IO::new(move || f(self.run()))
    }

    /// Chain with another IO computation.
    #[inline]
    pub fn and_then<B: 'static, F>(self, f: F) -> IO<B>
    where
        F: FnOnce(A) -> IO<B> + 'static,
    {
        IO::new(move || f(self.run()).run())
    }

    /// Sequence, keeping the second result.
    #[inline]
    pub fn then<B: 'static>(self, next: IO<B>) -> IO<B> {
        IO::new(move || {
            let _ = self.run();
            next.run()
        })
    }

    /// Sequence, keeping the first result.
    #[inline]
    pub fn before<B: 'static>(self, other: IO<B>) -> IO<A> {
        IO::new(move || {
            let a = self.run();
            let _ = other.run();
            a
        })
    }

    /// Run only if condition is true.
    #[inline]
    pub fn when(self, condition: bool) -> IO<Option<A>> {
        IO::new(move || if condition { Some(self.run()) } else { None })
    }

    /// Run only if condition is false.
    #[inline]
    pub fn unless(self, condition: bool) -> IO<Option<A>> {
        self.when(!condition)
    }
}

/// Create a pure IO value.
#[inline]
pub fn io_pure<A: 'static>(value: A) -> IO<A> {
    IO::new(move || value)
}

/// Create an IO computation from a function.
#[inline]
pub fn io<A: 'static, F>(f: F) -> IO<A>
where
    F: FnOnce() -> A + 'static,
{
    IO::new(f)
}

/// Create an IO computation that does nothing.
#[inline]
pub fn io_unit() -> IO<()> {
    IO::new(|| ())
}

// =============================================================================
// IO Combinators
// =============================================================================

/// Sequence multiple IO computations.
pub fn io_sequence<A: 'static>(computations: Vec<IO<A>>) -> IO<Vec<A>> {
    IO::new(move || computations.into_iter().map(IO::run).collect())
}

/// Sequence, discarding results.
pub fn io_sequence_<A: 'static>(computations: Vec<IO<A>>) -> IO<()> {
    IO::new(move || {
        for io in computations {
            io.run();
        }
    })
}

/// Run two IO computations and combine results.
pub fn io_both<A: 'static, B: 'static>(first: IO<A>, second: IO<B>) -> IO<(A, B)> {
    IO::new(move || (first.run(), second.run()))
}

/// Apply a function in IO to a value in IO.
pub fn io_ap<A: 'static, B: 'static, F>(io_f: IO<F>, io_a: IO<A>) -> IO<B>
where
    F: FnOnce(A) -> B + 'static,
{
    IO::new(move || {
        let f = io_f.run();
        let a = io_a.run();
        f(a)
    })
}

/// Run the IO computation once and replicate its result N times.
///
/// The effect executes a single time; the produced value is cloned. For
/// run-it-each-time semantics use [`io_replicate_m`].
pub fn io_replicate<A: Clone + 'static>(n: usize, io_a: IO<A>) -> IO<Vec<A>>
where
{
    IO::new(move || {
        let value = io_a.run();
        core::iter::repeat_n(value, n).collect()
    })
}

/// Repeat an IO computation N times, running it each time.
pub fn io_replicate_m<A: 'static, F>(n: usize, mut f: F) -> IO<Vec<A>>
where
    F: FnMut() -> IO<A> + 'static,
{
    IO::new(move || (0..n).map(|_| f().run()).collect())
}

/// Conditional IO.
pub fn io_if_then_else<A: 'static>(condition: bool, if_true: IO<A>, if_false: IO<A>) -> IO<A> {
    IO::new(move || {
        if condition {
            if_true.run()
        } else {
            if_false.run()
        }
    })
}

// =============================================================================
// IO with Resources
// =============================================================================

/// Run an IO computation with a resource, ensuring cleanup.
///
/// `release` runs after the use-phase IO completes — including when it
/// panics (the resource is held by a drop guard), so the bracket guarantee
/// holds on unwind as well.
///
/// # Panics
///
/// Panics only if the internal guard invariant (the resource is stored
/// immediately before the use phase) is violated, which indicates a bug
/// in this crate. Panics raised by `acquire`, `use_resource`, or
/// `release` themselves propagate unchanged.
///
/// # Example
///
/// ```rust
/// use ordofp_core::easy::{io, io_bracket};
///
/// let result = io_bracket(
///     io(|| 7),
///     |r: &i32| {
///         let r = *r;
///         io(move || r * 6)
///     },
///     |_r| {},
/// )
/// .run();
/// assert_eq!(result, 42);
/// ```
pub fn io_bracket<R: 'static, A: 'static>(
    acquire: IO<R>,
    use_resource: impl FnOnce(&R) -> IO<A> + 'static,
    release: impl FnOnce(R) + 'static,
) -> IO<A> {
    // Holds the resource through the use phase; Drop releases it even on
    // unwind, which is what makes the bracket guarantee real.
    struct Custos<R, F: FnOnce(R)> {
        resource: Option<R>,
        release: Option<F>,
    }
    impl<R, F: FnOnce(R)> Drop for Custos<R, F> {
        fn drop(&mut self) {
            if let (Some(resource), Some(release)) = (self.resource.take(), self.release.take()) {
                release(resource);
            }
        }
    }
    IO::new(move || {
        let guard = Custos {
            resource: Some(acquire.run()),
            release: Some(release),
        };
        let io_a = use_resource(
            guard
                .resource
                .as_ref()
                .expect("io_bracket invariant: resource set just above"),
        );
        io_a.run()
        // `guard` drops here, releasing the resource (also on unwind).
    })
}

/// Run an IO computation with a clonable resource.
pub fn io_with_resource<R: Clone + 'static, A: 'static>(
    acquire: IO<R>,
    use_resource: impl FnOnce(&R) -> IO<A> + 'static,
    release: impl FnOnce(R) + 'static,
) -> IO<A> {
    IO::new(move || {
        let resource = acquire.run();
        let result = use_resource(&resource).run();
        release(resource);
        result
    })
}

// =============================================================================
// IO with Errors
// =============================================================================

/// An IO computation that may fail.
pub struct IOResult<A, E> {
    run_fn: Box<dyn FnOnce() -> Result<A, E>>,
}

impl<A: 'static, E: 'static> IOResult<A, E> {
    /// Create a new fallible IO computation.
    #[inline]
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce() -> Result<A, E> + 'static,
    {
        IOResult {
            run_fn: Box::new(f),
        }
    }

    /// Run the computation.
    ///
    /// # Errors
    ///
    /// Returns whatever `Err` the wrapped closure produces when the
    /// deferred effect finally executes; running adds no failure modes
    /// of its own.
    #[inline]
    pub fn run(self) -> Result<A, E> {
        (self.run_fn)()
    }

    /// Create a successful computation.
    #[inline]
    pub fn ok(value: A) -> Self {
        IOResult::new(move || Ok(value))
    }

    /// Create a failed computation.
    #[inline]
    pub fn err(error: E) -> Self {
        IOResult::new(move || Err(error))
    }

    /// Map over the success value.
    #[inline]
    pub fn map<B: 'static, F>(self, f: F) -> IOResult<B, E>
    where
        F: FnOnce(A) -> B + 'static,
    {
        IOResult::new(move || self.run().map(f))
    }

    /// Map over the error.
    #[inline]
    pub fn map_err<E2: 'static, F>(self, f: F) -> IOResult<A, E2>
    where
        F: FnOnce(E) -> E2 + 'static,
    {
        IOResult::new(move || self.run().map_err(f))
    }

    /// Chain with another fallible computation.
    #[inline]
    pub fn and_then<B: 'static, F>(self, f: F) -> IOResult<B, E>
    where
        F: FnOnce(A) -> IOResult<B, E> + 'static,
    {
        IOResult::new(move || self.run().and_then(|a| f(a).run()))
    }

    /// Recover from an error.
    #[inline]
    pub fn or_else<F>(self, f: F) -> IOResult<A, E>
    where
        F: FnOnce(E) -> IOResult<A, E> + 'static,
    {
        IOResult::new(move || self.run().or_else(|e| f(e).run()))
    }

    /// Convert to IO, panicking on error.
    ///
    /// # Panics
    ///
    /// The returned `IO`, when run, panics if this computation produces
    /// `Err`; the panic message includes the error's `Debug` rendering.
    /// Use [`IOResult::unwrap_or`] to substitute a default instead.
    #[inline]
    pub fn unwrap(self) -> IO<A>
    where
        E: core::fmt::Debug,
    {
        IO::new(move || self.run().unwrap())
    }

    /// Convert to IO with a default value.
    #[inline]
    pub fn unwrap_or(self, default: A) -> IO<A> {
        IO::new(move || self.run().unwrap_or(default))
    }
}

/// Create a fallible IO from a function.
#[inline]
pub fn io_try<A: 'static, E: 'static, F>(f: F) -> IOResult<A, E>
where
    F: FnOnce() -> Result<A, E> + 'static,
{
    IOResult::new(f)
}

/// Lift an IO into `IOResult`.
#[inline]
pub fn io_lift<A: 'static, E: 'static>(io_a: IO<A>) -> IOResult<A, E> {
    IOResult::new(move || Ok(io_a.run()))
}

// =============================================================================
// Lazy Computations
// =============================================================================

/// A lazily evaluated value.
pub struct Lazy<A> {
    thunk: core::cell::OnceCell<A>,
    compute: core::cell::Cell<Option<Box<dyn FnOnce() -> A>>>,
}

impl<A> Lazy<A> {
    /// Create a new lazy value.
    #[inline]
    pub fn new<F: FnOnce() -> A + 'static>(f: F) -> Self {
        Lazy {
            thunk: core::cell::OnceCell::new(),
            compute: core::cell::Cell::new(Some(Box::new(f))),
        }
    }

    /// Force evaluation and get the value.
    ///
    /// The initializer runs at most once; subsequent calls return the
    /// cached value.
    ///
    /// # Panics
    ///
    /// Panics if the initializer closure is unavailable when the cell is
    /// still empty: this happens when a previous `force` panicked partway
    /// through, or when the initializer reentrantly calls `force` on the
    /// same `Lazy`. Absent those, this never panics.
    #[inline]
    pub fn force(&self) -> &A {
        self.thunk.get_or_init(|| {
            let f = self.compute.take().expect("Lazy: already forced");
            f()
        })
    }

    /// Check if the value has been computed.
    #[inline]
    pub fn is_forced(&self) -> bool {
        self.thunk.get().is_some()
    }
}

/// Create a lazy value.
pub fn lazy<A: 'static, F: FnOnce() -> A + 'static>(f: F) -> Lazy<A> {
    Lazy::new(f)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_io_basic() {
        let io_val = io(|| 42);
        assert_eq!(io_val.run(), 42);
    }

    #[test]
    fn test_io_map() {
        let io_val = io(|| 21).map(|x| x * 2);
        assert_eq!(io_val.run(), 42);
    }

    #[test]
    fn test_io_and_then() {
        let io_val = io(|| 21).and_then(|x| io(move || x * 2));
        assert_eq!(io_val.run(), 42);
    }

    #[test]
    fn test_io_sequence() {
        let ios = alloc::vec![io(|| 1), io(|| 2), io(|| 3)];
        let results = io_sequence(ios).run();
        assert_eq!(results, alloc::vec![1, 2, 3]);
    }

    #[test]
    fn test_io_both() {
        let result = io_both(io(|| 1), io(|| 2)).run();
        assert_eq!(result, (1, 2));
    }

    #[test]
    fn test_io_bracket_releases_resource() {
        use core::sync::atomic::{AtomicBool, Ordering};
        static RELEASED: AtomicBool = AtomicBool::new(false);
        RELEASED.store(false, Ordering::SeqCst);
        let result = io_bracket(
            io(|| 7),
            |r: &i32| {
                let r = *r;
                io(move || r * 6)
            },
            |_r| RELEASED.store(true, Ordering::SeqCst),
        )
        .run();
        assert_eq!(result, 42);
        assert!(
            RELEASED.load(Ordering::SeqCst),
            "io_bracket must run release after use"
        );
    }

    #[test]
    fn test_io_bracket_releases_on_panic() {
        use core::sync::atomic::{AtomicBool, Ordering};
        static RELEASED_ON_PANIC: AtomicBool = AtomicBool::new(false);
        RELEASED_ON_PANIC.store(false, Ordering::SeqCst);
        let bracket = io_bracket(
            io(|| 7),
            |_r: &i32| io(|| -> i32 { panic!("use phase fails") }),
            |_r| RELEASED_ON_PANIC.store(true, Ordering::SeqCst),
        );
        let outcome =
            std::panic::catch_unwind(core::panic::AssertUnwindSafe(move || bracket.run()));
        assert!(outcome.is_err(), "use phase must have panicked");
        assert!(
            RELEASED_ON_PANIC.load(Ordering::SeqCst),
            "io_bracket must run release even when the use phase panics"
        );
    }

    #[test]
    fn test_io_when() {
        let result_some = io(|| 42).when(true).run();
        assert_eq!(result_some, Some(42));

        let result_none = io(|| 42).when(false).run();
        assert_eq!(result_none, None);
    }

    #[test]
    fn test_io_result() {
        let success = IOResult::<i32, &str>::ok(42);
        assert_eq!(success.run(), Ok(42));

        let failure = IOResult::<i32, &str>::err("oops");
        assert_eq!(failure.run(), Err("oops"));
    }

    #[test]
    fn test_io_result_and_then() {
        let result = IOResult::<i32, &str>::ok(21).and_then(|x| IOResult::ok(x * 2));
        assert_eq!(result.run(), Ok(42));
    }

    #[test]
    fn test_lazy() {
        let lazy_val = lazy(|| 42);

        assert!(!lazy_val.is_forced());
        assert_eq!(*lazy_val.force(), 42);
        assert!(lazy_val.is_forced());
        // Second force returns cached value
        assert_eq!(*lazy_val.force(), 42);
    }
}
