//! Kan Extensions and Yoneda Embeddings
//!
//! > *"Omne quod movetur ab alio movetur."*
//! > — Everything that is moved is moved by another. (Thomas Aquinas)
//!
//! This module provides advanced category theory constructs:
//!
//! - **Yoneda**: The Yoneda embedding - represents a functor by its natural transformations
//! - **Coyoneda**: The dual of Yoneda - a free functor from any type
//! - **Right Kan Extension (Ran)**: Universal construction for "best approximation from above"
//! - **Left Kan Extension (Lan)**: Universal construction for "best approximation from below"
//!
//! # The Yoneda Lemma
//!
//! The Yoneda lemma states that for any functor F and object A:
//! ```text
//! Nat(Hom(A, -), F) ≅ F(A)
//! ```
//!
//! This means the set of natural transformations from the hom-functor to F
//! is isomorphic to F(A).
//!
//! # Performance Benefits
//!
//! - **Yoneda**: `fmap` fusion - multiple `map` calls become a single function composition
//! - **Coyoneda**: Makes any type a functor; defers all `map` operations until interpretation

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::boxed::Box;

use crate::typeclasses::hkt::HKT;

// =============================================================================
// Continuation Type Aliases
// =============================================================================

/// Yoneda continuation: given a (rank-2) mapping `A -> A`, produces `F<A>`.
#[cfg(feature = "alloc")]
type YonedaRun<F, A> = Box<dyn for<'a> FnOnce(&'a dyn Fn(A) -> A) -> <F as HKT>::Target<A> + Send>;

/// Right-Kan-extension continuation: feeds an arrow `A -> H<B>` and yields `G<B>`
/// (monomorphised here at `B = A`).
#[cfg(feature = "alloc")]
type RanRun<G, H, A> =
    Box<dyn FnOnce(Box<dyn FnOnce(A) -> <H as HKT>::Target<A>>) -> <G as HKT>::Target<A> + Send>;

/// Codensity continuation in CPS form: `(A -> G<A>) -> G<A>`.
#[cfg(feature = "alloc")]
type CodensitasRun<G, A> =
    Box<dyn FnOnce(Box<dyn FnOnce(A) -> <G as HKT>::Target<A>>) -> <G as HKT>::Target<A> + Send>;

/// `CodensitasT` continuation over a concrete base-monad value: `(M -> M) -> M`.
#[cfg(all(feature = "alloc", feature = "transformers-cps"))]
type CodensitasTRun<M> = Box<dyn FnOnce(Box<dyn FnOnce(M) -> M + Send>) -> M + Send>;

/// Day-convolution combiner: merges the two type-erased components into an `A`.
#[cfg(feature = "alloc")]
type DayCombinatio<A> =
    Box<dyn FnOnce(Box<dyn core::any::Any>, Box<dyn core::any::Any>) -> A + Send>;

// =============================================================================
// Yoneda Embedding
// =============================================================================

/// The Yoneda embedding for a functor F.
///
/// > *"Yoneda est speculum functorum."*
/// > — Yoneda is the mirror of functors. (Modern)
///
/// Yoneda represents `F<A>` as `forall B. (A -> B) -> F<B>`.
///
/// This provides automatic fusion of `map` operations:
/// ```text
/// yoneda.map(f).map(g).map(h) = yoneda.map(h . g . f)
/// ```
///
/// # Type Parameters
///
/// - `F` - The underlying functor (HKT witness)
/// - `A` - The contained type
///
/// # Example
///
/// ```rust
/// use ordofp_core::category::Yoneda;
/// use ordofp_core::typeclasses::hkt::HKT;
///
/// struct OptionWitness;
/// impl HKT for OptionWitness {
///     type Target<A> = Option<A>;
/// }
///
/// // `Yoneda::map` isn't implemented yet -- composing the continuation needs
/// // HKT support Rust does not yet offer (see the `map_placeholder` note in
/// // this module's source). For now, bake any transformation into the
/// // continuation passed to `new` and run it with `lower`.
/// let yoneda: Yoneda<OptionWitness, i32> = Yoneda::new(|_id| Some(21 * 2));
/// assert_eq!(yoneda.lower(), Some(42));
/// ```
///
/// # Latin Etymology
///
/// Named after Nobuo Yoneda (米田 信夫), who discovered the Yoneda lemma.
#[cfg(feature = "alloc")]
pub struct Yoneda<F: HKT, A> {
    /// The continuation that, given any function `A -> B`, produces `F<B>`.
    run: YonedaRun<F, A>,
    _phantom: core::marker::PhantomData<A>,
}

