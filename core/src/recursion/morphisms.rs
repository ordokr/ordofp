//! Recursion scheme morphisms.
//!
//! > *"Per varios casus, per tot discrimina rerum."*
//! > — Through various chances, through so many changes of things. (Virgil)
//!
//! This module provides the core recursion scheme functions:
//! - Basic: cata, ana, hylo
//! - Extended: para, apo
//! - Course-of-values: histo, futu
//! - Auxiliary: zygo
//! - General: chrono, dyna
//!
//! # Stack Overflow Risk
//!
//! All recursion schemes in this module are implemented via direct Rust recursion
//! (no heap-allocated trampoline). Depth is bounded only by the depth of the input
//! structure. For deeply nested structures (> ~10 000 layers on a default 8 MiB
//! stack), callers should either:
//! - increase the thread stack size via `std::thread::Builder::stack_size`, or
//! - convert the input to a shallower representation before folding.
//!
//! A trampoline / iterative rewrite is a future work item.

use crate::typeclasses::hkt::{FunctorHKT, HKT};

#[cfg(feature = "alloc")]
use super::cofree::Cofree;
#[cfg(feature = "alloc")]
use super::free::{Aut, Free};
#[cfg(feature = "alloc")]
use super::traits::{Corecursiva, FunctorBasis, Recursiva};

// =============================================================================
// CATAMORPHISM (cata) - Fold
// =============================================================================

/// Catamorphism - generalized fold.
///
/// > *"Κατά (kata) = down, μορφή (morphe) = form"*
/// > — Transformation downward through the structure.
///
/// Tears down a recursive structure layer by layer, applying
/// an algebra at each level.
///
/// # Type
///
/// ```text
/// cata :: (Recursiva t, Base t ~ f) => (f a -> a) -> t -> a
/// ```
///
/// # Example
///
/// ```rust
/// use ordofp_core::recursion::{cata, ListF, ListFWitness};
/// use ordofp_core::fix::Fix;
///
/// // [1, 2, 3] as a Fix<ListFWitness<i32>>
/// let list: Fix<ListFWitness<i32>> = Fix::new(ListF::ConsF(
///     1,
///     Fix::new(ListF::ConsF(2, Fix::new(ListF::ConsF(3, Fix::new(ListF::NilF))))),
/// ));
///
/// // Sum all elements in the list
/// let sum: i32 = cata(|layer| match layer {
///     ListF::NilF => 0,
///     ListF::ConsF(x, acc) => x + acc,
/// }, list);
/// assert_eq!(sum, 6);
/// ```
#[cfg(feature = "alloc")]
#[inline]
pub fn cata<T, A, Alg>(alg: Alg, t: T) -> A
where
    T: Recursiva,
    Alg: Fn(<T::Base as HKT>::Target<A>) -> A + Clone,
{
    cata_impl(&alg, t)
}

#[cfg(feature = "alloc")]
#[inline]
fn cata_impl<T, A, Alg>(alg: &Alg, t: T) -> A
where
    T: Recursiva,
    Alg: Fn(<T::Base as HKT>::Target<A>) -> A,
{
    let layer = t.project();
    let mapped = T::Base::map(layer, |sub| cata_impl(alg, sub));
    alg(mapped)
}

// =============================================================================
// ANAMORPHISM (ana) - Unfold
// =============================================================================

/// Anamorphism - generalized unfold.
///
/// > *"Ἀνά (ana) = up, μορφή (morphe) = form"*
/// > — Transformation upward, building the structure.
///
/// Builds up a recursive structure from a seed, applying
/// a coalgebra at each level.
///
/// # Type
///
/// ```text
/// ana :: (Corecursiva t, Base t ~ f) => (a -> f a) -> a -> t
/// ```
///
/// # Example
///
/// ```rust
/// use ordofp_core::recursion::{ana, cata, ListF, ListFWitness};
/// use ordofp_core::fix::Fix;
///
/// // Build a list from a range: [5, 4, 3, 2, 1]
/// let list: Fix<ListFWitness<i32>> = ana(|n| {
///     if n <= 0 {
///         ListF::NilF
///     } else {
///         ListF::ConsF(n, n - 1)
///     }
/// }, 5);
///
/// // Fold it back down to check the built structure
/// let sum: i32 = cata(|layer| match layer {
///     ListF::NilF => 0,
///     ListF::ConsF(x, acc) => x + acc,
/// }, list);
/// assert_eq!(sum, 15); // 5 + 4 + 3 + 2 + 1
/// ```
#[cfg(feature = "alloc")]
#[inline]
pub fn ana<T, A, Coalg>(coalg: Coalg, seed: A) -> T
where
    T: Corecursiva,
    Coalg: Fn(A) -> <T::Base as HKT>::Target<A> + Clone,
{
    ana_impl(&coalg, seed)
}

