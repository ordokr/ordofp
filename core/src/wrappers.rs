//! Wrapper types for alternative Compositio/Unitas behaviors.
//!
//! > *\"Multiplicatio est additio eiusdem numeri.\"*
//! > — Multiplication is the addition of the same number. (Medieval arithmetic)
//!
//! These newtypes allow selecting different algebraic instances for the same underlying type.
//! For example, numbers can be combined via addition (default) or multiplication (`Multiplicatio`).
//!
//! # Scholastic Names
//!
//! - `Aggregatio` (Sum) - additive combination
//! - `Multiplicatio` (Product) - multiplicative combination  
//! - `Omnis` (All) - universal quantifier, conjunction
//! - `Aliquid` (Any) - existential quantifier, disjunction
//! - `Primus` (First) - keeps the first/leftmost value
//! - `Ultimus` (Last) - keeps the last/rightmost value
//! - `Reflexio` (Endo) - self-referential morphism
//!
//! # Example
//!
//! ```rust
//! use ordofp_core::wrappers::{Multiplicatio, Max, Min, Omnis, Aliquid, Aggregatio, Primus, Ultimus};
//!
//! // Multiplicatio: multiplication instead of addition
//! // (requires Compositio/Unitas from typeclasses module)
//! let p = Multiplicatio(3);
//! assert_eq!(p.0, 3);
//!
//! // Max/Min: ordered combination
//! let max = Max(5);
//! let min = Min(2);
//!
//! // Omnis/Aliquid: boolean logic (bitwise for integers)
//! let all = Omnis(true);
//! let any = Aliquid(false);
//! ```

use core::cmp::Ordering;
use core::ops::{BitAnd, BitOr};

/// Generates the standard `new` constructor and `into_inner` extractor for a
/// single-field tuple-struct wrapper.
///
/// Two arms: one with no trait bound on `T`, one with a single `Ord` bound
/// (covers every wrapper in this module). The struct definition, derives, and
/// doc comments remain written by hand so each wrapper's uniqueness stays
/// visible at its own site.
macro_rules! newtype_wrapper {
    ($name:ident) => {
        impl<T> $name<T> {
            #[doc = concat!("Create a new ", stringify!($name), " wrapper.")]
            #[inline]
            pub fn new(value: T) -> Self {
                $name(value)
            }

            /// Get the inner value.
            #[inline]
            pub fn into_inner(self) -> T {
                self.0
            }
        }
    };
    ($name:ident, T: Ord) => {
        impl<T: Ord> $name<T> {
            #[doc = concat!("Create a new ", stringify!($name), " wrapper.")]
            #[inline]
            pub fn new(value: T) -> Self {
                $name(value)
            }

            /// Get the inner value.
            #[inline]
            pub fn into_inner(self) -> T {
                self.0
            }
        }
    };
}

/// Multiplicatio: Wrapper for multiplicative combination.
///
/// > *\"Multiplicatio est additio eiusdem numeri.\"*
/// > — Multiplication is the addition of the same number.
///
/// While numbers default to additive Compositio (`combine` = `+`),
/// `Multiplicatio<T>` uses multiplication (`combine` = `*`).
///
/// # Example
///
/// ```rust
/// use ordofp_core::wrappers::Multiplicatio;
/// // With Compositio: Multiplicatio(2).combine(&Multiplicatio(3)) == Multiplicatio(6)
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Multiplicatio<T>(pub T);
newtype_wrapper!(Multiplicatio);

/// Aggregatio: Wrapper for additive combination (explicit version of default behavior).
///
/// > *\"Aggregatio est compositio plurium in unum.\"*
/// > — Aggregation is the composition of many into one.
///
/// This is mostly for symmetry with `Multiplicatio`. Numbers already use addition
/// by default, but `Aggregatio` makes the intent explicit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Aggregatio<T>(pub T);
newtype_wrapper!(Aggregatio);

/// Wrapper for maximum-based combination.
///
/// `combine` returns the larger of two values.
///
/// # Example
///
/// ```rust
/// use ordofp_core::wrappers::Max;
/// // With Semigroup: Max(3).combine(&Max(5)) == Max(5)
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Max<T: Ord>(pub T);
newtype_wrapper!(Max, T: Ord);

impl<T: Ord> Max<T> {
    /// Combine two Max values, returning the larger.
    #[inline]
    pub fn max_of(self, other: Self) -> Self {
        match self.0.cmp(&other.0) {
            Ordering::Less => other,
            _ => self,
        }
    }
}