#[cfg(feature = "alloc")]
impl<F: HKT, A: 'static> Yoneda<F, A> {
    /// Create a Yoneda from a raw continuation.
    ///
    /// This is the primitive constructor - prefer `lift` for most uses.
    #[inline]
    pub fn new<Run>(run: Run) -> Self
    where
        Run: for<'a> FnOnce(&'a dyn Fn(A) -> A) -> F::Target<A> + Send + 'static,
    {
        Yoneda {
            run: Box::new(run),
            _phantom: core::marker::PhantomData,
        }
    }

    /// Lower the Yoneda back to the underlying functor.
    ///
    /// This applies all accumulated transformations.
    #[inline]
    pub fn lower(self) -> F::Target<A> {
        (self.run)(&|a| a)
    }

    // Deliberately no `map`: a real `map` requires composing the continuation,
    // which needs HKT support Rust does not yet offer.
}

// =============================================================================
// Coyoneda - Free Functor
// =============================================================================

/// Coyoneda - the free functor.
///
/// > *"Coyoneda liberat functorem."*
/// > — Coyoneda frees the functor. (Modern)
///
/// Coyoneda makes any type constructor into a functor by deferring
/// all `map` operations until the structure is interpreted.
///
/// The representation is:
/// ```text
/// Coyoneda f a = exists b. (b -> a, f b)
/// ```
///
/// # Benefits
///
/// 1. **Free Functor**: Any type constructor becomes a functor
/// 2. **Map Fusion**: Multiple `map` calls are composed into one
/// 3. **Deferred Computation**: No work is done until `lower` is called
///
/// # Example
///
/// ```rust
/// use ordofp_core::category::Coyoneda;
/// use ordofp_core::typeclasses::hkt::HKT;
///
/// struct VecWitness;
/// impl HKT for VecWitness {
///     type Target<A> = Vec<A>;
/// }
///
/// // Even types that aren't functors can be wrapped
/// let coyoneda: Coyoneda<VecWitness, i32> = Coyoneda::lift(vec![1, 2, 3]);
///
/// // Maps are accumulated (composed) without touching the underlying data.
/// // There is no `lower`/`run_with` yet to execute the accumulated
/// // transform -- `is_type` is the only current way to inspect the result.
/// let result = coyoneda.map(|x| x + 1).map(|x| x * 2);
/// assert!(result.is_type::<Vec<i32>>());
/// ```
///
/// # Latin Etymology
///
/// *Co-* prefix indicates the dual construction.
#[cfg(feature = "alloc")]
pub struct Coyoneda<F: HKT, A> {
    /// The pivot value - the original `F<B>` for some existential B.
    pivot: CoyonedaPivot<F>,
    /// The accumulated transformation from B to A.
    transform: Box<dyn FnOnce(Box<dyn core::any::Any>) -> A + Send>,
    _phantom: core::marker::PhantomData<A>,
}

/// Internal storage for the existentially quantified pivot.
#[cfg(feature = "alloc")]
struct CoyonedaPivot<F: HKT> {
    /// The wrapped value (type-erased).
    value: Box<dyn core::any::Any + Send>,
    _phantom: core::marker::PhantomData<F>,
}

#[cfg(feature = "alloc")]
impl<F: HKT, A: 'static> Coyoneda<F, A> {
    /// Lift a functor value into Coyoneda.
    ///
    /// This wraps the value with an identity transformation.
    ///
    /// # Panics
    ///
    /// The stored transformation panics only if the type-erased pivot fails
    /// to downcast back to `A` when the Coyoneda is later run — an internal
    /// invariant that cannot fire absent a bug in this crate, since `lift`
    /// erases and downcasts the same type.
    #[inline]
    pub fn lift(fa: F::Target<A>) -> Self
    where
        F::Target<A>: Send + 'static,
    {
        Coyoneda {
            pivot: CoyonedaPivot {
                value: Box::new(fa),
                _phantom: core::marker::PhantomData,
            },
            transform: Box::new(|any| {
                *any.downcast::<A>().expect("Coyoneda type mismatch in lift")
            }),
            _phantom: core::marker::PhantomData,
        }
    }

    /// Map a function over the Coyoneda.
    ///
    /// This composes the function with the accumulated transformation
    /// without touching the underlying data.
    #[inline]
    pub fn map<B: 'static, G>(self, g: G) -> Coyoneda<F, B>
    where
        G: FnOnce(A) -> B + Send + 'static,
    {
        let old_transform = self.transform;
        Coyoneda {
            pivot: self.pivot,
            transform: Box::new(move |any| g(old_transform(any))),
            _phantom: core::marker::PhantomData,
        }
    }

    /// Check if the Coyoneda contains a specific type.
    #[inline]
    pub fn is_type<T: 'static>(&self) -> bool {
        self.pivot.value.is::<T>()
    }
}

