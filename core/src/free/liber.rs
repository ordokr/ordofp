//! Liber - Free Monad
//!
//! > *"Liber est qui non servit."*
//! > — Free is he who does not serve. (Seneca)
//!
//! The Free monad builds a monad from any functor, allowing DSL
//! construction and multiple interpretations.

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::boxed::Box;

use crate::typeclasses::hkt::{FunctorHKT, HKT};

// =============================================================================
// Liber - Free Monad
// =============================================================================

/// Free monad over functor F.
///
/// `Liber<F, A>` represents a computation that either:
/// - `Purus(a)`: immediately returns a value `a`
/// - `Suspensus(fa)`: suspends a computation `F<Liber<F, A>>`
///
/// The Free monad gives you a monad for any functor, enabling:
/// - Building DSLs as data structures
/// - Multiple interpretations via natural transformations
/// - Testability through mock interpreters
///
/// ```text
/// data Free f a = Pure a | Free (f (Free f a))
/// ```
///
/// # Latin Etymology
///
/// *Liber* = free, at liberty
///
/// # Example
///
/// ```rust
/// use ordofp_core::free::{Liber, OptionFWitness, plica_liber};
///
/// // Build a program over the `Option` functor
/// let program: Liber<OptionFWitness, i32> = Liber::purus(42);
///
/// // Chain operations
/// let chained = program.flat_map(|x| Liber::purus(x + 1));
///
/// // Interpret back into `Option` via the identity natural transformation
/// let result = plica_liber::<OptionFWitness, OptionFWitness, i32, _>(|fa| fa, chained);
/// assert_eq!(result, Some(43));
/// ```
#[cfg(feature = "alloc")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Liber<F: FunctorHKT, A> {
    /// Pure value - computation completes immediately.
    ///
    /// # Latin Etymology
    /// *Purus* = pure, clean
    Purus(A),

    /// Suspended computation - one layer of the functor.
    ///
    /// # Latin Etymology
    /// *Suspensus* = hanging, suspended
    Suspensus(Box<F::Target<Liber<F, A>>>),
}

#[cfg(feature = "alloc")]
impl<F: FunctorHKT, A> Liber<F, A> {
    /// Create a pure (immediate) value.
    ///
    /// This is the `return` / `pure` of the monad.
    ///
    /// ```rust
    /// use ordofp_core::free::{Liber, OptionFWitness};
    ///
    /// let x: Liber<OptionFWitness, i32> = Liber::purus(42);
    /// assert!(x.est_purus());
    /// ```
    #[inline]
    pub fn purus(a: A) -> Self {
        Liber::Purus(a)
    }

    /// Create a suspended computation from a functor value.
    #[inline]
    pub fn suspensus(fa: F::Target<Liber<F, A>>) -> Self {
        Liber::Suspensus(Box::new(fa))
    }

    /// Check if this is a pure value.
    #[inline]
    pub fn est_purus(&self) -> bool {
        matches!(self, Liber::Purus(_))
    }

    /// Check if this is a suspended computation.
    #[inline]
    pub fn est_suspensus(&self) -> bool {
        matches!(self, Liber::Suspensus(_))
    }

    /// Map a function over the result type.
    ///
    /// This is the functor `fmap` operation.
    #[inline]
    pub fn map<B, G>(self, f: G) -> Liber<F, B>
    where
        G: Fn(A) -> B + Clone,
    {
        match self {
            Liber::Purus(a) => Liber::Purus(f(a)),
            Liber::Suspensus(fa) => {
                let mapped = F::map(*fa, |child| child.map(f.clone()));
                Liber::Suspensus(Box::new(mapped))
            }
        }
    }

    /// Monadic bind (flatMap).
    ///
    /// This is the core monadic operation that allows sequencing
    /// computations in the Free monad.
    ///
    /// ```rust
    /// use ordofp_core::free::{Liber, OptionFWitness};
    ///
    /// let program: Liber<OptionFWitness, i32> = Liber::purus(42)
    ///     .flat_map(|x| Liber::purus(x + 1))
    ///     .flat_map(|x| Liber::purus(x * 2));
    /// match program {
    ///     Liber::Purus(x) => assert_eq!(x, 86),
    ///     Liber::Suspensus(_) => panic!("expected Purus"),
    /// }
    /// ```
    #[inline]
    pub fn flat_map<B, G>(self, f: G) -> Liber<F, B>
    where
        G: Fn(A) -> Liber<F, B> + Clone,
    {
        match self {
            Liber::Purus(a) => f(a),
            Liber::Suspensus(fa) => {
                let mapped = F::map(*fa, |child| child.flat_map(f.clone()));
                Liber::Suspensus(Box::new(mapped))
            }
        }
    }

