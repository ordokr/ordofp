//! Easy Error Handling
//!
//! Simplified error handling patterns that hide the complexity
//! of effect-based error management.
//!
//! # Example
//!
//! ```rust
//! use ordofp_core::easy::*;
//!
//! let result: Result<i32, String> = run_with_error(|| {
//!     let x: i32 = "42".parse().map_err(|_| "parse error".to_string())?;
//!     Ok(x * 2)
//! });
//! assert_eq!(result, Ok(84));
//! ```

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

// =============================================================================
// Basic Error Operations
// =============================================================================

/// Run a computation that may fail.
///
/// # Errors
///
/// Propagates whatever `Err` the `computation` closure itself returns;
/// this function adds no failure modes of its own.
///
/// # Example
///
/// ```rust
/// use ordofp_core::easy::run_with_error;
///
/// let result: Result<i32, &str> = run_with_error(|| Ok(42));
/// assert_eq!(result, Ok(42));
/// ```
pub fn run_with_error<A, E, F>(computation: F) -> Result<A, E>
where
    F: FnOnce() -> Result<A, E>,
{
    computation()
}

/// Run a computation, converting panics to errors.
///
/// Note: This requires std and `catch_unwind` support.
///
/// # Errors
///
/// Returns `Err` if the `computation` panics. The panic payload is
/// rendered as the error message when it is a `&str` or `String`;
/// any other payload becomes `"Unknown panic"`.
#[cfg(feature = "std")]
pub fn run_catching<A, F>(computation: F) -> Result<A, String>
where
    F: FnOnce() -> A + std::panic::UnwindSafe,
{
    std::panic::catch_unwind(computation).map_err(|e| {
        if let Some(s) = e.downcast_ref::<&str>() {
            String::from(*s)
        } else if let Some(s) = e.downcast_ref::<String>() {
            s.clone()
        } else {
            String::from("Unknown panic")
        }
    })
}

/// Run a fallible computation, providing a default on error.
///
/// # Example
///
/// ```rust
/// use ordofp_core::easy::run_or_default;
///
/// let result = run_or_default(|| Err("oops"), 42);
/// assert_eq!(result, 42);
/// ```
pub fn run_or_default<A, E, F>(computation: F, default: A) -> A
where
    F: FnOnce() -> Result<A, E>,
{
    computation().unwrap_or(default)
}

/// Run a fallible computation, using a fallback on error.
///
/// # Example
///
/// ```rust
/// use ordofp_core::easy::run_or_else;
///
/// let result = run_or_else(|| Err("oops"), |_| 42);
/// assert_eq!(result, 42);
/// ```
pub fn run_or_else<A, E, F, G>(computation: F, fallback: G) -> A
where
    F: FnOnce() -> Result<A, E>,
    G: FnOnce(E) -> A,
{
    computation().unwrap_or_else(fallback)
}

// =============================================================================
// Error Composition
// =============================================================================

/// Sequence multiple fallible operations.
///
/// # Errors
///
/// Returns the first `Err` produced inside the `computation` closure
/// (typically via the `?` operator); no failure modes are added here.
///
/// # Example
///
/// ```rust
/// use ordofp_core::easy::try_all;
///
/// let step1 = || -> Result<i32, &str> { Ok(1) };
/// let step2 = |a: i32| -> Result<i32, &str> { Ok(a + 1) };
///
/// let result = try_all(|| -> Result<i32, &str> {
///     let a = step1()?;
///     let b = step2(a)?;
///     Ok(b)
/// });
/// assert_eq!(result, Ok(2));
/// ```
pub fn try_all<A, E, F>(computation: F) -> Result<A, E>
where
    F: FnOnce() -> Result<A, E>,
{
    computation()
}

/// Chain two fallible operations.
///
/// # Errors
///
/// Returns the error of `first` if it fails; otherwise `second` is run
/// on the success value and its error, if any, is returned.
///
/// # Example
///
/// ```rust
/// use ordofp_core::easy::try_chain;
///
/// let result: Result<i32, &str> = try_chain(
///     || Ok(21),
///     |x| Ok(x * 2),
/// );
/// assert_eq!(result, Ok(42));
/// ```
pub fn try_chain<A, B, E, F1, F2>(first: F1, second: F2) -> Result<B, E>
where
    F1: FnOnce() -> Result<A, E>,
    F2: FnOnce(A) -> Result<B, E>,
{
    first().and_then(second)
}