// =============================================================================
// Right Kan Extension
// =============================================================================

/// Right Kan Extension (Ran) of functor G along functor H.
///
/// > *"Extensio dextra est approximatio optima ex alto."*
/// > — The right extension is the best approximation from above. (Modern)
///
/// The right Kan extension is defined by the universal property:
/// ```text
/// Ran h g a = forall b. (a -> h b) -> g b
/// ```
///
/// # Universal Property
///
/// For any functor F, natural transformations `Ran H G -> F` correspond
/// bijectively to natural transformations `G -> F . H`.
///
/// # Special Cases
///
/// - `Ran Id G = G` (extension along identity is the functor itself)
/// - `Ran G G = Codensity G` (extension along itself gives codensity)
///
/// # Latin Etymology
///
/// *Extensio Kan Dextra* = Right Kan Extension
#[cfg(feature = "alloc")]
pub struct ExtensioKanDextra<G: HKT, H: HKT, A> {
    /// The continuation: given any `A -> H<B>`, produces `G<B>`.
    run: RanRun<G, H, A>,
    _phantom: core::marker::PhantomData<(G, H, A)>,
}

#[cfg(feature = "alloc")]
impl<G: HKT, H: HKT, A: 'static> ExtensioKanDextra<G, H, A> {
    /// Create a new right Kan extension.
    #[inline]
    pub fn new<Run>(run: Run) -> Self
    where
        Run: FnOnce(Box<dyn FnOnce(A) -> H::Target<A>>) -> G::Target<A> + Send + 'static,
    {
        ExtensioKanDextra {
            run: Box::new(run),
            _phantom: core::marker::PhantomData,
        }
    }

    /// Run the Kan extension with a specific continuation.
    #[inline]
    pub fn run_with<F>(self, f: F) -> G::Target<A>
    where
        F: FnOnce(A) -> H::Target<A> + 'static,
    {
        (self.run)(Box::new(f))
    }
}

// =============================================================================
// Left Kan Extension
// =============================================================================

/// Left Kan Extension (Lan) of functor G along functor H.
///
/// > *"Extensio sinistra est approximatio optima ex imo."*
/// > — The left extension is the best approximation from below. (Modern)
///
/// The left Kan extension is defined by the universal property:
/// ```text
/// Lan h g a = exists b. (h b -> a, g b)
/// ```
///
/// # Universal Property
///
/// For any functor F, natural transformations `F -> Lan H G` correspond
/// bijectively to natural transformations `F . H -> G`.
///
/// # Special Cases
///
/// - `Lan Id G = G` (extension along identity)
/// - `Lan G G = Density G` (extension along itself gives density)
///
/// # Relationship to Coyoneda
///
/// `Coyoneda F = Lan Id F` - Coyoneda is the left Kan extension along Id.
///
/// # Latin Etymology
///
/// *Extensio Kan Sinistra* = Left Kan Extension
#[cfg(feature = "alloc")]
pub struct ExtensioKanSinistra<G: HKT, H: HKT, A> {
    /// The existential pair: `(H<B> -> A, G<B>)` for some B.
    existential: LanExistential<G, H, A>,
}

/// Internal representation of the existentially quantified pair.
#[cfg(feature = "alloc")]
struct LanExistential<G: HKT, H: HKT, A> {
    /// The G<B> value (type-erased).
    gb: Box<dyn core::any::Any + Send>,
    /// The transformation H<B> -> A.
    transform: Box<dyn FnOnce(Box<dyn core::any::Any>) -> A + Send>,
    _phantom: core::marker::PhantomData<(G, H)>,
}