    /// Lift a functor value into the Free monad.
    ///
    /// This is the fundamental operation for building Free monad programs.
    /// It takes a single functor operation and wraps it.
    ///
    /// ```rust
    /// use ordofp_core::free::{Liber, OptionFWitness};
    ///
    /// // Lift a single `Option` operation into the Free monad
    /// let lifted: Liber<OptionFWitness, i32> = Liber::lift_f(Some(42));
    /// assert!(lifted.est_suspensus());
    /// ```
    #[inline]
    pub fn lift_f(fa: F::Target<A>) -> Self
    where
        A: Clone,
    {
        Liber::Suspensus(Box::new(F::map(fa, Liber::purus)))
    }
}

// =============================================================================
// Fold Free - Interpretation
// =============================================================================

/// Fold a Free monad using a natural transformation.
///
/// This is the key function for interpreting Free monad programs.
/// Given a natural transformation `η: F ~> G` where `G` is a monad,
/// we can interpret `Liber<F, A>` into `G<A>`.
///
/// # Latin Etymology
///
/// *Plico Liber* = fold the free
///
/// # Type Parameters
///
/// * `F` - The functor of the Free monad
/// * `G` - The target monad for interpretation
/// * `Nat` - The natural transformation from F to G
/// * `A` - The result type
///
/// # Example
///
/// ```rust
/// use ordofp_core::free::{Liber, OptionFWitness, plica_liber};
///
/// let program: Liber<OptionFWitness, i32> = Liber::lift_f(Some(42));
///
/// // Interpret an `Option`-based program back into `Option` via the
/// // identity natural transformation.
/// let result = plica_liber::<OptionFWitness, OptionFWitness, i32, _>(|fa| fa, program);
/// assert_eq!(result, Some(42));
/// ```
#[cfg(feature = "alloc")]
#[inline]
pub fn plica_liber<F, G, A, NatFn>(nat: NatFn, free: Liber<F, A>) -> G::Target<A>
where
    F: FunctorHKT,
    G: FunctorHKT + MonadHKT,
    NatFn: Fn(F::Target<Liber<F, A>>) -> G::Target<Liber<F, A>> + Clone,
{
    plica_liber_impl::<F, G, A, NatFn>(nat, free)
}

#[cfg(feature = "alloc")]
#[inline]
fn plica_liber_impl<F, G, A, NatFn>(nat: NatFn, free: Liber<F, A>) -> G::Target<A>
where
    F: FunctorHKT,
    G: FunctorHKT + MonadHKT,
    NatFn: Fn(F::Target<Liber<F, A>>) -> G::Target<Liber<F, A>> + Clone,
{
    match free {
        Liber::Purus(a) => G::purus(a),
        Liber::Suspensus(fa) => {
            let ga: G::Target<Liber<F, A>> = nat(*fa);
            G::flat_map(ga, move |next| {
                plica_liber_impl::<F, G, A, NatFn>(nat, next)
            })
        }
    }
}

// =============================================================================
// MonadHKT - Monad for HKT witnesses
// =============================================================================

/// Monad operations for HKT witnesses.
///
/// This extends `FunctorHKT` with monadic operations.
pub trait MonadHKT: FunctorHKT {
    /// Wrap a pure value.
    fn purus<A>(a: A) -> Self::Target<A>;

    /// Monadic bind.
    fn flat_map<A, B, F>(fa: Self::Target<A>, f: F) -> Self::Target<B>
    where
        F: FnOnce(A) -> Self::Target<B>;
}

/// `MonadHKT` for Option.
impl MonadHKT for super::nat::OptionFWitness {
    fn purus<A>(a: A) -> Option<A> {
        Some(a)
    }

    fn flat_map<A, B, F>(fa: Option<A>, f: F) -> Option<B>
    where
        F: FnOnce(A) -> Option<B>,
    {
        fa.and_then(f)
    }
}

/// `MonadHKT` for Result.
impl<E: Clone> MonadHKT for super::nat::ResultFWitness<E> {
    fn purus<A>(a: A) -> Result<A, E> {
        Ok(a)
    }

    fn flat_map<A, B, F>(fa: Result<A, E>, f: F) -> Result<B, E>
    where
        F: FnOnce(A) -> Result<B, E>,
    {
        fa.and_then(f)
    }
}

/// `MonadHKT` for Identity.
impl MonadHKT for super::nat::IdentitasFWitness {
    fn purus<A>(a: A) -> A {
        a
    }

    fn flat_map<A, B, F>(fa: A, f: F) -> B
    where
        F: FnOnce(A) -> B,
    {
        f(fa)
    }
}

// =============================================================================
// Liber HKT Witness
// =============================================================================

/// HKT witness for Liber.
#[cfg(feature = "alloc")]
pub struct LiberWitness<F: FunctorHKT>(core::marker::PhantomData<F>);

#[cfg(feature = "alloc")]
impl<F: FunctorHKT> HKT for LiberWitness<F> {
    type Target<A> = Liber<F, A>;
}

