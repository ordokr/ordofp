//! Type-Level Peano Natural Numbers
//!
//! > *"Numerorum omnium principium est unitas."*
//! > — The principle of all numbers is unity. (Boethius)
//!
//! This module provides type-level natural numbers using the Peano encoding,
//! enabling compile-time arithmetic and length tracking.
//!
//! # Design
//!
//! Peano numbers are defined inductively:
//! - `Zero` represents 0
//! - `Succ<N>` represents N + 1
//!
//! This allows the type system to reason about quantities at compile time.
//!
//! # Example
//!
//! ```rust
//! use ordofp_core::dependent::peano::{Zero, Succ, N1, N2, N3, Naturalis};
//!
//! // Type-level numbers
//! type Five = Succ<Succ<Succ<Succ<Succ<Zero>>>>>;
//!
//! // Get runtime value
//! let n: usize = <N3 as Naturalis>::VALUE;
//! assert_eq!(n, 3);
//! ```

use core::marker::PhantomData;
use core::ops::Add;

// =============================================================================
// Core Peano Types
// =============================================================================

/// Type-level zero.
///
/// # Latin Etymology
/// *Nihil* means "nothing" - representing the absence of quantity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Zero;

/// Type-level successor (N + 1).
///
/// # Latin Etymology
/// *Successor* means "one who follows" - representing the next natural number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Succ<N>(PhantomData<N>);

impl<N> Succ<N> {
    /// Create a new successor type.
    pub const fn new() -> Self {
        Succ(PhantomData)
    }
}

// =============================================================================
// Type Aliases for Common Numbers
// =============================================================================

/// Type alias for 1
pub type N1 = Succ<Zero>;
/// Type alias for 2
pub type N2 = Succ<N1>;
/// Type alias for 3
pub type N3 = Succ<N2>;
/// Type alias for 4
pub type N4 = Succ<N3>;
/// Type alias for 5
pub type N5 = Succ<N4>;
/// Type alias for 6
pub type N6 = Succ<N5>;
/// Type alias for 7
pub type N7 = Succ<N6>;
/// Type alias for 8
pub type N8 = Succ<N7>;
/// Type alias for 9
pub type N9 = Succ<N8>;
/// Type alias for 10
pub type N10 = Succ<N9>;

// =============================================================================
// Naturalis Trait - Runtime Value Extraction
// =============================================================================

/// Trait for type-level natural numbers.
///
/// Provides the ability to get the runtime value of a type-level number.
///
/// # Latin Etymology
/// *Naturalis* means "natural" - as in natural numbers.
pub trait Naturalis {
    /// The runtime value of this type-level number.
    const VALUE: usize;

    /// Get the value at runtime.
    #[inline]
    fn value() -> usize {
        Self::VALUE
    }
}

impl Naturalis for Zero {
    const VALUE: usize = 0;
}

impl<N: Naturalis> Naturalis for Succ<N> {
    const VALUE: usize = N::VALUE + 1;
}

// =============================================================================
// NonNihil - Proof of Non-Zero
// =============================================================================

/// Marker trait for non-zero naturals.
///
/// # Latin Etymology
/// *Non Nihil* means "not nothing" - a non-zero number.
pub trait NonNihil: Naturalis {}

impl<N> NonNihil for Succ<N> where Succ<N>: Naturalis {}

// =============================================================================
// Type-Level Arithmetic
// =============================================================================

/// Type-level addition.
///
/// Computes `N + M` at the type level.
///
/// # Latin Etymology
/// *Additio* means "addition".
pub trait Additio<M>: Naturalis {
    /// The result of adding M to Self.
    type Summa: Naturalis;
}

impl<M: Naturalis> Additio<M> for Zero {
    type Summa = M;
}

impl<N: Naturalis + Additio<M>, M: Naturalis> Additio<M> for Succ<N>
where
    <N as Additio<M>>::Summa: Naturalis,
{
    type Summa = Succ<<N as Additio<M>>::Summa>;
}

/// Type-level multiplication.
///
/// Computes `N * M` at the type level.
///
/// # Latin Etymology
/// *Multiplicatio* means "multiplication".
pub trait Multiplicatio<M>: Naturalis {
    /// The result of multiplying Self by M.
    type Productum: Naturalis;
}

impl<M: Naturalis> Multiplicatio<M> for Zero {
    type Productum = Zero;
}

