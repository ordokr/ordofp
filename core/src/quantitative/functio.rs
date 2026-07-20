//! `FunctioLinearis` - Linear function types
//!
//! > *"Functio semel applicanda"*
//! > — A function to be applied once. (Neo-Latin)
//!
//! This module provides linear function types corresponding to linear logic's
//! linear implication (lollipop: ⊸).

extern crate alloc;

use alloc::boxed::Box;
use core::fmt;

// =============================================================================
// FunctioLinearis - Linear Function (⊸)
// =============================================================================

/// A linear function that must be called exactly once.
///
/// In linear logic, `A ⊸ B` (linear implication, "lollipop") represents
/// a function that consumes its argument exactly once to produce a result.
///
/// # Latin Etymology
///
/// *Functio* = performance, function
/// *Linearis* = linear
///
/// # Example
///
/// ```rust
/// use ordofp_core::quantitative::FunctioLinearis;
///
/// let double = FunctioLinearis::new(|x: i32| x * 2);
/// let result = double.apply(21);
/// assert_eq!(result, 42);
///
/// // double is now consumed and cannot be used again
/// ```
pub struct FunctioLinearis<A, B> {
    f: Box<dyn FnOnce(A) -> B + Send>,
}

impl<A, B> FunctioLinearis<A, B> {
    /// Create a new linear function.
    ///
    /// The function must be called exactly once.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::quantitative::FunctioLinearis;
    ///
    /// let f = FunctioLinearis::new(|x: i32| x + 1);
    /// ```
    #[inline]
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce(A) -> B + Send + 'static,
    {
        FunctioLinearis { f: Box::new(f) }
    }

    /// Apply the linear function to an argument.
    ///
    /// This consumes the function.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::quantitative::FunctioLinearis;
    ///
    /// let f = FunctioLinearis::new(|x: i32| x * 2);
    /// let result = f.apply(21);
    /// assert_eq!(result, 42);
    /// ```
    #[inline]
    pub fn apply(self, a: A) -> B {
        (self.f)(a)
    }

    /// Compose two linear functions.
    ///
    /// Creates `g ∘ f` where `f` is applied first, then `g`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::quantitative::FunctioLinearis;
    ///
    /// let f = FunctioLinearis::new(|x: i32| x + 1);
    /// let g = FunctioLinearis::new(|x: i32| x * 2);
    ///
    /// let composed = f.compose(g); // g(f(x))
    /// let result = composed.apply(5);
    /// assert_eq!(result, 12); // (5 + 1) * 2
    /// ```
    #[inline]
    pub fn compose<C>(self, g: FunctioLinearis<B, C>) -> FunctioLinearis<A, C>
    where
        A: 'static,
        B: 'static,
        C: 'static,
    {
        FunctioLinearis::new(move |a: A| g.apply(self.apply(a)))
    }

    /// Compose in the other direction.
    ///
    /// Creates `f ∘ g` where `g` is applied first, then `f`.
    #[inline]
    pub fn and_then<C>(self, g: FunctioLinearis<B, C>) -> FunctioLinearis<A, C>
    where
        A: 'static,
        B: 'static,
        C: 'static,
    {
        self.compose(g)
    }

    /// Map over the output of the function.
    #[inline]
    pub fn map_output<C, F>(self, f: F) -> FunctioLinearis<A, C>
    where
        F: FnOnce(B) -> C + Send + 'static,
        A: 'static,
        B: 'static,
    {
        FunctioLinearis::new(move |a: A| f(self.apply(a)))
    }

    /// Contramap over the input of the function.
    #[inline]
    pub fn contramap_input<Z, F>(self, f: F) -> FunctioLinearis<Z, B>
    where
        F: FnOnce(Z) -> A + Send + 'static,
        A: 'static,
        B: 'static,
    {
        FunctioLinearis::new(move |z: Z| self.apply(f(z)))
    }
}

impl<A: Send + 'static> FunctioLinearis<A, A> {
    /// Create an identity linear function.
    pub fn identity() -> Self {
        FunctioLinearis::new(|a: A| a)
    }
}

impl<A: 'static, B: 'static> fmt::Debug for FunctioLinearis<A, B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "FunctioLinearis<{} ⊸ {}>",
            core::any::type_name::<A>(),
            core::any::type_name::<B>()
        )
    }
}

// =============================================================================
// FunctioSemel - Type Alias
// =============================================================================

/// Type alias emphasizing the linear (once) nature of the function.
pub type FunctioSemel<A, B> = FunctioLinearis<A, B>;

// =============================================================================
// Free Functions
// =============================================================================

/// Apply a linear function to a value.
///
/// This is the application rule for linear implication.
///
/// # Example
///
/// ```rust
/// use ordofp_core::quantitative::{FunctioLinearis, linear_apply};
///
/// let f = FunctioLinearis::new(|x: i32| x * 2);
/// let result = linear_apply(f, 21);
/// assert_eq!(result, 42);
/// ```
#[inline]
pub fn linear_apply<A, B>(f: FunctioLinearis<A, B>, a: A) -> B {
    f.apply(a)
}

/// Compose two linear functions (g after f).
///
/// # Example
///
/// ```rust
/// use ordofp_core::quantitative::{FunctioLinearis, linear_compose};
///
/// let f = FunctioLinearis::new(|x: i32| x + 1);
/// let g = FunctioLinearis::new(|x: i32| x * 2);
///
/// let h = linear_compose(f, g);
/// assert_eq!(h.apply(5), 12); // (5 + 1) * 2
/// ```
#[inline]
pub fn linear_compose<A: 'static, B: 'static, C: 'static>(
    f: FunctioLinearis<A, B>,
    g: FunctioLinearis<B, C>,
) -> FunctioLinearis<A, C> {
    f.compose(g)
}

