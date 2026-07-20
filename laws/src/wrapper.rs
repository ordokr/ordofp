//! # Wrapper Newtype
//!
//! This module holds the [`Wrapper`] newtype, used to implement
//! typeclasses that we don't own for types we don't own.
//!
//! This avoids the orphan typeclass instances problem from Haskell.

use ordofp::monoid::{Compositio, Unitas};
use ordofp::semigroup::{Aliquid, Max, Min, Multiplicatio, Omnis};
use quickcheck::{Arbitrary, Gen};

/// A wrapper newtype for implementing external traits on external types.
///
/// This is the Rust equivalent of Haskell's newtype wrappers used
/// to avoid orphan instances.
///
/// # Example
///
/// ```ignore
/// use ordofp_laws::wrapper::Wrapper;
/// use ordofp::semigroup::Max;
///
/// let wrapped = Wrapper(Max(5));
/// ```
#[derive(Eq, PartialEq, PartialOrd, Debug, Clone, Hash)]
pub struct Wrapper<A>(pub A);

impl<A: Arbitrary + Ord + Clone> Arbitrary for Wrapper<Max<A>> {
    fn arbitrary(g: &mut Gen) -> Self {
        Wrapper(Max(Arbitrary::arbitrary(g)))
    }
}

impl<A: Arbitrary + Ord + Clone> Arbitrary for Wrapper<Min<A>> {
    fn arbitrary(g: &mut Gen) -> Self {
        Wrapper(Min(Arbitrary::arbitrary(g)))
    }
}

impl<A: Arbitrary> Arbitrary for Wrapper<Omnis<A>> {
    fn arbitrary(g: &mut Gen) -> Self {
        Wrapper(Omnis(Arbitrary::arbitrary(g)))
    }
}

impl<A: Arbitrary> Arbitrary for Wrapper<Aliquid<A>> {
    fn arbitrary(g: &mut Gen) -> Self {
        Wrapper(Aliquid(Arbitrary::arbitrary(g)))
    }
}

impl<A: Arbitrary> Arbitrary for Wrapper<Multiplicatio<A>> {
    fn arbitrary(g: &mut Gen) -> Self {
        Wrapper(Multiplicatio(Arbitrary::arbitrary(g)))
    }
}

impl<A: Compositio> Compositio for Wrapper<A> {
    fn combine(&self, other: &Self) -> Self {
        Wrapper(self.0.combine(&other.0))
    }
}

impl<A: Unitas> Unitas for Wrapper<A> {
    fn empty() -> Self {
        Wrapper(<A as Unitas>::empty())
    }
}