impl<N, M> Multiplicatio<M> for Succ<N>
where
    N: Naturalis + Multiplicatio<M>,
    M: Naturalis + Additio<<N as Multiplicatio<M>>::Productum>,
    <N as Multiplicatio<M>>::Productum: Naturalis,
    <M as Additio<<N as Multiplicatio<M>>::Productum>>::Summa: Naturalis,
{
    // (n+1) * m = m + (n * m)
    type Productum = <M as Additio<<N as Multiplicatio<M>>::Productum>>::Summa;
}

// =============================================================================
// Type-Level Comparison
// =============================================================================

/// Type-level less-than comparison.
///
/// `Minor<M>` is implemented when `Self < M`.
///
/// # Latin Etymology
/// *Minor* means "smaller, less".
pub trait Minor<M>: Naturalis {}

// Zero is less than any successor
impl<M: Naturalis> Minor<Succ<M>> for Zero {}

// Succ<N> < Succ<M> when N < M
impl<N: Naturalis + Minor<M>, M: Naturalis> Minor<Succ<M>> for Succ<N> {}

/// Type-level less-than-or-equal comparison.
///
/// `MinorVelAequus<M>` is implemented when `Self <= M`.
///
/// # Latin Etymology
/// *Minor vel Aequus* means "smaller or equal".
pub trait MinorVelAequus<M>: Naturalis {}

// 0 <= anything
impl<M: Naturalis> MinorVelAequus<M> for Zero {}

// Succ<N> <= Succ<M> when N <= M
impl<N: Naturalis + MinorVelAequus<M>, M: Naturalis> MinorVelAequus<Succ<M>> for Succ<N> {}

/// Type-level greater-than comparison.
///
/// `Maior<M>` is implemented when `Self > M`.
///
/// # Latin Etymology
/// *Maior* means "greater, larger".
pub trait Maior<M>: Naturalis {}

// Any successor is greater than zero
impl<N: Naturalis> Maior<Zero> for Succ<N> {}

// Succ<N> > Succ<M> when N > M
impl<N: Naturalis + Maior<M>, M: Naturalis> Maior<Succ<M>> for Succ<N> {}

/// Type-level equality.
///
/// `Aequus<M>` is implemented when `Self == M`.
///
/// # Latin Etymology
/// *Aequus* means "equal, level".
pub trait Aequus<M>: Naturalis {}

impl Aequus<Zero> for Zero {}

impl<N: Naturalis + Aequus<M>, M: Naturalis> Aequus<Succ<M>> for Succ<N> {}

// =============================================================================
// Type-Level Predecessor
// =============================================================================

/// Type-level predecessor (N - 1).
///
/// Only implemented for non-zero numbers.
///
/// # Latin Etymology
/// *Praecessor* means "one who goes before".
pub trait Praecessor: NonNihil {
    /// The predecessor type (N - 1).
    type Prior: Naturalis;
}

impl<N: Naturalis> Praecessor for Succ<N> {
    type Prior = N;
}

// =============================================================================
// Type-Level Subtraction (Saturating)
// =============================================================================

/// Type-level saturating subtraction.
///
/// Computes `max(Self - M, 0)` at the type level.
///
/// # Latin Etymology
/// *Subtractio* means "subtraction, taking away".
pub trait Subtractio<M>: Naturalis {
    /// The result of subtracting M from Self (saturating at zero).
    type Differentia: Naturalis;
}

// n - 0 = n
impl<N: Naturalis> Subtractio<Zero> for N {
    type Differentia = N;
}

// 0 - m = 0 (saturating)
impl<M: Naturalis> Subtractio<Succ<M>> for Zero {
    type Differentia = Zero;
}

// (n+1) - (m+1) = n - m
impl<N, M> Subtractio<Succ<M>> for Succ<N>
where
    N: Naturalis + Subtractio<M>,
    M: Naturalis,
{
    type Differentia = <N as Subtractio<M>>::Differentia;
}

// =============================================================================
// Type-Level Minimum and Maximum
// =============================================================================

/// Type-level minimum.
///
/// # Latin Etymology
/// *Minimus* means "smallest".
pub trait Minimus<M>: Naturalis {
    /// The smaller of Self and M.
    type Min: Naturalis;
}

impl<M: Naturalis> Minimus<M> for Zero {
    type Min = Zero;
}