#[cfg(feature = "alloc")]
#[inline]
fn ana_impl<T, A, Coalg>(coalg: &Coalg, seed: A) -> T
where
    T: Corecursiva,
    Coalg: Fn(A) -> <T::Base as HKT>::Target<A>,
{
    let layer = coalg(seed);
    let mapped = T::Base::map(layer, |sub| ana_impl(coalg, sub));
    T::embed(mapped)
}

// =============================================================================
// HYLOMORPHISM (hylo) - Refold
// =============================================================================

/// Hylomorphism - generalized refold.
///
/// > *"Ὕλη (hyle) = matter, μορφή (morphe) = form"*
/// > — Matter taking form.
///
/// Combines ana and cata: unfolds from a seed, then folds to a result.
/// More efficient than composing ana and cata because it doesn't
/// build the intermediate structure.
///
/// # Type
///
/// ```text
/// hylo :: Functor f => (f b -> b) -> (a -> f a) -> a -> b
/// ```
///
/// # Example
///
/// ```rust
/// use ordofp_core::recursion::{hylo, NatF, NatFWitness};
///
/// // 5! without ever materializing the intermediate unary `NatF` structure.
/// //
/// // The coalgebra unfolds 5 into unary form (SuccF(SuccF(...(ZeroF)))); the
/// // algebra folds it back up carrying a (count, product) accumulator, so
/// // each `SuccF` layer both counts up from 0 and multiplies by that count.
/// let (_, factorial) = hylo::<NatFWitness, usize, (usize, usize), _, _>(
///     |layer| match layer {
///         NatF::ZeroF => (0, 1),
///         NatF::SuccF(prev) => (prev.0 + 1, (prev.0 + 1) * prev.1),
///     },
///     |n| if n == 0 { NatF::ZeroF } else { NatF::SuccF(n - 1) },
///     5,
/// );
/// assert_eq!(factorial, 120); // 5! = 120
/// ```
#[cfg(feature = "alloc")]
#[inline]
pub fn hylo<F, A, B, Alg, Coalg>(alg: Alg, coalg: Coalg, seed: A) -> B
where
    F: FunctorHKT,
    Alg: Fn(F::Target<B>) -> B + Clone,
    Coalg: Fn(A) -> F::Target<A> + Clone,
{
    hylo_impl::<F, A, B, Alg, Coalg>(&alg, &coalg, seed)
}

#[cfg(feature = "alloc")]
#[inline]
fn hylo_impl<F, A, B, Alg, Coalg>(alg: &Alg, coalg: &Coalg, seed: A) -> B
where
    F: FunctorHKT,
    Alg: Fn(F::Target<B>) -> B,
    Coalg: Fn(A) -> F::Target<A>,
{
    let layer = coalg(seed);
    let mapped = F::map(layer, |sub| {
        hylo_impl::<F, A, B, Alg, Coalg>(alg, coalg, sub)
    });
    alg(mapped)
}

// =============================================================================
// PARAMORPHISM (para) - Fold with subtree access
// =============================================================================

