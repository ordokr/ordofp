//! Higher-Kinded Type Simulation - Genera Superiora
//!
//! > *"Genus supremum"*
//! > — The highest genus. (Scholastic philosophy)
//!
//! This module provides advanced HKT simulation using Rust's GATs,
//! inspired by the `kinds` crate and PureScript's type system.
//!
//! # Relationship to `typeclasses::hkt`
//!
//! This module overlaps with [`crate::typeclasses::hkt`]: both express the
//! same witness idea. `hkt`'s `HKT`/`FunctorHKT` is the lighter pattern the
//! rest of the crate actually consumes (e.g. `recursion::base` witnesses);
//! `Genus` is the richer simulation (Plug/Unplug, natural transformations).
//! Prefer `hkt` when interoperating with other `OrdoFP` modules.
//!
//! # Design
//!
//! The HKT simulation uses three key patterns:
//! - **Genus**: Represents a type constructor `* -> *`
//! - **Plug/Unplug**: Extract and reconstruct types from/to HKT form
//! - **Natural Transformations**: Polymorphic functions between functors
//!
//! # Scholastic Naming
//!
//! | English | Latin | Etymology |
//! |---------|-------|-----------|
//! | Genus | Genus | *genus* = kind, type, class |
//! | Apply | Applicare | *applicare* = to attach, apply |
//! | Plug | Inserere | *inserere* = to put in |
//! | Unplug | Extrahere | *extrahere* = to draw out |
//! | Transform | Transformare | *transformare* = to change form |
//!
//! # Example
//!
//! ```rust
//! use ordofp_core::typeclasses::genus::{FunctorGenus, OptionGenus};
//!
//! // Map over the built-in Option genus
//! let result = <OptionGenus as FunctorGenus>::fmap_genus(
//!     Some(21),
//!     |x| x * 2
//! );
//! assert_eq!(result, Some(42));
//! ```

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::marker::PhantomData;

// =============================================================================
// Core Genus Trait
// =============================================================================

/// A higher-kinded type marker.
///
/// `Genus` represents a type constructor `* -> *`, allowing us to abstract
/// over type constructors like `Option`, `Vec`, `Result<_, E>`, etc.
///
/// # Type Parameters
///
/// The associated type `Applicatum<A>` represents the type constructor
/// applied to type `A`.
///
/// # Example
///
/// ```rust
/// use ordofp_core::typeclasses::genus::Genus;
///
/// struct OptionGenus;
/// impl Genus for OptionGenus {
///     type Applicatum<A> = Option<A>;
/// }
///
/// struct VecGenus;
/// impl Genus for VecGenus {
///     type Applicatum<A> = Vec<A>;
/// }
/// ```
pub trait Genus {
    /// The type constructor applied to type parameter `A`.
    type Applicatum<A>;
}

// =============================================================================
// Built-in Genus Implementations
// =============================================================================

/// Genus marker for `Option`.
#[derive(Debug, Clone, Copy, Default)]
pub struct OptionGenus;

impl Genus for OptionGenus {
    type Applicatum<A> = Option<A>;
}

/// Genus marker for `Vec`.
#[derive(Debug, Clone, Copy, Default)]
pub struct VecGenus;

impl Genus for VecGenus {
    type Applicatum<A> = Vec<A>;
}

/// Genus marker for `Result<_, E>` with fixed error type.
#[derive(Debug, Clone, Copy)]
pub struct ResultGenus<E>(PhantomData<E>);

impl<E> Default for ResultGenus<E> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<E> ResultGenus<E> {
    /// Create a new Result genus with the given error type.
    pub const fn new() -> Self {
        ResultGenus(PhantomData)
    }
}

impl<E> Genus for ResultGenus<E> {
    type Applicatum<A> = Result<A, E>;
}

/// Genus marker for `Box`.
#[derive(Debug, Clone, Copy, Default)]
pub struct BoxGenus;

impl Genus for BoxGenus {
    type Applicatum<A> = Box<A>;
}

/// Genus marker for the Identity functor.
#[derive(Debug, Clone, Copy, Default)]
pub struct IdentitasGenus;

