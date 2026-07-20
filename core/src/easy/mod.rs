//! Easy API - Simplified Interface for `OrdoFP`
//!
//! This module provides a beginner-friendly API that hides the complexity
//! of effect types, row polymorphism, and type-level programming.
//!
//! # Philosophy
//!
//! The Easy API follows these principles:
//!
//! 1. **Zero type annotations required** - Types are inferred
//! 2. **Familiar patterns** - Looks like regular Rust
//! 3. **Escape hatches** - Can drop down to full API when needed
//!
//! # Usage
//!
//! ```rust
//! use ordofp_core::easy::*;
//!
//! // Simple state management
//! let result = run_with_state(0, |state| {
//!     *state += 1;
//!     *state * 2
//! });
//! assert_eq!(result, 2);
//!
//! // Reader pattern for configuration
//! #[derive(Default)]
//! struct Config {
//!     timeout_ms: u64,
//! }
//!
//! let result = run_with_config(&Config::default(), |config: &Config| {
//!     config.timeout_ms
//! });
//! assert_eq!(result, 0);
//!
//! // Error handling
//! let result: Result<i32, String> = run_with_error(|| {
//!     Ok(42)
//! });
//! assert_eq!(result, Ok(42));
//! ```

mod error;
mod io;
mod reader;
mod result_ext;
mod state;

pub use error::*;
pub use io::*;
pub use reader::*;
pub use result_ext::*;
pub use state::*;

// =============================================================================
// Do-Notation Style Combinators
// =============================================================================

/// Chain computations together, passing the result of each to the next.
///
/// This provides a do-notation-like experience without macros.
///
/// # Example
///
/// ```rust
/// use ordofp_core::easy::*;
///
/// let result = chain(
///     || 1,
///     |x| x + 1,
///     |x| x * 2,
/// );
/// assert_eq!(result, 4);
/// ```
pub fn chain<A, B, C, F1, F2, F3>(first: F1, second: F2, third: F3) -> C
where
    F1: FnOnce() -> A,
    F2: FnOnce(A) -> B,
    F3: FnOnce(B) -> C,
{
    third(second(first()))
}

/// Chain two computations.
pub fn chain2<A, B, F1, F2>(first: F1, second: F2) -> B
where
    F1: FnOnce() -> A,
    F2: FnOnce(A) -> B,
{
    second(first())
}

/// Chain four computations.
pub fn chain4<A, B, C, D, F1, F2, F3, F4>(first: F1, second: F2, third: F3, fourth: F4) -> D
where
    F1: FnOnce() -> A,
    F2: FnOnce(A) -> B,
    F3: FnOnce(B) -> C,
    F4: FnOnce(C) -> D,
{
    fourth(third(second(first())))
}

// =============================================================================
// Pure Computations
// =============================================================================

/// Lift a pure value into a computation context.
///
/// # Example
///
/// ```rust
/// use ordofp_core::easy::pure;
///
/// let x = pure(42);
/// assert_eq!(x, 42);
/// ```
#[inline]
pub fn pure<T>(value: T) -> T {
    value
}

/// Sequence two computations, discarding the first result.
///
/// # Example
///
/// ```rust
/// use ordofp_core::easy::sequence;
///
/// let result = sequence(
///     || println!("first"),
///     || 42,
/// );
/// assert_eq!(result, 42);
/// ```
pub fn sequence<A, B, F1, F2>(first: F1, second: F2) -> B
where
    F1: FnOnce() -> A,
    F2: FnOnce() -> B,
{
    let _ = first();
    second()
}

/// Apply a function to the result of a computation.
///
/// # Example
///
/// ```rust
/// use ordofp_core::easy::map;
///
/// let result = map(|| 21, |x| x * 2);
/// assert_eq!(result, 42);
/// ```
pub fn map<A, B, F, G>(computation: F, mapper: G) -> B
where
    F: FnOnce() -> A,
    G: FnOnce(A) -> B,
{
    mapper(computation())
}

/// Flatten nested computations.
///
/// # Example
///
/// ```rust
/// use ordofp_core::easy::flatten;
///
/// let result = flatten(|| || 42);
/// assert_eq!(result, 42);
/// ```
pub fn flatten<A, F, G>(computation: F) -> A
where
    F: FnOnce() -> G,
    G: FnOnce() -> A,
{
    computation()()
}

// =============================================================================
// Tuple Combinators
// =============================================================================

/// Run two computations and combine their results into a tuple.
///
/// # Example
///
/// ```rust
/// use ordofp_core::easy::both;
///
/// let (a, b) = both(|| 1, || 2);
/// assert_eq!((a, b), (1, 2));
/// ```
pub fn both<A, B, F1, F2>(first: F1, second: F2) -> (A, B)
where
    F1: FnOnce() -> A,
    F2: FnOnce() -> B,
{
    (first(), second())
}

