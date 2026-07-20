//! `MonadError` type class - error handling in monadic context.
//!
//! > *"Errare humanum est, sed in errando perseverare diabolicum."*
//! > — To err is human, but to persist in error is diabolical. (Seneca)
//!
//! The `MonadError` trait extends `Monad` with error-specific operations,
//! enabling structured error handling without breaking the computation chain.
//!
//! # Laws (*Leges Erroris*)
//!
//! 1. **Left Catch**: `throw(e).catch(h) == h(e)`
//! 2. **Right Catch**: `m.catch(throw) == m`
//! 3. **Associativity**: `m.catch(h1).catch(h2) == m.catch(|e| h1(e).catch(h2))`
//!
//! # Example
//!
//! ```rust
//! use ordofp_core::typeclasses::monad_error::MonadError;
//!
//! // Create an error
//! let err: Result<i32, &str> = Result::throw("not found");
//! assert_eq!(err, Err("not found"));
//!
//! // Catch and recover
//! let recovered = err.catch(|_| Ok(0));
//! assert_eq!(recovered, Ok(0));
//!
//! // Successful values pass through
//! let ok: Result<i32, &str> = Ok(42);
//! let still_ok = ok.catch(|_| Ok(0));
//! assert_eq!(still_ok, Ok(42));
//! ```

/// A Monad with error handling capabilities.
///
/// `MonadError` provides `throw` to raise errors and `catch` to handle them
/// within monadic composition.
///
/// # Type Parameters
///
/// * `E` - The error type that can be thrown and caught
///
/// # Laws
///
/// 1. **Left Catch**: `throw(e).catch(h) == h(e)`
/// 2. **Right Catch**: `m.catch(throw) == m`
/// 3. **Associativity**: `m.catch(h1).catch(h2) == m.catch(|e| h1(e).catch(h2))`
pub trait MonadError<E> {
    /// The success type contained in the monad.
    type Inner;

    /// Creates an error value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::typeclasses::monad_error::MonadError;
    ///
    /// let err: Result<i32, &str> = Result::throw("error");
    /// assert_eq!(err, Err("error"));
    /// ```
    fn throw(error: E) -> Self;

    /// Catches an error and attempts recovery.
    ///
    /// If the monad is in an error state, applies the handler function.
    /// Otherwise, returns the successful value unchanged.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::typeclasses::monad_error::MonadError;
    ///
    /// let err: Result<i32, &str> = Err("not found");
    /// let recovered = err.catch(|e| {
    ///     if *e == "not found" { Ok(0) } else { Err(*e) }
    /// });
    /// assert_eq!(recovered, Ok(0));
    /// ```
    fn catch<F>(self, handler: F) -> Self
    where
        F: FnOnce(&E) -> Self;

    /// Catches an error, taking ownership of the error value.
    ///
    /// More efficient than `catch` when the error doesn't need to be preserved.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::typeclasses::monad_error::MonadError;
    ///
    /// let err: Result<i32, String> = Err("error".to_string());
    /// let recovered = err.catch_owned(|e| Ok(e.len() as i32));
    /// assert_eq!(recovered, Ok(5));
    /// ```
    fn catch_owned<F>(self, handler: F) -> Self
    where
        F: FnOnce(E) -> Self;

    /// Attempts a computation, returning a default on error.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::typeclasses::monad_error::MonadError;
    ///
    /// let err: Result<i32, &str> = Err("failed");
    /// let with_default = err.or_default(42);
    /// assert_eq!(with_default, Ok(42));
    /// ```
    #[inline]
    fn or_default(self, default: Self::Inner) -> Self
    where
        Self: Sized,
    {
        self.catch(|_| Self::pure(default))
    }

    /// Creates a successful value.
    fn pure(value: Self::Inner) -> Self;

    /// Adapts the error type using a mapping function.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::typeclasses::monad_error::MonadError;
    ///
    /// let err: Result<i32, i32> = Err(404);
    /// let mapped: Result<i32, String> = err.adapt_error(|e| format!("Error: {}", e));
    /// assert_eq!(mapped, Err("Error: 404".to_string()));
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `Err(f(e))` when `self` carries an error `e`; success
    /// values pass through untouched and `f` is never called for them.
    fn adapt_error<F, E2>(self, f: F) -> Result<Self::Inner, E2>
    where
        Self: Sized,
        F: FnOnce(E) -> E2;
}

// ============================================================================
// Implementation for Result
// ============================================================================

impl<A, E> MonadError<E> for Result<A, E> {
    type Inner = A;

    #[inline]
    fn throw(error: E) -> Self {
        Err(error)
    }

    #[inline]
    fn catch<F>(self, handler: F) -> Self
    where
        F: FnOnce(&E) -> Self,
    {
        match &self {
            Ok(_) => self,
            Err(e) => handler(e),
        }
    }

    #[inline]
    fn catch_owned<F>(self, handler: F) -> Self
    where
        F: FnOnce(E) -> Self,
    {
        match self {
            Ok(a) => Ok(a),
            Err(e) => handler(e),
        }
    }

    #[inline]
    fn pure(value: A) -> Self {
        Ok(value)
    }

    #[inline]
    fn adapt_error<F, E2>(self, f: F) -> Result<A, E2>
    where
        F: FnOnce(E) -> E2,
    {
        self.map_err(f)
    }
}