impl Genus for IdentitasGenus {
    type Applicatum<A> = A;
}

// =============================================================================
// Functor for Genus
// =============================================================================

/// Functor type class for higher-kinded types.
///
/// `FunctorGenus<G>` provides the `fmap` operation for a type constructor
/// represented by `Genus` `G`.
///
/// # Laws
///
/// 1. Identity: `fmap_genus(fa, |x| x) == fa`
/// 2. Composition: `fmap_genus(fmap_genus(fa, f), g) == fmap_genus(fa, |x| g(f(x)))`
///
/// # Example
///
/// ```rust
/// use ordofp_core::typeclasses::genus::{FunctorGenus, OptionGenus};
///
/// let result = <OptionGenus as FunctorGenus>::fmap_genus(
///     Some(5),
///     |x| x * 2
/// );
/// assert_eq!(result, Some(10));
/// ```
pub trait FunctorGenus: Genus {
    /// Map a function over the genus.
    fn fmap_genus<A, B, F>(fa: Self::Applicatum<A>, f: F) -> Self::Applicatum<B>
    where
        F: FnOnce(A) -> B;
}

impl FunctorGenus for OptionGenus {
    #[inline]
    fn fmap_genus<A, B, F>(fa: Option<A>, f: F) -> Option<B>
    where
        F: FnOnce(A) -> B,
    {
        fa.map(f)
    }
}

// Note: VecGenus doesn't implement FunctorGenus with FnOnce because
// Vec iteration requires FnMut. Use the standard Iterator map instead.

impl<E> FunctorGenus for ResultGenus<E> {
    #[inline]
    fn fmap_genus<A, B, F>(fa: Result<A, E>, f: F) -> Result<B, E>
    where
        F: FnOnce(A) -> B,
    {
        fa.map(f)
    }
}

impl FunctorGenus for BoxGenus {
    #[inline]
    fn fmap_genus<A, B, F>(fa: Box<A>, f: F) -> Box<B>
    where
        F: FnOnce(A) -> B,
    {
        Box::new(f(*fa))
    }
}

impl FunctorGenus for IdentitasGenus {
    #[inline]
    fn fmap_genus<A, B, F>(fa: A, f: F) -> B
    where
        F: FnOnce(A) -> B,
    {
        f(fa)
    }
}

// =============================================================================
// Applicative for Genus
// =============================================================================

/// Applicative type class for higher-kinded types.
///
/// `ApplicativeGenus<G>` extends `FunctorGenus` with `pure` and `ap`.
///
/// # Laws
///
/// 1. Identity: `ap(pure(|x| x), v) == v`
/// 2. Homomorphism: `ap(pure(f), pure(x)) == pure(f(x))`
/// 3. Interchange: `ap(u, pure(y)) == ap(pure(|f| f(y)), u)`
/// 4. Composition: `ap(ap(ap(pure(compose), u), v), w) == ap(u, ap(v, w))`
pub trait ApplicativeGenus: FunctorGenus {
    /// Lift a pure value into the genus.
    fn purus_genus<A>(a: A) -> Self::Applicatum<A>;

    /// Apply a wrapped function to a wrapped value.
    fn ap_genus<A, B, F>(ff: Self::Applicatum<F>, fa: Self::Applicatum<A>) -> Self::Applicatum<B>
    where
        F: FnOnce(A) -> B;
}

impl ApplicativeGenus for OptionGenus {
    #[inline]
    fn purus_genus<A>(a: A) -> Option<A> {
        Some(a)
    }

    #[inline]
    fn ap_genus<A, B, F>(ff: Option<F>, fa: Option<A>) -> Option<B>
    where
        F: FnOnce(A) -> B,
    {
        match (ff, fa) {
            (Some(f), Some(a)) => Some(f(a)),
            _ => None,
        }
    }
}

// Note: VecGenus doesn't implement ApplicativeGenus with FnOnce because
// Vec requires applying a function to multiple elements.

impl<E> ApplicativeGenus for ResultGenus<E> {
    #[inline]
    fn purus_genus<A>(a: A) -> Result<A, E> {
        Ok(a)
    }