/// Chain three fallible operations.
///
/// # Errors
///
/// Short-circuits on the first failing step: the error of `first`,
/// else of `second`, else of `third`. Later steps are not run once
/// an earlier one has failed.
pub fn try_chain3<A, B, C, E, F1, F2, F3>(first: F1, second: F2, third: F3) -> Result<C, E>
where
    F1: FnOnce() -> Result<A, E>,
    F2: FnOnce(A) -> Result<B, E>,
    F3: FnOnce(B) -> Result<C, E>,
{
    first().and_then(second).and_then(third)
}

/// Run multiple fallible operations, collecting all results.
///
/// # Errors
///
/// Returns the first `Err` encountered while running `operations` in
/// order; operations after the failing one are not run. Use
/// [`partition_results`] to keep going and gather every error instead.
///
/// # Example
///
/// ```rust
/// use ordofp_core::easy::try_collect;
///
/// let ops: [fn() -> Result<i32, &'static str>; 3] = [|| Ok(1), || Ok(2), || Ok(3)];
/// let results = try_collect(&ops);
/// assert_eq!(results, Ok(vec![1, 2, 3]));
/// ```
pub fn try_collect<A, E, F>(operations: &[F]) -> Result<Vec<A>, E>
where
    F: Fn() -> Result<A, E>,
{
    operations.iter().map(|f| f()).collect()
}

/// Run two fallible operations and combine results.
///
/// # Errors
///
/// Returns the error of `first` if it fails (in which case `second`
/// is never run), otherwise the error of `second` if that fails.
///
/// # Example
///
/// ```rust
/// use ordofp_core::easy::try_both;
///
/// let result: Result<(i32, i32), &str> = try_both(|| Ok(1), || Ok(2));
/// assert_eq!(result, Ok((1, 2)));
/// ```
pub fn try_both<A, B, E, F1, F2>(first: F1, second: F2) -> Result<(A, B), E>
where
    F1: FnOnce() -> Result<A, E>,
    F2: FnOnce() -> Result<B, E>,
{
    Ok((first()?, second()?))
}

// =============================================================================
// Error Recovery
// =============================================================================

/// Retry a fallible operation up to N times.
///
/// # Errors
///
/// Returns the error from the final attempt when all `max_attempts`
/// invocations of `operation` fail; earlier errors are discarded.
///
/// # Panics
///
/// Panics if `max_attempts` is `0` — there is then no error to return.
///
/// # Example
///
/// ```rust
/// use ordofp_core::easy::retry;
///
/// let mut attempts = 0;
/// let result = retry(3, || {
///     attempts += 1;
///     if attempts < 3 { Err("not yet") } else { Ok(42) }
/// });
/// assert_eq!(result, Ok(42));
/// ```
pub fn retry<A, E, F>(max_attempts: usize, mut operation: F) -> Result<A, E>
where
    F: FnMut() -> Result<A, E>,
{
    let mut last_error = None;
    for _ in 0..max_attempts {
        match operation() {
            Ok(a) => return Ok(a),
            Err(e) => last_error = Some(e),
        }
    }
    Err(last_error.expect("retry: max_attempts must be > 0"))
}

/// Retry with a condition for retrying.
///
/// # Errors
///
/// Returns an error immediately if `should_retry` rejects it (no
/// further attempts are made), or the final attempt's error once all
/// `max_attempts` invocations of `operation` have failed.
///
/// # Panics
///
/// Panics if `max_attempts` is `0` — there is then no error to return.
pub fn retry_if<A, E, F, P>(max_attempts: usize, mut operation: F, should_retry: P) -> Result<A, E>
where
    F: FnMut() -> Result<A, E>,
    P: Fn(&E) -> bool,
{
    let mut last_error = None;
    for _ in 0..max_attempts {
        match operation() {
            Ok(a) => return Ok(a),
            Err(e) => {
                if !should_retry(&e) {
                    return Err(e);
                }
                last_error = Some(e);
            }
        }
    }
    Err(last_error.expect("retry_if: max_attempts must be > 0"))
}

/// Try the first operation, falling back to the second on error.
///
/// # Errors
///
/// Returns the error of `second` when both operations fail; the error
/// from `first` is discarded once the fallback is attempted.
///
/// # Example
///
/// ```rust
/// use ordofp_core::easy::fallback;
///
/// let result: Result<i32, &str> = fallback(
///     || Err("first failed"),
///     || Ok(42),
/// );
/// assert_eq!(result, Ok(42));
/// ```
pub fn fallback<A, E, F1, F2>(first: F1, second: F2) -> Result<A, E>
where
    F1: FnOnce() -> Result<A, E>,
    F2: FnOnce() -> Result<A, E>,
{
    first().or_else(|_| second())
}

