//! Bifunctor type class for types with two type parameters.
//!
//! A `Bifunctor` is a type constructor that takes two type arguments and is a
//! functor in both of them. This means you can map over both type parameters
//! independently or simultaneously.
//!
//! # Example
//!
//! ```rust
//! use ordofp_core::bifunctor::Bifunctor;
//!
//! // Map over both sides of a Result
//! let result: Result<i32, &str> = Ok(42);
//! let mapped = result.bimap(|x| x * 2, |e| e.len());
//! assert_eq!(mapped, Ok(84));
//!
//! let err: Result<i32, &str> = Err("error");
//! let mapped = err.bimap(|x| x * 2, |e| e.len());
//! assert_eq!(mapped, Err(5));
//!
//! // Map over a tuple
//! let tuple = (10, "hello");
//! let mapped = tuple.bimap(|x| x * 2, |s: &str| s.len());
//! assert_eq!(mapped, (20, 5));
//! ```

/// A functor over two type parameters.
///
/// Bifunctor provides the `bimap` operation that allows mapping over both
/// type parameters simultaneously.
///
/// # Laws
///
/// A lawful Bifunctor must satisfy:
///
/// 1. **Identity**: `x.bimap(id, id) == x`
/// 2. **Composition**: `x.bimap(f, g).bimap(h, i) == x.bimap(|a| h(f(a)), |b| i(g(b)))`
pub trait Bifunctor {
    /// The first type parameter.
    type Left;
    /// The second type parameter.
    type Right;
    /// The output type after transformation.
    type Target<A, B>;

    /// Map over both type parameters simultaneously.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::bifunctor::Bifunctor;
    ///
    /// let result: Result<i32, String> = Ok(42);
    /// let mapped = result.bimap(|x| x.to_string(), |e| e.len());
    /// assert_eq!(mapped, Ok("42".to_string()));
    /// ```
    fn bimap<A, B, F, G>(self, f: F, g: G) -> Self::Target<A, B>
    where
        F: FnOnce(Self::Left) -> A,
        G: FnOnce(Self::Right) -> B;

    /// Map over the first (left) type parameter only.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::bifunctor::Bifunctor;
    ///
    /// let result: Result<i32, &str> = Ok(42);
    /// let mapped = result.map_left(|x| x.to_string());
    /// assert_eq!(mapped, Ok("42".to_string()));
    /// ```
    #[inline]
    fn map_left<A, F>(self, f: F) -> Self::Target<A, Self::Right>
    where
        Self: Sized,
        Self::Right: Sized,
        F: FnOnce(Self::Left) -> A,
    {
        self.bimap(f, |x| x)
    }

    /// Map over the second (right) type parameter only.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::bifunctor::Bifunctor;
    ///
    /// let result: Result<i32, &str> = Err("error");
    /// let mapped = result.map_right(|e| e.len());
    /// assert_eq!(mapped, Err(5));
    /// ```
    #[inline]
    fn map_right<B, G>(self, g: G) -> Self::Target<Self::Left, B>
    where
        Self: Sized,
        Self::Left: Sized,
        G: FnOnce(Self::Right) -> B,
    {
        self.bimap(|x| x, g)
    }
}

// Implementation for Result<T, E>
impl<T, E> Bifunctor for Result<T, E> {
    type Left = T;
    type Right = E;
    type Target<A, B> = Result<A, B>;

    #[inline]
    fn bimap<A, B, F, G>(self, f: F, g: G) -> Result<A, B>
    where
        F: FnOnce(T) -> A,
        G: FnOnce(E) -> B,
    {
        match self {
            Ok(t) => Ok(f(t)),
            Err(e) => Err(g(e)),
        }
    }
}

// Implementation for tuples (T, U)
impl<T, U> Bifunctor for (T, U) {
    type Left = T;
    type Right = U;
    type Target<A, B> = (A, B);