impl<N: Naturalis> Minimus<Zero> for Succ<N> {
    type Min = Zero;
}

impl<N, M> Minimus<Succ<M>> for Succ<N>
where
    N: Naturalis + Minimus<M>,
    M: Naturalis,
    <N as Minimus<M>>::Min: Naturalis,
{
    type Min = Succ<<N as Minimus<M>>::Min>;
}

/// Type-level maximum.
///
/// # Latin Etymology
/// *Maximus* means "greatest".
pub trait Maximus<M>: Naturalis {
    /// The greater of Self and M.
    type Max: Naturalis;
}

impl<M: Naturalis> Maximus<M> for Zero {
    type Max = M;
}

impl<N: Naturalis> Maximus<Zero> for Succ<N> {
    type Max = Succ<N>;
}

impl<N, M> Maximus<Succ<M>> for Succ<N>
where
    N: Naturalis + Maximus<M>,
    M: Naturalis,
    <N as Maximus<M>>::Max: Naturalis,
{
    type Max = Succ<<N as Maximus<M>>::Max>;
}

// =============================================================================
// Convenience Type Operators
// =============================================================================

/// Convenience type alias for addition.
pub type Sum<A, B> = <A as Additio<B>>::Summa;

/// Convenience type alias for multiplication.
pub type Prod<A, B> = <A as Multiplicatio<B>>::Productum;

/// Convenience type alias for subtraction.
pub type Diff<A, B> = <A as Subtractio<B>>::Differentia;

/// Convenience type alias for minimum.
pub type Min<A, B> = <A as Minimus<B>>::Min;

/// Convenience type alias for maximum.
pub type Max<A, B> = <A as Maximus<B>>::Max;

/// Convenience type alias for predecessor.
pub type Pred<N> = <N as Praecessor>::Prior;

// =============================================================================
// std::ops implementations for value-level convenience
// =============================================================================

impl Add<Zero> for Zero {
    type Output = Zero;
    #[inline]
    fn add(self, _: Zero) -> Zero {
        Zero
    }
}

impl<N> Add<Succ<N>> for Zero {
    type Output = Succ<N>;
    #[inline]
    fn add(self, rhs: Succ<N>) -> Succ<N> {
        rhs
    }
}

