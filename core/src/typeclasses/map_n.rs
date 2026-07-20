// In map_n functions, single-char names (a, b, c, d) are conventional and idiomatic
// for tuple elements being combined. This follows Haskell/Scala conventions.

//! # `MapN` - Combining Multiple Functor Values
//!
//! > *"Ex pluribus unum."* — Out of many, one. (E pluribus unum)
//!
//! This module provides helper functions for combining multiple functor values
//! with a function. These are convenience wrappers around the Semigroupal's
//! `productum` operation followed by a map.
//!
//! ## Quick Start
//!
//! ```rust
//! use ordofp_core::typeclasses::map_n::{map2, map3, map4};
//!
//! // Combine two Option values
//! let result = map2(Some(1), Some(2), |a, b| a + b);
//! assert_eq!(result, Some(3));
//!
//! // Short-circuits on None
//! let result = map2(Some(1), None::<i32>, |a, b| a + b);
//! assert_eq!(result, None);
//!
//! // Combine three values
//! let result = map3(Some(1), Some(2), Some(3), |a, b, c| a + b + c);
//! assert_eq!(result, Some(6));
//!
//! // Combine four values  
//! let result = map4(Some(1), Some(2), Some(3), Some(4), |a, b, c, d| a + b + c + d);
//! assert_eq!(result, Some(10));
//! ```
//!
//! ## With Result
//!
//! ```rust
//! use ordofp_core::typeclasses::map_n::{map2_result, map3_result};
//!
//! let result: Result<i32, &str> = map2_result(Ok(1), Ok(2), |a, b| a + b);
//! assert_eq!(result, Ok(3));
//!
//! // When second is Err, type annotations needed for closure params
//! let err: Result<i32, &str> = map2_result(Ok(1i32), Err("error"), |a: i32, b: i32| a + b);
//! assert_eq!(err, Err("error"));
//! ```

// ========== Option MapN ==========

/// Combines two Option values with a binary function.
///
/// Returns `None` if either input is `None`.
///
/// # Examples
///
/// ```rust
/// use ordofp_core::typeclasses::map_n::map2;
///
/// let result = map2(Some(1), Some(2), |a, b| a + b);
/// assert_eq!(result, Some(3));
///
/// let none_result = map2(Some(1), None::<i32>, |a, b| a + b);
/// assert_eq!(none_result, None);
/// ```
#[inline]
pub fn map2<A, B, C, F>(fa: Option<A>, fb: Option<B>, f: F) -> Option<C>
where
    F: FnOnce(A, B) -> C,
{
    match (fa, fb) {
        (Some(a), Some(b)) => Some(f(a, b)),
        _ => None,
    }
}

/// Combines three Option values with a ternary function.
///
/// Returns `None` if any input is `None`.
///
/// # Examples
///
/// ```rust
/// use ordofp_core::typeclasses::map_n::map3;
///
/// let result = map3(Some(1), Some(2), Some(3), |a, b, c| a + b + c);
/// assert_eq!(result, Some(6));
/// ```
#[inline]
pub fn map3<A, B, C, D, F>(fa: Option<A>, fb: Option<B>, fc: Option<C>, f: F) -> Option<D>
where
    F: FnOnce(A, B, C) -> D,
{
    match (fa, fb, fc) {
        (Some(a), Some(b), Some(c)) => Some(f(a, b, c)),
        _ => None,
    }
}

/// Combines four Option values with a quaternary function.
///
/// Returns `None` if any input is `None`.
///
/// # Examples
///
/// ```rust
/// use ordofp_core::typeclasses::map_n::map4;
///
/// let result = map4(Some(1), Some(2), Some(3), Some(4), |a, b, c, d| a + b + c + d);
/// assert_eq!(result, Some(10));
/// ```
#[inline]
pub fn map4<A, B, C, D, E, F>(
    fa: Option<A>,
    fb: Option<B>,
    fc: Option<C>,
    fd: Option<D>,
    f: F,
) -> Option<E>
where
    F: FnOnce(A, B, C, D) -> E,
{
    match (fa, fb, fc, fd) {
        (Some(a), Some(b), Some(c), Some(d)) => Some(f(a, b, c, d)),
        _ => None,
    }
}

/// Combines five Option values with a quinary function.
///
/// Returns `None` if any input is `None`.
///
/// # Examples
///
/// ```rust
/// use ordofp_core::typeclasses::map_n::map5;
///
/// let result = map5(Some(1), Some(2), Some(3), Some(4), Some(5), |a, b, c, d, e| a + b + c + d + e);
/// assert_eq!(result, Some(15));
/// ```
#[inline]
pub fn map5<A, B, C, D, E, G, F>(
    fa: Option<A>,
    fb: Option<B>,
    fc: Option<C>,
    fd: Option<D>,
    fe: Option<E>,
    f: F,
) -> Option<G>
where
    F: FnOnce(A, B, C, D, E) -> G,
{
    match (fa, fb, fc, fd, fe) {
        (Some(a), Some(b), Some(c), Some(d), Some(e)) => Some(f(a, b, c, d, e)),
        _ => None,
    }
}

