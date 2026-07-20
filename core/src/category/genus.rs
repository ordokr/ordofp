//! Genus - Universal Higher-Kinded Type Abstraction
//!
//! > *"Genus est quod de pluribus specie differentibus praedicatur."*
//! > — A genus is what is predicated of many things differing in species. (Aristotle)
//!
//! The **Genus** trait provides a unified abstraction for higher-kinded types (HKTs)
//! in Rust, bridging the gap between Rust's type system and category-theoretic
//! concepts. It serves as the foundation for functors, applicatives, monads, and
//! more advanced abstractions like Kan extensions.
//!
//! # Design Philosophy
//!
//! In Scholastic philosophy, *genus* is the broadest classification of being.
//! Similarly, `Genus` is the broadest classification of type constructors,
//! enabling polymorphism over type constructors themselves.
//!
//! # Architecture
//!
//! ```text
//! Genus (HKT witness)
//!    │
//!    ├── FunctorGenus (fmap)
//!    │      │
//!    │      ├── ApplicatioGenus (pure, ap)
//!    │      │      │
//!    │      │      └── MonadGenus (flatMap)
//!    │      │
//!    │      └── TraversableGenus (traverse)
//!    │
//!    └── FoldableGenus (fold)
//! ```
//!
//! # Example
//!
//! Rust has no native higher-kinded types, so `Genus` uses a separate
//! zero-sized "witness" type to stand in for a type constructor. Adding
//! support for your own container means defining a witness and implementing
//! `Genus`/`FunctorGenus` for it (legal for local types under the orphan
//! rule; you cannot do this for a type constructor you don't own, like
//! `Option`, from outside this crate):
//!
//! ```rust
//! use ordofp_core::category::genus::{Genus, FunctorGenus};
//!
//! // Your own wrapper type.
//! struct MyBox<T>(T);
//!
//! // The witness that stands in for `MyBox` at the type-constructor level.
//! struct MyBoxGenus;
//!
//! impl Genus for MyBoxGenus {
//!     type Applied<A> = MyBox<A> where A: Send + Sync;
//! }
//!
//! impl FunctorGenus for MyBoxGenus {
//!     fn fmap<A, B, F>(fa: MyBox<A>, mut f: F) -> MyBox<B>
//!     where
//!         A: Send + Sync,
//!         B: Send + Sync,
//!         F: FnMut(A) -> B + Send + Sync,
//!     {
//!         MyBox(f(fa.0))
//!     }
//! }
//!
//! let doubled = MyBoxGenus::fmap(MyBox(21), |x| x * 2);
//! assert_eq!(doubled.0, 42);
//! ```
//!
//! # MSRV Requirements
//!
//! This module uses GATs (stable) on Edition 2024; the crate itself requires the pinned nightly (see rust-toolchain.toml).

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use core::marker::PhantomData;

// =============================================================================
// Genus - The Universal HKT Trait
// =============================================================================

/// The universal higher-kinded type abstraction.
///
/// `Genus` represents a type constructor `* -> *` (a type that takes one type
/// parameter). It's the foundation for all functorial abstractions.
///
/// # Latin Etymology
///
/// *Genus* (n.) = kind, type, class, genus - the broadest logical classification.
///
/// # Laws
///
/// Genus itself has no laws - it's a pure abstraction. Laws are introduced
/// by more specific traits like `FunctorGenus` and `MonadGenus`.
pub trait Genus: Send + Sync + 'static {
    /// The type constructor applied to type `A`.
    ///
    /// For `OptionGenus`, `Applied<i32>` = `Option<i32>`.
    type Applied<A>: Send + Sync
    where
        A: Send + Sync;
}

// =============================================================================
// FunctorGenus - Mappable Type Constructors
// =============================================================================

/// A functor for higher-kinded types.
///
/// > *"Functor est morphismus inter categorias."*
/// > — A functor is a morphism between categories.
///
/// `FunctorGenus` provides the `fmap` operation for type constructors,
/// allowing functions to be lifted into the functor's context.
///
/// # Laws
///
/// 1. **Identity**: `fmap(fa, |x| x) = fa`
/// 2. **Composition**: `fmap(fmap(fa, f), g) = fmap(fa, |x| g(f(x)))`
///
/// # Example
///
/// ```rust
/// use ordofp_core::category::genus::{FunctorGenus, OptionGenus};
///
/// let opt = Some(42);
/// let doubled = OptionGenus::fmap(opt, |x| x * 2);
/// assert_eq!(doubled, Some(84));
/// ```
pub trait FunctorGenus: Genus {
    /// Map a function over the functor.
    fn fmap<A, B, F>(fa: Self::Applied<A>, f: F) -> Self::Applied<B>
    where
        A: Send + Sync,
        B: Send + Sync,
        F: FnMut(A) -> B + Send + Sync;

