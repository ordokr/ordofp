//! Effect Representations - Per-Effect Runtime Shapes
//!
//! This module provides one runtime representation per effect pattern
//! (pure / state / reader / error / general continuation). Only `PureRepr`
//! and `ErrorRepr` are allocation-free; `StateRepr`, `ReaderRepr`, and
//! `ContRepr` box their closures — "optimal" here means *shaped for the
//! effect*, not zero-cost.
//!
//! These representations are also **standalone**: `Eff` itself does not
//! select among them — the `EffRepr` marker trait records which repr fits
//! which row, but nothing performs compile-time representation selection.

use alloc::boxed::Box;
use core::marker::PhantomData;

use super::row::{EffectRow, ErrorRow, Pure, ReaderRow, StateRow};

// =============================================================================
// Pure Representation
// =============================================================================

/// Zero-cost representation for pure computations.
///
/// When a computation has no effects, it's represented as just the value.
/// No boxing, no indirection, no overhead.
#[repr(transparent)]
pub struct PureRepr<A> {
    /// The pure value.
    pub value: A,
}

impl<A> PureRepr<A> {
    /// Create a new pure representation.
    #[inline(always)]
    pub const fn new(value: A) -> Self {
        PureRepr { value }
    }

    /// Extract the value.
    #[inline(always)]
    pub fn run(self) -> A {
        self.value
    }

    /// Map over the value.
    #[inline(always)]
    pub fn map<B, F: FnOnce(A) -> B>(self, f: F) -> PureRepr<B> {
        PureRepr::new(f(self.value))
    }
}

// =============================================================================
// State Representation
// =============================================================================

/// Representation for state-only computations.
///
/// State computations are represented as `S -> (A, S)` functions. Note the
/// function is **boxed** — each construction/`map`/`and_then` allocates; for
/// an allocation-free state representation see `optim::state_fast`.
pub struct StateRepr<S, A> {
    /// The state transition function.
    run_fn: Box<dyn FnOnce(S) -> (A, S)>,
}

impl<S: 'static, A: 'static> StateRepr<S, A> {
    /// Create a new state representation.
    #[inline]
    pub fn new<F: FnOnce(S) -> (A, S) + 'static>(f: F) -> Self {
        StateRepr {
            run_fn: Box::new(f),
        }
    }

    /// Run the state computation.
    #[inline]
    pub fn run(self, initial: S) -> (A, S) {
        (self.run_fn)(initial)
    }

    /// Read the current state without modifying it.
    ///
    /// Returns a clone of the state as the result value while leaving the state
    /// itself unchanged. This is the low-level `get` primitive for `StateRepr`;
    /// it is equivalent to the state-monad operation `λs. (s, s)`.
    ///
    /// Requires `S: Clone` because the state must be both returned as the result
    /// and kept intact for subsequent computations in the chain.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::nexus::StateRepr;
    ///
    /// let comp = StateRepr::<i32, i32>::get().map(|s| s + 1);
    /// assert_eq!(comp.run(10), (11, 10)); // result = 11, state unchanged
    /// ```
    #[inline]
    pub fn get() -> StateRepr<S, S>
    where
        S: Clone,
    {
        StateRepr::new(|s: S| (s.clone(), s))
    }

    /// Set the state.
    #[inline]
    pub fn put(new_state: S) -> StateRepr<S, ()> {
        StateRepr::new(move |_| ((), new_state))
    }

    /// Modify the state.
    #[inline]
    pub fn modify<F: FnOnce(S) -> S + 'static>(f: F) -> StateRepr<S, ()> {
        StateRepr::new(move |s| ((), f(s)))
    }

    /// Map over the result.
    pub fn map<B: 'static, F: FnOnce(A) -> B + 'static>(self, f: F) -> StateRepr<S, B> {
        StateRepr::new(move |s| {
            let (a, s2) = (self.run_fn)(s);
            (f(a), s2)
        })
    }

    /// Chain two state computations.
    pub fn and_then<B: 'static, F: FnOnce(A) -> StateRepr<S, B> + 'static>(
        self,
        f: F,
    ) -> StateRepr<S, B> {
        StateRepr::new(move |s| {
            let (a, s2) = (self.run_fn)(s);
            f(a).run(s2)
        })
    }
}