/// Flip the arguments of a curried linear function.
///
/// Transforms `A ⊸ (B ⊸ C)` to `B ⊸ (A ⊸ C)`.
#[inline]
pub fn linear_flip<A: 'static + Send, B: 'static + Send, C: 'static + Send>(
    f: FunctioLinearis<A, FunctioLinearis<B, C>>,
) -> FunctioLinearis<B, FunctioLinearis<A, C>> {
    FunctioLinearis::new(move |b: B| {
        FunctioLinearis::new(move |a: A| {
            let inner: FunctioLinearis<B, C> = f.apply(a);
            inner.apply(b)
        })
    })
}

/// Create a constant linear function.
///
/// Returns a function that ignores its argument and returns the given value.
#[inline]
pub fn linear_const<A: 'static, B: Send + 'static>(b: B) -> FunctioLinearis<A, B> {
    FunctioLinearis::new(move |_: A| b)
}

/// Curry a binary linear function.
///
/// Transforms `(A, B) ⊸ C` to `A ⊸ (B ⊸ C)`.
#[inline]
pub fn linear_curry<A: 'static + Send, B: 'static + Send, C: 'static + Send>(
    f: FunctioLinearis<(A, B), C>,
) -> FunctioLinearis<A, FunctioLinearis<B, C>> {
    FunctioLinearis::new(move |a: A| FunctioLinearis::new(move |b: B| f.apply((a, b))))
}

/// Uncurry a curried linear function.
///
/// Transforms `A ⊸ (B ⊸ C)` to `(A, B) ⊸ C`.
#[inline]
pub fn linear_uncurry<A: 'static + Send, B: 'static + Send, C: 'static + Send>(
    f: FunctioLinearis<A, FunctioLinearis<B, C>>,
) -> FunctioLinearis<(A, B), C> {
    FunctioLinearis::new(move |(a, b): (A, B)| {
        let inner: FunctioLinearis<B, C> = f.apply(a);
        inner.apply(b)
    })
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::format;
    use alloc::string::ToString;

    #[test]
    fn test_functio_linearis_new_apply() {
        let f = FunctioLinearis::new(|x: i32| x * 2);
        let result = f.apply(21);
        assert_eq!(result, 42);
    }

    #[test]
    fn test_functio_linearis_compose() {
        let f = FunctioLinearis::new(|x: i32| x + 1);
        let g = FunctioLinearis::new(|x: i32| x * 2);

        let composed = f.compose(g);
        let result = composed.apply(5);
        assert_eq!(result, 12); // (5 + 1) * 2
    }

    #[test]
    fn test_functio_linearis_and_then() {
        let f = FunctioLinearis::new(|x: i32| x + 1);
        let g = FunctioLinearis::new(|x: i32| x * 2);

        let chained = f.and_then(g);
        let result = chained.apply(5);
        assert_eq!(result, 12);
    }

    #[test]
    fn test_functio_linearis_map_output() {
        let f = FunctioLinearis::new(|x: i32| x + 1);
        let mapped = f.map_output(|y| y.to_string());
        assert_eq!(mapped.apply(41), "42");
    }

    #[test]
    fn test_functio_linearis_contramap_input() {
        let f = FunctioLinearis::new(|x: i32| x * 2);
        let contramapped = f.contramap_input(|s: &str| {
            s.parse::<i32>()
                .expect("test input should be a valid integer string")
        });
        assert_eq!(contramapped.apply("21"), 42);
    }

    #[test]
    fn test_functio_linearis_identity() {
        let id: FunctioLinearis<i32, i32> = FunctioLinearis::identity();
        assert_eq!(id.apply(42), 42);
    }

    #[test]
    fn test_linear_apply_fn() {
        let f = FunctioLinearis::new(|x: i32| x * 2);
        let result = linear_apply(f, 21);
        assert_eq!(result, 42);
    }

    #[test]
    fn test_linear_compose_fn() {
        let f = FunctioLinearis::new(|x: i32| x + 1);
        let g = FunctioLinearis::new(|x: i32| x * 2);

        let h = linear_compose(f, g);
        assert_eq!(h.apply(5), 12);
    }

    #[test]
    fn test_linear_const() {
        let f: FunctioLinearis<i32, &str> = linear_const("hello");
        assert_eq!(f.apply(42), "hello");
    }

    #[test]
    fn test_linear_curry() {
        let f = FunctioLinearis::new(|(a, b): (i32, i32)| a + b);
        let curried = linear_curry(f);
        let result = curried.apply(10).apply(32);
        assert_eq!(result, 42);
    }

    #[test]
    fn test_linear_uncurry() {
        let f = FunctioLinearis::new(|a: i32| FunctioLinearis::new(move |b: i32| a + b));
        let uncurried = linear_uncurry(f);
        let result = uncurried.apply((10, 32));
        assert_eq!(result, 42);
    }

    #[test]
    fn test_linear_flip() {
        let f =
            FunctioLinearis::new(|a: i32| FunctioLinearis::new(move |b: &str| format!("{b}: {a}")));
        let flipped = linear_flip(f);
        let result = flipped.apply("answer").apply(42);
        assert_eq!(result, "answer: 42");
    }

    #[test]
    fn test_debug() {
        let f: FunctioLinearis<i32, i32> = FunctioLinearis::new(|x| x);
        let debug_str = format!("{f:?}");
        assert!(debug_str.contains("FunctioLinearis"));
    }
}
