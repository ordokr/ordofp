//! Free monad - for futumorphisms and productive recursion.
//!
//! > *"Libertas est potestas faciendi quod velis."*
//! > — Freedom is the power to do what you wish.
//!
//! The Free monad allows embedding pure values or suspended computations,
//! enabling futumorphisms to produce multiple layers at once.

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::boxed::Box;

use crate::typeclasses::hkt::{FunctorHKT, HKT};

/// Free monad - suspended computation or pure value.
///
/// `Free F A` is either a pure `A` or a suspended `F (Free F A)`.
/// This allows anamorphisms to produce multiple layers at once
/// or terminate early with a pure value.
///
/// ```text
/// data Free f a = Pure a | Free (f (Free f a))
/// ```
///
/// # Scholastic Name: Liber
///
/// > *"Liber arbitrium."*
/// > — Free will/choice.
#[cfg(feature = "alloc")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Free<F: FunctorHKT, A> {
    /// Pure value - terminate recursion early.
    Purus(A),
    /// Suspended computation - continue unfolding.
    Suspensus(Box<F::Target<Free<F, A>>>),
}

#[cfg(feature = "alloc")]
impl<F: FunctorHKT, A> Free<F, A> {
    /// Create a pure value.
    #[inline]
    pub fn purus(a: A) -> Self {
        Free::Purus(a)
    }

    /// Create a suspended computation.
    #[inline]
    pub fn suspensus(layer: F::Target<Free<F, A>>) -> Self {
        Free::Suspensus(Box::new(layer))
    }

    /// Check if this is a pure value.
    #[inline]
    pub fn is_pure(&self) -> bool {
        matches!(self, Free::Purus(_))
    }

    /// Check if this is suspended.
    #[inline]
    pub fn is_suspended(&self) -> bool {
        matches!(self, Free::Suspensus(_))
    }

    /// Map over the value type.
    #[inline]
    pub fn map<B, G>(self, f: G) -> Free<F, B>
    where
        G: Fn(A) -> B + Clone,
    {
        self.map_impl(f)
    }

    fn map_impl<B, G>(self, f: G) -> Free<F, B>
    where
        G: Fn(A) -> B + Clone,
    {
        match self {
            Free::Purus(a) => Free::Purus(f(a)),
            Free::Suspensus(layer) => {
                let mapped = F::map(*layer, |child| child.map_impl(f.clone()));
                Free::Suspensus(Box::new(mapped))
            }
        }
    }

    /// Monadic bind (`flat_map`).
    #[inline]
    pub fn flat_map<B, G>(self, f: G) -> Free<F, B>
    where
        G: Fn(A) -> Free<F, B> + Clone,
    {
        self.flat_map_impl(f)
    }

    fn flat_map_impl<B, G>(self, f: G) -> Free<F, B>
    where
        G: Fn(A) -> Free<F, B> + Clone,
    {
        match self {
            Free::Purus(a) => f(a),
            Free::Suspensus(layer) => {
                let mapped = F::map(*layer, |child| child.flat_map_impl(f.clone()));
                Free::Suspensus(Box::new(mapped))
            }
        }
    }

    /// Lift a functor value into the free monad.
    #[inline]
    pub fn lift_f(fa: F::Target<A>) -> Self
    where
        A: Clone,
    {
        Free::Suspensus(Box::new(F::map(fa, Free::purus)))
    }
}

/// Witness for Free as an HKT.
#[cfg(feature = "alloc")]
pub struct FreeWitness<F: FunctorHKT>(core::marker::PhantomData<F>);

#[cfg(feature = "alloc")]
impl<F: FunctorHKT> HKT for FreeWitness<F> {
    type Target<A> = Free<F, A>;
}

// =============================================================================
// Free Monad Iteration
// =============================================================================

/// Iterate a Free monad structure.
///
/// Repeatedly unwraps the Free until reaching a pure value.
#[cfg(feature = "alloc")]
#[inline]
pub fn iter_free<F, A, Step>(free: Free<F, A>, step: Step) -> A
where
    F: FunctorHKT,
    Step: Fn(F::Target<A>) -> A + Clone,
{
    iter_free_impl(free, &step)
}

#[cfg(feature = "alloc")]
fn iter_free_impl<F, A, Step>(free: Free<F, A>, step: &Step) -> A
where
    F: FunctorHKT,
    Step: Fn(F::Target<A>) -> A,
{
    match free {
        Free::Purus(a) => a,
        Free::Suspensus(layer) => {
            let mapped = F::map(*layer, |child| iter_free_impl(child, step));
            step(mapped)
        }
    }
}

// =============================================================================
// Free for specific base functors
// =============================================================================

/// Free monad over natural numbers.
#[cfg(feature = "alloc")]
pub type FreeNat<A> = Free<crate::recursion::base::NatFWitness, A>;

/// Free monad over lists.
#[cfg(feature = "alloc")]
pub type FreeList<E, A> = Free<crate::recursion::base::ListFWitness<E>, A>;

/// Free monad over expressions.
#[cfg(feature = "alloc")]
pub type FreeExpr<A> = Free<crate::recursion::base::ExprFWitness, A>;

// =============================================================================
// Either for Apomorphisms
// =============================================================================

/// Either type for apomorphisms - allows early termination.
///
/// In an apomorphism, the coalgebra can return either:
/// - `Sinister(fixed)` - embed this pre-built structure directly
/// - `Dexter(seed)` - continue unfolding from this seed
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Aut<L, R> {
    /// Left - early termination with pre-built structure.
    Sinister(L),
    /// Right - continue unfolding.
    Dexter(R),
}

impl<L, R> Aut<L, R> {
    /// Create a left value.
    #[inline]
    pub fn sinister(l: L) -> Self {
        Aut::Sinister(l)
    }

    /// Create a right value.
    #[inline]
    pub fn dexter(r: R) -> Self {
        Aut::Dexter(r)
    }

    /// Check if left.
    #[inline]
    pub fn is_sinister(&self) -> bool {
        matches!(self, Aut::Sinister(_))
    }

    /// Check if right.
    #[inline]
    pub fn is_dexter(&self) -> bool {
        matches!(self, Aut::Dexter(_))
    }

    /// Map over the left value.
    #[inline]
    pub fn map_sinister<B, F>(self, f: F) -> Aut<B, R>
    where
        F: FnOnce(L) -> B,
    {
        match self {
            Aut::Sinister(l) => Aut::Sinister(f(l)),
            Aut::Dexter(r) => Aut::Dexter(r),
        }
    }

    /// Map over the right value.
    #[inline]
    pub fn map_dexter<B, F>(self, f: F) -> Aut<L, B>
    where
        F: FnOnce(R) -> B,
    {
        match self {
            Aut::Sinister(l) => Aut::Sinister(l),
            Aut::Dexter(r) => Aut::Dexter(f(r)),
        }
    }

    /// Bimap over both values.
    #[inline]
    pub fn bimap<A, B, F, G>(self, f: F, g: G) -> Aut<A, B>
    where
        F: FnOnce(L) -> A,
        G: FnOnce(R) -> B,
    {
        match self {
            Aut::Sinister(l) => Aut::Sinister(f(l)),
            Aut::Dexter(r) => Aut::Dexter(g(r)),
        }
    }
}