#[cfg(feature = "alloc")]
impl<F: FunctorHKT> FunctorHKT for LiberWitness<F> {
    /// # Panics
    ///
    /// Panics if `fa` is `Liber::Suspensus` — this witness only maps pure
    /// (`Purus`) values; mapping through a suspended layer would require a
    /// `Clone` bound on the mapping function that this trait cannot express.
    fn map<A, B, G>(fa: Liber<F, A>, mut f: G) -> Liber<F, B>
    where
        G: FnMut(A) -> B,
    {
        // Note: map requires Clone for the recursive case
        // This is a simplified version for pure values
        match fa {
            Liber::Purus(a) => Liber::Purus(f(a)),
            Liber::Suspensus(_) => {
                // For the full implementation, we'd need Clone on G
                panic!("map on suspended Liber requires Clone")
            }
        }
    }
}

// =============================================================================
// Iteration (simplified interpreter)
// =============================================================================

/// Iterate a Free monad, collapsing it with a step function.
///
/// This is a simpler form of interpretation when you don't need
/// a full natural transformation.
///
/// # Latin Etymology
///
/// *Itero Liber* = iterate the free
#[cfg(feature = "alloc")]
#[inline]
pub fn itero_liber<F, A, Step>(free: Liber<F, A>, step: Step) -> A
where
    F: FunctorHKT,
    Step: Fn(F::Target<A>) -> A + Clone,
{
    match free {
        Liber::Purus(a) => a,
        Liber::Suspensus(fa) => {
            let mapped = F::map(*fa, |child| itero_liber(child, step.clone()));
            step(mapped)
        }
    }
}

// =============================================================================
// Utility Functions
// =============================================================================

/// Join nested Free monads.
///
/// Collapses `Liber<F, Liber<F, A>>` into `Liber<F, A>`.
#[cfg(feature = "alloc")]
#[inline]
pub fn join_liber<F: FunctorHKT, A>(nested: Liber<F, Liber<F, A>>) -> Liber<F, A> {
    nested.flat_map(|x| x)
}

/// Wrap a value in Liber (alias for purus).
#[cfg(feature = "alloc")]
#[inline]
pub fn purus_liber<F: FunctorHKT, A>(a: A) -> Liber<F, A> {
    Liber::purus(a)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::super::nat::OptionFWitness;
    use super::*;

    #[test]
    fn test_purus() {
        let free: Liber<OptionFWitness, i32> = Liber::purus(42);
        assert!(free.est_purus());
        assert!(!free.est_suspensus());
    }

    #[test]
    fn test_map_purus() {
        let free: Liber<OptionFWitness, i32> = Liber::purus(42);
        let mapped = free.map(|x| x * 2);

        match mapped {
            Liber::Purus(x) => assert_eq!(x, 84),
            _ => panic!("Expected Purus"),
        }
    }

    #[test]
    fn test_flat_map_purus() {
        let free: Liber<OptionFWitness, i32> = Liber::purus(42);
        let chained = free.flat_map(|x| Liber::purus(x + 1));

        match chained {
            Liber::Purus(x) => assert_eq!(x, 43),
            _ => panic!("Expected Purus"),
        }
    }

    #[test]
    fn test_chain_operations() {
        let free: Liber<OptionFWitness, i32> = Liber::purus(10);
        let result = free
            .flat_map(|x| Liber::purus(x + 5))
            .flat_map(|x| Liber::purus(x * 2))
            .map(|x| x - 10);

        match result {
            Liber::Purus(x) => assert_eq!(x, 20), // ((10 + 5) * 2) - 10 = 20
            _ => panic!("Expected Purus"),
        }
    }

    #[test]
    fn test_monad_left_identity() {
        // pure a >>= f  ≡  f a
        let a = 42;
        let f = |x: i32| Liber::<OptionFWitness, i32>::purus(x * 2);

        let left: Liber<OptionFWitness, i32> = Liber::purus(a).flat_map(f);
        let right: Liber<OptionFWitness, i32> = f(a);

        match (left, right) {
            (Liber::Purus(l), Liber::Purus(r)) => assert_eq!(l, r),
            _ => panic!("Both should be Purus"),
        }
    }

    #[test]
    fn test_monad_right_identity() {
        // m >>= pure  ≡  m
        let m: Liber<OptionFWitness, i32> = Liber::purus(42);
        let result = m.flat_map(Liber::purus);

        match result {
            Liber::Purus(r) => assert_eq!(r, 42),
            _ => panic!("Should be Purus"),
        }
    }

    #[test]
    fn test_join() {
        let nested: Liber<OptionFWitness, Liber<OptionFWitness, i32>> =
            Liber::purus(Liber::purus(42));
        let joined = join_liber(nested);

        match joined {
            Liber::Purus(x) => assert_eq!(x, 42),
            _ => panic!("Expected Purus"),
        }
    }
}