    /// Replace all values with a constant.
    fn replace<A, B>(fa: Self::Applied<A>, b: B) -> Self::Applied<B>
    where
        A: Send + Sync,
        B: Clone + Send + Sync,
    {
        Self::fmap(fa, |_| b.clone())
    }

    /// Void - discard the value, keeping only the structure.
    fn void<A>(fa: Self::Applied<A>) -> Self::Applied<()>
    where
        A: Send + Sync,
    {
        Self::fmap(fa, |_| ())
    }
}

// =============================================================================
// ApplicatioGenus - Applicative Functors
// =============================================================================

/// An applicative functor for higher-kinded types.
///
/// > *"Applicatio est elevatio functionis ad contextum."*
/// > — Application is the lifting of a function to a context.
///
/// `ApplicatioGenus` extends `FunctorGenus` with the ability to:
/// - Lift pure values into the functor (`purus`)
/// - Apply wrapped functions to wrapped values (`ap`)
///
/// # Laws
///
/// 1. **Identity**: `ap(purus(|x| x), v) = v`
/// 2. **Composition**: `ap(ap(ap(purus(compose), u), v), w) = ap(u, ap(v, w))`
/// 3. **Homomorphism**: `ap(purus(f), purus(x)) = purus(f(x))`
/// 4. **Interchange**: `ap(u, purus(y)) = ap(purus(|f| f(y)), u)`
pub trait ApplicatioGenus: FunctorGenus {
    /// Lift a pure value into the applicative context.
    fn purus<A>(a: A) -> Self::Applied<A>
    where
        A: Send + Sync;

    /// Apply a wrapped function to a wrapped value.
    fn ap<A, B, F>(ff: Self::Applied<F>, fa: Self::Applied<A>) -> Self::Applied<B>
    where
        A: Send + Sync,
        B: Send + Sync,
        F: FnMut(A) -> B + Send + Sync;

    /// Lift a binary function.
    ///
    /// Note: The default implementation is not available due to Rust's type system
    /// limitations with closures. Concrete implementations should override this.
    fn lift2<A, B, C, F>(f: F, fa: Self::Applied<A>, fb: Self::Applied<B>) -> Self::Applied<C>
    where
        A: Send + Sync,
        B: Send + Sync,
        C: Send + Sync,
        F: FnMut(A, B) -> C + Send + Sync;

    /// Sequence two actions, discarding the first result.
    fn sequence_right<A, B>(fa: Self::Applied<A>, fb: Self::Applied<B>) -> Self::Applied<B>
    where
        A: Send + Sync,
        B: Send + Sync,
    {
        Self::lift2(|_, b| b, fa, fb)
    }

    /// Sequence two actions, discarding the second result.
    fn sequence_left<A, B>(fa: Self::Applied<A>, fb: Self::Applied<B>) -> Self::Applied<A>
    where
        A: Send + Sync,
        B: Send + Sync,
    {
        Self::lift2(|a, _| a, fa, fb)
    }
}

// =============================================================================
// MonadGenus - Monadic Type Constructors
// =============================================================================

/// A monad for higher-kinded types.
///
/// > *"Monas est unitas indivisibilis."*
/// > — A monad is an indivisible unity. (Leibniz)
///
/// `MonadGenus` extends `ApplicatioGenus` with the ability to sequence
/// computations that depend on previous results.
///
/// # Laws
///
/// 1. **Left Identity**: `flat_map(purus(a), f) = f(a)`
/// 2. **Right Identity**: `flat_map(m, purus) = m`
/// 3. **Associativity**: `flat_map(flat_map(m, f), g) = flat_map(m, |x| flat_map(f(x), g))`
pub trait MonadGenus: ApplicatioGenus {
    /// Bind / flatMap - chain computations.
    fn flat_map<A, B, F>(fa: Self::Applied<A>, f: F) -> Self::Applied<B>
    where
        A: Send + Sync,
        B: Send + Sync,
        F: FnMut(A) -> Self::Applied<B> + Send + Sync;

