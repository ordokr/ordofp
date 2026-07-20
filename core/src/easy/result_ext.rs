//! Result Extension Traits
//!
//! Additional methods for working with Result types in a functional style.
//!
//! # Example
//!
//! ```rust
//! use ordofp_core::easy::*;
//!
//! let result: Result<i32, &str> = Ok(42);
//!
//! // Tap into successful values
//! let tapped = result.tap(|x| println!("Got: {}", x));
//!
//! // Convert to Option
//! let opt = result.ok_or_none();
//! ```

use alloc::vec::Vec;

// =============================================================================
// Result Extension Trait
// =============================================================================

/// Extension methods for Result types.
pub trait ResultExt<T, E> {
    /// Execute a side-effect on the success value without consuming it.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::easy::ResultExt;
    ///
    /// let result: Result<i32, &str> = Ok(42);
    /// let _ = result.tap(|x| println!("Value: {}", x));
    /// ```
    fn tap<F: FnOnce(&T)>(self, f: F) -> Self;

    /// Execute a side-effect on the error value without consuming it.
    fn tap_err<F: FnOnce(&E)>(self, f: F) -> Self;

    /// Convert Ok to Some, Err to None (alias for `ok()`).
    fn ok_or_none(self) -> Option<T>;

    /// Convert Err to Some, Ok to None (alias for `err()`).
    fn err_or_none(self) -> Option<E>;

    /// Swap Ok and Err.
    ///
    /// # Errors
    ///
    /// Returns `Err` exactly when `self` was `Ok`: the success value
    /// becomes the error and vice versa.
    fn swap(self) -> Result<E, T>;

    /// Apply a function if Ok, otherwise return the error unchanged.
    ///
    /// # Errors
    ///
    /// Propagates the existing error when `self` is `Err`; otherwise
    /// returns whatever error `f` produces from the success value.
    fn and_try<U, F>(self, f: F) -> Result<U, E>
    where
        F: FnOnce(T) -> Result<U, E>;

    /// Convert Result<Result<T, E>, E> to Result<T, E>.
    ///
    /// # Errors
    ///
    /// Returns the outer error when `self` is `Err`, or the inner error
    /// when the contained value converts into an `Err`.
    fn flatten_result(self) -> Result<T, E>
    where
        T: Into<Result<T, E>>;

    /// Get the value or a default.
    fn unwrap_or_default_with<F: FnOnce() -> T>(self, f: F) -> T;
}

impl<T, E> ResultExt<T, E> for Result<T, E> {
    #[inline]
    fn tap<F: FnOnce(&T)>(self, f: F) -> Self {
        if let Ok(ref value) = self {
            f(value);
        }
        self
    }

    #[inline]
    fn tap_err<F: FnOnce(&E)>(self, f: F) -> Self {
        if let Err(ref err) = self {
            f(err);
        }
        self
    }

    #[inline]
    fn ok_or_none(self) -> Option<T> {
        self.ok()
    }

    #[inline]
    fn err_or_none(self) -> Option<E> {
        self.err()
    }

    #[inline]
    fn swap(self) -> Result<E, T> {
        match self {
            Ok(t) => Err(t),
            Err(e) => Ok(e),
        }
    }

    #[inline]
    fn and_try<U, F>(self, f: F) -> Result<U, E>
    where
        F: FnOnce(T) -> Result<U, E>,
    {
        self.and_then(f)
    }

    #[inline]
    fn flatten_result(self) -> Result<T, E>
    where
        T: Into<Result<T, E>>,
    {
        self.and_then(core::convert::Into::into)
    }

    #[inline]
    fn unwrap_or_default_with<F: FnOnce() -> T>(self, f: F) -> T {
        self.unwrap_or_else(|_| f())
    }
}

// =============================================================================
// Option Extension Trait
// =============================================================================

/// Extension methods for Option types.
pub trait OptionExt<T> {
    /// Execute a side-effect on the value without consuming it.
    fn tap<F: FnOnce(&T)>(self, f: F) -> Self;

    /// Execute a side-effect if None.
    fn tap_none<F: FnOnce()>(self, f: F) -> Self;

    /// Convert to Result with a default error.
    ///
    /// # Errors
    ///
    /// Returns `Err(err)` when the option is `None`.
    fn ok_or_err<E>(self, err: E) -> Result<T, E>;

    /// Convert to Result with a lazily computed error.
    ///
    /// # Errors
    ///
    /// Returns `Err(f())` when the option is `None`; `f` is only
    /// invoked in that case.
    fn ok_or_else_err<E, F: FnOnce() -> E>(self, f: F) -> Result<T, E>;