/// Try multiple fallback options in order.
///
/// # Errors
///
/// Returns the error of the last option when every option in the
/// slice fails; errors from earlier options are discarded.
///
/// # Panics
///
/// Panics if `options` is empty — there is then no error to return.
pub fn fallback_chain<A, E, F>(options: &[F]) -> Result<A, E>
where
    F: Fn() -> Result<A, E>,
{
    let mut last_error = None;
    for option in options {
        match option() {
            Ok(a) => return Ok(a),
            Err(e) => last_error = Some(e),
        }
    }
    Err(last_error.expect("fallback_chain: options must not be empty"))
}

// =============================================================================
// Error Accumulation
// =============================================================================

/// Accumulate errors from multiple validations.
///
/// # Errors
///
/// Every predicate is run against `value`; if any fail, returns a
/// `Vec` containing the error of each failed validation, in slice
/// order. This accumulates rather than short-circuiting.
///
/// # Example
///
/// ```rust
/// use ordofp_core::easy::validate_all_errors;
///
/// let result = validate_all_errors(42, &[
///     (|x| *x > 0, "must be positive"),
///     (|x: &i32| *x < 100, "must be < 100"),
/// ]);
/// assert_eq!(result, Ok(42));
/// ```
pub fn validate_all_errors<T, E: Clone>(
    value: T,
    validations: &[crate::easy::Validation<T, E>],
) -> Result<T, Vec<E>> {
    let errors: Vec<E> = validations
        .iter()
        .filter(|(pred, _)| !pred(&value))
        .map(|(_, err)| err.clone())
        .collect();

    if errors.is_empty() {
        Ok(value)
    } else {
        Err(errors)
    }
}

/// Collect all errors from multiple operations.
///
/// Returns all successes and all errors.
pub fn partition_results<A, E, F>(operations: &[F]) -> (Vec<A>, Vec<E>)
where
    F: Fn() -> Result<A, E>,
{
    let mut successes = Vec::with_capacity(operations.len());
    let mut errors = Vec::new();

    for op in operations {
        match op() {
            Ok(a) => successes.push(a),
            Err(e) => errors.push(e),
        }
    }

    (successes, errors)
}

// =============================================================================
// Error Types
// =============================================================================

/// A simple error with a message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleError {
    message: String,
}

impl SimpleError {
    /// Create a new simple error.
    pub fn new(message: impl Into<String>) -> Self {
        SimpleError {
            message: message.into(),
        }
    }

    /// Get the error message.
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for SimpleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

/// Create a simple error.
pub fn error(message: impl Into<String>) -> SimpleError {
    SimpleError::new(message)
}

/// A multi-error that collects multiple errors.
#[derive(Debug, Clone)]
pub struct MultiError<E> {
    errors: Vec<E>,
}

impl<E> MultiError<E> {
    /// Create an empty multi-error.
    pub fn new() -> Self {
        MultiError { errors: Vec::new() }
    }

    /// Create from a single error.
    pub fn single(error: E) -> Self {
        MultiError {
            errors: alloc::vec![error],
        }
    }

    /// Create from multiple errors.
    pub fn many(errors: Vec<E>) -> Self {
        MultiError { errors }
    }

    /// Add an error.
    pub fn push(&mut self, error: E) {
        self.errors.push(error);
    }

    /// Check if there are any errors.
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    /// Get the number of errors.
    pub fn len(&self) -> usize {
        self.errors.len()
    }

    /// Get all errors.
    pub fn errors(&self) -> &[E] {
        &self.errors
    }

    /// Convert to Result (Ok if empty, Err otherwise).
    ///
    /// # Errors
    ///
    /// Returns `Err(self)` — the accumulated errors — when at least
    /// one error has been collected; `value` is discarded in that case.
    pub fn into_result<A>(self, value: A) -> Result<A, Self> {
        if self.is_empty() {
            Ok(value)
        } else {
            Err(self)
        }
    }
}

impl<E> Default for MultiError<E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<E: fmt::Display> fmt::Display for MultiError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Multiple errors ({}):", self.errors.len())?;
        for (i, e) in self.errors.iter().enumerate() {
            write!(f, "\n  {}: {}", i + 1, e)?;
        }
        Ok(())
    }
}