// ============================================================================
// Convenience functions
// ============================================================================

/// Creates an error in a `MonadError` context.
///
/// # Example
///
/// ```rust
/// use ordofp_core::typeclasses::monad_error::throw;
///
/// let err: Result<i32, &str> = throw("error");
/// assert_eq!(err, Err("error"));
/// ```
#[inline]
pub fn throw<M, E>(error: E) -> M
where
    M: MonadError<E>,
{
    M::throw(error)
}

/// Ensures a condition holds, throwing an error if not.
///
/// # Example
///
/// ```rust
/// use ordofp_core::typeclasses::monad_error::ensure;
///
/// let valid: Result<(), &str> = ensure(true, "condition failed");
/// assert_eq!(valid, Ok(()));
///
/// let invalid: Result<(), &str> = ensure(false, "condition failed");
/// assert_eq!(invalid, Err("condition failed"));
/// ```
///
/// # Errors
///
/// Returns `Err(error)` when `condition` is `false`. The error value is
/// constructed eagerly — use [`ensure_lazy`] when it is expensive.
#[inline]
pub fn ensure<E>(condition: bool, error: E) -> Result<(), E> {
    if condition { Ok(()) } else { Err(error) }
}

/// Ensures a condition holds, throwing an error if not (lazy error).
///
/// # Example
///
/// ```rust
/// use ordofp_core::typeclasses::monad_error::ensure_lazy;
///
/// let valid: Result<(), String> = ensure_lazy(true, || "expensive error".to_string());
/// assert_eq!(valid, Ok(()));
/// ```
///
/// # Errors
///
/// Returns `Err(error())` when `condition` is `false`; the closure is
/// only invoked in that case.
#[inline]
pub fn ensure_lazy<E, F>(condition: bool, error: F) -> Result<(), E>
where
    F: FnOnce() -> E,
{
    if condition { Ok(()) } else { Err(error()) }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_result_throw() {
        let err: Result<i32, &str> = Result::throw("error");
        assert_eq!(err, Err("error"));
    }

    #[test]
    fn test_result_catch_err() {
        let err: Result<i32, &str> = Err("not found");
        let recovered = err.catch(|_| Ok(42));
        assert_eq!(recovered, Ok(42));
    }

    #[test]
    fn test_result_catch_ok() {
        let ok: Result<i32, &str> = Ok(42);
        let still_ok = ok.catch(|_| Ok(0));
        assert_eq!(still_ok, Ok(42));
    }

    #[test]
    fn test_result_catch_owned() {
        let err: Result<i32, i32> = Err(5);
        let recovered = err.catch_owned(|e| Ok(e * 2));
        assert_eq!(recovered, Ok(10));
    }

    #[test]
    fn test_result_or_default() {
        let err: Result<i32, &str> = Err("error");
        let with_default = err.or_default(100);
        assert_eq!(with_default, Ok(100));

        let ok: Result<i32, &str> = Ok(42);
        let still_ok = ok.or_default(100);
        assert_eq!(still_ok, Ok(42));
    }

    #[test]
    fn test_result_adapt_error() {
        let err: Result<i32, i32> = Err(404);
        let mapped: Result<i32, &str> = err.adapt_error(|_| "not found");
        assert_eq!(mapped, Err("not found"));

        let ok: Result<i32, i32> = Ok(42);
        let still_ok: Result<i32, &str> = ok.adapt_error(|_| "error");
        assert_eq!(still_ok, Ok(42));
    }

    #[test]
    fn test_throw_function() {
        let err: Result<i32, &str> = throw("error");
        assert_eq!(err, Err("error"));
    }

    #[test]
    fn test_ensure() {
        let valid: Result<(), &str> = ensure(5 > 3, "should not happen");
        assert_eq!(valid, Ok(()));

        let invalid: Result<(), &str> = ensure(5 < 3, "5 is not less than 3");
        assert_eq!(invalid, Err("5 is not less than 3"));
    }

    // MonadError Laws
    #[test]
    fn test_left_catch_law() {
        // throw(e).catch(h) == h(e)
        let e = "error";
        let handler = |_: &&str| Ok::<i32, &str>(42);

        let left: Result<i32, &str> = Result::throw(e).catch(handler);
        let right: Result<i32, &str> = handler(&e);

        assert_eq!(left, right);
    }

    #[test]
    fn test_right_catch_law() {
        // m.catch(throw) == m
        let m: Result<i32, &str> = Ok(42);
        let caught = m.catch(|e| Result::throw(*e));
        assert_eq!(caught, m);

        let err: Result<i32, &str> = Err("error");
        let caught_err = err.catch(|e| Result::throw(*e));
        assert_eq!(caught_err, err);
    }

    #[test]
    fn test_associativity_catch_law() {
        // m.catch(h1).catch(h2) == m.catch(|e| h1(e).catch(h2))
        let m: Result<i32, &str> = Err("error");
        let h1 = |_: &&str| Err::<i32, &str>("h1 error");
        let h2 = |_: &&str| Ok::<i32, &str>(100);

        let left = m.catch(h1).catch(h2);
        let right = m.catch(|e| h1(e).catch(h2));

        assert_eq!(left, right);
    }
}