    /// Filter the option with a predicate.
    fn filter_with<P: FnOnce(&T) -> bool>(self, predicate: P) -> Self;

    /// Get value or compute default.
    fn unwrap_or_compute<F: FnOnce() -> T>(self, f: F) -> T;

    /// Zip with another option.
    fn zip_with<U, R, F: FnOnce(T, U) -> R>(self, other: Option<U>, f: F) -> Option<R>;
}

impl<T> OptionExt<T> for Option<T> {
    #[inline]
    fn tap<F: FnOnce(&T)>(self, f: F) -> Self {
        if let Some(ref value) = self {
            f(value);
        }
        self
    }

    #[inline]
    fn tap_none<F: FnOnce()>(self, f: F) -> Self {
        if self.is_none() {
            f();
        }
        self
    }

    #[inline]
    fn ok_or_err<E>(self, err: E) -> Result<T, E> {
        self.ok_or(err)
    }

    #[inline]
    fn ok_or_else_err<E, F: FnOnce() -> E>(self, f: F) -> Result<T, E> {
        self.ok_or_else(f)
    }

    #[inline]
    fn filter_with<P: FnOnce(&T) -> bool>(self, predicate: P) -> Self {
        match self {
            Some(ref value) if predicate(value) => self,
            _ => None,
        }
    }

    #[inline]
    fn unwrap_or_compute<F: FnOnce() -> T>(self, f: F) -> T {
        self.unwrap_or_else(f)
    }

    #[inline]
    fn zip_with<U, R, F: FnOnce(T, U) -> R>(self, other: Option<U>, f: F) -> Option<R> {
        match (self, other) {
            (Some(a), Some(b)) => Some(f(a, b)),
            _ => None,
        }
    }
}

// =============================================================================
// Conversion Helpers
// =============================================================================

/// Convert a boolean to a Result.
///
/// # Errors
///
/// Returns `Err(err_value)` when `condition` is `false`; `ok_value` is
/// dropped in that case. Both values are eagerly constructed — use
/// [`bool_to_result_with`] when either side is expensive.
///
/// # Example
///
/// ```rust
/// use ordofp_core::easy::bool_to_result;
///
/// let result = bool_to_result(true, 42, "error");
/// assert_eq!(result, Ok(42));
/// ```
pub fn bool_to_result<T, E>(condition: bool, ok_value: T, err_value: E) -> Result<T, E> {
    if condition {
        Ok(ok_value)
    } else {
        Err(err_value)
    }
}

/// Convert a boolean to a Result with lazy evaluation.
///
/// # Errors
///
/// Returns `Err(err())` when `condition` is `false`; only the chosen
/// branch's closure is invoked.
pub fn bool_to_result_with<T, E, F, G>(condition: bool, ok: F, err: G) -> Result<T, E>
where
    F: FnOnce() -> T,
    G: FnOnce() -> E,
{
    if condition { Ok(ok()) } else { Err(err()) }
}

/// Convert a boolean to an Option.
pub fn bool_to_option<T>(condition: bool, value: T) -> Option<T> {
    if condition { Some(value) } else { None }
}

/// Convert a boolean to an Option with lazy evaluation.
pub fn bool_to_option_with<T, F: FnOnce() -> T>(condition: bool, f: F) -> Option<T> {
    if condition { Some(f()) } else { None }
}

// =============================================================================
// Collection Helpers
// =============================================================================

/// Collect results, stopping at first error.
///
/// # Errors
///
/// Returns the first `Err` yielded by the iterator; items after it are
/// not consumed. Use [`partition_results_vec`] to gather every error.
///
/// # Example
///
/// ```rust
/// use ordofp_core::easy::collect_results;
///
/// let results: Vec<Result<i32, &str>> = vec![Ok(1), Ok(2), Ok(3)];
/// let collected = collect_results(results);
/// assert_eq!(collected, Ok(vec![1, 2, 3]));
/// ```
pub fn collect_results<T, E, I>(iter: I) -> Result<Vec<T>, E>
where
    I: IntoIterator<Item = Result<T, E>>,
{
    iter.into_iter().collect()
}

/// Collect results, gathering all errors.
///
/// # Example
///
/// ```rust
/// use ordofp_core::easy::partition_results_vec;
///
/// let results: Vec<Result<i32, &str>> = vec![Ok(1), Err("a"), Ok(2), Err("b")];
/// let (successes, errors) = partition_results_vec(results);
/// assert_eq!(successes, vec![1, 2]);
/// assert_eq!(errors, vec!["a", "b"]);
/// ```
pub fn partition_results_vec<T, E, I>(iter: I) -> (Vec<T>, Vec<E>)
where
    I: IntoIterator<Item = Result<T, E>>,
{
    let mut successes = Vec::new();
    let mut errors = Vec::new();

    for result in iter {
        match result {
            Ok(t) => successes.push(t),
            Err(e) => errors.push(e),
        }
    }

    (successes, errors)
}

