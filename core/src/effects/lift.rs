//! Effect Lifting - Utilities for lifting values into effectful contexts
//!
//! > *"Elevare ad effectum"*
//! > — To lift to effect. (Neo-Latin)
//!
//! This module provides utilities for lifting pure values and functions
//! into effectful contexts.

use super::Effectus;
use core::future::Future;

/// Trait for lifting values into effectful computations.
///
/// `LiftEffectus` allows wrapping pure values in an effect context,
/// similar to `pure` or `return` in monads.
///
/// # Example
///
/// ```rust
/// use ordofp_core::effects::{Effectus, IoEffectus, LiftEffectus, lift_pure};
///
/// fn example<E: Effectus>() -> impl LiftEffectus<E, Output = i32> {
///     lift_pure(42)
/// }
///
/// let lifted = example::<IoEffectus>();
/// assert_eq!(lifted.run(), 42);
/// ```
pub trait LiftEffectus<E: Effectus> {
    /// The type of value produced.
    type Output;

    /// Execute the lifted computation.
    fn run(self) -> Self::Output;
}

/// A pure value lifted into an effect context.
///
/// This represents a computation that simply returns a value
/// without performing any actual effects.
#[derive(Debug, Clone, Copy)]
pub struct PureLift<A, E> {
    value: A,
    _effect: core::marker::PhantomData<E>,
}

impl<A, E: Effectus> PureLift<A, E> {
    /// Create a new pure lift.
    #[inline]
    pub fn new(value: A) -> Self {
        PureLift {
            value,
            _effect: core::marker::PhantomData,
        }
    }
}

impl<A, E: Effectus> LiftEffectus<E> for PureLift<A, E> {
    type Output = A;

    #[inline]
    fn run(self) -> Self::Output {
        self.value
    }
}

/// Lift a pure value into an effect context.
///
/// This is analogous to `pure` or `return` in monadic programming.
///
/// # Example
///
/// ```rust
/// use ordofp_core::effects::{LiftEffectus, lift_pure, IoEffectus};
///
/// let lifted = lift_pure::<_, IoEffectus>(42);
/// assert_eq!(lifted.run(), 42);
/// ```
#[inline]
pub fn lift_pure<A, E: Effectus>(value: A) -> PureLift<A, E> {
    PureLift::new(value)
}

/// Lift an IO operation into an `IoEffectus` context.
///
/// This marks a computation as having IO effects.
///
/// # Example
///
/// ```rust
/// use ordofp_core::effects::{LiftEffectus, lift_io};
///
/// let io_op = lift_io(|| {
///     println!("Hello, world!");
///     42
/// });
/// assert_eq!(io_op.run(), 42);
/// ```
#[inline]
pub fn lift_io<A, F>(f: F) -> IoLift<A, F>
where
    F: FnOnce() -> A,
{
    IoLift {
        f,
        _output: core::marker::PhantomData,
    }
}

/// An IO operation lifted into an `IoEffectus` context.
#[derive(Debug)]
pub struct IoLift<A, F> {
    f: F,
    _output: core::marker::PhantomData<A>,
}

impl<A, F: FnOnce() -> A> LiftEffectus<super::IoEffectus> for IoLift<A, F> {
    type Output = A;

    #[inline]
    fn run(self) -> Self::Output {
        (self.f)()
    }
}

/// Lift an error-producing operation into an `ErrorEffectus` context.
///
/// This marks a computation as potentially producing errors.
///
/// # Example
///
/// ```rust
/// use ordofp_core::effects::{LiftEffectus, lift_error};
///
/// let result = lift_error::<i32, String, _>(|| {
///     if true { Ok(42) } else { Err("error".to_string()) }
/// });
/// assert_eq!(result.run(), Ok(42));
/// ```
#[inline]
pub fn lift_error<A, E, F>(f: F) -> ErrorLift<A, E, F>
where
    F: FnOnce() -> Result<A, E>,
{
    ErrorLift {
        f,
        _phantom: core::marker::PhantomData,
    }
}

/// An error-producing operation lifted into an `ErrorEffectus` context.
#[derive(Debug)]
pub struct ErrorLift<A, E, F> {
    f: F,
    _phantom: core::marker::PhantomData<(A, E)>,
}

impl<A, E: Send + Sync + 'static, F: FnOnce() -> Result<A, E>> LiftEffectus<super::ErrorEffectus<E>>
    for ErrorLift<A, E, F>
{
    type Output = Result<A, E>;

    #[inline]
    fn run(self) -> Self::Output {
        (self.f)()
    }
}