    /// Flatten a nested structure.
    fn flatten<A>(ffa: Self::Applied<Self::Applied<A>>) -> Self::Applied<A>
    where
        A: Send + Sync,
        Self::Applied<A>: Send + Sync,
    {
        Self::flat_map(ffa, |fa| fa)
    }

    /// Execute an action and ignore its result, then return the given value.
    fn as_<A, B>(fa: Self::Applied<A>, b: B) -> Self::Applied<B>
    where
        A: Send + Sync,
        B: Clone + Send + Sync,
    {
        Self::flat_map(fa, |_| Self::purus(b.clone()))
    }
}

// =============================================================================
// FoldableGenus - Foldable Type Constructors
// =============================================================================

/// A foldable type constructor.
///
/// > *"Plicare est reducere ad unum."*
/// > — To fold is to reduce to one.
///
/// `FoldableGenus` provides the ability to collapse a structure into a summary value.
pub trait FoldableGenus: Genus {
    /// Left-associative fold.
    fn fold_left<A, B, F>(fa: Self::Applied<A>, init: B, f: F) -> B
    where
        A: Send + Sync,
        F: FnMut(B, A) -> B;

    /// Right-associative fold.
    fn fold_right<A, B, F>(fa: Self::Applied<A>, init: B, f: F) -> B
    where
        A: Send + Sync,
        F: FnMut(A, B) -> B;

    /// Fold with a monoid: map every element to `B` and combine the results,
    /// starting from `B::empty()`.
    ///
    /// (No default implementation on purpose: a correct `fold_map` must
    /// combine every element through a real monoid — a naive default would
    /// discard the accumulator.)
    fn fold_map<A, B, F>(fa: Self::Applied<A>, mut f: F) -> B
    where
        A: Send + Sync,
        B: crate::typeclasses::Unitas + Send + Sync,
        F: FnMut(A) -> B,
    {
        Self::fold_left(fa, B::empty(), |acc, a| acc.combine(&f(a)))
    }
}

// =============================================================================
// TraversableGenus - Traversable Type Constructors
// =============================================================================

/// A traversable type constructor.
///
/// > *"Traversare est iter facere per structuram."*
/// > — To traverse is to make a journey through a structure.
///
/// `TraversableGenus` provides the ability to traverse a structure while
/// performing effects and collecting results.
pub trait TraversableGenus: FunctorGenus + FoldableGenus {
    /// Traverse a structure with an effectful function.
    ///
    /// The type parameter `G` is the target genus (applicative functor).
    fn traverse<A, B, G, F>(fa: Self::Applied<A>, f: F) -> G::Applied<Self::Applied<B>>
    where
        A: Send + Sync,
        B: Send + Sync,
        G: ApplicatioGenus,
        G::Applied<Self::Applied<B>>: Send + Sync,
        F: FnMut(A) -> G::Applied<B>;

    /// Sequence a structure of effects into an effect of structure.
    fn sequence<A, G>(fga: Self::Applied<G::Applied<A>>) -> G::Applied<Self::Applied<A>>
    where
        A: Send + Sync,
        G: ApplicatioGenus,
        G::Applied<A>: Send + Sync,
        G::Applied<Self::Applied<A>>: Send + Sync,
    {
        Self::traverse::<G::Applied<A>, A, G, _>(fga, |ga| ga)
    }
}

// =============================================================================
// Standard Genus Implementations
// =============================================================================

/// Genus witness for `Option`.
#[derive(Debug, Clone, Copy, Default)]
pub struct OptionGenus;

impl Genus for OptionGenus {
    type Applied<A>
        = Option<A>
    where
        A: Send + Sync;
}

impl FunctorGenus for OptionGenus {
    #[inline]
    fn fmap<A, B, F>(fa: Option<A>, f: F) -> Option<B>
    where
        A: Send + Sync,
        B: Send + Sync,
        F: FnMut(A) -> B + Send + Sync,
    {
        fa.map(f)
    }
}

impl ApplicatioGenus for OptionGenus {
    #[inline]
    fn purus<A>(a: A) -> Option<A>
    where
        A: Send + Sync,
    {
        Some(a)
    }

    #[inline]
    fn ap<A, B, F>(ff: Option<F>, fa: Option<A>) -> Option<B>
    where
        A: Send + Sync,
        B: Send + Sync,
        F: FnMut(A) -> B + Send + Sync,
    {
        match (ff, fa) {
            (Some(mut f), Some(a)) => Some(f(a)),
            _ => None,
        }
    }

