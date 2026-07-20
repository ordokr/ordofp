//! Natural transformations between functors.
//!
//! > *"Transformatio naturalis est morphismus inter functores."*
//! > — A natural transformation is a morphism between functors.
//!
//! This module provides [`TransformatioNaturalis`] (`FunctionK` / Natural Transformation),
//! which represents a structure-preserving transformation between functors.
//!
//! # Etymology
//!
//! - **`TransformatioNaturalis`** (Latin): "natural transformation"
//!   - *transformatio*: "a changing of form"
//!   - *naturalis*: "natural, by nature"
//!   - In category theory, these are the morphisms in the category of functors
//!
//! # Theory
//!
//! A natural transformation `η: F ~> G` is a family of morphisms:
//! ```text
//! η_A: F<A> -> G<A>
//! ```
//! that is natural in `A`, meaning for any `f: A -> B`:
//! ```text
//! G.map(f) ∘ η_A = η_B ∘ F.map(f)
//! ```
//!
//! This commutative diagram expresses that the transformation is uniform
//! across all types - it transforms the *structure*, not the *contents*.
//!
//! # Example
//!
//! ```rust
//! use ordofp_core::typeclasses::{TransformatioNaturalis, AdPrimum, AdUltimum};
//!
//! // Transform Vec to Option by taking the first element
//! let nat = AdPrimum;
//! let result = nat.transformare(vec![1, 2, 3]);
//! assert_eq!(result, Some(1));
//!
//! // Transform Vec to Option by taking the last element
//! let nat = AdUltimum;
//! let result = nat.transformare(vec![1, 2, 3]);
//! assert_eq!(result, Some(3));
//! ```

use alloc::vec;
use alloc::vec::Vec;
use core::marker::PhantomData;

/// A natural transformation from functor `F` to functor `G`.
///
/// This is a structure-preserving transformation between functors,
/// also known as `FunctionK` or `~>` in other FP libraries.
///
/// # Type Parameters
///
/// - `F`: The source functor
/// - `G`: The target functor
///
/// # Laws
///
/// Naturality: For any function `f: A -> B`:
/// ```text
/// nat.transformare(fa.map(f)) = nat.transformare(fa).map(f)
/// ```
///
/// # Example
///
/// ```rust
/// use ordofp_core::typeclasses::{TransformatioNaturalis, AdPrimum};
///
/// let nat = AdPrimum;
/// let vec = vec![10, 20, 30];
/// let opt = nat.transformare(vec);
/// assert_eq!(opt, Some(10));
/// ```
pub trait TransformatioNaturalis<F, G> {
    /// Transform a value from functor `F` to functor `G`.
    ///
    /// # Latin Etymology
    /// *transformare*: "to change in form, transform"
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::typeclasses::{TransformatioNaturalis, AdPrimum};
    ///
    /// let nat = AdPrimum;
    /// assert_eq!(nat.transformare(vec![1, 2, 3]), Some(1));
    /// assert_eq!(nat.transformare(Vec::<i32>::new()), None);
    /// ```
    fn transformare(&self, fa: F) -> G;

    /// Compose this natural transformation with another.
    ///
    /// Given `η: F ~> G` and `θ: G ~> H`, produces `θ ∘ η: F ~> H`.
    ///
    /// # Latin Etymology
    /// *componere*: "to put together, compose"
    #[inline]
    fn componere<H, N>(self, other: N) -> Compositio<F, G, H, Self, N>
    where
        Self: Sized,
        N: TransformatioNaturalis<G, H>,
    {
        Compositio {
            first: self,
            second: other,
            _phantom: PhantomData,
        }
    }

    /// Alias for `componere` - compose with another transformation.
    ///
    /// This transformation is applied first, then the other.
    #[inline]
    fn and_then<H, N>(self, other: N) -> Compositio<F, G, H, Self, N>
    where
        Self: Sized,
        N: TransformatioNaturalis<G, H>,
    {
        self.componere(other)
    }
}

/// Composition of two natural transformations.
///
/// Represents `θ ∘ η` where `η: F ~> G` and `θ: G ~> H`.
///
/// **Name collision warning:** this struct shares the name `Compositio`
/// with the *Semigroup trait* in `typeclasses::semigroup`. At the
/// `typeclasses` module root the trait wins (it is re-exported explicitly,
/// shadowing this glob re-export) — to use this struct, path it explicitly
/// as `typeclasses::natural_transformation::Compositio`.
pub struct Compositio<F, G, H, N1, N2>
where
    N1: TransformatioNaturalis<F, G>,
    N2: TransformatioNaturalis<G, H>,
{
    first: N1,
    second: N2,
    _phantom: PhantomData<(F, G, H)>,
}

impl<F, G, H, N1, N2> TransformatioNaturalis<F, H> for Compositio<F, G, H, N1, N2>
where
    N1: TransformatioNaturalis<F, G>,
    N2: TransformatioNaturalis<G, H>,
{
    #[inline]
    fn transformare(&self, fa: F) -> H {
        let g = self.first.transformare(fa);
        self.second.transformare(g)
    }
}

// ============================================================================
// Identity natural transformation
// ============================================================================