/// Wrapper for minimum-based combination.
///
/// `combine` returns the smaller of two values.
///
/// # Example
///
/// ```rust
/// use ordofp_core::wrappers::Min;
/// // With Semigroup: Min(3).combine(&Min(5)) == Min(3)
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Min<T: Ord>(pub T);
newtype_wrapper!(Min, T: Ord);

impl<T: Ord> Min<T> {
    /// Combine two Min values, returning the smaller.
    #[inline]
    pub fn min_of(self, other: Self) -> Self {
        match self.0.cmp(&other.0) {
            Ordering::Less => self,
            _ => other,
        }
    }
}

/// Omnis: Wrapper for conjunctive (AND) combination.
///
/// > *\"Omnis propositio universalis affirmativa.\"*
/// > — Every universal affirmative proposition. (Aristotle, Prior Analytics)
///
/// For booleans: `combine` = logical AND.
/// For integers: `combine` = bitwise AND.
///
/// # Example
///
/// ```rust
/// use ordofp_core::wrappers::Omnis;
/// // With Compositio:
/// // Omnis(true).combine(&Omnis(false)) == Omnis(false)
/// // Omnis(0b1100).combine(&Omnis(0b1010)) == Omnis(0b1000)
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Omnis<T>(pub T);
newtype_wrapper!(Omnis);

impl<T: BitAnd<Output = T>> Omnis<T> {
    /// Combine two Omnis values using bitwise AND.
    #[inline]
    pub fn and_with(self, other: Self) -> Self {
        Omnis(self.0.bitand(other.0))
    }
}

/// Aliquid: Wrapper for disjunctive (OR) combination.
///
/// > *\"Aliquid est ens quod est aliud quid.\"*
/// > — Something is a being that is other than nothing. (Aquinas)
///
/// For booleans: `combine` = logical OR.
/// For integers: `combine` = bitwise OR.
///
/// # Example
///
/// ```rust
/// use ordofp_core::wrappers::Aliquid;
/// // With Compositio:
/// // Aliquid(true).combine(&Aliquid(false)) == Aliquid(true)
/// // Aliquid(0b1100).combine(&Aliquid(0b0011)) == Aliquid(0b1111)
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Aliquid<T>(pub T);
newtype_wrapper!(Aliquid);

impl<T: BitOr<Output = T>> Aliquid<T> {
    /// Combine two Aliquid values using bitwise OR.
    #[inline]
    pub fn or_with(self, other: Self) -> Self {
        Aliquid(self.0.bitor(other.0))
    }
}

/// Primus: Wrapper that keeps the first (leftmost) value.
///
/// > *\"Primum in intentione, ultimum in executione.\"*
/// > — First in intention, last in execution. (Scholastic maxim)
///
/// `combine` always returns the first operand.
///
/// # Example
///
/// ```rust
/// use ordofp_core::wrappers::Primus;
/// // With Compositio: Primus(1).combine(&Primus(2)) == Primus(1)
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Primus<T>(pub T);
newtype_wrapper!(Primus);

/// Ultimus: Wrapper that keeps the last (rightmost) value.
///
/// > *\"Ultimus finis beatitudo est.\"*
/// > — The ultimate end is happiness. (Aquinas, Summa Theologica)
///
/// `combine` always returns the second operand.
///
/// # Example
///
/// ```rust
/// use ordofp_core::wrappers::Ultimus;
/// // With Compositio: Ultimus(1).combine(&Ultimus(2)) == Ultimus(2)
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Ultimus<T>(pub T);
newtype_wrapper!(Ultimus);

/// Reflexio: Wrapper for endomorphisms (functions from T to T).
///
/// > *\"Reflexio est actus quo intellectus in seipsum redit.\"*
/// > — Reflection is the act by which the intellect returns to itself. (Aquinas)
///
/// `combine` composes the functions.
/// This is useful for building up transformations.
#[derive(Clone)]
pub struct Reflexio<T> {
    /// The wrapped function.
    pub run: fn(T) -> T,
}

impl<T> Reflexio<T> {
    /// Create a new Reflexio wrapper.
    #[inline]
    pub fn new(f: fn(T) -> T) -> Self {
        Reflexio { run: f }
    }

    /// Apply the endomorphism to a value.
    #[inline]
    pub fn apply(&self, x: T) -> T {
        (self.run)(x)
    }
}

// Note: Reflexio composition is not supported because Reflexio stores a function pointer
// (fn(T) -> T), not a closure. To compose endomorphisms, apply them sequentially
// or use a different representation like Box<dyn Fn(T) -> T>.