    #[inline]
    fn ap_genus<A, B, F>(ff: Result<F, E>, fa: Result<A, E>) -> Result<B, E>
    where
        F: FnOnce(A) -> B,
    {
        match (ff, fa) {
            (Ok(f), Ok(a)) => Ok(f(a)),
            (Err(e), _) => Err(e),
            (_, Err(e)) => Err(e),
        }
    }
}

impl ApplicativeGenus for BoxGenus {
    #[inline]
    fn purus_genus<A>(a: A) -> Box<A> {
        Box::new(a)
    }

    #[inline]
    fn ap_genus<A, B, F>(ff: Box<F>, fa: Box<A>) -> Box<B>
    where
        F: FnOnce(A) -> B,
    {
        Box::new((*ff)(*fa))
    }
}

impl ApplicativeGenus for IdentitasGenus {
    #[inline]
    fn purus_genus<A>(a: A) -> A {
        a
    }

    #[inline]
    fn ap_genus<A, B, F>(ff: F, fa: A) -> B
    where
        F: FnOnce(A) -> B,
    {
        ff(fa)
    }
}

// =============================================================================
// Monad for Genus
// =============================================================================

/// Monad type class for higher-kinded types.
///
/// `MonadGenus<G>` extends `ApplicativeGenus` with `flat_map` (bind).
///
/// # Laws
///
/// 1. Left identity: `flat_map(pure(a), f) == f(a)`
/// 2. Right identity: `flat_map(m, pure) == m`
/// 3. Associativity: `flat_map(flat_map(m, f), g) == flat_map(m, |x| flat_map(f(x), g))`
pub trait MonadGenus: ApplicativeGenus {
    /// Sequentially compose two actions.
    fn flat_map_genus<A, B, F>(fa: Self::Applicatum<A>, f: F) -> Self::Applicatum<B>
    where
        F: FnOnce(A) -> Self::Applicatum<B>;

    /// Flatten a nested genus.
    #[inline]
    fn flatten_genus<A>(ffa: Self::Applicatum<Self::Applicatum<A>>) -> Self::Applicatum<A> {
        Self::flat_map_genus(ffa, |x| x)
    }
}

impl MonadGenus for OptionGenus {
    #[inline]
    fn flat_map_genus<A, B, F>(fa: Option<A>, f: F) -> Option<B>
    where
        F: FnOnce(A) -> Option<B>,
    {
        fa.and_then(f)
    }
}

// Note: VecGenus doesn't implement MonadGenus with FnOnce because
// Vec requires applying a function to multiple elements.

impl<E> MonadGenus for ResultGenus<E> {
    #[inline]
    fn flat_map_genus<A, B, F>(fa: Result<A, E>, f: F) -> Result<B, E>
    where
        F: FnOnce(A) -> Result<B, E>,
    {
        fa.and_then(f)
    }
}

impl MonadGenus for BoxGenus {
    #[inline]
    fn flat_map_genus<A, B, F>(fa: Box<A>, f: F) -> Box<B>
    where
        F: FnOnce(A) -> Box<B>,
    {
        f(*fa)
    }
}

impl MonadGenus for IdentitasGenus {
    #[inline]
    fn flat_map_genus<A, B, F>(fa: A, f: F) -> B
    where
        F: FnOnce(A) -> B,
    {
        f(fa)
    }
}

// =============================================================================
// Plug/Unplug Pattern
// =============================================================================

/// Trait for types that can be "unplugged" to reveal their HKT structure.
///
/// `Extrahere` (Latin: "to draw out") extracts the type parameter from
/// a concrete type, revealing the underlying genus.
///
/// # Example
///
/// ```rust
/// use ordofp_core::typeclasses::genus::Extrahere;
///
/// // Option<i32>'s genus is OptionGenus, its element type i32;
/// // extrahere(self) yields the inner value (if any)
/// let opt = Some(42);
/// let value: Option<i32> = opt.extrahere();
/// assert_eq!(value, Some(42));
/// ```
pub trait Extrahere {
    /// The genus (type constructor) of this type.
    type Genus: Genus;