/// Lift a state operation into a `StatusEffectus` context.
///
/// This marks a computation as using state.
///
/// # Example
///
/// ```rust
/// use ordofp_core::effects::lift_state;
///
/// let state_op = lift_state::<i32, i32, _>(|s| (s + 1, s));
/// let (new_state, value) = state_op.run_state(5);
/// assert_eq!(new_state, 6);
/// assert_eq!(value, 5);
/// ```
#[inline]
pub fn lift_state<S, A, F>(f: F) -> StateLift<S, A, F>
where
    F: FnOnce(S) -> (S, A),
{
    StateLift {
        f,
        _state: core::marker::PhantomData,
    }
}

/// A state operation lifted into a `StatusEffectus` context.
#[derive(Debug)]
pub struct StateLift<S, A, F> {
    f: F,
    _state: core::marker::PhantomData<(S, A)>,
}

impl<S: Send + Sync + 'static, A, F: FnOnce(S) -> (S, A)> StateLift<S, A, F> {
    /// Run the state operation with an initial state.
    #[inline]
    pub fn run_state(self, state: S) -> (S, A) {
        (self.f)(state)
    }
}

/// Lift an async operation into an `AsyncEffectus` context.
///
/// This marks a computation as asynchronous.
///
/// # Example
///
/// ```rust
/// use ordofp_core::effects::lift_async;
///
/// let async_op = lift_async(async {
///     42
/// });
/// let _future = async_op.into_future();
/// ```
#[inline]
pub fn lift_async<A, F>(f: F) -> AsyncLift<F>
where
    F: Future<Output = A>,
{
    AsyncLift { f }
}

/// An async operation lifted into an `AsyncEffectus` context.
pub struct AsyncLift<F> {
    f: F,
}

impl<A, F: Future<Output = A>> AsyncLift<F> {
    /// Get the inner future.
    #[inline]
    pub fn into_future(self) -> F {
        self.f
    }
}

/// Lift a reader operation into a `ReaderEffectus` context.
///
/// This marks a computation as reading from an environment.
///
/// # Example
///
/// ```rust
/// use ordofp_core::effects::lift_reader;
///
/// let reader_op = lift_reader::<String, usize, _>(|env: &String| env.len());
/// let result = reader_op.run_reader(&"hello".to_string());
/// assert_eq!(result, 5);
/// ```
#[inline]
pub fn lift_reader<R, A, F>(f: F) -> ReaderLift<R, A, F>
where
    F: FnOnce(&R) -> A,
{
    ReaderLift {
        f,
        _env: core::marker::PhantomData,
    }
}

/// A reader operation lifted into a `ReaderEffectus` context.
#[derive(Debug)]
pub struct ReaderLift<R, A, F> {
    f: F,
    _env: core::marker::PhantomData<(R, A)>,
}

impl<R: Send + Sync + 'static, A, F: FnOnce(&R) -> A> ReaderLift<R, A, F> {
    /// Run the reader operation with an environment.
    #[inline]
    pub fn run_reader(self, env: &R) -> A {
        (self.f)(env)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effects::IoEffectus;
    use alloc::string::{String, ToString};

    #[test]
    fn test_lift_pure() {
        let lifted = lift_pure::<_, IoEffectus>(42);
        assert_eq!(lifted.run(), 42);
    }

    #[test]
    fn test_lift_io() {
        let io_op = lift_io(|| 42);
        assert_eq!(io_op.run(), 42);
    }

    #[test]
    fn test_lift_error_ok() {
        let result = lift_error::<i32, String, _>(|| Ok(42));
        assert_eq!(result.run(), Ok(42));
    }

    #[test]
    fn test_lift_error_err() {
        let result = lift_error::<i32, String, _>(|| Err("error".to_string()));
        assert_eq!(result.run(), Err("error".to_string()));
    }

    #[test]
    fn test_lift_state() {
        let state_op = lift_state::<i32, i32, _>(|s| (s + 1, s));
        let (new_state, value) = state_op.run_state(5);
        assert_eq!(new_state, 6);
        assert_eq!(value, 5);
    }

    #[test]
    fn test_lift_reader() {
        let reader_op = lift_reader::<String, usize, _>(|env: &String| env.len());
        let result = reader_op.run_reader(&"hello".to_string());
        assert_eq!(result, 5);
    }
}