impl<T> core::fmt::Debug for Reflexio<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Reflexio").finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multiplicatio() {
        let p1 = Multiplicatio(3);
        let p2 = Multiplicatio(4);
        assert_eq!(p1.0 * p2.0, 12);
        assert_eq!(p1.into_inner(), 3);
    }

    #[test]
    fn test_aggregatio() {
        let s1 = Aggregatio(3);
        let s2 = Aggregatio(4);
        assert_eq!(s1.0 + s2.0, 7);
    }

    #[test]
    fn test_max() {
        let m1 = Max(3);
        let m2 = Max(5);
        assert_eq!(m1.max_of(m2), Max(5));
        assert_eq!(m2.max_of(m1), Max(5));
    }

    #[test]
    fn test_min() {
        let m1 = Min(3);
        let m2 = Min(5);
        assert_eq!(m1.min_of(m2), Min(3));
        assert_eq!(m2.min_of(m1), Min(3));
    }

    #[test]
    fn test_max_equal_values() {
        let m = Max(7);
        assert_eq!(m.max_of(Max(7)), Max(7));
    }

    #[test]
    fn test_min_equal_values() {
        let m = Min(7);
        assert_eq!(m.min_of(Min(7)), Min(7));
    }

    #[test]
    fn test_omnis() {
        assert_eq!(Omnis(true).and_with(Omnis(true)), Omnis(true));
        assert_eq!(Omnis(true).and_with(Omnis(false)), Omnis(false));
        assert_eq!(Omnis(0b1100u8).and_with(Omnis(0b1010u8)), Omnis(0b1000u8));
    }

    #[test]
    fn test_aliquid() {
        assert_eq!(Aliquid(false).or_with(Aliquid(false)), Aliquid(false));
        assert_eq!(Aliquid(true).or_with(Aliquid(false)), Aliquid(true));
        assert_eq!(
            Aliquid(0b1100u8).or_with(Aliquid(0b0011u8)),
            Aliquid(0b1111u8)
        );
    }

    #[test]
    fn test_omnis_zero_is_annihilator() {
        // AND with zero must always yield zero regardless of the other operand.
        assert_eq!(Omnis(0u8).and_with(Omnis(0xFF)), Omnis(0u8));
        assert_eq!(Omnis(0xFF_u8).and_with(Omnis(0u8)), Omnis(0u8));
        // Zero is its own fixed-point under AND.
        assert_eq!(Omnis(0u8).and_with(Omnis(0u8)), Omnis(0u8));
    }

    #[test]
    fn test_aliquid_zero_is_identity() {
        // OR with zero must leave the other operand unchanged.
        assert_eq!(Aliquid(0u8).or_with(Aliquid(0b1010u8)), Aliquid(0b1010u8));
        assert_eq!(Aliquid(0b1010u8).or_with(Aliquid(0u8)), Aliquid(0b1010u8));
        // Zero is its own fixed-point under OR.
        assert_eq!(Aliquid(0u8).or_with(Aliquid(0u8)), Aliquid(0u8));
    }

    #[test]
    fn test_primus() {
        let f1 = Primus(1);
        let f2 = Primus(2);
        assert_eq!(f1.0, 1);
        assert_eq!(f2.0, 2);
    }

    #[test]
    fn test_primus_combine_always_keeps_first() {
        use crate::typeclasses::Compositio;
        // Primus must ignore the second operand — this is its defining invariant.
        assert_eq!(Primus(42).combine(&Primus(99)), Primus(42));
        // Associativity: chaining three values still yields the leftmost.
        let a = Primus("a");
        let b = Primus("b");
        let c = Primus("c");
        assert_eq!(a.combine(&b).combine(&c), a.combine(&b.combine(&c)));
    }

    #[test]
    fn test_ultimus() {
        let l1 = Ultimus(1);
        let l2 = Ultimus(2);
        assert_eq!(l1.0, 1);
        assert_eq!(l2.0, 2);
    }

    #[test]
    fn test_ultimus_combine_always_keeps_last() {
        use crate::typeclasses::Compositio;
        // Ultimus must ignore the first operand — this is its defining invariant.
        assert_eq!(Ultimus(42).combine(&Ultimus(99)), Ultimus(99));
        // Associativity: chaining three values still yields the rightmost.
        let a = Ultimus("a");
        let b = Ultimus("b");
        let c = Ultimus("c");
        assert_eq!(a.combine(&b).combine(&c), a.combine(&b.combine(&c)));
    }

    #[test]
    fn test_reflexio() {
        let add_one = Reflexio::new(|x: i32| x + 1);
        let double = Reflexio::new(|x: i32| x * 2);

        assert_eq!(add_one.apply(5), 6);
        assert_eq!(double.apply(5), 10);

        // Compose manually: double(add_one(x)) = (x + 1) * 2
        let result = double.apply(add_one.apply(5));
        assert_eq!(result, 12);
    }
}
