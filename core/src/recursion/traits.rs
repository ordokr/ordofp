//! Core traits for recursion schemes.
//!
//! > *"Forma dat esse rei."*
//! > — Form gives being to a thing. (Aquinas)

#[cfg(feature = "alloc")]
use alloc::boxed::Box;

use crate::typeclasses::hkt::{FunctorHKT, HKT};

/// The base functor type family.
///
/// Every recursive type has an associated "base functor" that represents
/// one layer of the recursive structure with the recursion replaced by
/// a type parameter.
///
/// # Example
///
/// For a recursive list type `List<A>`, the base functor would be:
///
/// ```rust
/// enum ListF<A, R> {
///     Nil,
///     Cons(A, R),
/// }
///
/// let layer: ListF<i32, ()> = ListF::Cons(42, ());
/// match layer {
///     ListF::Cons(x, ()) => assert_eq!(x, 42),
///     ListF::Nil => panic!("expected Cons"),
/// }
/// ```
///
/// Where `R` is the recursion variable.
pub trait FunctorBasis {
    /// The base functor type.
    /// `F` is a type that implements `FunctorHKT`.
    type Base: FunctorHKT;
}

/// Recursiva - Trait for types that can be recursively folded.
///
/// > *"Descensus ab universali ad particulare."*
/// > — Descent from the universal to the particular.
///
/// A type is `Recursiva` if we can project it into its base functor
/// applied to itself. This enables catamorphisms (folds).
///
/// # Laws
///
/// For any `Recursiva` type with `embed` as its inverse:
///
/// ```text
/// embed(project(t)) == t  // embed-project identity
/// ```
#[cfg(feature = "alloc")]
pub trait Recursiva: FunctorBasis + Sized {
    /// Project one layer of recursion.
    ///
    /// Takes a recursive structure and exposes one layer,
    /// with recursive positions containing the original type.
    fn project(self) -> <Self::Base as HKT>::Target<Self>;
}

/// Corecursiva - Trait for types that can be recursively unfolded.
///
/// > *"Ascensus a particulari ad universale."*
/// > — Ascent from the particular to the universal.
///
/// A type is `Corecursiva` if we can embed its base functor
/// applied to itself back into the type. This enables anamorphisms (unfolds).
///
/// # Laws
///
/// For any `Corecursiva` type with `project` as its inverse:
///
/// ```text
/// project(embed(f)) == f  // project-embed identity
/// ```
#[cfg(feature = "alloc")]
pub trait Corecursiva: FunctorBasis + Sized {
    /// Embed one layer of base functor.
    ///
    /// Takes a layer of the base functor with recursive positions
    /// and wraps it into the recursive type.
    fn embed(layer: <Self::Base as HKT>::Target<Self>) -> Self;
}

/// Birecursiva - Trait for types that are both recursive and corecursive.
///
/// > *"Idem est principium et finis."*
/// > — The beginning and the end are the same.
///
/// Most useful recursive data types implement both traits.
#[cfg(feature = "alloc")]
pub trait Birecursiva: Recursiva + Corecursiva {}

#[cfg(feature = "alloc")]
impl<T: Recursiva + Corecursiva> Birecursiva for T {}

// =============================================================================
// Fix Implementation
// =============================================================================

#[cfg(feature = "alloc")]
use crate::fix::Fix;

#[cfg(feature = "alloc")]
impl<F: FunctorHKT> FunctorBasis for Fix<F> {
    type Base = F;
}

#[cfg(feature = "alloc")]
impl<F: FunctorHKT> Recursiva for Fix<F> {
    #[inline]
    fn project(self) -> F::Target<Self> {
        *self.0
    }
}

#[cfg(feature = "alloc")]
impl<F: FunctorHKT> Corecursiva for Fix<F> {
    #[inline]
    fn embed(layer: F::Target<Self>) -> Self {
        Fix(Box::new(layer))
    }
}