// =============================================================================
// Reader Representation
// =============================================================================

/// Representation for reader-only computations.
///
/// Reader computations are represented as `&E -> A` functions. Note the
/// function is **boxed** — each construction/`map`/`and_then` allocates; for
/// an allocation-free reader representation see `optim::reader_fast`.
pub struct ReaderRepr<E, A> {
    /// The reader function.
    run_fn: Box<dyn FnOnce(&E) -> A>,
}

impl<E: 'static, A: 'static> ReaderRepr<E, A> {
    /// Create a new reader representation.
    #[inline]
    pub fn new<F: FnOnce(&E) -> A + 'static>(f: F) -> Self {
        ReaderRepr {
            run_fn: Box::new(f),
        }
    }

    /// Run the reader computation.
    #[inline]
    pub fn run(self, env: &E) -> A {
        (self.run_fn)(env)
    }

    /// Get the environment.
    #[inline]
    pub fn ask() -> ReaderRepr<E, E>
    where
        E: Clone,
    {
        ReaderRepr::new(|e: &E| e.clone())
    }

    /// Extract from the environment.
    #[inline]
    pub fn asks<F: FnOnce(&E) -> A + 'static>(f: F) -> Self {
        ReaderRepr::new(f)
    }

    /// Map over the result.
    pub fn map<B: 'static, F: FnOnce(A) -> B + 'static>(self, f: F) -> ReaderRepr<E, B> {
        ReaderRepr::new(move |e| f((self.run_fn)(e)))
    }

    /// Chain two reader computations.
    pub fn and_then<B: 'static, F: FnOnce(A) -> ReaderRepr<E, B> + 'static>(
        self,
        f: F,
    ) -> ReaderRepr<E, B> {
        ReaderRepr::new(move |e| {
            let a = (self.run_fn)(e);
            f(a).run(e)
        })
    }
}

// =============================================================================
// Error Representation
// =============================================================================

/// Zero-cost representation for error-only computations.
///
/// Error computations are represented as `Result<A, E>`.
pub struct ErrorRepr<E, A> {
    /// The result value.
    pub result: Result<A, E>,
}

impl<E, A> ErrorRepr<E, A> {
    /// Create a success value.
    #[inline(always)]
    pub const fn ok(value: A) -> Self {
        ErrorRepr { result: Ok(value) }
    }

    /// Create an error value.
    #[inline(always)]
    pub const fn err(error: E) -> Self {
        ErrorRepr { result: Err(error) }
    }

    /// Run the error computation.
    ///
    /// # Errors
    ///
    /// Returns the stored `Err(e)` when this representation holds a
    /// failure (built via [`err`](Self::err) or a short-circuited
    /// `and_then` chain). The representation *is* the `Result`, so
    /// running adds no failure modes of its own.
    #[inline(always)]
    pub fn run(self) -> Result<A, E> {
        self.result
    }

    /// Map over the result.
    pub fn map<B, F: FnOnce(A) -> B>(self, f: F) -> ErrorRepr<E, B> {
        ErrorRepr {
            result: self.result.map(f),
        }
    }

    /// Chain error computations.
    pub fn and_then<B, F: FnOnce(A) -> ErrorRepr<E, B>>(self, f: F) -> ErrorRepr<E, B> {
        match self.result {
            Ok(a) => f(a),
            Err(e) => ErrorRepr::err(e),
        }
    }
}

// =============================================================================
// Continuation Representation (General Case)
// =============================================================================

/// General representation using continuations.
///
/// This is the fallback for computations with multiple effects. The
/// continuation is **boxed**; every `pure`/`map` allocates.
pub struct ContRepr<R: EffectRow, A> {
    /// The computation.
    inner: Box<dyn FnOnce() -> A>,
    /// Phantom for effect row.
    _marker: PhantomData<R>,
}