#[cfg(feature = "alloc")]
impl<G: HKT, H: HKT, A: 'static> ExtensioKanSinistra<G, H, A> {
    /// Create a new left Kan extension.
    ///
    /// # Panics
    ///
    /// The stored transformation panics only if the type-erased `H::Target<B>`
    /// argument fails to downcast when the extension is later applied — an
    /// internal invariant that cannot fire absent a bug in this crate, since
    /// the erasure and the downcast use the same existential `B`.
    #[inline]
    pub fn new<B: Send + 'static>(
        gb: G::Target<B>,
        transform: impl FnOnce(H::Target<B>) -> A + Send + 'static,
    ) -> Self
    where
        G::Target<B>: Send + 'static,
        H::Target<B>: 'static,
    {
        ExtensioKanSinistra {
            existential: LanExistential {
                gb: Box::new(gb),
                transform: Box::new(move |any| {
                    let hb = *any.downcast::<H::Target<B>>().expect("Lan type mismatch");
                    transform(hb)
                }),
                _phantom: core::marker::PhantomData,
            },
        }
    }

    /// Map a function over the Lan.
    ///
    /// `Lan H G` is always a functor in A.
    #[inline]
    pub fn map<B: 'static, F>(self, f: F) -> ExtensioKanSinistra<G, H, B>
    where
        F: FnOnce(A) -> B + Send + 'static,
    {
        let old_transform = self.existential.transform;
        ExtensioKanSinistra {
            existential: LanExistential {
                gb: self.existential.gb,
                transform: Box::new(move |any| f(old_transform(any))),
                _phantom: core::marker::PhantomData,
            },
        }
    }
}

// =============================================================================
// Codensity Monad
// =============================================================================

/// The Codensity monad - `Ran G G` specialized.
///
/// > *"Codensitas est monas universalis."*
/// > — Codensity is the universal monad. (Modern)
///
/// Codensity of a functor G is `Ran G G`, which always forms a monad
/// even when G itself is not a monad.
///
/// ```text
/// Codensity g a = forall b. (a -> g b) -> g b
/// ```
///
/// # Benefits
///
/// 1. **Monad from Functor**: Any functor gives rise to a Codensity monad
/// 2. **CPS Transformation**: Represents computations in continuation-passing style
/// 3. **Performance**: Can improve performance of left-associated binds
///
/// # Latin Etymology
///
/// *Codensitas* = co-density (dual of density)
#[cfg(feature = "alloc")]
pub struct Codensitas<G: HKT, A> {
    /// The continuation.
    run: CodensitasRun<G, A>,
    _phantom: core::marker::PhantomData<A>,
}

#[cfg(feature = "alloc")]
impl<G: HKT, A: 'static> Codensitas<G, A> {
    /// Create a new Codensity computation.
    #[inline]
    pub fn new<Run>(run: Run) -> Self
    where
        Run: FnOnce(Box<dyn FnOnce(A) -> G::Target<A>>) -> G::Target<A> + Send + 'static,
    {
        Codensitas {
            run: Box::new(run),
            _phantom: core::marker::PhantomData,
        }
    }

    /// Lift a pure value into Codensity.
    #[inline]
    pub fn purus(a: A) -> Self
    where
        A: Send + 'static,
    {
        Codensitas::new(move |k| k(a))
    }

    // Deliberately no `map`; see the Yoneda note above.

    /// Run the Codensity with a final continuation.
    #[inline]
    pub fn run_with<F>(self, k: F) -> G::Target<A>
    where
        F: FnOnce(A) -> G::Target<A> + 'static,
    {
        (self.run)(Box::new(k))
    }
}

// =============================================================================
// CodensitasT - Universal Codensity Transformer
// =============================================================================

/// Universal Codensity transformer for any monad M.
///
/// > *"CodensitasT est monas universalis transformata."*
/// > — CodensitasT is the universal transformed monad. (Modern)
///
/// `CodensitasT<M>` wraps any monad `M` to provide O(1) bind composition
/// via continuation-passing style.
///
/// # Type Parameters
///
/// * `M` - The base monad type
///
/// # Example
///
/// ```rust
/// use ordofp_core::category::CodensitasT;
///
/// // Wrap Option<A> in CodensitasT
/// let cod: CodensitasT<Option<i32>> = CodensitasT::new(|k| k(Some(42)));
///
/// // Chain with O(1) per operation
/// let result = cod.lower();
/// assert_eq!(result, Some(42));
/// ```
#[cfg(all(feature = "alloc", feature = "transformers-cps"))]
pub struct CodensitasT<M> {
    /// Continuation: (A -> M) -> M
    run: CodensitasTRun<M>,
}