/// Paramorphism - fold with access to subtrees.
///
/// > *"Παρά (para) = beside, μορφή (morphe) = form"*
/// > — Transformation alongside the form.
///
/// Like cata, but the algebra also receives the original subtree
/// alongside its folded result.
///
/// # Type
///
/// ```text
/// para :: (Recursiva t, Base t ~ f) => (f (t, a) -> a) -> t -> a
/// ```
#[cfg(feature = "alloc")]
#[inline]
pub fn para<T, A, Alg>(alg: Alg, t: T) -> A
where
    T: Recursiva + Clone,
    Alg: Fn(<T::Base as HKT>::Target<(T, A)>) -> A + Clone,
{
    para_impl(&alg, t)
}

#[cfg(feature = "alloc")]
#[inline]
fn para_impl<T, A, Alg>(alg: &Alg, t: T) -> A
where
    T: Recursiva + Clone,
    Alg: Fn(<T::Base as HKT>::Target<(T, A)>) -> A,
{
    let layer = t.project();
    let mapped = T::Base::map(layer, |sub: T| {
        let sub_clone = sub.clone();
        let result = para_impl(alg, sub);
        (sub_clone, result)
    });
    alg(mapped)
}

// =============================================================================
// APOMORPHISM (apo) - Unfold with early termination
// =============================================================================

/// Apomorphism - unfold with early termination.
///
/// > *"Ἀπό (apo) = away from, μορφή (morphe) = form"*
/// > — Transformation away from form.
///
/// Like ana, but can terminate early by returning a pre-built structure.
///
/// # Type
///
/// ```text
/// apo :: (Corecursiva t, Base t ~ f) => (a -> f (Either t a)) -> a -> t
/// ```
#[cfg(feature = "alloc")]
#[inline]
pub fn apo<T, A, Coalg>(coalg: Coalg, seed: A) -> T
where
    T: Corecursiva + Clone,
    Coalg: Fn(A) -> <T::Base as HKT>::Target<Aut<T, A>> + Clone,
{
    apo_impl(&coalg, seed)
}

#[cfg(feature = "alloc")]
#[inline]
fn apo_impl<T, A, Coalg>(coalg: &Coalg, seed: A) -> T
where
    T: Corecursiva + Clone,
    Coalg: Fn(A) -> <T::Base as HKT>::Target<Aut<T, A>>,
{
    let layer = coalg(seed);
    let mapped = T::Base::map(layer, |either| match either {
        Aut::Sinister(t) => t,
        Aut::Dexter(a) => apo_impl(coalg, a),
    });
    T::embed(mapped)
}

// =============================================================================
// HISTOMORPHISM (histo) - Fold with history
// =============================================================================

/// Histomorphism - fold with access to computation history.
///
/// > *"Historia = history, μορφή (morphe) = form"*
/// > — Transformation through history.
///
/// Like cata, but the algebra receives a Cofree structure containing
/// all previously computed values (course-of-values recursion).
///
/// # Type
///
/// ```text
/// histo :: (Recursiva t, Base t ~ f) => (f (Cofree f a) -> a) -> t -> a
/// ```
#[cfg(feature = "alloc")]
#[inline]
pub fn histo<T, A, Alg>(alg: Alg, t: T) -> A
where
    T: Recursiva,
    Alg: Fn(<T::Base as HKT>::Target<Cofree<T::Base, A>>) -> A + Clone,
    <T::Base as HKT>::Target<Cofree<T::Base, A>>: Clone,
{
    // Build a Cofree structure annotated with results, then extract the root
    let cofree = histo_build(&alg, t);
    cofree.attribute
}

#[cfg(feature = "alloc")]
#[inline]
fn histo_build<T, A, Alg>(alg: &Alg, t: T) -> Cofree<T::Base, A>
where
    T: Recursiva,
    Alg: Fn(<T::Base as HKT>::Target<Cofree<T::Base, A>>) -> A,
    <T::Base as HKT>::Target<Cofree<T::Base, A>>: Clone,
{
    let layer = t.project();
    let children = T::Base::map(layer, |sub| histo_build(alg, sub));
    let attr = alg(children.clone());
    Cofree::new(attr, children)
}

// =============================================================================
// FUTUMORPHISM (futu) - Unfold with multiple layers
// =============================================================================

