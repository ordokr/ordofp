//! Linear Monad - Monadic operations preserving linearity
//!
//! > *"Ligatio linearis"*
//! > — Linear binding. (Neo-Latin)

use super::{FunctorLinearis, Linearis};

/// A monad that operates on linear values.
///
/// `MonadLinearis` extends the monad concept to linear types,
/// providing `bind` (`flat_map`) operations that preserve single-use semantics.
///
/// # Laws
///
/// Linear monads satisfy the standard monad laws:
///
/// 1. **Left Identity**: `pure(a).bind(f) ≡ f(a)`
/// 2. **Right Identity**: `m.bind(pure) ≡ m`
/// 3. **Associativity**: `m.bind(f).bind(g) ≡ m.bind(|x| f(x).bind(g))`
///
/// # Example
///
/// ```rust
/// use ordofp_core::linear::{Linearis, MonadLinearis};
///
/// let x = Linearis::purus_linear(5);
/// let y = x.bind_linear(|n| Linearis::purus_linear(n * 2));
/// assert_eq!(y.consume(), 10);
/// ```
pub trait MonadLinearis: FunctorLinearis {
    /// Wrap a value in the monadic context.
    ///
    /// This is the `return` or `pure` operation for linear monads.
    fn purus_linear(value: Self::Elem) -> Self;

    /// Sequentially compose two linear computations.
    ///
    /// This is the `>>=` (bind) operation. It takes a linear value
    /// and a function that produces a new linear value, chaining them.
    fn bind_linear<B, F>(self, f: F) -> Self::Output<B>
    where
        F: FnOnce(Self::Elem) -> Self::Output<B>;

    /// Flatten a nested linear monad.
    ///
    /// Converts `M<M<A>>` to `M<A>`.
    #[inline]
    fn flatten_linear(self) -> Self::Output<Self::Elem>
    where
        Self::Elem: Into<Self::Output<Self::Elem>>,
    {
        self.bind_linear(core::convert::Into::into)
    }
}

impl<T> MonadLinearis for Linearis<T> {
    #[inline]
    fn purus_linear(value: T) -> Self {
        Linearis::new(value)
    }

    #[inline]
    fn bind_linear<B, F>(self, f: F) -> Linearis<B>
    where
        F: FnOnce(T) -> Linearis<B>,
    {
        f(self.consume())
    }
}

impl<T> MonadLinearis for Option<T> {
    #[inline]
    fn purus_linear(value: T) -> Self {
        Some(value)
    }

    #[inline]
    fn bind_linear<B, F>(self, f: F) -> Option<B>
    where
        F: FnOnce(T) -> Option<B>,
    {
        self.and_then(f)
    }
}

impl<T, E> MonadLinearis for Result<T, E> {
    #[inline]
    fn purus_linear(value: T) -> Self {
        Ok(value)
    }

    #[inline]
    fn bind_linear<B, F>(self, f: F) -> Result<B, E>
    where
        F: FnOnce(T) -> Result<B, E>,
    {
        self.and_then(f)
    }
}

/// Trait for binding operations on linear values.
///
/// This is a separate trait to allow for more flexible implementations
/// and to make the binding operation more visible in code.
pub trait BindLinearis<A>: Sized {
    /// The output type after binding.
    type Output<B>;

    /// Bind a linear function to this value.
    fn bind<B, F>(self, f: F) -> Self::Output<B>
    where
        F: FnOnce(A) -> Self::Output<B>;

    /// Sequence two computations, discarding the result of the first.
    #[inline]
    fn then<B>(self, other: Self::Output<B>) -> Self::Output<B>
    where
        Self::Output<B>: Sized,
    {
        self.bind(|_| other)
    }
}

impl<A> BindLinearis<A> for Linearis<A> {
    type Output<B> = Linearis<B>;

    #[inline]
    fn bind<B, F>(self, f: F) -> Linearis<B>
    where
        F: FnOnce(A) -> Linearis<B>,
    {
        f(self.consume())
    }
}

impl<A> BindLinearis<A> for Option<A> {
    type Output<B> = Option<B>;

    #[inline]
    fn bind<B, F>(self, f: F) -> Option<B>
    where
        F: FnOnce(A) -> Option<B>,
    {
        self.and_then(f)
    }
}

impl<A, E> BindLinearis<A> for Result<A, E> {
    type Output<B> = Result<B, E>;

    #[inline]
    fn bind<B, F>(self, f: F) -> Result<B, E>
    where
        F: FnOnce(A) -> Result<B, E>,
    {
        self.and_then(f)
    }
}

