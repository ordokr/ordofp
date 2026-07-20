//! Error Effect - Short-Circuit Error Handling
//!
//! The Error effect provides short-circuit error handling, allowing
//! computations to fail early with an error value.
//!
//! # Handler Laws
//!
//! The Error handler must satisfy these algebraic laws:
//!
//! ## Catch-Throw Law
//! ```text
//! catch(throw(e), h) = h(e)
//! ```
//! Catching a thrown error invokes the handler with that error.
//!
//! ## Catch-Pure Law
//! ```text
//! catch(pure(x), h) = pure(x)
//! ```
//! Catching on a successful value returns that value unchanged.
//!
//! ## Throw-Bind Law
//! ```text
//! throw(e).and_then(f) = throw(e)
//! ```
//! Binding after a throw propagates the error (short-circuit).
//!
//! # Verification Tier
//!
//! **Tier 1**: Laws are tested via property-based tests in `nexus::laws`.
//!
//! # Performance
//!
//! When a computation uses only the Error effect, it compiles to
//! the same code as using Rust's native `Result` type.

use core::marker::PhantomData;

use crate::nexus::effect::{Eff, EffectMarker};
use crate::nexus::row::{ERROR_BIT, Row};

// =============================================================================
// Error Effect Type
// =============================================================================

/// The Error effect marker type.
///
/// `ErrorEffect<E>` represents computations that can fail with
/// an error of type `E`.
#[derive(Copy, Clone, Debug)]
pub struct ErrorEffect<E> {
    _marker: PhantomData<E>,
}

impl<E> EffectMarker for ErrorEffect<E> {
    const BIT: u128 = ERROR_BIT;
    const NAME: &'static str = "Error";
}

/// Type alias for a row containing only Error.
pub type ErrorRow = Row<ERROR_BIT>;

// =============================================================================
// Error Operations
// =============================================================================

/// Operations that can be performed with the Error effect.
pub enum ErrorOp<E> {
    /// Raise an error.
    Throw(E),
    /// Catch an error.
    Catch,
}

// =============================================================================
// Error Effect Functions
// =============================================================================

/// Raise an error.
///
/// # Example
///
/// ```rust
/// use ordofp_core::nexus::Eff;
/// use ordofp_core::nexus::effects::error::{ErrorRow, error_ok, error_throw};
///
/// fn divide(a: i32, b: i32) -> Eff<ErrorRow, i32> {
///     if b == 0 {
///         error_throw("Division by zero")
///     } else {
///         error_ok::<&str, i32>(a / b)
///     }
/// }
///
/// // Building the computation does not perform any effect: `error_throw` and
/// // `error_ok` currently return a stub `Eff` that only panics if it is ever
/// // forced by a handler (no Error-row handler exists yet). See
/// // `ErrorComputation` in this module for a fully working error type.
/// let _computed = divide(10, 2);
/// let _failed = divide(10, 0);
/// ```
pub fn error_throw<E: 'static, A: 'static>(_error: E) -> Eff<ErrorRow, A> {
    Eff::lazy(|| crate::cold_panic!("error_throw requires Error handler"))
}

/// Succeed with a value.
pub fn error_ok<E: 'static, A: 'static>(value: A) -> Eff<ErrorRow, A> {
    Eff::from_value(value)
}

// =============================================================================
// Concrete Error Implementation
// =============================================================================

/// A concrete error computation that can be run.
///
/// This is essentially `Result<A, E>` with monadic operations.
/// Uses `#[repr(transparent)]` to have zero overhead over Result.
#[must_use = "error computations do nothing unless run"]
#[repr(transparent)]
pub struct ErrorComputation<E, A> {
    /// The result.
    result: Result<A, E>,
}

impl<E, A> ErrorComputation<E, A> {
    /// Create a successful computation.
    #[inline(always)]
    pub fn ok(value: A) -> Self {
        ErrorComputation { result: Ok(value) }
    }

    /// Create a failed computation.
    #[inline(always)]
    pub fn err(error: E) -> Self {
        ErrorComputation { result: Err(error) }
    }