    #[inline]
    fn lift2<A, B, C, F>(mut f: F, fa: Option<A>, fb: Option<B>) -> Option<C>
    where
        A: Send + Sync,
        B: Send + Sync,
        C: Send + Sync,
        F: FnMut(A, B) -> C + Send + Sync,
    {
        match (fa, fb) {
            (Some(a), Some(b)) => Some(f(a, b)),
            _ => None,
        }
    }
}

impl MonadGenus for OptionGenus {
    #[inline]
    fn flat_map<A, B, F>(fa: Option<A>, f: F) -> Option<B>
    where
        A: Send + Sync,
        B: Send + Sync,
        F: FnMut(A) -> Option<B> + Send + Sync,
    {
        fa.and_then(f)
    }
}

impl FoldableGenus for OptionGenus {
    #[inline]
    fn fold_left<A, B, F>(fa: Option<A>, init: B, mut f: F) -> B
    where
        A: Send + Sync,
        F: FnMut(B, A) -> B,
    {
        match fa {
            Some(a) => f(init, a),
            None => init,
        }
    }

    #[inline]
    fn fold_right<A, B, F>(fa: Option<A>, init: B, mut f: F) -> B
    where
        A: Send + Sync,
        F: FnMut(A, B) -> B,
    {
        match fa {
            Some(a) => f(a, init),
            None => init,
        }
    }
}

/// Genus witness for `Result<_, E>`.
#[derive(Debug, Clone, Copy)]
pub struct ResultGenus<E>(PhantomData<E>);

impl<E> Default for ResultGenus<E> {
    fn default() -> Self {
        ResultGenus(PhantomData)
    }
}

impl<E: Send + Sync + 'static> Genus for ResultGenus<E> {
    type Applied<A>
        = Result<A, E>
    where
        A: Send + Sync;
}

impl<E: Send + Sync + 'static> FunctorGenus for ResultGenus<E> {
    #[inline]
    fn fmap<A, B, F>(fa: Result<A, E>, f: F) -> Result<B, E>
    where
        A: Send + Sync,
        B: Send + Sync,
        F: FnMut(A) -> B + Send + Sync,
    {
        fa.map(f)
    }
}

impl<E: Send + Sync + 'static> ApplicatioGenus for ResultGenus<E> {
    #[inline]
    fn purus<A>(a: A) -> Result<A, E>
    where
        A: Send + Sync,
    {
        Ok(a)
    }

    #[inline]
    fn ap<A, B, F>(ff: Result<F, E>, fa: Result<A, E>) -> Result<B, E>
    where
        A: Send + Sync,
        B: Send + Sync,
        F: FnMut(A) -> B + Send + Sync,
    {
        match (ff, fa) {
            (Ok(mut f), Ok(a)) => Ok(f(a)),
            (Err(e), _) => Err(e),
            (_, Err(e)) => Err(e),
        }
    }

    #[inline]
    fn lift2<A, B, C, F>(mut f: F, fa: Result<A, E>, fb: Result<B, E>) -> Result<C, E>
    where
        A: Send + Sync,
        B: Send + Sync,
        C: Send + Sync,
        F: FnMut(A, B) -> C + Send + Sync,
    {
        match (fa, fb) {
            (Ok(a), Ok(b)) => Ok(f(a, b)),
            (Err(e), _) => Err(e),
            (_, Err(e)) => Err(e),
        }
    }
}

impl<E: Send + Sync + 'static> MonadGenus for ResultGenus<E> {
    #[inline]
    fn flat_map<A, B, F>(fa: Result<A, E>, f: F) -> Result<B, E>
    where
        A: Send + Sync,
        B: Send + Sync,
        F: FnMut(A) -> Result<B, E> + Send + Sync,
    {
        fa.and_then(f)
    }
}

impl<E: Send + Sync + 'static> FoldableGenus for ResultGenus<E> {
    #[inline]
    fn fold_left<A, B, F>(fa: Result<A, E>, init: B, mut f: F) -> B
    where
        A: Send + Sync,
        F: FnMut(B, A) -> B,
    {
        match fa {
            Ok(a) => f(init, a),
            Err(_) => init,
        }
    }

    #[inline]
    fn fold_right<A, B, F>(fa: Result<A, E>, init: B, mut f: F) -> B
    where
        A: Send + Sync,
        F: FnMut(A, B) -> B,
    {
        match fa {
            Ok(a) => f(a, init),
            Err(_) => init,
        }
    }
}