/// Combines six Option values with a senary function.
#[inline]
pub fn map6<A, B, C, D, E, G, H, F>(
    fa: Option<A>,
    fb: Option<B>,
    fc: Option<C>,
    fd: Option<D>,
    fe: Option<E>,
    fg: Option<G>,
    f: F,
) -> Option<H>
where
    F: FnOnce(A, B, C, D, E, G) -> H,
{
    match (fa, fb, fc, fd, fe, fg) {
        (Some(a), Some(b), Some(c), Some(d), Some(e), Some(g)) => Some(f(a, b, c, d, e, g)),
        _ => None,
    }
}

// ========== Result MapN ==========

/// Combines two Result values with a binary function.
///
/// Returns the first error encountered, or the combined result.
///
/// # Examples
///
/// ```rust
/// use ordofp_core::typeclasses::map_n::map2_result;
///
/// let result: Result<i32, &str> = map2_result(Ok(1), Ok(2), |a, b| a + b);
/// assert_eq!(result, Ok(3));
///
/// // When second is Err, type annotations needed for closure params
/// let err: Result<i32, &str> = map2_result(Ok(1i32), Err("error"), |a: i32, b: i32| a + b);
/// assert_eq!(err, Err("error"));
/// ```
///
/// # Errors
///
/// Returns the leftmost `Err` among the inputs (`fa` before `fb`);
/// `f` is only applied when both are `Ok`. Errors do not accumulate —
/// use `Probatum` for accumulating validation.
#[inline]
pub fn map2_result<A, B, C, E, F>(fa: Result<A, E>, fb: Result<B, E>, f: F) -> Result<C, E>
where
    F: FnOnce(A, B) -> C,
{
    match (fa, fb) {
        (Ok(a), Ok(b)) => Ok(f(a, b)),
        (Err(e), _) => Err(e),
        (_, Err(e)) => Err(e),
    }
}

/// Combines three Result values with a ternary function.
///
/// Returns the first error encountered, or the combined result.
///
/// # Examples
///
/// ```rust
/// use ordofp_core::typeclasses::map_n::map3_result;
///
/// let result: Result<i32, &str> = map3_result(Ok(1), Ok(2), Ok(3), |a, b, c| a + b + c);
/// assert_eq!(result, Ok(6));
/// ```
///
/// # Errors
///
/// Returns the leftmost `Err` among the three inputs; `f` is only
/// applied when all are `Ok`.
#[inline]
pub fn map3_result<A, B, C, D, E, F>(
    fa: Result<A, E>,
    fb: Result<B, E>,
    fc: Result<C, E>,
    f: F,
) -> Result<D, E>
where
    F: FnOnce(A, B, C) -> D,
{
    match (fa, fb, fc) {
        (Ok(a), Ok(b), Ok(c)) => Ok(f(a, b, c)),
        (Err(e), _, _) => Err(e),
        (_, Err(e), _) => Err(e),
        (_, _, Err(e)) => Err(e),
    }
}

/// Combines four Result values with a quaternary function.
///
/// Returns the first error encountered, or the combined result.
///
/// # Examples
///
/// ```rust
/// use ordofp_core::typeclasses::map_n::map4_result;
///
/// let result: Result<i32, &str> = map4_result(Ok(1), Ok(2), Ok(3), Ok(4), |a, b, c, d| a + b + c + d);
/// assert_eq!(result, Ok(10));
/// ```
///
/// # Errors
///
/// Returns the leftmost `Err` among the four inputs; `f` is only
/// applied when all are `Ok`.
#[inline]
pub fn map4_result<A, B, C, D, E, G, F>(
    fa: Result<A, E>,
    fb: Result<B, E>,
    fc: Result<C, E>,
    fd: Result<D, E>,
    f: F,
) -> Result<G, E>
where
    F: FnOnce(A, B, C, D) -> G,
{
    match (fa, fb, fc, fd) {
        (Ok(a), Ok(b), Ok(c), Ok(d)) => Ok(f(a, b, c, d)),
        (Err(e), _, _, _) => Err(e),
        (_, Err(e), _, _) => Err(e),
        (_, _, Err(e), _) => Err(e),
        (_, _, _, Err(e)) => Err(e),
    }
}