    /// Create from a Result.
    #[inline(always)]
    pub fn from_result(result: Result<A, E>) -> Self {
        ErrorComputation { result }
    }

    /// Run the computation.
    ///
    /// # Errors
    ///
    /// Returns the stored `Err(e)` when this computation was built from a
    /// failure (via [`err`](Self::err), a failing `from_result`, or a
    /// short-circuited chain). Running performs no work of its own — the
    /// representation is just the underlying `Result` — so no new errors
    /// arise here.
    #[inline(always)]
    pub fn run(self) -> Result<A, E> {
        self.result
    }

    /// Map over a successful result.
    #[inline(always)]
    pub fn map<B, F: FnOnce(A) -> B>(self, f: F) -> ErrorComputation<E, B> {
        ErrorComputation {
            result: self.result.map(f),
        }
    }

    /// Map over an error.
    #[inline(always)]
    pub fn map_err<E2, F: FnOnce(E) -> E2>(self, f: F) -> ErrorComputation<E2, A> {
        ErrorComputation {
            result: self.result.map_err(f),
        }
    }

    /// Chain two error computations.
    #[inline(always)]
    pub fn and_then<B, F: FnOnce(A) -> ErrorComputation<E, B>>(
        self,
        f: F,
    ) -> ErrorComputation<E, B> {
        match self.result {
            Ok(a) => f(a),
            Err(e) => ErrorComputation::err(e),
        }
    }

    /// Handle an error.
    #[inline(always)]
    pub fn or_else<F: FnOnce(E) -> ErrorComputation<E, A>>(self, f: F) -> ErrorComputation<E, A> {
        match self.result {
            Ok(a) => ErrorComputation::ok(a),
            Err(e) => f(e),
        }
    }

    /// Provide a default value on error.
    #[inline]
    pub fn unwrap_or(self, default: A) -> A {
        self.result.unwrap_or(default)
    }

    /// Provide a computed default on error.
    #[inline]
    pub fn unwrap_or_else<F: FnOnce(E) -> A>(self, f: F) -> A {
        self.result.unwrap_or_else(f)
    }

    /// Convert to Option, discarding the error.
    #[inline]
    pub fn ok_option(self) -> Option<A> {
        self.result.ok()
    }

    /// Convert to Option, discarding the success value.
    #[inline]
    pub fn err_option(self) -> Option<E> {
        self.result.err()
    }
}

impl<E, A> From<Result<A, E>> for ErrorComputation<E, A> {
    fn from(result: Result<A, E>) -> Self {
        ErrorComputation::from_result(result)
    }
}

impl<E, A> From<ErrorComputation<E, A>> for Result<A, E> {
    fn from(comp: ErrorComputation<E, A>) -> Self {
        comp.run()
    }
}

// =============================================================================
// Combinators
// =============================================================================

/// Try multiple computations, returning the first success.
pub fn first_success<E, A>(
    computations: alloc::vec::Vec<ErrorComputation<E, A>>,
) -> ErrorComputation<alloc::vec::Vec<E>, A> {
    // Pre-allocate error accumulator with worst-case capacity so that
    // the all-failure path never reallocates.
    let mut errors = alloc::vec::Vec::with_capacity(computations.len());

    for comp in computations {
        match comp.run() {
            Ok(a) => return ErrorComputation::ok(a),
            Err(e) => errors.push(e),
        }
    }

    ErrorComputation::err(errors)
}

/// Collect successes, failing if any computation fails.
pub fn sequence_results<E, A>(
    computations: alloc::vec::Vec<ErrorComputation<E, A>>,
) -> ErrorComputation<E, alloc::vec::Vec<A>> {
    // Pre-allocate result accumulator with the total count.
    let mut results = alloc::vec::Vec::with_capacity(computations.len());

    for comp in computations {
        match comp.run() {
            Ok(a) => results.push(a),
            Err(e) => return ErrorComputation::err(e),
        }
    }

    ErrorComputation::ok(results)
}