/// Genus witness for `Vec`.
#[cfg(feature = "alloc")]
#[derive(Debug, Clone, Copy, Default)]
pub struct VecGenus;

#[cfg(feature = "alloc")]
impl Genus for VecGenus {
    type Applied<A>
        = Vec<A>
    where
        A: Send + Sync;
}

#[cfg(feature = "alloc")]
impl FunctorGenus for VecGenus {
    #[inline]
    fn fmap<A, B, F>(fa: Vec<A>, f: F) -> Vec<B>
    where
        A: Send + Sync,
        B: Send + Sync,
        F: FnMut(A) -> B + Send + Sync,
    {
        fa.into_iter().map(f).collect()
    }
}

#[cfg(feature = "alloc")]
impl FoldableGenus for VecGenus {
    #[inline]
    fn fold_left<A, B, F>(fa: Vec<A>, init: B, f: F) -> B
    where
        A: Send + Sync,
        F: FnMut(B, A) -> B,
    {
        fa.into_iter().fold(init, f)
    }

    #[inline]
    fn fold_right<A, B, F>(fa: Vec<A>, init: B, mut f: F) -> B
    where
        A: Send + Sync,
        F: FnMut(A, B) -> B,
    {
        fa.into_iter().rev().fold(init, |acc, a| f(a, acc))
    }
}

/// Genus witness for Identity (no wrapper).
#[derive(Debug, Clone, Copy, Default)]
pub struct IdentitasGenus;

impl Genus for IdentitasGenus {
    type Applied<A>
        = A
    where
        A: Send + Sync;
}

impl FunctorGenus for IdentitasGenus {
    #[inline]
    fn fmap<A, B, F>(fa: A, mut f: F) -> B
    where
        A: Send + Sync,
        B: Send + Sync,
        F: FnMut(A) -> B + Send + Sync,
    {
        f(fa)
    }
}

impl ApplicatioGenus for IdentitasGenus {
    #[inline]
    fn purus<A>(a: A) -> A
    where
        A: Send + Sync,
    {
        a
    }

    #[inline]
    fn ap<A, B, F>(ff: F, fa: A) -> B
    where
        A: Send + Sync,
        B: Send + Sync,
        F: FnMut(A) -> B + Send + Sync,
    {
        let mut ff = ff;
        ff(fa)
    }

    #[inline]
    fn lift2<A, B, C, F>(mut f: F, fa: A, fb: B) -> C
    where
        A: Send + Sync,
        B: Send + Sync,
        C: Send + Sync,
        F: FnMut(A, B) -> C + Send + Sync,
    {
        f(fa, fb)
    }
}

impl MonadGenus for IdentitasGenus {
    #[inline]
    fn flat_map<A, B, F>(fa: A, mut f: F) -> B
    where
        A: Send + Sync,
        B: Send + Sync,
        F: FnMut(A) -> B + Send + Sync,
    {
        f(fa)
    }
}

// =============================================================================
// Natural Transformations between Genera
// =============================================================================

/// A natural transformation between two Genera.
///
/// > *"Transformatio naturalis est morphismus functorum."*
/// > — A natural transformation is a morphism of functors.
///
/// For genera `F` and `G`, a natural transformation `η: F ~> G` provides
/// a way to transform `F::Applied<A>` to `G::Applied<A>` for all `A`.
pub trait TransformatioGenera<F: Genus, G: Genus> {
    /// Transform from genus F to genus G.
    fn transforma<A>(fa: F::Applied<A>) -> G::Applied<A>
    where
        A: Send + Sync;
}

/// Identity transformation - transforms a genus to itself.
#[derive(Debug, Clone, Copy, Default)]
pub struct TransformatioIdentitasGenera<F>(PhantomData<F>);

impl<F: Genus> TransformatioGenera<F, F> for TransformatioIdentitasGenera<F> {
    #[inline]
    fn transforma<A>(fa: F::Applied<A>) -> F::Applied<A>
    where
        A: Send + Sync,
    {
        fa
    }
}

/// Composition of natural transformations.
#[derive(Debug, Clone, Copy)]
pub struct TransformatioCompositaGenera<F, G, H, T1, T2> {
    _marker: PhantomData<(F, G, H, T1, T2)>,
}

