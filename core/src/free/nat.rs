//! Natural Transformations - Morphisms between functors
//!
//! > *"Natura non facit saltus."*
//! > — Nature does not make jumps. (Leibniz)
//!
//! A natural transformation is a mapping between functors that
//! preserves their structure. It's the key to interpreting Free monads.

use crate::typeclasses::hkt::{FunctorHKT, HKT};

// =============================================================================
// Natural Transformation
// =============================================================================

/// A natural transformation from functor F to functor G.
///
/// Given functors `F` and `G`, a natural transformation `η: F ~> G` is a
/// family of morphisms `η_A: F<A> -> G<A>` for every type `A`, such that
/// for any `f: A -> B`:
///
/// ```text
/// η_B ∘ F.map(f) = G.map(f) ∘ η_A
/// ```
///
/// This means the transformation commutes with mapping - you can either
/// transform then map, or map then transform, and get the same result.
///
/// # Latin Etymology
///
/// *Transformatio Naturalis* = Natural transformation
///
/// # Example
///
/// ```rust
/// use ordofp_core::free::{OptionFWitness, TransformatioNaturalis};
/// use ordofp_core::typeclasses::hkt::HKT;
///
/// struct VecFWitness;
/// impl HKT for VecFWitness {
///     type Target<T> = Vec<T>;
/// }
///
/// // Transform Option to Vec
/// struct OptionToVec;
///
/// impl TransformatioNaturalis<OptionFWitness, VecFWitness> for OptionToVec {
///     fn transforma<A>(fa: Option<A>) -> Vec<A> {
///         match fa {
///             Some(a) => vec![a],
///             None => vec![],
///         }
///     }
/// }
///
/// assert_eq!(OptionToVec::transforma(Some(42)), vec![42]);
/// assert_eq!(OptionToVec::transforma::<i32>(None), Vec::<i32>::new());
/// ```
pub trait TransformatioNaturalis<F: HKT, G: HKT> {
    /// Transform a value from functor F to functor G.
    fn transforma<A>(fa: F::Target<A>) -> G::Target<A>;
}

/// Function-based natural transformation.
///
/// For simple cases where the transformation can be expressed as a function.
pub struct TransformatioFn<F: HKT, G: HKT, Func> {
    f: Func,
    _f: core::marker::PhantomData<F>,
    _g: core::marker::PhantomData<G>,
}

impl<F: HKT, G: HKT, Func> TransformatioFn<F, G, Func> {
    /// Create a new function-based natural transformation.
    #[inline]
    pub fn new(f: Func) -> Self {
        TransformatioFn {
            f,
            _f: core::marker::PhantomData,
            _g: core::marker::PhantomData,
        }
    }

    /// Apply the transformation to a value in functor `F`.
    #[inline]
    pub fn apply<A>(&self, fa: F::Target<A>) -> G::Target<A>
    where
        Func: Fn(F::Target<A>) -> G::Target<A>,
    {
        (self.f)(fa)
    }
}

// =============================================================================
// Identity Natural Transformation
// =============================================================================

/// The identity natural transformation `F ~> F`.
///
/// Simply passes values through unchanged.
///
/// # Latin Etymology
///
/// *Identitas* = sameness
pub struct TransformatioIdentitas<F>(core::marker::PhantomData<F>);

impl<F> Default for TransformatioIdentitas<F> {
    #[inline]
    fn default() -> Self {
        TransformatioIdentitas(core::marker::PhantomData)
    }
}

impl<F: HKT> TransformatioNaturalis<F, F> for TransformatioIdentitas<F> {
    #[inline]
    fn transforma<A>(fa: F::Target<A>) -> F::Target<A> {
        fa
    }
}

// =============================================================================
// Composition of Natural Transformations
// =============================================================================

/// Composition of two natural transformations.
///
/// Given `η: F ~> G` and `θ: G ~> H`, produces `θ ∘ η: F ~> H`.
///
/// # Latin Etymology
///
/// *Compositio* = putting together
pub struct TransformatioCompositio<F, G, H, Eta, Theta> {
    _eta: core::marker::PhantomData<Eta>,
    _theta: core::marker::PhantomData<Theta>,
    _f: core::marker::PhantomData<F>,
    _g: core::marker::PhantomData<G>,
    _h: core::marker::PhantomData<H>,
}

impl<F: HKT, G: HKT, H: HKT, Eta, Theta> TransformatioNaturalis<F, H>
    for TransformatioCompositio<F, G, H, Eta, Theta>