/// Futumorphism - unfold producing multiple layers.
///
/// > *"Futurum = future, μορφή (morphe) = form"*
/// > — Transformation toward the future.
///
/// Like ana, but the coalgebra can produce multiple layers at once
/// using the Free monad.
///
/// # Type
///
/// ```text
/// futu :: (Corecursiva t, Base t ~ f) => (a -> f (Free f a)) -> a -> t
/// ```
#[cfg(feature = "alloc")]
#[inline]
pub fn futu<T, A, Coalg>(coalg: Coalg, seed: A) -> T
where
    T: Corecursiva,
    Coalg: Fn(A) -> <T::Base as HKT>::Target<Free<T::Base, A>> + Clone,
{
    futu_impl(&coalg, seed)
}

#[cfg(feature = "alloc")]
#[inline]
fn futu_impl<T, A, Coalg>(coalg: &Coalg, seed: A) -> T
where
    T: Corecursiva,
    Coalg: Fn(A) -> <T::Base as HKT>::Target<Free<T::Base, A>>,
{
    let layer = coalg(seed);
    let mapped = T::Base::map(layer, |free| free_to_t(coalg, free));
    T::embed(mapped)
}

#[cfg(feature = "alloc")]
#[inline]
fn free_to_t<T, A, Coalg>(coalg: &Coalg, free: Free<T::Base, A>) -> T
where
    T: Corecursiva,
    Coalg: Fn(A) -> <T::Base as HKT>::Target<Free<T::Base, A>>,
{
    match free {
        Free::Purus(a) => futu_impl(coalg, a),
        Free::Suspensus(layer) => {
            let mapped = T::Base::map(*layer, |inner_free| free_to_t(coalg, inner_free));
            T::embed(mapped)
        }
    }
}

// =============================================================================
// ZYGOMORPHISM (zygo) - Fold with auxiliary algebra
// =============================================================================

/// Zygomorphism - fold with auxiliary computation.
///
/// > *"Ζυγός (zygos) = yoke, μορφή (morphe) = form"*
/// > — Twin transformations yoked together.
///
/// Runs two folds simultaneously: an auxiliary fold and a main fold
/// that can use the auxiliary results.
///
/// # Type
///
/// ```text
/// zygo :: (Recursiva t, Base t ~ f) => (f b -> b) -> (f (b, a) -> a) -> t -> a
/// ```
#[cfg(feature = "alloc")]
#[inline]
pub fn zygo<T, A, B, AuxAlg, Alg>(aux_alg: AuxAlg, alg: Alg, t: T) -> A
where
    T: Recursiva,
    AuxAlg: Fn(<T::Base as HKT>::Target<B>) -> B + Clone,
    Alg: Fn(<T::Base as HKT>::Target<(B, A)>) -> A + Clone,
    <T::Base as HKT>::Target<(B, A)>: Clone,
{
    let (_, result) = zygo_impl(&aux_alg, &alg, t);
    result
}

#[cfg(feature = "alloc")]
#[inline]
fn zygo_impl<T, A, B, AuxAlg, Alg>(aux_alg: &AuxAlg, alg: &Alg, t: T) -> (B, A)
where
    T: Recursiva,
    AuxAlg: Fn(<T::Base as HKT>::Target<B>) -> B,
    Alg: Fn(<T::Base as HKT>::Target<(B, A)>) -> A,
    <T::Base as HKT>::Target<(B, A)>: Clone,
{
    let layer = t.project();
    let mapped = T::Base::map(layer, |sub| zygo_impl(aux_alg, alg, sub));

    // Extract the B values for auxiliary computation
    let aux_layer = T::Base::map(mapped.clone(), |(b, _)| b);
    let main_layer = mapped;

    let aux_result = aux_alg(aux_layer);
    let main_result = alg(main_layer);

    (aux_result, main_result)
}

// =============================================================================
// CHRONOMORPHISM (chrono) - Generalized refold
// =============================================================================