impl<N, M> Add<M> for Succ<N>
where
    N: Add<M>,
{
    type Output = Succ<N::Output>;
    #[inline]
    fn add(self, _rhs: M) -> Self::Output {
        Succ::new()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
// Witness fns like `is_less_than<N: Minor<M>, M>()` use their type params only
// as bounds — instantiating them is the compile-time proof.
#[allow(clippy::extra_unused_type_parameters)]
mod tests {
    use super::*;

    #[test]
    fn test_naturalis_value() {
        assert_eq!(Zero::VALUE, 0);
        assert_eq!(<N1 as Naturalis>::VALUE, 1);
        assert_eq!(<N2 as Naturalis>::VALUE, 2);
        assert_eq!(<N3 as Naturalis>::VALUE, 3);
        assert_eq!(<N5 as Naturalis>::VALUE, 5);
        assert_eq!(<N10 as Naturalis>::VALUE, 10);
    }

    #[test]
    fn test_non_nihil() {
        fn requires_non_zero<N: NonNihil>() -> usize {
            N::VALUE
        }

        assert_eq!(requires_non_zero::<N1>(), 1);
        assert_eq!(requires_non_zero::<N5>(), 5);
        // This would not compile:
        // requires_non_zero::<Zero>();
    }

    #[test]
    fn test_additio() {
        // 0 + 3 = 3
        type ZeroPlusThree = Sum<Zero, N3>;
        assert_eq!(<ZeroPlusThree as Naturalis>::VALUE, 3);

        // 2 + 3 = 5
        type TwoPlusThree = Sum<N2, N3>;
        assert_eq!(<TwoPlusThree as Naturalis>::VALUE, 5);

        // 3 + 0 = 3
        type ThreePlusZero = Sum<N3, Zero>;
        assert_eq!(<ThreePlusZero as Naturalis>::VALUE, 3);
    }

    #[test]
    fn test_multiplicatio() {
        // 0 * 5 = 0
        type ZeroTimesFive = Prod<Zero, N5>;
        assert_eq!(<ZeroTimesFive as Naturalis>::VALUE, 0);

        // 2 * 3 = 6
        type TwoTimesThree = Prod<N2, N3>;
        assert_eq!(<TwoTimesThree as Naturalis>::VALUE, 6);

        // 3 * 2 = 6
        type ThreeTimesTwo = Prod<N3, N2>;
        assert_eq!(<ThreeTimesTwo as Naturalis>::VALUE, 6);
    }

    #[test]
    fn test_subtractio() {
        // 5 - 3 = 2
        type FiveMinusThree = Diff<N5, N3>;
        assert_eq!(<FiveMinusThree as Naturalis>::VALUE, 2);

        // 3 - 5 = 0 (saturating)
        type ThreeMinusFive = Diff<N3, N5>;
        assert_eq!(<ThreeMinusFive as Naturalis>::VALUE, 0);

        // 5 - 0 = 5
        type FiveMinusZero = Diff<N5, Zero>;
        assert_eq!(<FiveMinusZero as Naturalis>::VALUE, 5);
    }

    #[test]
    fn test_comparison_minor() {
        fn is_less_than<N: Minor<M>, M>() -> bool {
            true
        }

        // 0 < 1
        assert!(is_less_than::<Zero, N1>());
        // 2 < 5
        assert!(is_less_than::<N2, N5>());
        // 0 < 10
        assert!(is_less_than::<Zero, N10>());
    }

    #[test]
    fn test_comparison_maior() {
        fn is_greater_than<N: Maior<M>, M>() -> bool {
            true
        }

        // 1 > 0
        assert!(is_greater_than::<N1, Zero>());
        // 5 > 2
        assert!(is_greater_than::<N5, N2>());
    }

    #[test]
    fn test_comparison_aequus() {
        fn is_equal<N: Aequus<M>, M>() -> bool {
            true
        }

        assert!(is_equal::<Zero, Zero>());
        assert!(is_equal::<N3, N3>());
        assert!(is_equal::<N10, N10>());
    }

    #[test]
    fn test_min_max() {
        // min(3, 5) = 3
        type MinThreeFive = Min<N3, N5>;
        assert_eq!(<MinThreeFive as Naturalis>::VALUE, 3);

        // max(3, 5) = 5
        type MaxThreeFive = Max<N3, N5>;
        assert_eq!(<MaxThreeFive as Naturalis>::VALUE, 5);

        // min(0, 5) = 0
        type MinZeroFive = Min<Zero, N5>;
        assert_eq!(<MinZeroFive as Naturalis>::VALUE, 0);

        // max(0, 5) = 5
        type MaxZeroFive = Max<Zero, N5>;
        assert_eq!(<MaxZeroFive as Naturalis>::VALUE, 5);
    }

    #[test]
    fn test_praecessor() {
        type PredFive = Pred<N5>;
        assert_eq!(<PredFive as Naturalis>::VALUE, 4);

        type PredOne = Pred<N1>;
        assert_eq!(<PredOne as Naturalis>::VALUE, 0);
    }

    #[test]
    fn test_ops_add() {
        let _sum: Succ<Succ<Zero>> = Zero + Succ::<Succ<Zero>>::new();
        let _sum2: Succ<Succ<Succ<Zero>>> = Succ::<Zero>::new() + Succ::<Succ<Zero>>::new();
    }

    /// `MinorVelAequus` (≤) must hold both for strict inequality (`N < M`)
    /// and the boundary case (`N == M`).  The `Zero ≤ anything` base case
    /// and the reflexive case `N ≤ N` are the two most important edge cases
    /// not covered by the `Minor` / `Maior` tests above.
    #[test]
    fn test_comparison_minor_vel_aequus() {
        fn is_leq<N: MinorVelAequus<M>, M>() -> bool {
            true
        }

        // Base case: 0 <= 0 (reflexive equality at zero)
        assert!(is_leq::<Zero, Zero>());
        // 0 <= any successor (Zero is less than all Succ<_>)
        assert!(is_leq::<Zero, N1>());
        assert!(is_leq::<Zero, N5>());
        // Strict inequality: 2 <= 5
        assert!(is_leq::<N2, N5>());
        // Boundary / reflexive: N <= N for a non-zero number
        assert!(is_leq::<N3, N3>());
        assert!(is_leq::<N5, N5>());
    }
}