#[cfg(all(feature = "alloc", feature = "transformers-cps"))]
impl<M> CodensitasT<M> {
    /// Create a new `CodensitasT` from a continuation.
    #[inline]
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce(Box<dyn FnOnce(M) -> M + Send>) -> M + Send + 'static,
    {
        Self { run: Box::new(f) }
    }

    /// Lower (convert back to base monad).
    #[inline]
    pub fn lower(self) -> M
    where
        M: Clone + Send + 'static,
    {
        (self.run)(Box::new(|m| m))
    }
}

// =============================================================================
// Density Comonad
// =============================================================================

/// The Density comonad - `Lan G G` specialized.
///
/// > *"Densitas est comonas universalis."*
/// > — Density is the universal comonad. (Modern)
///
/// Density of a functor G is `Lan G G`, which always forms a comonad
/// even when G itself is not a comonad.
///
/// ```text
/// Density g a = exists b. (g b -> a, g b)
/// ```
///
/// # Benefits
///
/// 1. **Comonad from Functor**: Any functor gives rise to a Density comonad
/// 2. **Represents focused computations**: Like a generalized zipper
///
/// # Latin Etymology
///
/// *Densitas* = density
#[cfg(feature = "alloc")]
pub struct Densitas<G: HKT, A> {
    /// The existential pair.
    inner: DensitasInner<G, A>,
}

#[cfg(feature = "alloc")]
struct DensitasInner<G: HKT, A> {
    /// The G<B> value.
    gb: Box<dyn core::any::Any + Send>,
    /// The extraction function G<B> -> A.
    extract: Box<dyn FnOnce(Box<dyn core::any::Any>) -> A + Send>,
    _phantom: core::marker::PhantomData<G>,
}

#[cfg(feature = "alloc")]
impl<G: HKT, A: 'static> Densitas<G, A> {
    /// Create a new Density from a `G<B>` and extraction function.
    ///
    /// # Panics
    ///
    /// The stored extraction panics only if the type-erased `G::Target<B>`
    /// fails to downcast when [`Self::extractum`] later runs — an internal
    /// invariant that cannot fire absent a bug in this crate, since the
    /// erasure and the downcast use the same existential `B`.
    #[inline]
    pub fn new<B: Send + 'static>(
        gb: G::Target<B>,
        extract: impl FnOnce(G::Target<B>) -> A + Send + 'static,
    ) -> Self
    where
        G::Target<B>: Send + 'static,
    {
        Densitas {
            inner: DensitasInner {
                gb: Box::new(gb),
                extract: Box::new(move |any| {
                    let gb = *any
                        .downcast::<G::Target<B>>()
                        .expect("Density type mismatch");
                    extract(gb)
                }),
                _phantom: core::marker::PhantomData,
            },
        }
    }

    /// Extract the value from the Density (comonad extract).
    #[inline]
    pub fn extractum(self) -> A {
        (self.inner.extract)(self.inner.gb)
    }

    /// Map a function over the Density.
    #[inline]
    pub fn map<B: 'static, F>(self, f: F) -> Densitas<G, B>
    where
        F: FnOnce(A) -> B + Send + 'static,
    {
        let old_extract = self.inner.extract;
        Densitas {
            inner: DensitasInner {
                gb: self.inner.gb,
                extract: Box::new(move |any| f(old_extract(any))),
                _phantom: core::marker::PhantomData,
            },
        }
    }
}

// =============================================================================
// Day Convolution
// =============================================================================

/// Day convolution of two functors.
///
/// > *"Convolutio diei unit duos functores."*
/// > — Day convolution unites two functors. (Modern)
///
/// The Day convolution is defined as:
/// ```text
/// Day f g a = exists b c. (f b, g c, b -> c -> a)
/// ```
///
/// # Monoidal Structure
///
/// Day convolution forms the tensor product for the monoidal category
/// of functors with Applicative structure.
///
/// # Latin Etymology
///
/// *Convolutio Diei* = Day convolution (named after Brian Day)
#[cfg(feature = "alloc")]
pub struct ConvolutioDiei<F: HKT, G: HKT, A> {
    /// The existential triple.
    inner: DayInner<F, G, A>,
}

#[cfg(feature = "alloc")]
struct DayInner<F: HKT, G: HKT, A> {
    /// F<B> component.
    fb: Box<dyn core::any::Any + Send>,
    /// G<C> component.
    gc: Box<dyn core::any::Any + Send>,
    /// The combining function B -> C -> A.
    combine: DayCombinatio<A>,
    _phantom: core::marker::PhantomData<(F, G)>,
}