/// Chronomorphism - generalized refold with history and future.
///
/// > *"Χρόνος (chronos) = time, μορφή (morphe) = form"*
/// > — Transformation through time.
///
/// Combines histo and futu: unfolds with multiple layers,
/// folds with access to history.
///
/// # Type
///
/// ```text
/// chrono :: Functor f => (f (Cofree f b) -> b) -> (a -> f (Free f a)) -> a -> b
/// ```
#[cfg(feature = "alloc")]
#[inline]
pub fn chrono<F, A, B, Alg, Coalg>(alg: Alg, coalg: Coalg, seed: A) -> B
where
    F: FunctorHKT,
    Alg: Fn(F::Target<Cofree<F, B>>) -> B + Clone,
    Coalg: Fn(A) -> F::Target<Free<F, A>> + Clone,
    F::Target<Cofree<F, B>>: Clone,
{
    let cofree = chrono_build::<F, A, B, Alg, Coalg>(&alg, &coalg, seed);
    cofree.attribute
}

#[cfg(feature = "alloc")]
#[inline]
fn chrono_build<F, A, B, Alg, Coalg>(alg: &Alg, coalg: &Coalg, seed: A) -> Cofree<F, B>
where
    F: FunctorHKT,
    Alg: Fn(F::Target<Cofree<F, B>>) -> B,
    Coalg: Fn(A) -> F::Target<Free<F, A>>,
    F::Target<Cofree<F, B>>: Clone,
{
    let layer = coalg(seed);
    let children = F::map(layer, |free| {
        chrono_free::<F, A, B, Alg, Coalg>(alg, coalg, free)
    });
    let attr = alg(children.clone());
    Cofree::new(attr, children)
}

#[cfg(feature = "alloc")]
#[inline]
fn chrono_free<F, A, B, Alg, Coalg>(alg: &Alg, coalg: &Coalg, free: Free<F, A>) -> Cofree<F, B>
where
    F: FunctorHKT,
    Alg: Fn(F::Target<Cofree<F, B>>) -> B,
    Coalg: Fn(A) -> F::Target<Free<F, A>>,
    F::Target<Cofree<F, B>>: Clone,
{
    match free {
        Free::Purus(a) => chrono_build(alg, coalg, a),
        Free::Suspensus(layer) => {
            let children = F::map(*layer, |inner| chrono_free(alg, coalg, inner));
            let attr = alg(children.clone());
            Cofree::new(attr, children)
        }
    }
}

// =============================================================================
// DYNAMORPHISM (dyna) - Efficient histo + ana
// =============================================================================

/// Dynamorphism - course-of-values unfold-then-fold.
///
/// More efficient than composing ana and histo.
///
/// # Type
///
/// ```text
/// dyna :: Functor f => (f (Cofree f b) -> b) -> (a -> f a) -> a -> b
/// ```
#[cfg(feature = "alloc")]
#[inline]
pub fn dyna<F, A, B, Alg, Coalg>(alg: Alg, coalg: Coalg, seed: A) -> B
where
    F: FunctorHKT,
    Alg: Fn(F::Target<Cofree<F, B>>) -> B + Clone,
    Coalg: Fn(A) -> F::Target<A> + Clone,
    F::Target<Cofree<F, B>>: Clone,
{
    let cofree = dyna_build::<F, A, B, Alg, Coalg>(&alg, &coalg, seed);
    cofree.attribute
}

#[cfg(feature = "alloc")]
#[inline]
fn dyna_build<F, A, B, Alg, Coalg>(alg: &Alg, coalg: &Coalg, seed: A) -> Cofree<F, B>
where
    F: FunctorHKT,
    Alg: Fn(F::Target<Cofree<F, B>>) -> B,
    Coalg: Fn(A) -> F::Target<A>,
    F::Target<Cofree<F, B>>: Clone,
{
    let layer = coalg(seed);
    let children = F::map(layer, |sub| dyna_build(alg, coalg, sub));
    let attr = alg(children.clone());
    Cofree::new(attr, children)
}

// =============================================================================
// MENDLER-STYLE MORPHISMS
// =============================================================================