/// Run three computations and combine their results.
pub fn all3<A, B, C, F1, F2, F3>(f1: F1, f2: F2, f3: F3) -> (A, B, C)
where
    F1: FnOnce() -> A,
    F2: FnOnce() -> B,
    F3: FnOnce() -> C,
{
    (f1(), f2(), f3())
}

/// Run four computations and combine their results.
pub fn all4<A, B, C, D, F1, F2, F3, F4>(f1: F1, f2: F2, f3: F3, f4: F4) -> (A, B, C, D)
where
    F1: FnOnce() -> A,
    F2: FnOnce() -> B,
    F3: FnOnce() -> C,
    F4: FnOnce() -> D,
{
    (f1(), f2(), f3(), f4())
}

// =============================================================================
// Conditional Combinators
// =============================================================================

/// Conditional computation based on a predicate.
///
/// # Example
///
/// ```rust
/// use ordofp_core::easy::when;
///
/// let result = when(true, || 42, || 0);
/// assert_eq!(result, 42);
/// ```
pub fn when<A, F1, F2>(condition: bool, if_true: F1, if_false: F2) -> A
where
    F1: FnOnce() -> A,
    F2: FnOnce() -> A,
{
    if condition { if_true() } else { if_false() }
}

/// Execute a computation only if a condition is true.
///
/// # Example
///
/// ```rust
/// use ordofp_core::easy::when_true;
///
/// let result = when_true(true, || 42);
/// assert_eq!(result, Some(42));
/// ```
pub fn when_true<A, F>(condition: bool, computation: F) -> Option<A>
where
    F: FnOnce() -> A,
{
    if condition { Some(computation()) } else { None }
}

/// Execute a computation only if a condition is false.
pub fn when_false<A, F>(condition: bool, computation: F) -> Option<A>
where
    F: FnOnce() -> A,
{
    (!condition).then(computation)
}

// =============================================================================
// Loop Combinators
// =============================================================================

/// Repeat a computation N times, collecting results.
///
/// # Example
///
/// ```rust
/// use ordofp_core::easy::repeat;
///
/// let results = repeat(3, |i| i * 2);
/// assert_eq!(results, vec![0, 2, 4]);
/// ```
pub fn repeat<A, F>(count: usize, computation: F) -> alloc::vec::Vec<A>
where
    F: FnMut(usize) -> A,
{
    (0..count).map(computation).collect()
}

/// Iterate while a condition is true.
///
/// # Example
///
/// ```rust
/// use core::cell::Cell;
/// use ordofp_core::easy::iterate_while;
///
/// let i = Cell::new(0);
/// let result = iterate_while(|| i.get() < 5, || { i.set(i.get() + 1); i.get() });
/// assert_eq!(result, vec![1, 2, 3, 4, 5]);
/// ```
pub fn iterate_while<A, P, F>(mut predicate: P, mut computation: F) -> alloc::vec::Vec<A>
where
    P: FnMut() -> bool,
    F: FnMut() -> A,
{
    let mut results = alloc::vec::Vec::new();
    while predicate() {
        results.push(computation());
    }
    results
}

/// Fold over a range.
///
/// # Example
///
/// ```rust
/// use ordofp_core::easy::fold_range;
///
/// let sum = fold_range(0..5, 0, |acc, i| acc + i);
/// assert_eq!(sum, 10);
/// ```
pub fn fold_range<A, F>(range: core::ops::Range<usize>, init: A, mut folder: F) -> A
where
    F: FnMut(A, usize) -> A,
{
    let mut acc = init;
    for i in range {
        acc = folder(acc, i);
    }
    acc
}

// =============================================================================
// Resource Management
// =============================================================================

/// Execute a computation with a resource, closing it on the normal path.
///
/// Similar in shape to try-with-resources, but **not panic-safe**: `close`
/// runs only when `use_resource` returns normally. If `use_resource` panics,
/// `close` is skipped and the resource is simply dropped during unwinding —
/// any cleanup beyond `Drop` will not happen. For guaranteed cleanup use a
/// RAII guard type instead.
///
/// # Example
///
/// ```rust,no_run
/// // Uses real file IO, so this example is `no_run` (the file may not exist
/// // and the doctest binary is not built with the crate's own `std` feature).
/// use ordofp_core::easy::with_resource;
/// use std::fs::File;
/// use std::io::Read;
///
/// let contents: String = with_resource(
///     || File::open("test.txt").expect("failed to open file"),
///     |file: &File| {
///         let mut file = file.try_clone().expect("failed to clone handle");
///         let mut buf = String::new();
///         file.read_to_string(&mut buf).expect("failed to read");
///         buf
///     },
///     |_file| {},
/// );
/// println!("{contents}");
/// ```
pub fn with_resource<R, A, Open, Use, Close>(open: Open, use_resource: Use, close: Close) -> A
where
    Open: FnOnce() -> R,
    Use: FnOnce(&R) -> A,
    Close: FnOnce(R),
{
    let resource = open();
    let result = use_resource(&resource);
    close(resource);
    result
}