/// The identity natural transformation for any functor.
///
/// # Latin Etymology
/// *Identitas*: "sameness, identity"
pub struct IdentitasNat<F>(PhantomData<F>);

impl<F> IdentitasNat<F> {
    /// Create a new identity natural transformation.
    #[inline]
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<F> Default for IdentitasNat<F> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<F> TransformatioNaturalis<F, F> for IdentitasNat<F> {
    #[inline]
    fn transformare(&self, fa: F) -> F {
        fa
    }
}

/// Create an identity natural transformation.
#[inline]
pub fn identitas_nat<F>() -> IdentitasNat<F> {
    IdentitasNat::new()
}

// ============================================================================
// Vec to Option transformations
// ============================================================================

/// Natural transformation from `Vec` to `Option` by taking the first element.
///
/// # Latin Etymology
/// *`AdPrimum`*: "to the first"
///
/// # Example
///
/// ```rust
/// use ordofp_core::typeclasses::{TransformatioNaturalis, AdPrimum};
///
/// let nat = AdPrimum;
/// assert_eq!(nat.transformare(vec![1, 2, 3]), Some(1));
/// assert_eq!(nat.transformare(Vec::<i32>::new()), None);
/// ```
pub struct AdPrimum;

impl<T> TransformatioNaturalis<Vec<T>, Option<T>> for AdPrimum {
    #[inline]
    fn transformare(&self, fa: Vec<T>) -> Option<T> {
        fa.into_iter().next()
    }
}

/// Natural transformation from `Vec` to `Option` by taking the last element.
///
/// # Latin Etymology
/// *`AdUltimum`*: "to the last"
///
/// # Example
///
/// ```rust
/// use ordofp_core::typeclasses::{TransformatioNaturalis, AdUltimum};
///
/// let nat = AdUltimum;
/// assert_eq!(nat.transformare(vec![1, 2, 3]), Some(3));
/// assert_eq!(nat.transformare(Vec::<i32>::new()), None);
/// ```
pub struct AdUltimum;

impl<T> TransformatioNaturalis<Vec<T>, Option<T>> for AdUltimum {
    #[inline]
    fn transformare(&self, fa: Vec<T>) -> Option<T> {
        fa.into_iter().last()
    }
}

/// Natural transformation from `Vec` to `Option` by taking the nth element.
///
/// # Latin Etymology
/// *`AdNumerum`*: "to the number"
///
/// # Example
///
/// ```rust
/// use ordofp_core::typeclasses::{TransformatioNaturalis, AdNumerum};
///
/// let nat = AdNumerum(1);
/// assert_eq!(nat.transformare(vec![10, 20, 30]), Some(20));
/// assert_eq!(nat.transformare(vec![10]), None);
/// ```
pub struct AdNumerum(pub usize);

impl<T> TransformatioNaturalis<Vec<T>, Option<T>> for AdNumerum {
    #[inline]
    fn transformare(&self, fa: Vec<T>) -> Option<T> {
        fa.into_iter().nth(self.0)
    }
}

// ============================================================================
// Vec to Result transformations
// ============================================================================

/// Natural transformation from `Vec` to `Result` by taking the first element.
///
/// Returns `Err(E)` if the vec is empty.
///
/// # Latin Etymology
/// *`AdPrimumAut`*: "to the first, or (else error)"
///
/// # Example
///
/// ```rust
/// use ordofp_core::typeclasses::{TransformatioNaturalis, AdPrimumAut};
///
/// let nat = AdPrimumAut("empty");
/// assert_eq!(nat.transformare(vec![1, 2, 3]), Ok(1));
/// assert_eq!(nat.transformare(Vec::<i32>::new()), Err("empty"));
/// ```
pub struct AdPrimumAut<E>(pub E);

impl<T, E: Clone> TransformatioNaturalis<Vec<T>, Result<T, E>> for AdPrimumAut<E> {
    #[inline]
    fn transformare(&self, fa: Vec<T>) -> Result<T, E> {
        fa.into_iter().next().ok_or_else(|| self.0.clone())
    }
}

/// Natural transformation from `Vec` to `Result` by taking the last element.
///
/// # Latin Etymology
/// *`AdUltimumAut`*: "to the last, or (else error)"
pub struct AdUltimumAut<E>(pub E);

impl<T, E: Clone> TransformatioNaturalis<Vec<T>, Result<T, E>> for AdUltimumAut<E> {
    #[inline]
    fn transformare(&self, fa: Vec<T>) -> Result<T, E> {
        fa.into_iter().last().ok_or_else(|| self.0.clone())
    }
}

/// Natural transformation from `Vec` to `Result` by taking the nth element.
///
/// # Latin Etymology
/// *`AdNumerumAut`*: "to the number, or (else error)"
pub struct AdNumerumAut<E>(pub usize, pub E);

impl<T, E: Clone> TransformatioNaturalis<Vec<T>, Result<T, E>> for AdNumerumAut<E> {
    #[inline]
    fn transformare(&self, fa: Vec<T>) -> Result<T, E> {
        fa.into_iter().nth(self.0).ok_or_else(|| self.1.clone())
    }
}

// ============================================================================
// Option transformations
// ============================================================================

/// Natural transformation from `Option` to `Vec`.
///
/// # Latin Etymology
/// *`OptionAdVec`*: "Option to Vec"
///
/// # Example
///
/// ```rust
/// use ordofp_core::typeclasses::{TransformatioNaturalis, OptionAdVec};
///
/// let nat = OptionAdVec;
/// assert_eq!(nat.transformare(Some(42)), vec![42]);
/// assert_eq!(nat.transformare(None::<i32>), vec![]);
/// ```
pub struct OptionAdVec;

impl<T> TransformatioNaturalis<Option<T>, Vec<T>> for OptionAdVec {
    #[inline]
    fn transformare(&self, fa: Option<T>) -> Vec<T> {
        match fa {
            Some(x) => vec![x],
            None => Vec::new(),
        }
    }
}

/// Natural transformation from `Result` to `Vec`.
///
/// # Latin Etymology
/// *`ResultAdVec`*: "Result to Vec"
pub struct ResultAdVec;

impl<T, E> TransformatioNaturalis<Result<T, E>, Vec<T>> for ResultAdVec {
    #[inline]
    fn transformare(&self, fa: Result<T, E>) -> Vec<T> {
        match fa {
            Ok(x) => vec![x],
            Err(_) => Vec::new(),
        }
    }
}

/// Natural transformation from `Result` to `Option`, discarding the error.
///
/// # Latin Etymology
/// *`ResultAdOption`*: "Result to Option"
pub struct ResultAdOption;

impl<T, E> TransformatioNaturalis<Result<T, E>, Option<T>> for ResultAdOption {
    #[inline]
    fn transformare(&self, fa: Result<T, E>) -> Option<T> {
        fa.ok()
    }
}

// ============================================================================
// Function as natural transformation
// ============================================================================

/// Blanket implementation: any function `Fn(F) -> G` is a natural transformation.
impl<F, G, Func> TransformatioNaturalis<F, G> for Func
where
    Func: Fn(F) -> G,
{
    #[inline]
    fn transformare(&self, fa: F) -> G {
        self(fa)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ad_primum() {
        let nat = AdPrimum;
        assert_eq!(nat.transformare(vec![1, 2, 3]), Some(1));
        assert_eq!(nat.transformare(Vec::<i32>::new()), None);
    }

    #[test]
    fn test_ad_ultimum() {
        let nat = AdUltimum;
        assert_eq!(nat.transformare(vec![1, 2, 3]), Some(3));
        assert_eq!(nat.transformare(Vec::<i32>::new()), None);
    }

    #[test]
    fn test_ad_numerum() {
        let nat = AdNumerum(1);
        assert_eq!(nat.transformare(vec![10, 20, 30]), Some(20));
        assert_eq!(nat.transformare(vec![10]), None);
    }

    #[test]
    fn test_ad_primum_aut() {
        let nat = AdPrimumAut("empty");
        assert_eq!(nat.transformare(vec![1, 2, 3]), Ok(1));
        assert_eq!(nat.transformare(Vec::<i32>::new()), Err("empty"));
    }

    #[test]
    fn test_option_ad_vec() {
        let nat = OptionAdVec;
        assert_eq!(nat.transformare(Some(42)), vec![42]);
        assert_eq!(nat.transformare(None::<i32>), Vec::<i32>::new());
    }

    #[test]
    fn test_result_ad_option() {
        let nat = ResultAdOption;
        assert_eq!(nat.transformare(Ok::<_, ()>(42)), Some(42));
        assert_eq!(nat.transformare(Err::<i32, _>("error")), None);
    }

    #[test]
    fn test_composition() {
        // Compose: Vec -> Option -> Vec
        let to_option = AdPrimum;
        let to_vec = OptionAdVec;
        let composed = to_option.and_then(to_vec);

        assert_eq!(composed.transformare(vec![1, 2, 3]), vec![1]);
        assert_eq!(composed.transformare(Vec::<i32>::new()), Vec::<i32>::new());
    }

    #[test]
    fn test_identity() {
        let id: IdentitasNat<Vec<i32>> = identitas_nat();
        assert_eq!(id.transformare(vec![1, 2, 3]), vec![1, 2, 3]);
    }

    #[test]
    fn test_function_as_nat() {
        // Any function can be used as a natural transformation
        let f = |v: Vec<i32>| -> Option<i32> { v.into_iter().max() };

        assert_eq!(f.transformare(vec![1, 5, 3]), Some(5));
        assert_eq!(f.transformare(vec![]), None);
    }

    #[test]
    fn test_naturality_ad_primum() {
        // Verify naturality: nat.transformare(fa.map(f)) = nat.transformare(fa).map(f)
        let nat = AdPrimum;
        let vec = vec![1, 2, 3];
        let f = |x: i32| x * 2;

        // Left side: transform then map
        let left: Option<i32> = nat.transformare(vec.clone()).map(f);

        // Right side: map then transform
        let mapped: Vec<i32> = vec.into_iter().map(f).collect();
        let right = nat.transformare(mapped);

        assert_eq!(left, right);
    }
}