impl<F, G, H, T1, T2> TransformatioGenera<F, H> for TransformatioCompositaGenera<F, G, H, T1, T2>
where
    F: Genus,
    G: Genus,
    H: Genus,
    T1: TransformatioGenera<F, G>,
    T2: TransformatioGenera<G, H>,
{
    #[inline]
    fn transforma<A>(fa: F::Applied<A>) -> H::Applied<A>
    where
        A: Send + Sync,
    {
        let ga = T1::transforma(fa);
        T2::transforma(ga)
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Regression for the accumulator-discarding fold_map default: over a
    /// multi-element structure it must combine every mapped element, not
    /// return `f(last)`.
    #[cfg(feature = "alloc")]
    #[test]
    fn fold_map_combines_all_elements() {
        use alloc::string::{String, ToString};

        let folded: String = VecGenus::fold_map(alloc::vec![1, 2, 3], |x: i32| x.to_string());
        assert_eq!(folded, "123");

        // Empty structure yields the monoid identity.
        let empty: String = VecGenus::fold_map(alloc::vec![] as Vec<i32>, |x: i32| x.to_string());
        assert_eq!(empty, "");
    }

    #[cfg(feature = "alloc")]
    extern crate alloc;
    #[cfg(feature = "alloc")]
    use alloc::vec;

    #[test]
    fn test_option_genus_functor() {
        let opt = Some(42);
        let doubled = OptionGenus::fmap(opt, |x| x * 2);
        assert_eq!(doubled, Some(84));
    }

    #[test]
    fn test_option_genus_none() {
        let opt: Option<i32> = None;
        let doubled = OptionGenus::fmap(opt, |x| x * 2);
        assert_eq!(doubled, None);
    }

    #[test]
    fn test_option_genus_applicative() {
        let a = Some(10);
        let result = OptionGenus::purus(42);
        assert_eq!(result, Some(42));

        let f: Option<fn(i32) -> i32> = Some(|x| x + 1);
        let applied = OptionGenus::ap(f, a);
        assert_eq!(applied, Some(11));
    }

    #[test]
    fn test_option_genus_monad() {
        let opt = Some(42);
        let result = OptionGenus::flat_map(opt, |x| if x > 0 { Some(x * 2) } else { None });
        assert_eq!(result, Some(84));
    }

    #[test]
    fn test_option_genus_monad_none() {
        let opt = Some(-1);
        let result = OptionGenus::flat_map(opt, |x| if x > 0 { Some(x * 2) } else { None });
        assert_eq!(result, None);
    }

    #[test]
    fn test_result_genus_functor() {
        let res: Result<i32, &str> = Ok(42);
        let doubled = ResultGenus::<&str>::fmap(res, |x| x * 2);
        assert_eq!(doubled, Ok(84));
    }

    #[test]
    fn test_result_genus_error() {
        let res: Result<i32, &str> = Err("error");
        let doubled = ResultGenus::<&str>::fmap(res, |x| x * 2);
        assert_eq!(doubled, Err("error"));
    }

    #[test]
    fn test_result_genus_monad() {
        let res: Result<i32, &str> = Ok(42);
        let result =
            ResultGenus::flat_map(res, |x| if x > 0 { Ok(x * 2) } else { Err("negative") });
        assert_eq!(result, Ok(84));
    }

    #[test]
    fn test_foldable_result_ok() {
        let res: Result<i32, &str> = Ok(42);
        let sum = ResultGenus::<&str>::fold_left(res, 0, |acc, x| acc + x);
        assert_eq!(sum, 42);
    }

    #[test]
    fn test_foldable_result_err() {
        let res: Result<i32, &str> = Err("error");
        let sum = ResultGenus::<&str>::fold_left(res, 10, |acc, x| acc + x);
        assert_eq!(sum, 10);
    }

    #[test]
    fn test_foldable_result_fold_right_ok() {
        let res: Result<i32, &str> = Ok(5);
        let result = ResultGenus::<&str>::fold_right(res, 10, |x, acc| x * acc);
        assert_eq!(result, 50);
    }

    #[test]
    fn test_foldable_result_fold_right_err() {
        let res: Result<i32, &str> = Err("error");
        let result = ResultGenus::<&str>::fold_right(res, 10, |x, acc| x * acc);
        assert_eq!(result, 10);
    }

    #[test]
    fn test_identity_genus() {
        let val = 42;
        let doubled = IdentitasGenus::fmap(val, |x| x * 2);
        assert_eq!(doubled, 84);
    }

    #[test]
    fn test_identity_genus_monad() {
        let val = 42;
        let result = IdentitasGenus::flat_map(val, |x| x + 1);
        assert_eq!(result, 43);
    }

    #[test]
    fn test_foldable_option() {
        let opt = Some(42);
        let sum = OptionGenus::fold_left(opt, 0, |acc, x| acc + x);
        assert_eq!(sum, 42);

        let none: Option<i32> = None;
        let sum_none = OptionGenus::fold_left(none, 0, |acc, x| acc + x);
        assert_eq!(sum_none, 0);
    }

    #[test]
    fn test_functor_laws_identity() {
        // fmap id = id
        let opt = Some(42);
        let result = OptionGenus::fmap(opt, |x| x);
        assert_eq!(result, Some(42));
    }

    #[test]
    fn test_monad_laws_left_identity() {
        // flat_map(purus(a), f) = f(a)
        let a = 42;
        let f = |x: i32| Some(x * 2);

        let left = OptionGenus::flat_map(OptionGenus::purus(a), f);
        let right = f(a);

        assert_eq!(left, right);
    }

    #[test]
    fn test_monad_laws_right_identity() {
        // flat_map(m, purus) = m
        let m = Some(42);
        let result = OptionGenus::flat_map(m, OptionGenus::purus);
        assert_eq!(result, Some(42));
    }

    // =========================================================================
    // VecGenus Tests
    // =========================================================================

    #[test]
    #[cfg(feature = "alloc")]
    fn test_vec_genus_fmap_empty() {
        let vec: Vec<i32> = vec![];
        let result = VecGenus::fmap(vec, |x| x * 2);
        assert_eq!(result, Vec::<i32>::new());
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_vec_genus_fmap_single_element() {
        let vec = vec![42];
        let result = VecGenus::fmap(vec, |x| x * 2);
        assert_eq!(result, vec![84]);
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_vec_genus_fmap_multiple_elements() {
        let vec = vec![1, 2, 3, 4, 5];
        let result = VecGenus::fmap(vec, |x| x * 2);
        assert_eq!(result, vec![2, 4, 6, 8, 10]);
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_vec_genus_fmap_string_transform() {
        let vec = vec!["hello", "world", "rust"];
        let result = VecGenus::fmap(vec, str::len);
        assert_eq!(result, vec![5, 5, 4]);
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_vec_genus_functor_law_identity() {
        // fmap id = id
        let vec = vec![1, 2, 3, 4, 5];
        let expected = vec.clone();
        let result = VecGenus::fmap(vec, |x| x);
        assert_eq!(result, expected);
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_vec_genus_functor_law_composition() {
        // fmap (g . f) = fmap g . fmap f
        let vec = vec![1, 2, 3, 4, 5];
        let f = |x: i32| x * 2;
        let g = |x: i32| x + 10;

        // Left side: fmap (g . f)
        let left = VecGenus::fmap(vec.clone(), |x| g(f(x)));

        // Right side: fmap g . fmap f
        let right = VecGenus::fmap(VecGenus::fmap(vec, f), g);

        assert_eq!(left, right);
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_vec_genus_fmap_with_complex_closure() {
        let vec = vec![1, 2, 3, 4, 5];
        let result = VecGenus::fmap(vec, |x| if x % 2 == 0 { x * 10 } else { x * 100 });
        assert_eq!(result, vec![100, 20, 300, 40, 500]);
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_vec_genus_void() {
        let vec = vec![1, 2, 3];
        let result = VecGenus::void(vec);
        assert_eq!(result, vec![(), (), ()]);
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_vec_genus_replace() {
        let vec = vec![1, 2, 3];
        let result = VecGenus::replace(vec, 42);
        assert_eq!(result, vec![42, 42, 42]);
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_vec_genus_fmap_large_vector() {
        // Test with a larger vector to ensure no issues with multiple elements
        let vec: Vec<i32> = (1..=100).collect();
        let result = VecGenus::fmap(vec, |x| x * x);
        let expected: Vec<i32> = (1..=100).map(|x| x * x).collect();
        assert_eq!(result, expected);
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_vec_genus_fmap_preserves_order() {
        let vec = vec![5, 3, 8, 1, 9, 2];
        let result = VecGenus::fmap(vec, |x| x * 2);
        assert_eq!(result, vec![10, 6, 16, 2, 18, 4]);
    }
}