/// Execute a computation with a mutable resource.
///
/// Same caveat as [`with_resource`]: `close` is skipped if `use_resource`
/// panics — cleanup happens only on the normal return path.
pub fn with_resource_mut<R, A, Open, Use, Close>(open: Open, use_resource: Use, close: Close) -> A
where
    Open: FnOnce() -> R,
    Use: FnOnce(&mut R) -> A,
    Close: FnOnce(R),
{
    let mut resource = open();
    let result = use_resource(&mut resource);
    close(resource);
    result
}

// =============================================================================
// Validation Combinators
// =============================================================================

/// Validate a value with a predicate, returning an Option.
///
/// # Example
///
/// ```rust
/// use ordofp_core::easy::validate;
///
/// let valid = validate(42, |x| *x > 0);
/// assert_eq!(valid, Some(42));
///
/// let invalid = validate(-1, |x| *x > 0);
/// assert_eq!(invalid, None);
/// ```
pub fn validate<T, P>(value: T, predicate: P) -> Option<T>
where
    P: FnOnce(&T) -> bool,
{
    if predicate(&value) { Some(value) } else { None }
}

/// Validate a value, returning a Result with an error message.
///
/// # Errors
///
/// Returns `Err(error)` when `predicate(&value)` is `false`; the value
/// is dropped in that case.
///
/// # Example
///
/// ```rust
/// use ordofp_core::easy::validate_or;
///
/// let result: Result<i32, &str> = validate_or(42, |x| *x > 0, "must be positive");
/// assert!(result.is_ok());
/// ```
pub fn validate_or<T, E, P>(value: T, predicate: P, error: E) -> Result<T, E>
where
    P: FnOnce(&T) -> bool,
{
    if predicate(&value) {
        Ok(value)
    } else {
        Err(error)
    }
}

/// A single validation: a predicate over `&T` paired with the error to emit
/// when the predicate fails.
pub type Validation<T, E> = (fn(&T) -> bool, E);

/// Validate multiple conditions, collecting all errors.
///
/// # Errors
///
/// Every predicate is run against `value`; if any fail, returns a `Vec`
/// with the error of each failed validation in slice order. All
/// validations are checked — this accumulates rather than
/// short-circuiting on the first failure.
///
/// # Example
///
/// ```rust
/// use ordofp_core::easy::validate_all;
///
/// let result = validate_all(
///     42,
///     &[
///         (|x| *x > 0, "must be positive"),
///         (|x: &i32| *x < 100, "must be less than 100"),
///     ],
/// );
/// assert!(result.is_ok());
/// ```
pub fn validate_all<T, E: Clone>(
    value: T,
    validations: &[Validation<T, E>],
) -> Result<T, alloc::vec::Vec<E>> {
    let errors: alloc::vec::Vec<E> = validations
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

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain() {
        let result = chain(|| 1, |x| x + 1, |x| x * 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn test_chain2() {
        let result = chain2(|| 10, |x| x * 2);
        assert_eq!(result, 20);
    }

    #[test]
    fn test_pure() {
        assert_eq!(pure(42), 42);
    }

    #[test]
    fn test_map() {
        let result = map(|| 21, |x| x * 2);
        assert_eq!(result, 42);
    }

    #[test]
    fn test_both() {
        let (a, b) = both(|| 1, || 2);
        assert_eq!((a, b), (1, 2));
    }

    #[test]
    fn test_when() {
        assert_eq!(when(true, || 1, || 0), 1);
        assert_eq!(when(false, || 1, || 0), 0);
    }

    #[test]
    fn test_when_true() {
        assert_eq!(when_true(true, || 42), Some(42));
        assert_eq!(when_true(false, || 42), None);
    }

    #[test]
    fn test_repeat() {
        let results = repeat(3, |i| i * 2);
        assert_eq!(results, alloc::vec![0, 2, 4]);
    }

    #[test]
    fn test_fold_range() {
        let sum = fold_range(0..5, 0, |acc, i| acc + i);
        assert_eq!(sum, 10);
    }

    #[test]
    fn test_validate() {
        assert_eq!(validate(42, |x| *x > 0), Some(42));
        assert_eq!(validate(-1, |x: &i32| *x > 0), None);
    }

    #[test]
    fn test_validate_or() {
        let result: Result<i32, &str> = validate_or(42, |x| *x > 0, "must be positive");
        assert!(result.is_ok());

        let result: Result<i32, &str> = validate_or(-1, |x| *x > 0, "must be positive");
        assert!(result.is_err());
    }
}