/// Combines five Result values with a quinary function.
///
/// # Errors
///
/// Returns the leftmost `Err` among the five inputs; `f` is only
/// applied when all are `Ok`.
#[inline]
pub fn map5_result<A, B, C, D, E, G, H, F>(
    fa: Result<A, H>,
    fb: Result<B, H>,
    fc: Result<C, H>,
    fd: Result<D, H>,
    fe: Result<E, H>,
    f: F,
) -> Result<G, H>
where
    F: FnOnce(A, B, C, D, E) -> G,
{
    match (fa, fb, fc, fd, fe) {
        (Ok(a), Ok(b), Ok(c), Ok(d), Ok(e)) => Ok(f(a, b, c, d, e)),
        (Err(e), _, _, _, _) => Err(e),
        (_, Err(e), _, _, _) => Err(e),
        (_, _, Err(e), _, _) => Err(e),
        (_, _, _, Err(e), _) => Err(e),
        (_, _, _, _, Err(e)) => Err(e),
    }
}

// ========== Tuple MapN (Applicative style) ==========

/// Applies a binary function to a tuple of values.
///
/// This is a simple helper for applying a function to paired values.
///
/// # Examples
///
/// ```rust
/// use ordofp_core::typeclasses::map_n::tuple2_map;
///
/// let result = tuple2_map((1, 2), |a, b| a + b);
/// assert_eq!(result, 3);
/// ```
#[inline]
pub fn tuple2_map<A, B, C, F>((a, b): (A, B), f: F) -> C
where
    F: FnOnce(A, B) -> C,
{
    f(a, b)
}

/// Applies a ternary function to a tuple of values.
#[inline]
pub fn tuple3_map<A, B, C, D, F>((a, b, c): (A, B, C), f: F) -> D
where
    F: FnOnce(A, B, C) -> D,
{
    f(a, b, c)
}

/// Applies a quaternary function to a tuple of values.
#[inline]
pub fn tuple4_map<A, B, C, D, E, F>((a, b, c, d): (A, B, C, D), f: F) -> E
where
    F: FnOnce(A, B, C, D) -> E,
{
    f(a, b, c, d)
}

// ========== Sequence helpers ==========

/// Converts a tuple of Options into an Option of tuple.
///
/// Returns `None` if any element is `None`.
///
/// # Examples
///
/// ```rust
/// use ordofp_core::typeclasses::map_n::sequence2;
///
/// let result = sequence2((Some(1), Some(2)));
/// assert_eq!(result, Some((1, 2)));
///
/// let none = sequence2((Some(1), None::<i32>));
/// assert_eq!(none, None);
/// ```
#[inline]
pub fn sequence2<A, B>(tuple: (Option<A>, Option<B>)) -> Option<(A, B)> {
    match tuple {
        (Some(a), Some(b)) => Some((a, b)),
        _ => None,
    }
}

/// Converts a tuple of three Options into an Option of tuple.
#[inline]
pub fn sequence3<A, B, C>(tuple: (Option<A>, Option<B>, Option<C>)) -> Option<(A, B, C)> {
    match tuple {
        (Some(a), Some(b), Some(c)) => Some((a, b, c)),
        _ => None,
    }
}

/// Converts a tuple of four Options into an Option of tuple.
#[inline]
pub fn sequence4<A, B, C, D>(
    tuple: (Option<A>, Option<B>, Option<C>, Option<D>),
) -> Option<(A, B, C, D)> {
    match tuple {
        (Some(a), Some(b), Some(c), Some(d)) => Some((a, b, c, d)),
        _ => None,
    }
}

/// Converts a tuple of Results into a Result of tuple.
///
/// Returns the first error encountered, or the combined tuple.
///
/// # Examples
///
/// ```rust
/// use ordofp_core::typeclasses::map_n::sequence2_result;
///
/// let result: Result<(i32, i32), &str> = sequence2_result((Ok(1), Ok(2)));
/// assert_eq!(result, Ok((1, 2)));
///
/// let err: Result<(i32, i32), &str> = sequence2_result((Ok(1), Err("error")));
/// assert_eq!(err, Err("error"));
/// ```
///
/// # Errors
///
/// Returns the leftmost `Err` in the tuple; the other component's
/// success value is discarded.
#[inline]
pub fn sequence2_result<A, B, E>(tuple: (Result<A, E>, Result<B, E>)) -> Result<(A, B), E> {
    match tuple {
        (Ok(a), Ok(b)) => Ok((a, b)),
        (Err(e), _) => Err(e),
        (_, Err(e)) => Err(e),
    }
}

/// Converts a tuple of three Results into a Result of tuple.
///
/// # Errors
///
/// Returns the leftmost `Err` in the tuple; the other components'
/// success values are discarded.
#[inline]
pub fn sequence3_result<A, B, C, E>(
    tuple: (Result<A, E>, Result<B, E>, Result<C, E>),
) -> Result<(A, B, C), E> {
    match tuple {
        (Ok(a), Ok(b), Ok(c)) => Ok((a, b, c)),
        (Err(e), _, _) => Err(e),
        (_, Err(e), _) => Err(e),
        (_, _, Err(e)) => Err(e),
    }
}