// =============================================================================
// Result Extension Helpers
// =============================================================================

/// Convert Option to Result with an error message.
///
/// # Errors
///
/// Returns a [`SimpleError`] carrying `error` when `option` is `None`.
pub fn require<T>(option: Option<T>, error: impl Into<String>) -> Result<T, SimpleError> {
    option.ok_or_else(|| SimpleError::new(error))
}

/// Convert bool to Result.
///
/// # Errors
///
/// Returns a [`SimpleError`] carrying `error` when `condition` is `false`.
pub fn require_true(condition: bool, error: impl Into<String>) -> Result<(), SimpleError> {
    if condition {
        Ok(())
    } else {
        Err(SimpleError::new(error))
    }
}

/// Ensure a condition holds, returning the value if true.
///
/// # Errors
///
/// Returns a [`SimpleError`] carrying `error` when `condition(&value)`
/// is `false`; the value is dropped in that case.
pub fn ensure<T>(
    value: T,
    condition: impl FnOnce(&T) -> bool,
    error: impl Into<String>,
) -> Result<T, SimpleError> {
    if condition(&value) {
        Ok(value)
    } else {
        Err(SimpleError::new(error))
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_with_error() {
        let result: Result<i32, &str> = run_with_error(|| Ok(42));
        assert_eq!(result, Ok(42));

        let result: Result<i32, &str> = run_with_error(|| Err("oops"));
        assert!(result.is_err());
    }

    #[test]
    fn test_run_or_default() {
        let result = run_or_default(|| Err::<i32, _>("oops"), 42);
        assert_eq!(result, 42);

        let result = run_or_default(|| Ok::<_, &str>(10), 42);
        assert_eq!(result, 10);
    }

    #[test]
    fn test_try_chain() {
        let result = try_chain(|| Ok::<_, &str>(21), |x| Ok(x * 2));
        assert_eq!(result, Ok(42));
    }

    #[test]
    fn test_try_both() {
        let result = try_both(|| Ok::<_, &str>(1), || Ok(2));
        assert_eq!(result, Ok((1, 2)));
    }

    #[test]
    fn test_retry() {
        let mut attempts = 0;
        let result = retry(3, || {
            attempts += 1;
            if attempts < 3 { Err("not yet") } else { Ok(42) }
        });
        assert_eq!(result, Ok(42));
        assert_eq!(attempts, 3);
    }

    #[test]
    fn test_fallback() {
        let result: Result<i32, &str> = fallback(|| Err("first failed"), || Ok(42));
        assert_eq!(result, Ok(42));
    }

    #[test]
    fn test_validate_all_errors() {
        let result = validate_all_errors(
            42,
            &[
                (|x| *x > 0, "must be positive"),
                (|x: &i32| *x < 100, "must be < 100"),
            ],
        );
        assert_eq!(result, Ok(42));

        let result = validate_all_errors(
            -5,
            &[
                (|x| *x > 0, "must be positive"),
                (|x: &i32| *x < 100, "must be < 100"),
            ],
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().len(), 1);
    }

    #[test]
    fn test_partition_results() {
        let ops: Vec<fn() -> Result<i32, &'static str>> =
            alloc::vec![|| Ok(1), || Err("error"), || Ok(2),];
        let (successes, errors) = partition_results(&ops);
        assert_eq!(successes, alloc::vec![1, 2]);
        assert_eq!(errors, alloc::vec!["error"]);
    }

    #[test]
    fn test_simple_error() {
        let err = error("something went wrong");
        assert_eq!(err.message(), "something went wrong");
    }

    #[test]
    fn test_multi_error() {
        let mut multi = MultiError::new();
        multi.push("error 1");
        multi.push("error 2");

        assert_eq!(multi.len(), 2);
        assert!(!multi.is_empty());
    }

    #[test]
    fn test_require() {
        let result = require(Some(42), "value required");
        assert_eq!(result, Ok(42));

        let result = require::<i32>(None, "value required");
        assert!(result.is_err());
    }

    #[test]
    fn test_ensure() {
        let result = ensure(42, |x| *x > 0, "must be positive");
        assert_eq!(result, Ok(42));

        let result = ensure(-1, |x| *x > 0, "must be positive");
        assert!(result.is_err());
    }
}