/// Kleisli composition for linear monads.
///
/// Composes two functions that return linear monads.
///
/// # Example
///
/// ```rust
/// use ordofp_core::linear::{Linearis, kleisli_compose_linear};
///
/// let f = |x: i32| Linearis::new(x + 1);
/// let g = |x: i32| Linearis::new(x * 2);
///
/// let composed = kleisli_compose_linear(f, g);
/// let result = composed(5);
/// assert_eq!(result.consume(), 12); // (5 + 1) * 2
/// ```
#[inline]
pub fn kleisli_compose_linear<A, B, C, F, G>(f: F, g: G) -> impl FnOnce(A) -> Linearis<C>
where
    F: FnOnce(A) -> Linearis<B>,
    G: FnOnce(B) -> Linearis<C>,
{
    move |a| f(a).bind_linear(g)
}

/// Join two nested linear values.
///
/// Flattens `Linearis<Linearis<A>>` to `Linearis<A>`.
#[inline]
pub fn join_linear<A>(nested: Linearis<Linearis<A>>) -> Linearis<A> {
    nested.consume()
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    #[test]
    fn test_linearis_purus_linear() {
        let x = Linearis::purus_linear(42);
        assert_eq!(x.consume(), 42);
    }

    #[test]
    fn test_linearis_bind_linear() {
        let x = Linearis::new(5);
        let y = x.bind_linear(|n| Linearis::new(n * 2));
        assert_eq!(y.consume(), 10);
    }

    #[test]
    fn test_option_monad_linear() {
        let x = Option::purus_linear(5);
        let y = x.bind_linear(|n| Some(n * 2));
        assert_eq!(y, Some(10));

        let none: Option<i32> = None;
        let result = none.bind_linear(|n| Some(n * 2));
        assert_eq!(result, None);
    }

    #[test]
    fn test_result_monad_linear() {
        let x: Result<i32, &str> = Result::purus_linear(5);
        let y = x.bind_linear(|n| Ok(n * 2));
        assert_eq!(y, Ok(10));
    }

    #[test]
    fn test_bind_trait() {
        let x = Linearis::new(5);
        let y = x.bind(|n| Linearis::new(n + 10));
        assert_eq!(y.consume(), 15);
    }

    #[test]
    fn test_then() {
        let x = Linearis::new(5);
        let y = x.then(Linearis::new("done"));
        assert_eq!(y.consume(), "done");
    }

    #[test]
    fn test_kleisli_compose() {
        let f = |x: i32| Linearis::new(x + 1);
        let g = |x: i32| Linearis::new(x * 2);

        let composed = kleisli_compose_linear(f, g);
        let result = composed(5);
        assert_eq!(result.consume(), 12); // (5 + 1) * 2
    }

    #[test]
    fn test_join_linear() {
        let nested = Linearis::new(Linearis::new(42));
        let flat = join_linear(nested);
        assert_eq!(flat.consume(), 42);
    }

    // Monad Laws
    #[test]
    fn test_left_identity() {
        let a = 5;
        let f = |x: i32| Linearis::new(x * 2);

        let left = Linearis::purus_linear(a).bind_linear(f);
        let right = f(a);

        assert_eq!(left.consume(), right.consume());
    }

    #[test]
    fn test_right_identity() {
        let m = Linearis::new(5);
        let m_clone = Linearis::new(5);

        let result = m.bind_linear(Linearis::purus_linear);

        assert_eq!(result.consume(), m_clone.consume());
    }

    #[test]
    fn test_associativity() {
        let _m = Linearis::new(5);
        let f = |x: i32| Linearis::new(x + 1);
        let g = |x: i32| Linearis::new(x * 2);

        // m.bind(f).bind(g)
        let m1 = Linearis::new(5);
        let left = m1.bind_linear(f).bind_linear(g);

        // m.bind(|x| f(x).bind(g))
        let m2 = Linearis::new(5);
        let right = m2.bind_linear(|x| {
            let fx = Linearis::new(x + 1);
            fx.bind_linear(g)
        });

        assert_eq!(left.consume(), right.consume());
    }

    #[test]
    fn test_chaining() {
        let result = Linearis::new(5i32)
            .bind_linear(|x| Linearis::new(x + 1))
            .bind_linear(|x| Linearis::new(x * 2))
            .bind_linear(|x: i32| Linearis::new(x.to_string()))
            .consume();

        assert_eq!(result, "12");
    }
}