/// A tuple of four `Result`s sharing one error type — input of
/// [`sequence4_result`].
pub type Results4<A, B, C, D, E> = (Result<A, E>, Result<B, E>, Result<C, E>, Result<D, E>);

/// Converts a tuple of four Results into a Result of tuple.
///
/// # Errors
///
/// Returns the leftmost `Err` in the tuple; the other components'
/// success values are discarded.
#[inline]
pub fn sequence4_result<A, B, C, D, E>(tuple: Results4<A, B, C, D, E>) -> Result<(A, B, C, D), E> {
    match tuple {
        (Ok(a), Ok(b), Ok(c), Ok(d)) => Ok((a, b, c, d)),
        (Err(e), _, _, _) => Err(e),
        (_, Err(e), _, _) => Err(e),
        (_, _, Err(e), _) => Err(e),
        (_, _, _, Err(e)) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map2_some() {
        let result = map2(Some(1), Some(2), |a, b| a + b);
        assert_eq!(result, Some(3));
    }

    #[test]
    fn test_map2_none_first() {
        let result = map2(None::<i32>, Some(2), |a, b| a + b);
        assert_eq!(result, None);
    }

    #[test]
    fn test_map2_none_second() {
        let result = map2(Some(1), None::<i32>, |a, b| a + b);
        assert_eq!(result, None);
    }

    #[test]
    fn test_map3() {
        let result = map3(Some(1), Some(2), Some(3), |a, b, c| a + b + c);
        assert_eq!(result, Some(6));
    }

    #[test]
    fn test_map4() {
        let result = map4(Some(1), Some(2), Some(3), Some(4), |a, b, c, d| {
            a + b + c + d
        });
        assert_eq!(result, Some(10));
    }

    #[test]
    fn test_map5() {
        let result = map5(
            Some(1),
            Some(2),
            Some(3),
            Some(4),
            Some(5),
            |a, b, c, d, e| a + b + c + d + e,
        );
        assert_eq!(result, Some(15));
    }

    #[test]
    fn test_map2_result_ok() {
        let result: Result<i32, &str> = map2_result(Ok(1), Ok(2), |a, b| a + b);
        assert_eq!(result, Ok(3));
    }

    #[test]
    fn test_map2_result_err_first() {
        let result: Result<i32, &str> = map2_result(Err("first"), Ok(2i32), |a: i32, b| a + b);
        assert_eq!(result, Err("first"));
    }

    #[test]
    fn test_map2_result_err_second() {
        let result: Result<i32, &str> = map2_result(Ok(1i32), Err("second"), |a, b: i32| a + b);
        assert_eq!(result, Err("second"));
    }

    #[test]
    fn test_map3_result() {
        let result: Result<i32, &str> = map3_result(Ok(1), Ok(2), Ok(3), |a, b, c| a + b + c);
        assert_eq!(result, Ok(6));
    }

    #[test]
    fn test_map4_result() {
        let result: Result<i32, &str> =
            map4_result(Ok(1), Ok(2), Ok(3), Ok(4), |a, b, c, d| a + b + c + d);
        assert_eq!(result, Ok(10));
    }

    #[test]
    fn test_tuple2_map() {
        let result = tuple2_map((1, 2), |a, b| a + b);
        assert_eq!(result, 3);
    }

    #[test]
    fn test_tuple3_map() {
        let result = tuple3_map((1, 2, 3), |a, b, c| a + b + c);
        assert_eq!(result, 6);
    }

    #[test]
    fn test_sequence2_some() {
        let result = sequence2((Some(1), Some(2)));
        assert_eq!(result, Some((1, 2)));
    }

    #[test]
    fn test_sequence2_none() {
        let result = sequence2((Some(1), None::<i32>));
        assert_eq!(result, None);
    }

    #[test]
    fn test_sequence3() {
        let result = sequence3((Some(1), Some(2), Some(3)));
        assert_eq!(result, Some((1, 2, 3)));
    }

    #[test]
    fn test_sequence2_result_ok() {
        let result: Result<(i32, i32), &str> = sequence2_result((Ok(1), Ok(2)));
        assert_eq!(result, Ok((1, 2)));
    }

    #[test]
    fn test_sequence2_result_err() {
        let result: Result<(i32, i32), &str> = sequence2_result((Ok(1), Err("error")));
        assert_eq!(result, Err("error"));
    }

    #[test]
    fn test_sequence3_result() {
        let result: Result<(i32, i32, i32), &str> = sequence3_result((Ok(1), Ok(2), Ok(3)));
        assert_eq!(result, Ok((1, 2, 3)));
    }
}