impl<R: EffectRow + 'static, A: 'static> ContRepr<R, A> {
    /// Create from a pure value.
    pub fn pure(value: A) -> Self {
        ContRepr {
            inner: Box::new(move || value),
            _marker: PhantomData,
        }
    }

    /// Run the computation.
    pub fn run(self) -> A {
        (self.inner)()
    }

    /// Map over the result.
    pub fn map<B: 'static, F: FnOnce(A) -> B + 'static>(self, f: F) -> ContRepr<R, B> {
        ContRepr {
            inner: Box::new(move || f((self.inner)())),
            _marker: PhantomData,
        }
    }
}

// =============================================================================
// Effect Representation Trait (Simplified)
// =============================================================================

/// Marker trait for effect representations.
///
/// Records which representation is valid for a given effect row. Note that
/// this trait is **not consumed by `Eff`** — no automatic representation
/// selection happens; the reprs are standalone building blocks.
pub trait EffRepr<R: EffectRow, A>: Sized {}

impl<A> EffRepr<Pure, A> for PureRepr<A> {}
impl<S: 'static, A: 'static> EffRepr<StateRow, A> for StateRepr<S, A> {}
impl<E: 'static, A: 'static> EffRepr<ReaderRow, A> for ReaderRepr<E, A> {}
impl<Err, A> EffRepr<ErrorRow, A> for ErrorRepr<Err, A> {}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pure_repr() {
        let repr = PureRepr::new(42);
        assert_eq!(repr.run(), 42);
    }

    #[test]
    fn test_pure_map() {
        let repr = PureRepr::new(21);
        let mapped = repr.map(|x| x * 2);
        assert_eq!(mapped.run(), 42);
    }

    #[test]
    fn test_state_repr() {
        let repr = StateRepr::<i32, i32>::get();
        let (value, state) = repr.run(42);
        assert_eq!(value, 42);
        assert_eq!(state, 42);
    }

    #[test]
    fn test_state_put() {
        let repr = StateRepr::<i32, ()>::put(100);
        let ((), state) = repr.run(42);
        assert_eq!(state, 100);
    }

    #[test]
    fn test_state_modify() {
        let repr = StateRepr::<i32, ()>::modify(|x| x + 10);
        let ((), state) = repr.run(32);
        assert_eq!(state, 42);
    }

    #[test]
    fn test_reader_repr() {
        let repr = ReaderRepr::<i32, i32>::ask();
        let value = repr.run(&42);
        assert_eq!(value, 42);
    }

    #[test]
    fn test_reader_asks() {
        let repr = ReaderRepr::<(i32, i32), i32>::asks(|(a, b)| a + b);
        let value = repr.run(&(20, 22));
        assert_eq!(value, 42);
    }

    #[test]
    fn test_error_ok() {
        let repr: ErrorRepr<&str, i32> = ErrorRepr::ok(42);
        assert_eq!(repr.run(), Ok(42));
    }

    #[test]
    fn test_error_err() {
        let repr: ErrorRepr<&str, i32> = ErrorRepr::err("oops");
        assert_eq!(repr.run(), Err("oops"));
    }

    #[test]
    fn test_error_map() {
        let repr: ErrorRepr<&str, i32> = ErrorRepr::ok(21);
        let mapped = repr.map(|x| x * 2);
        assert_eq!(mapped.run(), Ok(42));
    }

    #[test]
    fn test_cont_pure() {
        let repr: ContRepr<Pure, i32> = ContRepr::pure(42);
        assert_eq!(repr.run(), 42);
    }

    #[test]
    fn test_cont_map() {
        let repr: ContRepr<Pure, i32> = ContRepr::pure(21);
        let mapped = repr.map(|x| x * 2);
        assert_eq!(mapped.run(), 42);
    }
}