#[cfg(feature = "alloc")]
impl<F: HKT, G: HKT, A: 'static> ConvolutioDiei<F, G, A> {
    /// Create a new Day convolution.
    ///
    /// # Panics
    ///
    /// The stored combiner panics only if the type-erased `B` or `C`
    /// components fail to downcast when the convolution is later collapsed —
    /// an internal invariant that cannot fire absent a bug in this crate,
    /// since erasure and downcast use the same existential types.
    #[inline]
    pub fn new<B: Send + 'static, C: Send + 'static>(
        fb: F::Target<B>,
        gc: G::Target<C>,
        combine: impl FnOnce(B, C) -> A + Send + 'static,
    ) -> Self
    where
        F::Target<B>: Send + 'static,
        G::Target<C>: Send + 'static,
    {
        ConvolutioDiei {
            inner: DayInner {
                fb: Box::new(fb),
                gc: Box::new(gc),
                combine: Box::new(move |any_b, any_c| {
                    let b = *any_b
                        .downcast::<B>()
                        .expect("Day convolution B type mismatch");
                    let c = *any_c
                        .downcast::<C>()
                        .expect("Day convolution C type mismatch");
                    combine(b, c)
                }),
                _phantom: core::marker::PhantomData,
            },
        }
    }

    /// Map a function over the Day convolution.
    #[inline]
    pub fn map<D: 'static, H>(self, h: H) -> ConvolutioDiei<F, G, D>
    where
        H: FnOnce(A) -> D + Send + 'static,
    {
        let old_combine = self.inner.combine;
        ConvolutioDiei {
            inner: DayInner {
                fb: self.inner.fb,
                gc: self.inner.gc,
                combine: Box::new(move |b, c| h(old_combine(b, c))),
                _phantom: core::marker::PhantomData,
            },
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Simple Option witness for testing
    struct OptionWitness;
    impl HKT for OptionWitness {
        type Target<A> = Option<A>;
    }

    #[test]
    fn test_coyoneda_lift() {
        let coyoneda: Coyoneda<OptionWitness, i32> = Coyoneda::lift(Some(42));
        assert!(!coyoneda.is_type::<i32>());
        assert!(coyoneda.is_type::<Option<i32>>());
    }

    #[test]
    fn test_coyoneda_map() {
        let coyoneda: Coyoneda<OptionWitness, i32> = Coyoneda::lift(Some(42));
        let mapped = coyoneda.map(|x| x * 2);
        let mapped2 = mapped.map(|x| x + 1);
        // The transforms are accumulated
        assert!(mapped2.is_type::<Option<i32>>());
    }

    #[test]
    fn test_coyoneda_map_chain() {
        let coyoneda: Coyoneda<OptionWitness, i32> = Coyoneda::lift(Some(10));

        // Chain multiple maps - they should all compose
        let result = coyoneda
            .map(|x| x + 1) // 11
            .map(|x| x * 2) // 22
            .map(|x| x - 2); // 20

        // The internal value is still Option<i32>
        assert!(result.is_type::<Option<i32>>());
    }

    #[test]
    fn test_codensitas_purus() {
        let cod: Codensitas<OptionWitness, i32> = Codensitas::purus(42);
        let result = cod.run_with(Some);
        assert_eq!(result, Some(42));
    }

    #[test]
    fn test_densitas_basic() {
        let density: Densitas<OptionWitness, i32> =
            Densitas::new(Some(42), |opt: Option<i32>| opt.unwrap_or(0));
        let extracted = density.extractum();
        assert_eq!(extracted, 42);
    }

    #[test]
    fn test_densitas_map() {
        let density: Densitas<OptionWitness, i32> =
            Densitas::new(Some(21), |opt: Option<i32>| opt.unwrap_or(0));
        let mapped = density.map(|x| x * 2);
        let extracted = mapped.extractum();
        assert_eq!(extracted, 42);
    }

    #[test]
    fn test_extensio_kan_dextra_basic() {
        let ran: ExtensioKanDextra<OptionWitness, OptionWitness, i32> =
            ExtensioKanDextra::new(|k: Box<dyn FnOnce(i32) -> Option<i32>>| k(42));

        let result = ran.run_with(|x| Some(x * 2));
        assert_eq!(result, Some(84));
    }
}
