//! Linear Functor - Functor operations preserving linearity
//!
//! > *"Transformatio linearis"*
//! > — Linear transformation. (Neo-Latin)

use super::Linearis;

/// A functor that operates on linear values.
///
/// `FunctorLinearis` extends the functor concept to linear types,
/// ensuring that transformations preserve single-use semantics.
/// The key difference from regular `Functor` is that operations
/// consume their input rather than borrowing it.
///
/// # Laws
///
/// Linear functors satisfy the standard functor laws:
///
/// 1. **Identity**: `fmap_linear(id) ≡ id`
/// 2. **Composition**: `fmap_linear(f ∘ g) ≡ fmap_linear(f) ∘ fmap_linear(g)`
///
/// # Example
///
/// ```rust
/// use ordofp_core::linear::{Linearis, FunctorLinearis};
///
/// let x = Linearis::new(5);
/// let y = x.fmap_linear(|n| n * 2);
/// assert_eq!(y.consume(), 10);
/// ```
pub trait FunctorLinearis: Sized {
    /// The type of value contained in this functor.
    type Elem;

    /// The output type when mapping to type `B`.
    type Output<B>;

    /// Map a function over this linear functor, consuming it.
    ///
    /// This is the core functor operation for linear types. Unlike
    /// regular `fmap`, this consumes `self` because the value can
    /// only be used once.
    fn fmap_linear<B, F>(self, f: F) -> Self::Output<B>
    where
        F: FnOnce(Self::Elem) -> B;

    /// Replace the inner value with a constant, consuming the original.
    ///
    /// This is equivalent to `fmap_linear(|_| value)`.
    #[inline]
    fn replace_linear<B>(self, value: B) -> Self::Output<B> {
        self.fmap_linear(|_| value)
    }

    /// Void the inner value, mapping to `()`.
    #[inline]
    fn void_linear(self) -> Self::Output<()> {
        self.fmap_linear(|_| ())
    }
}

impl<T> FunctorLinearis for Linearis<T> {
    type Elem = T;
    type Output<B> = Linearis<B>;

    #[inline]
    fn fmap_linear<B, F>(self, f: F) -> Linearis<B>
    where
        F: FnOnce(T) -> B,
    {
        Linearis::new(f(self.consume()))
    }
}

impl<T> FunctorLinearis for Option<T> {
    type Elem = T;
    type Output<B> = Option<B>;

    #[inline]
    fn fmap_linear<B, F>(self, f: F) -> Option<B>
    where
        F: FnOnce(T) -> B,
    {
        self.map(f)
    }
}

impl<T, E> FunctorLinearis for Result<T, E> {
    type Elem = T;
    type Output<B> = Result<B, E>;

    #[inline]
    fn fmap_linear<B, F>(self, f: F) -> Result<B, E>
    where
        F: FnOnce(T) -> B,
    {
        self.map(f)
    }
}

/// Extension trait for linear functor operations on tuples.
pub trait FunctorLinearisTuple<A, B>: Sized {
    /// Map over the first element of a tuple.
    fn fmap_first<C, F>(self, f: F) -> (C, B)
    where
        F: FnOnce(A) -> C;

    /// Map over the second element of a tuple.
    fn fmap_second<C, F>(self, f: F) -> (A, C)
    where
        F: FnOnce(B) -> C;

    /// Map over both elements of a tuple.
    fn bimap<C, D, F, G>(self, f: F, g: G) -> (C, D)
    where
        F: FnOnce(A) -> C,
        G: FnOnce(B) -> D;
}

impl<A, B> FunctorLinearisTuple<A, B> for (A, B) {
    #[inline]
    fn fmap_first<C, F>(self, f: F) -> (C, B)
    where
        F: FnOnce(A) -> C,
    {
        (f(self.0), self.1)
    }

    #[inline]
    fn fmap_second<C, F>(self, f: F) -> (A, C)
    where
        F: FnOnce(B) -> C,
    {
        (self.0, f(self.1))
    }

    #[inline]
    fn bimap<C, D, F, G>(self, f: F, g: G) -> (C, D)
    where
        F: FnOnce(A) -> C,
        G: FnOnce(B) -> D,
    {
        (f(self.0), g(self.1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linearis_fmap_linear() {
        let x = Linearis::new(5);
        let y = x.fmap_linear(|n| n * 2);
        assert_eq!(y.consume(), 10);
    }

    #[test]
    fn test_linearis_replace_linear() {
        let x = Linearis::new(5);
        let y = x.replace_linear("replaced");
        assert_eq!(y.consume(), "replaced");
    }

    #[test]
    fn test_linearis_void_linear() {
        let x = Linearis::new(5);
        let y = x.void_linear();
        assert_eq!(y.consume(), ());
    }

    #[test]
    fn test_option_fmap_linear() {
        let x = Some(5);
        let y = x.fmap_linear(|n| n * 2);
        assert_eq!(y, Some(10));

        let none: Option<i32> = None;
        let result = none.fmap_linear(|n| n * 2);
        assert_eq!(result, None);
    }

    #[test]
    fn test_result_fmap_linear() {
        let x: Result<i32, &str> = Ok(5);
        let y = x.fmap_linear(|n| n * 2);
        assert_eq!(y, Ok(10));

        let err: Result<i32, &str> = Err("error");
        let result = err.fmap_linear(|n| n * 2);
        assert_eq!(result, Err("error"));
    }

    #[test]
    fn test_tuple_fmap_first() {
        let x = (5, "hello");
        let y = x.fmap_first(|n| n * 2);
        assert_eq!(y, (10, "hello"));
    }

    #[test]
    fn test_tuple_fmap_second() {
        let x = (5, "hello");
        let y = x.fmap_second(str::len);
        assert_eq!(y, (5, 5));
    }

    #[test]
    fn test_tuple_bimap() {
        let x = (5, "hello");
        let y = x.bimap(|n| n * 2, str::len);
        assert_eq!(y, (10, 5));
    }

    #[test]
    fn test_functor_identity_law() {
        let x = Linearis::new(42);
        let y = x.fmap_linear(|a| a);
        assert_eq!(y.consume(), 42);
    }

    #[test]
    fn test_functor_composition_law() {
        let f = |x: i32| x * 2;
        let g = |x: i32| x + 1;

        let x1 = Linearis::new(5);
        let result1 = x1.fmap_linear(|x| f(g(x)));

        let x2 = Linearis::new(5);
        let result2 = x2.fmap_linear(g).fmap_linear(f);

        assert_eq!(result1.consume(), result2.consume());
    }
}