/// Mendler-style catamorphism.
///
/// Doesn't require the base functor to implement Functor.
/// Instead, uses rank-2 polymorphism to ensure proper recursion.
///
/// # Type
///
/// ```text
/// mcata :: (forall x. (x -> a) -> f x -> a) -> Fix f -> a
/// ```
#[cfg(feature = "alloc")]
#[inline]
pub fn mcata<T, A, Alg>(alg: Alg, t: T) -> A
where
    T: Recursiva,
    Alg: for<'a> Fn(&'a dyn Fn(T) -> A, <T::Base as HKT>::Target<T>) -> A + Clone,
{
    mcata_impl(&alg, t)
}

#[cfg(feature = "alloc")]
#[inline]
fn mcata_impl<T, A, Alg>(alg: &Alg, t: T) -> A
where
    T: Recursiva,
    Alg: for<'a> Fn(&'a dyn Fn(T) -> A, <T::Base as HKT>::Target<T>) -> A,
{
    let layer = t.project();
    let recurse: &dyn Fn(T) -> A = &|sub| mcata_impl(alg, sub);
    alg(recurse, layer)
}

/// Mendler-style anamorphism.
///
/// Doesn't require the base functor to implement Functor.
///
/// # Type
///
/// ```text
/// mana :: (forall x. (a -> x) -> a -> f x) -> a -> Fix f
/// ```
#[cfg(feature = "alloc")]
#[inline]
pub fn mana<T, A, Coalg>(coalg: Coalg, seed: A) -> T
where
    T: Corecursiva,
    Coalg: for<'a> Fn(&'a dyn Fn(A) -> T, A) -> <T::Base as HKT>::Target<T> + Clone,
{
    mana_impl(&coalg, seed)
}

#[cfg(feature = "alloc")]
#[inline]
fn mana_impl<T, A, Coalg>(coalg: &Coalg, seed: A) -> T
where
    T: Corecursiva,
    Coalg: for<'a> Fn(&'a dyn Fn(A) -> T, A) -> <T::Base as HKT>::Target<T>,
{
    let recurse: &dyn Fn(A) -> T = &|sub| mana_impl(coalg, sub);
    let layer = coalg(recurse, seed);
    T::embed(layer)
}

/// Mendler-style hylomorphism.
///
/// Doesn't build intermediate structure.
#[cfg(feature = "alloc")]
#[inline]
pub fn mhylo<F, A, B, Alg, Coalg>(alg: Alg, coalg: Coalg, seed: A) -> B
where
    F: FunctorHKT,
    Alg: for<'a> Fn(&'a dyn Fn(A) -> B, F::Target<A>) -> B + Clone,
    Coalg: Fn(A) -> F::Target<A> + Clone,
{
    mhylo_impl::<F, A, B, Alg, Coalg>(&alg, &coalg, seed)
}

#[cfg(feature = "alloc")]
#[inline]
fn mhylo_impl<F, A, B, Alg, Coalg>(alg: &Alg, coalg: &Coalg, seed: A) -> B
where
    F: FunctorHKT,
    Alg: for<'a> Fn(&'a dyn Fn(A) -> B, F::Target<A>) -> B,
    Coalg: Fn(A) -> F::Target<A>,
{
    let layer = coalg(seed);
    let recurse: &dyn Fn(A) -> B = &|sub| mhylo_impl::<F, A, B, Alg, Coalg>(alg, coalg, sub);
    alg(recurse, layer)
}

// =============================================================================
// Helper type aliases
// =============================================================================

/// Algebra type for catamorphisms.
#[cfg(feature = "alloc")]
pub type Algebra<F, A> = dyn Fn(<F as HKT>::Target<A>) -> A;

/// Coalgebra type for anamorphisms.
#[cfg(feature = "alloc")]
pub type Coalgebra<F, A> = dyn Fn(A) -> <F as HKT>::Target<A>;

/// R-Algebra for paramorphisms.
#[cfg(feature = "alloc")]
pub type RAlgebra<T, A> = dyn Fn(<<T as FunctorBasis>::Base as HKT>::Target<(T, A)>) -> A;

/// R-Coalgebra for apomorphisms.
#[cfg(feature = "alloc")]
pub type RCoalgebra<T, A> = dyn Fn(A) -> <<T as FunctorBasis>::Base as HKT>::Target<Aut<T, A>>;