/// Partition computations into successes and failures.
pub fn partition_results<E, A>(
    computations: alloc::vec::Vec<ErrorComputation<E, A>>,
) -> (alloc::vec::Vec<A>, alloc::vec::Vec<E>) {
    // Pre-allocate both buckets conservatively at half of the total count
    // to avoid the most common case of repeated reallocation.
    let half = computations.len() / 2 + 1;
    let mut successes = alloc::vec::Vec::with_capacity(half);
    let mut failures = alloc::vec::Vec::with_capacity(half);

    for comp in computations {
        match comp.run() {
            Ok(a) => successes.push(a),
            Err(e) => failures.push(e),
        }
    }

    (successes, failures)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_ok() {
        let comp: ErrorComputation<&str, i32> = ErrorComputation::ok(42);
        assert_eq!(comp.run(), Ok(42));
    }

    #[test]
    fn test_error_err() {
        let comp: ErrorComputation<&str, i32> = ErrorComputation::err("oops");
        assert_eq!(comp.run(), Err("oops"));
    }

    #[test]
    fn test_error_map() {
        let comp: ErrorComputation<&str, i32> = ErrorComputation::ok(21);
        let mapped = comp.map(|x| x * 2);
        assert_eq!(mapped.run(), Ok(42));
    }

    #[test]
    fn test_error_map_err() {
        let comp: ErrorComputation<&str, i32> = ErrorComputation::err("oops");
        let mapped = comp.map_err(str::len);
        assert_eq!(mapped.run(), Err(4));
    }

    #[test]
    fn test_error_and_then_ok() {
        let comp: ErrorComputation<&str, i32> = ErrorComputation::ok(21);
        let chained = comp.and_then(|x| ErrorComputation::ok(x * 2));
        assert_eq!(chained.run(), Ok(42));
    }

    #[test]
    fn test_error_and_then_err() {
        let comp: ErrorComputation<&str, i32> = ErrorComputation::err("first");
        let chained = comp.and_then(|x| ErrorComputation::ok(x * 2));
        assert_eq!(chained.run(), Err("first"));
    }

    #[test]
    fn test_error_or_else() {
        let comp: ErrorComputation<&str, i32> = ErrorComputation::err("oops");
        let recovered = comp.or_else(|_| ErrorComputation::ok(42));
        assert_eq!(recovered.run(), Ok(42));
    }

    #[test]
    fn test_error_unwrap_or() {
        let err_comp: ErrorComputation<&str, i32> = ErrorComputation::err("oops");
        assert_eq!(err_comp.unwrap_or(42), 42);

        let ok_comp: ErrorComputation<&str, i32> = ErrorComputation::ok(100);
        assert_eq!(ok_comp.unwrap_or(42), 100);
    }

    #[test]
    fn test_first_success() {
        let comps = alloc::vec![
            ErrorComputation::err("a"),
            ErrorComputation::ok(42),
            ErrorComputation::err("b"),
        ];
        let result = first_success(comps);
        assert_eq!(result.run(), Ok(42));
    }

    #[test]
    fn test_first_success_all_fail() {
        let comps: alloc::vec::Vec<ErrorComputation<&str, i32>> =
            alloc::vec![ErrorComputation::err("a"), ErrorComputation::err("b"),];
        let result = first_success(comps);
        assert_eq!(result.run(), Err(alloc::vec!["a", "b"]));
    }

    #[test]
    fn test_sequence_results() {
        let comps: alloc::vec::Vec<ErrorComputation<&str, i32>> = alloc::vec![
            ErrorComputation::ok(1),
            ErrorComputation::ok(2),
            ErrorComputation::ok(3),
        ];
        let result = sequence_results(comps);
        assert_eq!(result.run(), Ok(alloc::vec![1, 2, 3]));
    }

    #[test]
    fn test_partition_results() {
        let comps = alloc::vec![
            ErrorComputation::ok(1),
            ErrorComputation::err("a"),
            ErrorComputation::ok(2),
        ];
        let (ok, err) = partition_results(comps);
        assert_eq!(ok, alloc::vec![1, 2]);
        assert_eq!(err, alloc::vec!["a"]);
    }
}