    /// The type parameter that was plugged in.
    type Elementum;

    /// Extract the inner value, if possible.
    fn extrahere(self) -> Option<Self::Elementum>;
}

impl<A> Extrahere for Option<A> {
    type Genus = OptionGenus;
    type Elementum = A;

    #[inline]
    fn extrahere(self) -> Option<A> {
        self
    }
}

impl<A> Extrahere for Vec<A> {
    type Genus = VecGenus;
    type Elementum = A;

    #[inline]
    fn extrahere(self) -> Option<A> {
        self.into_iter().next()
    }
}

impl<A, E> Extrahere for Result<A, E> {
    type Genus = ResultGenus<E>;
    type Elementum = A;

    #[inline]
    fn extrahere(self) -> Option<A> {
        self.ok()
    }
}

impl<A> Extrahere for Box<A> {
    type Genus = BoxGenus;
    type Elementum = A;

    #[inline]
    fn extrahere(self) -> Option<A> {
        Some(*self)
    }
}

/// Trait for "plugging" a type parameter into a genus.
///
/// `Inserere` (Latin: "to put in") reconstructs a concrete type
/// from a genus marker and a value.
pub trait Inserere<A>: Genus {
    /// Plug a value into the genus.
    fn inserere(a: A) -> Self::Applicatum<A>;
}

impl<A> Inserere<A> for OptionGenus {
    #[inline]
    fn inserere(a: A) -> Option<A> {
        Some(a)
    }
}

impl<A> Inserere<A> for VecGenus {
    #[inline]
    fn inserere(a: A) -> Vec<A> {
        alloc::vec![a]
    }
}

impl<A, E> Inserere<A> for ResultGenus<E> {
    #[inline]
    fn inserere(a: A) -> Result<A, E> {
        Ok(a)
    }
}

impl<A> Inserere<A> for BoxGenus {
    #[inline]
    fn inserere(a: A) -> Box<A> {
        Box::new(a)
    }
}

impl<A> Inserere<A> for IdentitasGenus {
    #[inline]
    fn inserere(a: A) -> A {
        a
    }
}

// =============================================================================
// Natural Transformations for Genus
// =============================================================================

/// A natural transformation between genera.
///
/// `TransformatioGenerum<F, G>` is a polymorphic function that transforms
/// any `F::Applicatum<A>` to `G::Applicatum<A>`, preserving the inner type.
///
/// # Laws
///
/// Naturality: For any `f: A -> B`:
/// ```text
/// G::fmap(transform(fa), f) == transform(F::fmap(fa, f))
/// ```
pub trait TransformatioGenerum<F: Genus, G: Genus> {
    /// Transform from genus F to genus G.
    fn transformare_genus<A>(fa: F::Applicatum<A>) -> G::Applicatum<A>;
}

/// Option to Vec natural transformation.
pub struct OptionAdVecGenus;

impl TransformatioGenerum<OptionGenus, VecGenus> for OptionAdVecGenus {
    #[inline]
    fn transformare_genus<A>(fa: Option<A>) -> Vec<A> {
        match fa {
            Some(a) => alloc::vec![a],
            None => Vec::new(),
        }
    }
}

/// Vec to Option natural transformation (head).
pub struct VecAdOptionGenus;

impl TransformatioGenerum<VecGenus, OptionGenus> for VecAdOptionGenus {
    #[inline]
    fn transformare_genus<A>(fa: Vec<A>) -> Option<A> {
        fa.into_iter().next()
    }
}

/// Result to Option natural transformation.
pub struct ResultAdOptionGenus<E>(PhantomData<E>);

