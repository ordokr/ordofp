//! Fixed point of a functor.
//!
//! Useful for defining recursive data structures.

use crate::typeclasses::hkt::FunctorHKT;
#[cfg(feature = "alloc")]
use alloc::boxed::Box;

/// The fixed point of a functor `F`.
///
/// `Fix f` is the type `t` such that `t ~ f t`.
#[cfg(feature = "alloc")]
#[derive(Debug)]
pub struct Fix<F: FunctorHKT>(pub Box<F::Target<Fix<F>>>);

#[cfg(feature = "alloc")]
impl<F: FunctorHKT> Clone for Fix<F>
where
    F::Target<Fix<F>>: Clone,
{
    fn clone(&self) -> Self {
        Fix(self.0.clone())
    }
}

#[cfg(feature = "alloc")]
impl<F: FunctorHKT> PartialEq for Fix<F>
where
    F::Target<Fix<F>>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

#[cfg(feature = "alloc")]
impl<F: FunctorHKT> Eq for Fix<F> where F::Target<Fix<F>>: Eq {}

#[cfg(feature = "alloc")]
impl<F: FunctorHKT> Fix<F> {
    /// Creates a new `Fix`.
    #[inline]
    pub fn new(x: F::Target<Fix<F>>) -> Self {
        Fix(Box::new(x))
    }

    /// Unwraps one layer of the fixed point.
    #[inline]
    pub fn unfix(self) -> F::Target<Fix<F>> {
        *self.0
    }

    /// Catamorphism (fold).
    ///
    /// Tears down the structure layer by layer.
    ///
    /// # Stack usage
    ///
    /// This implementation is **recursive** — each layer of `Fix` consumes one
    /// stack frame.  Deep structures (tens of thousands of layers) risk stack
    /// overflow.  For production use on unbounded data consider running inside a
    /// larger thread stack or converting to a trampoline.
    #[inline]
    pub fn cata<A, Alg>(self, mut alg: Alg) -> A
    where
        Alg: FnMut(F::Target<A>) -> A,
    {
        self.cata_impl(&mut alg)
    }

    fn cata_impl<A, Alg>(self, alg: &mut Alg) -> A
    where
        Alg: FnMut(F::Target<A>) -> A,
    {
        let inner = self.unfix();
        // recursively map cata over the inner structure
        let mapped = F::map(inner, |sub| sub.cata_impl(alg));
        alg(mapped)
    }

    /// Anamorphism (unfold).
    ///
    /// Builds up the structure layer by layer.
    ///
    /// # Stack usage
    ///
    /// Like `cata`, this is **recursive**.  The depth of recursion equals the
    /// depth of the resulting `Fix` structure.  Apply the same caution as for
    /// `cata` when the seed produces deeply-nested output.
    #[inline]
    pub fn ana<A, Coalg>(a: A, mut coalg: Coalg) -> Self
    where
        Coalg: FnMut(A) -> F::Target<A>,
    {
        Self::ana_impl(a, &mut coalg)
    }

    fn ana_impl<A, Coalg>(a: A, coalg: &mut Coalg) -> Self
    where
        Coalg: FnMut(A) -> F::Target<A>,
    {
        let layer = coalg(a);
        let mapped = F::map(layer, |sub| Fix::ana_impl(sub, coalg));
        Fix::new(mapped)
    }
}

#[cfg(all(test, feature = "alloc"))]
mod tests {
    use super::*;
    use crate::typeclasses::hkt::HKT;

    // A natural-number functor: NatF<A> = Zero | Succ(A)
    #[derive(Clone, Debug, PartialEq)]
    enum NatF<A> {
        Zero,
        Succ(A),
    }

    struct NatHKT;

    impl HKT for NatHKT {
        type Target<T> = NatF<T>;
    }

    impl FunctorHKT for NatHKT {
        fn map<A, B, G: FnMut(A) -> B>(fa: NatF<A>, mut f: G) -> NatF<B> {
            match fa {
                NatF::Zero => NatF::Zero,
                NatF::Succ(a) => NatF::Succ(f(a)),
            }
        }
    }

    #[test]
    fn cata_on_depth_zero_returns_base_case() {
        // Edge case: Fix(Zero) has no recursive children, so the catamorphism
        // algebra is invoked exactly once with the base variant.  This tests
        // that cata correctly terminates without any recursive descent.
        let zero: Fix<NatHKT> = Fix::new(NatF::Zero);
        let depth = zero.cata(|nf| match nf {
            NatF::Zero => 0usize,
            NatF::Succ(n) => n + 1,
        });
        assert_eq!(
            depth, 0,
            "cata on Fix(Zero) must return the base-case value"
        );
    }

    #[test]
    fn cata_counts_depth_correctly() {
        // Build Fix(Succ(Fix(Succ(Fix(Zero))))) = depth 2 manually.
        let two: Fix<NatHKT> = Fix::new(NatF::Succ(Fix::new(NatF::Succ(Fix::new(NatF::Zero)))));
        let depth = two.cata(|nf| match nf {
            NatF::Zero => 0usize,
            NatF::Succ(n) => n + 1,
        });
        assert_eq!(depth, 2, "cata must count nested Fix layers correctly");
    }

    #[test]
    fn ana_builds_natural_number_and_cata_round_trips() {
        // `ana` (anamorphism) unfolds a seed into a Fix structure layer by layer.
        // The coalgebra here decodes a usize into our NatF functor: 0 becomes Zero,
        // n becomes Succ(n-1).  Running `cata` over the result with the inverse
        // algebra must recover the original seed, verifying the ana/cata round-trip.
        let n = 5usize;
        let nat: Fix<NatHKT> = Fix::ana(n, |k| {
            if k == 0 {
                NatF::Zero
            } else {
                NatF::Succ(k - 1)
            }
        });

        // Verify the structure has the right depth via cata.
        let recovered = nat.cata(|nf| match nf {
            NatF::Zero => 0usize,
            NatF::Succ(m) => m + 1,
        });
        assert_eq!(recovered, n, "cata(ana(n)) must recover the original seed");
    }
}