/// Collect options, stopping at first None.
pub fn collect_options<T, I>(iter: I) -> Option<Vec<T>>
where
    I: IntoIterator<Item = Option<T>>,
{
    iter.into_iter().collect()
}

/// Filter and map in one pass.
pub fn filter_map_results<T, U, E, F, I>(iter: I, f: F) -> Vec<U>
where
    I: IntoIterator<Item = Result<T, E>>,
    F: Fn(T) -> U,
{
    iter.into_iter()
        .filter_map(core::result::Result::ok)
        .map(f)
        .collect()
}

// =============================================================================
// Combinators
// =============================================================================

/// Try multiple operations, returning the first success.
///
/// # Example
///
/// ```rust
/// use ordofp_core::easy::first_success;
///
/// let ops: [fn() -> Result<i32, &'static str>; 3] = [
///     || Err("first failed"),
///     || Err("second failed"),
///     || Ok(42),
/// ];
/// let result = first_success(&ops);
/// assert_eq!(result, Some(42));
/// ```
pub fn first_success<T, E, F>(operations: &[F]) -> Option<T>
where
    F: Fn() -> Result<T, E>,
{
    for op in operations {
        if let Ok(value) = op() {
            return Some(value);
        }
    }
    None
}

/// Try multiple operations, returning all successes.
pub fn all_successes<T, E, F>(operations: &[F]) -> Vec<T>
where
    F: Fn() -> Result<T, E>,
{
    operations.iter().filter_map(|op| op().ok()).collect()
}

/// Execute operations until one fails.
///
/// # Errors
///
/// Returns the error of the first failing operation; operations after
/// it are not run, and the successes gathered so far are discarded.
pub fn until_failure<T, E, F>(operations: &[F]) -> Result<Vec<T>, E>
where
    F: Fn() -> Result<T, E>,
{
    let mut results = Vec::new();
    for op in operations {
        results.push(op()?);
    }
    Ok(results)
}

// =============================================================================
// Lifting
// =============================================================================

/// Lift a function to work with Results.
///
/// # Example
///
/// ```rust
/// use ordofp_core::easy::lift_result;
///
/// let add_one = |x: i32| x + 1;
/// let lifted = lift_result::<i32, i32, &str, _>(add_one);
/// assert_eq!(lifted(Ok(41)), Ok(42));
/// ```
pub fn lift_result<T, U, E, F>(f: F) -> impl Fn(Result<T, E>) -> Result<U, E>
where
    F: Fn(T) -> U,
{
    move |r| r.map(&f)
}

/// Lift a function to work with Options.
pub fn lift_option<T, U, F>(f: F) -> impl Fn(Option<T>) -> Option<U>
where
    F: Fn(T) -> U,
{
    move |o| o.map(&f)
}

/// Lift a binary function to work with Results.
pub fn lift_result2<T1, T2, U, E, F>(f: F) -> impl Fn(Result<T1, E>, Result<T2, E>) -> Result<U, E>
where
    F: Fn(T1, T2) -> U,
{
    move |r1, r2| match (r1, r2) {
        (Ok(t1), Ok(t2)) => Ok(f(t1, t2)),
        (Err(e), _) | (_, Err(e)) => Err(e),
    }
}