    #[inline]
    fn bimap<A, B, F, G>(self, f: F, g: G) -> (A, B)
    where
        F: FnOnce(T) -> A,
        G: FnOnce(U) -> B,
    {
        (f(self.0), g(self.1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "alloc")]
    use alloc::{format, string::String, vec::Vec};

    #[test]
    fn test_result_bimap_ok() {
        let result: Result<i32, &str> = Ok(42);
        let mapped = result.bimap(|x| x * 2, str::len);
        assert_eq!(mapped, Ok(84));
    }

    #[test]
    fn test_result_bimap_err() {
        let result: Result<i32, &str> = Err("error");
        let mapped = result.bimap(|x| x * 2, str::len);
        assert_eq!(mapped, Err(5));
    }

    #[test]
    fn test_result_map_left_ok() {
        let result: Result<i32, &str> = Ok(42);
        let mapped = result.map_left(|x| x * 2);
        assert_eq!(mapped, Ok(84));
    }

    #[test]
    fn test_result_map_left_err() {
        let result: Result<i32, &str> = Err("error");
        let mapped = result.map_left(|x| x * 2);
        assert_eq!(mapped, Err("error"));
    }

    #[test]
    fn test_result_map_right_ok() {
        let result: Result<i32, &str> = Ok(42);
        let mapped = result.map_right(str::len);
        assert_eq!(mapped, Ok(42));
    }

    #[test]
    fn test_result_map_right_err() {
        let result: Result<i32, &str> = Err("error");
        let mapped = result.map_right(str::len);
        assert_eq!(mapped, Err(5));
    }

    #[test]
    fn test_tuple_bimap() {
        let tuple = (10, "hello");
        let mapped = tuple.bimap(|x| x * 2, |s: &str| s.len());
        assert_eq!(mapped, (20, 5));
    }

    #[test]
    fn test_tuple_map_left() {
        let tuple = (10i32, "hello");
        let mapped = tuple.map_left(|x| x * 2);
        assert_eq!(mapped, (20, "hello"));
    }

    #[test]
    fn test_tuple_map_right() {
        let tuple = (10, "hello");
        let mapped = tuple.map_right(|s: &str| s.len());
        assert_eq!(mapped, (10, 5));
    }

    #[test]
    fn test_bifunctor_identity_law() {
        // bimap(id, id) == id
        let result: Result<i32, &str> = Ok(42);
        let mapped = result.bimap(|x| x, |e| e);
        assert_eq!(mapped, Ok(42));

        let tuple = (10, "hello");
        let mapped = tuple.bimap(|x| x, |s| s);
        assert_eq!(mapped, (10, "hello"));
    }

    #[test]
    fn test_bifunctor_composition_law() {
        // bimap(f, g).bimap(h, i) == bimap(h . f, i . g)
        let tuple = (10, 20);

        let f = |x: i32| x + 1;
        let g = |x: i32| x * 2;
        let h = |x: i32| x * 10;
        let i = |x: i32| x - 5;

        // Composed
        let composed = tuple.bimap(f, g).bimap(h, i);

        // Single bimap with composed functions
        let single = tuple.bimap(|x| h(f(x)), |x| i(g(x)));

        assert_eq!(composed, single);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_complex_types() {
        // Test with more complex types
        let result: Result<Vec<i32>, String> = Ok(alloc::vec![1, 2, 3]);
        let mapped = result.bimap(|v| v.iter().sum::<i32>(), |e| format!("Error: {e}"));
        assert_eq!(mapped, Ok(6));

        let err_result: Result<Vec<i32>, String> = Err("oops".into());
        let mapped = err_result.bimap(|v| v.iter().sum::<i32>(), |e| format!("Error: {e}"));
        assert_eq!(mapped, Err("Error: oops".into()));
    }

    #[test]
    fn test_nested_tuples() {
        let tuple = ((1, 2), (3, 4));
        let mapped = tuple.bimap(|(a, b)| a + b, |(c, d)| c * d);
        assert_eq!(mapped, (3, 12));
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_result_map_error_equivalent() {
        // map_right is like map_err for Result
        let result: Result<i32, &str> = Err("file not found");
        let mapped = result.map_right(|e| format!("IO Error: {e}"));
        assert_eq!(mapped, Err("IO Error: file not found".into()));
    }
}