impl<E> Default for ResultAdOptionGenus<E> {
    #[inline]
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<E> TransformatioGenerum<ResultGenus<E>, OptionGenus> for ResultAdOptionGenus<E> {
    #[inline]
    fn transformare_genus<A>(fa: Result<A, E>) -> Option<A> {
        fa.ok()
    }
}

/// Identity transformation (identity natural transformation).
pub struct IdentitasTransformatio<G: Genus>(PhantomData<G>);

impl<G: Genus> Default for IdentitasTransformatio<G> {
    #[inline]
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<G: Genus> TransformatioGenerum<G, G> for IdentitasTransformatio<G> {
    #[inline]
    fn transformare_genus<A>(fa: G::Applicatum<A>) -> G::Applicatum<A> {
        fa
    }
}

// =============================================================================
// Traverse for Genus
// =============================================================================

/// Traversable type class for higher-kinded types.
///
/// `TraversableGenus<G>` allows sequencing effects across a structure.
pub trait TraversableGenus: FunctorGenus {
    /// Traverse with an applicative effect.
    fn traverse_genus<A, B, F, H>(
        fa: Self::Applicatum<A>,
        f: F,
    ) -> H::Applicatum<Self::Applicatum<B>>
    where
        F: FnOnce(A) -> H::Applicatum<B>,
        H: ApplicativeGenus;
}

impl TraversableGenus for OptionGenus {
    #[inline]
    fn traverse_genus<A, B, F, H>(fa: Option<A>, f: F) -> H::Applicatum<Option<B>>
    where
        F: FnOnce(A) -> H::Applicatum<B>,
        H: ApplicativeGenus,
    {
        match fa {
            Some(a) => H::fmap_genus(f(a), Some),
            None => H::purus_genus(None),
        }
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Lift a pure function to work on genus values.
#[inline]
pub fn lift_genus<G, A, B, F>(f: F) -> impl FnOnce(G::Applicatum<A>) -> G::Applicatum<B>
where
    G: FunctorGenus,
    F: FnOnce(A) -> B,
{
    move |fa| G::fmap_genus(fa, f)
}

/// Lift a binary function to work on genus values.
#[inline]
pub fn lift2_genus<G, A, B, C, F>(
    f: F,
    fa: G::Applicatum<A>,
    fb: G::Applicatum<B>,
) -> G::Applicatum<C>
where
    G: ApplicativeGenus,
    F: FnOnce(A, B) -> C,
{
    G::ap_genus(G::fmap_genus(fa, move |a| move |b: B| f(a, b)), fb)
}

/// Sequence two monadic actions, keeping the result of the second.
#[inline]
pub fn sequence_genus<G, A, B>(fa: G::Applicatum<A>, fb: G::Applicatum<B>) -> G::Applicatum<B>
where
    G: MonadGenus,
{
    G::flat_map_genus(fa, |_| fb)
}

/// Map a function over two genus values and combine the results.
#[inline]
pub fn map2_genus<G, A, B, C, F>(
    fa: G::Applicatum<A>,
    fb: G::Applicatum<B>,
    f: F,
) -> G::Applicatum<C>
where
    G: MonadGenus,
    F: FnOnce(A, B) -> C,
{
    G::flat_map_genus(fa, |a| G::fmap_genus(fb, |b| f(a, b)))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_functor_genus_option() {
        let result = <OptionGenus as FunctorGenus>::fmap_genus(Some(21), |x| x * 2);
        assert_eq!(result, Some(42));

        let none: Option<i32> = None;
        let result = <OptionGenus as FunctorGenus>::fmap_genus(none, |x| x * 2);
        assert_eq!(result, None);
    }

    // Note: VecGenus doesn't implement FunctorGenus with FnOnce

    #[test]
    fn test_functor_genus_result() {
        let ok: Result<i32, &str> = Ok(21);
        let result = <ResultGenus<&str> as FunctorGenus>::fmap_genus(ok, |x| x * 2);
        assert_eq!(result, Ok(42));

        let err: Result<i32, &str> = Err("error");
        let result = <ResultGenus<&str> as FunctorGenus>::fmap_genus(err, |x| x * 2);
        assert_eq!(result, Err("error"));
    }

    #[test]
    fn test_applicative_genus_option() {
        let result = <OptionGenus as ApplicativeGenus>::purus_genus(42);
        assert_eq!(result, Some(42));

        let f: Option<fn(i32) -> i32> = Some(|x| x * 2);
        let result = <OptionGenus as ApplicativeGenus>::ap_genus(f, Some(21));
        assert_eq!(result, Some(42));
    }

    #[test]
    fn test_monad_genus_option() {
        let result = <OptionGenus as MonadGenus>::flat_map_genus(Some(21), |x| Some(x * 2));
        assert_eq!(result, Some(42));

        let result = <OptionGenus as MonadGenus>::flat_map_genus(Some(21), |_x| None::<i32>);
        assert_eq!(result, None);
    }

    #[test]
    fn test_monad_genus_laws_option() {
        // Left identity: flat_map(pure(a), f) == f(a)
        let a = 5;
        let f = |x: i32| Some(x * 2);
        let left = <OptionGenus as MonadGenus>::flat_map_genus(
            <OptionGenus as ApplicativeGenus>::purus_genus(a),
            f,
        );
        let right = f(a);
        assert_eq!(left, right);

        // Right identity: flat_map(m, pure) == m
        let m = Some(42);
        let result = <OptionGenus as MonadGenus>::flat_map_genus(m, |x| {
            <OptionGenus as ApplicativeGenus>::purus_genus(x)
        });
        assert_eq!(result, Some(42));
    }

    #[test]
    fn test_extrahere_option() {
        let opt = Some(42);
        let extracted = opt.extrahere();
        assert_eq!(extracted, Some(42));

        let none: Option<i32> = None;
        let extracted = none.extrahere();
        assert_eq!(extracted, None);
    }

    #[test]
    fn test_extrahere_vec() {
        let vec = alloc::vec![1, 2, 3];
        let extracted = vec.extrahere();
        assert_eq!(extracted, Some(1));
    }

    #[test]
    fn test_inserere() {
        let opt = <OptionGenus as Inserere<i32>>::inserere(42);
        assert_eq!(opt, Some(42));

        let vec = <VecGenus as Inserere<i32>>::inserere(42);
        assert_eq!(vec, alloc::vec![42]);
    }

    #[test]
    fn test_natural_transformation_option_to_vec() {
        let result = OptionAdVecGenus::transformare_genus(Some(42));
        assert_eq!(result, alloc::vec![42]);

        let result = OptionAdVecGenus::transformare_genus(None::<i32>);
        assert_eq!(result, Vec::<i32>::new());
    }

    #[test]
    fn test_natural_transformation_vec_to_option() {
        let result = VecAdOptionGenus::transformare_genus(alloc::vec![1, 2, 3]);
        assert_eq!(result, Some(1));

        let result = VecAdOptionGenus::transformare_genus(Vec::<i32>::new());
        assert_eq!(result, None);
    }

    #[test]
    fn test_lift_genus() {
        let f = |x: i32| x * 2;
        let lifted = lift_genus::<OptionGenus, _, _, _>(f);
        assert_eq!(lifted(Some(21)), Some(42));
    }

    #[test]
    fn test_sequence_genus() {
        let result = sequence_genus::<OptionGenus, _, _>(Some(1), Some(42));
        assert_eq!(result, Some(42));

        let result = sequence_genus::<OptionGenus, _, _>(None::<i32>, Some(42));
        assert_eq!(result, None);
    }

    #[test]
    fn test_identitas_genus() {
        let result = <IdentitasGenus as FunctorGenus>::fmap_genus(42, |x| x * 2);
        assert_eq!(result, 84);

        let result = <IdentitasGenus as MonadGenus>::flat_map_genus(21, |x| x * 2);
        assert_eq!(result, 42);
    }

    #[test]
    fn test_box_genus() {
        let result = <BoxGenus as FunctorGenus>::fmap_genus(Box::new(21), |x| x * 2);
        assert_eq!(*result, 42);

        let result = <BoxGenus as MonadGenus>::flat_map_genus(Box::new(21), |x| Box::new(x * 2));
        assert_eq!(*result, 42);
    }

    #[test]
    fn test_traverse_genus() {
        // Traverse Option with Option applicative
        let result: Option<Option<i32>> =
            <OptionGenus as TraversableGenus>::traverse_genus::<i32, i32, _, OptionGenus>(
                Some(21),
                |x| Some(x * 2),
            );
        assert_eq!(result, Some(Some(42)));
    }
}