/// Lift a binary function to work with Options.
pub fn lift_option2<T1, T2, U, F>(f: F) -> impl Fn(Option<T1>, Option<T2>) -> Option<U>
where
    F: Fn(T1, T2) -> U,
{
    move |o1, o2| match (o1, o2) {
        (Some(t1), Some(t2)) => Some(f(t1, t2)),
        _ => None,
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_result_tap() {
        let mut tapped = false;
        let result: Result<i32, &str> = Ok(42);
        let _ = result.tap(|_| tapped = true);
        assert!(tapped);
    }

    #[test]
    fn test_result_tap_err() {
        let mut tapped = false;
        let result: Result<i32, &str> = Err("error");
        let _ = result.tap_err(|_| tapped = true);
        assert!(tapped);
    }

    #[test]
    fn test_result_swap() {
        let ok: Result<i32, &str> = Ok(42);
        let err: Result<i32, &str> = Err("error");

        assert_eq!(ok.swap(), Err(42));
        assert_eq!(err.swap(), Ok("error"));
    }

    #[test]
    fn test_result_is_ok_and() {
        let result: Result<i32, &str> = Ok(42);
        assert!(result.is_ok_and(|x| x > 0));
        assert!(!result.is_ok_and(|x| x < 0));
    }

    #[test]
    fn test_option_tap() {
        let mut tapped = false;
        let option = Some(42);
        let _ = option.tap(|_| tapped = true);
        assert!(tapped);
    }

    #[test]
    fn test_option_tap_none() {
        let mut tapped = false;
        let option: Option<i32> = None;
        let _ = option.tap_none(|| tapped = true);
        assert!(tapped);
    }

    #[test]
    fn test_option_zip_with() {
        let a = Some(2);
        let b = Some(3);
        let result = OptionExt::zip_with(a, b, |x, y| x * y);
        assert_eq!(result, Some(6));
    }

    #[test]
    fn test_option_zip_with_none_cases() {
        assert_eq!(
            OptionExt::zip_with(Some(2_i32), None, |x, y: i32| x + y),
            None
        );
        assert_eq!(
            OptionExt::zip_with(None, Some(3_i32), |x: i32, y| x + y),
            None
        );
        assert_eq!(
            OptionExt::zip_with(None::<i32>, None::<i32>, |x, y| x + y),
            None
        );
    }

    #[test]
    fn test_bool_to_result() {
        assert_eq!(bool_to_result(true, 42, "error"), Ok(42));
        assert_eq!(bool_to_result(false, 42, "error"), Err("error"));
    }

    #[test]
    fn test_bool_to_option() {
        assert_eq!(bool_to_option(true, 42), Some(42));
        assert_eq!(bool_to_option(false, 42), None);
    }

    #[test]
    fn test_collect_results() {
        let results: Vec<Result<i32, &str>> = alloc::vec![Ok(1), Ok(2), Ok(3)];
        assert_eq!(collect_results(results), Ok(alloc::vec![1, 2, 3]));

        let results: Vec<Result<i32, &str>> = alloc::vec![Ok(1), Err("error"), Ok(3)];
        assert!(collect_results(results).is_err());
    }

    #[test]
    fn test_partition_results_vec() {
        let results: Vec<Result<i32, &str>> = alloc::vec![Ok(1), Err("a"), Ok(2), Err("b")];
        let (successes, errors) = partition_results_vec(results);
        assert_eq!(successes, alloc::vec![1, 2]);
        assert_eq!(errors, alloc::vec!["a", "b"]);
    }

    #[test]
    fn test_first_success() {
        let ops: Vec<fn() -> Result<i32, &'static str>> =
            alloc::vec![|| Err("first"), || Err("second"), || Ok(42),];
        assert_eq!(first_success(&ops), Some(42));
    }

    #[test]
    fn test_first_success_all_fail() {
        let ops: Vec<fn() -> Result<i32, &'static str>> =
            alloc::vec![|| Err("first"), || Err("second"), || Err("third")];
        assert_eq!(first_success(&ops), None);
    }

    #[test]
    fn test_all_successes_empty_slice() {
        let ops: Vec<fn() -> Result<i32, &'static str>> = alloc::vec![];
        let result: Vec<i32> = all_successes(&ops);
        assert!(result.is_empty(), "empty ops should yield empty successes");
    }

    #[test]
    fn test_until_failure_empty_slice() {
        let ops: Vec<fn() -> Result<i32, &'static str>> = alloc::vec![];
        assert_eq!(until_failure(&ops), Ok(alloc::vec![]));
    }

    #[test]
    fn test_until_failure_stops_at_first_error() {
        // The third op must never run; if it does the test panics, catching the bug.
        let ops: Vec<fn() -> Result<i32, &'static str>> =
            alloc::vec![|| Ok(1), || Err("stop"), || panic!(
                "executed past first error"
            )];
        assert_eq!(until_failure(&ops), Err("stop"));
    }

    #[test]
    fn test_lift_result() {
        let add_one = |x: i32| x + 1;
        let lifted = lift_result(add_one);
        assert_eq!(lifted(Ok(41)), Ok(42));
        assert_eq!(lifted(Err("error")), Err("error"));
    }

    #[test]
    fn test_lift_option() {
        let add_one = |x: i32| x + 1;
        let lifted = lift_option(add_one);
        assert_eq!(lifted(Some(41)), Some(42));
        assert_eq!(lifted(None), None);
    }

    #[test]
    fn test_lift_result2() {
        let add = |x: i32, y: i32| x + y;
        let lifted = lift_result2(add);
        assert_eq!(lifted(Ok(20), Ok(22)), Ok(42));
        assert_eq!(lifted(Err("error"), Ok(22)), Err("error"));
    }
}