where
    Eta: TransformatioNaturalis<F, G>,
    Theta: TransformatioNaturalis<G, H>,
{
    #[inline]
    fn transforma<A>(fa: F::Target<A>) -> H::Target<A> {
        let ga: G::Target<A> = Eta::transforma(fa);
        Theta::transforma(ga)
    }
}

// =============================================================================
// Common Functor Witnesses for Natural Transformations
// =============================================================================

/// Witness for Option as an HKT.
#[derive(Clone)]
pub struct OptionFWitness;

impl HKT for OptionFWitness {
    type Target<T> = Option<T>;
}

impl FunctorHKT for OptionFWitness {
    #[inline]
    fn map<A, B, F>(fa: Option<A>, f: F) -> Option<B>
    where
        F: FnMut(A) -> B,
    {
        fa.map(f)
    }
}

/// Witness for Result as an HKT (fixed error type).
pub struct ResultFWitness<E>(core::marker::PhantomData<E>);

impl<E> HKT for ResultFWitness<E> {
    type Target<T> = Result<T, E>;
}

impl<E: Clone> FunctorHKT for ResultFWitness<E> {
    #[inline]
    fn map<A, B, F>(fa: Result<A, E>, f: F) -> Result<B, E>
    where
        F: FnMut(A) -> B,
    {
        fa.map(f)
    }
}

/// Witness for Identity functor.
///
/// # Latin Etymology
///
/// *Identitas* = sameness
pub struct IdentitasFWitness;

impl HKT for IdentitasFWitness {
    type Target<T> = T;
}

impl FunctorHKT for IdentitasFWitness {
    #[inline]
    fn map<A, B, F>(fa: A, mut f: F) -> B
    where
        F: FnMut(A) -> B,
    {
        f(fa)
    }
}

/// Witness for Const functor (ignores type parameter).
///
/// # Latin Etymology
///
/// *Constans* = constant, unchanging
#[cfg(feature = "alloc")]
pub struct ConstFWitness<C>(core::marker::PhantomData<C>);

#[cfg(feature = "alloc")]
impl<C> HKT for ConstFWitness<C> {
    type Target<T> = C;
}

// Note: Const is not a Functor in the usual sense since map would
// require C -> C, not involving the type parameter at all.

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Example identity transformation - demonstrates how to implement natural transformations
    struct OptionIdentity;

    impl TransformatioNaturalis<OptionFWitness, OptionFWitness> for OptionIdentity {
        fn transforma<A>(fa: Option<A>) -> Option<A> {
            fa
        }
    }

    #[test]
    fn test_identity_transformation() {
        let opt = Some(42);
        let result: Option<i32> = TransformatioIdentitas::<OptionFWitness>::transforma(opt);
        assert_eq!(result, Some(42));
        let by_example: Option<i32> = OptionIdentity::transforma(Some(42));
        assert_eq!(by_example, Some(42));
    }

    #[test]
    fn test_identity_on_none() {
        let opt: Option<i32> = None;
        let result: Option<i32> = TransformatioIdentitas::<OptionFWitness>::transforma(opt);
        assert_eq!(result, None);
    }

    #[test]
    fn test_result_witness() {
        let ok: Result<i32, &str> = Ok(42);
        let mapped: Result<i32, &str> = ResultFWitness::<&str>::map(ok, |x| x * 2);
        assert_eq!(mapped, Ok(84));
    }

    #[test]
    fn test_identity_witness() {
        let value = 42;
        let mapped = IdentitasFWitness::map(value, |x| x * 2);
        assert_eq!(mapped, 84);
    }

    #[test]
    fn test_composition_of_natural_transformations() {
        // Composing identity ∘ identity should still be identity for both Some and None.
        // This exercises `TransformatioCompositio`, which was previously untested.
        type Id = TransformatioIdentitas<OptionFWitness>;
        type Composed =
            TransformatioCompositio<OptionFWitness, OptionFWitness, OptionFWitness, Id, Id>;

        let result: Option<i32> = <Composed as TransformatioNaturalis<
            OptionFWitness,
            OptionFWitness,
        >>::transforma(Some(7));
        assert_eq!(
            result,
            Some(7),
            "composed identity must preserve Some value"
        );

        let result: Option<i32> =
            <Composed as TransformatioNaturalis<OptionFWitness, OptionFWitness>>::transforma(None);
        assert_eq!(result, None, "composed identity must preserve None");
    }
}
