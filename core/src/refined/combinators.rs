//! Predicate Combinators
//!
//! > *"Ex pluribus unum"*
//! > — From many, one. (Latin)
//!
//! This module provides combinators for composing predicates using
//! logical operators (and, or, not).

use core::marker::PhantomData;

use super::Praedicatum;

// =============================================================================
// Not Combinator
// =============================================================================

/// Negation of a predicate.
///
/// `Non<P>` is true when `P` is false.
///
/// # Latin Etymology
/// *Non* = not.
pub struct Non<P> {
    _predicate: PhantomData<P>,
}

impl<T, P: Praedicatum<T>> Praedicatum<T> for Non<P> {
    #[inline]
    fn check(value: &T) -> bool {
        !P::check(value)
    }

    #[inline]
    fn description() -> &'static str {
        "NOT condition"
    }

    #[inline]
    fn name() -> &'static str {
        "Non"
    }
}

/// Alias for Not combinator.
pub type Not<P> = Non<P>;

// =============================================================================
// And Combinator
// =============================================================================

/// Conjunction of two predicates.
///
/// `Et<P1, P2>` is true when both `P1` and `P2` are true.
///
/// # Latin Etymology
/// *Et* = and.
pub struct Et<P1, P2> {
    _predicates: PhantomData<(P1, P2)>,
}

impl<T, P1: Praedicatum<T>, P2: Praedicatum<T>> Praedicatum<T> for Et<P1, P2> {
    #[inline]
    fn check(value: &T) -> bool {
        P1::check(value) && P2::check(value)
    }

    #[inline]
    fn description() -> &'static str {
        "both conditions must be true"
    }

    #[inline]
    fn name() -> &'static str {
        "Et"
    }
}

/// Alias for And combinator.
pub type And<P1, P2> = Et<P1, P2>;

// =============================================================================
// Or Combinator
// =============================================================================

/// Disjunction of two predicates.
///
/// `Vel<P1, P2>` is true when either `P1` or `P2` (or both) are true.
///
/// # Latin Etymology
/// *Vel* = or (inclusive).
pub struct Vel<P1, P2> {
    _predicates: PhantomData<(P1, P2)>,
}

impl<T, P1: Praedicatum<T>, P2: Praedicatum<T>> Praedicatum<T> for Vel<P1, P2> {
    #[inline]
    fn check(value: &T) -> bool {
        P1::check(value) || P2::check(value)
    }

    #[inline]
    fn description() -> &'static str {
        "at least one condition must be true"
    }

    #[inline]
    fn name() -> &'static str {
        "Vel"
    }
}

/// Alias for Or combinator.
pub type Or<P1, P2> = Vel<P1, P2>;

// =============================================================================
// Xor Combinator
// =============================================================================

/// Exclusive disjunction of two predicates.
///
/// `Aut<P1, P2>` is true when exactly one of `P1` or `P2` is true.
///
/// # Latin Etymology
/// *Aut* = or (exclusive).
pub struct Aut<P1, P2> {
    _predicates: PhantomData<(P1, P2)>,
}

impl<T, P1: Praedicatum<T>, P2: Praedicatum<T>> Praedicatum<T> for Aut<P1, P2> {
    #[inline]
    fn check(value: &T) -> bool {
        P1::check(value) ^ P2::check(value)
    }

    #[inline]
    fn description() -> &'static str {
        "exactly one condition must be true"
    }

    #[inline]
    fn name() -> &'static str {
        "Aut"
    }
}

/// Alias for Xor combinator.
pub type Xor<P1, P2> = Aut<P1, P2>;

// =============================================================================
// Implication Combinator
// =============================================================================

/// Implication of two predicates.
///
/// `Implicatio<P1, P2>` is true when P1 implies P2 (if P1 then P2).
/// Equivalent to `!P1 || P2`.
///
/// # Latin Etymology
/// *Implicatio* = implication, involvement.
pub struct Implicatio<P1, P2> {
    _predicates: PhantomData<(P1, P2)>,
}

impl<T, P1: Praedicatum<T>, P2: Praedicatum<T>> Praedicatum<T> for Implicatio<P1, P2> {
    #[inline]
    fn check(value: &T) -> bool {
        !P1::check(value) || P2::check(value)
    }

    #[inline]
    fn description() -> &'static str {
        "if first condition then second condition"
    }

    #[inline]
    fn name() -> &'static str {
        "Implicatio"
    }
}

/// Alias for Implies combinator.
pub type Implies<P1, P2> = Implicatio<P1, P2>;

// =============================================================================
// Equivalence Combinator
// =============================================================================

/// Bi-conditional (equivalence) of two predicates.
///
/// `Aequivalentia<P1, P2>` is true when P1 and P2 have the same truth value.
///
/// # Latin Etymology
/// *Aequivalentia* = equivalence.
pub struct Aequivalentia<P1, P2> {
    _predicates: PhantomData<(P1, P2)>,
}

impl<T, P1: Praedicatum<T>, P2: Praedicatum<T>> Praedicatum<T> for Aequivalentia<P1, P2> {
    #[inline]
    fn check(value: &T) -> bool {
        P1::check(value) == P2::check(value)
    }

    #[inline]
    fn description() -> &'static str {
        "conditions must have same truth value"
    }

    #[inline]
    fn name() -> &'static str {
        "Aequivalentia"
    }
}

/// Alias for Iff (if and only if) combinator.
pub type Iff<P1, P2> = Aequivalentia<P1, P2>;

// =============================================================================
// All Combinator (Variadic And)
// =============================================================================

/// All predicates must be true (3-way and).
///
/// # Latin Etymology
/// *Omnes* = all.
pub struct Omnes<P1, P2, P3> {
    _predicates: PhantomData<(P1, P2, P3)>,
}

impl<T, P1, P2, P3> Praedicatum<T> for Omnes<P1, P2, P3>
where
    P1: Praedicatum<T>,
    P2: Praedicatum<T>,
    P3: Praedicatum<T>,
{
    #[inline]
    fn check(value: &T) -> bool {
        P1::check(value) && P2::check(value) && P3::check(value)
    }

    #[inline]
    fn description() -> &'static str {
        "all conditions must be true"
    }

    #[inline]
    fn name() -> &'static str {
        "Omnes"
    }
}

/// Alias for All combinator.
pub type All<P1, P2, P3> = Omnes<P1, P2, P3>;

// =============================================================================
// Any Combinator (Variadic Or)
// =============================================================================

/// At least one predicate must be true (3-way or).
///
/// # Latin Etymology
/// *Aliquis* = any, some.
pub struct Aliquis<P1, P2, P3> {
    _predicates: PhantomData<(P1, P2, P3)>,
}

impl<T, P1, P2, P3> Praedicatum<T> for Aliquis<P1, P2, P3>
where
    P1: Praedicatum<T>,
    P2: Praedicatum<T>,
    P3: Praedicatum<T>,
{
    #[inline]
    fn check(value: &T) -> bool {
        P1::check(value) || P2::check(value) || P3::check(value)
    }

    #[inline]
    fn description() -> &'static str {
        "at least one condition must be true"
    }

    #[inline]
    fn name() -> &'static str {
        "Aliquis"
    }
}

/// Alias for Any combinator.
pub type Any<P1, P2, P3> = Aliquis<P1, P2, P3>;

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::super::common::{Even, Negative, NonNegative, Positive};
    use super::*;

    #[test]
    fn test_not() {
        // Not Positive = non-positive (zero or negative)
        assert!(!Non::<Positive>::check(&42i32));
        assert!(Non::<Positive>::check(&0i32));
        assert!(Non::<Positive>::check(&-1i32));
    }

    #[test]
    fn test_and() {
        // Positive AND Even = positive even numbers
        assert!(Et::<Positive, Even>::check(&42i32));
        assert!(!Et::<Positive, Even>::check(&41i32)); // positive but odd
        assert!(!Et::<Positive, Even>::check(&-2i32)); // even but negative
        assert!(!Et::<Positive, Even>::check(&0i32)); // even but not positive
    }

    #[test]
    fn test_or() {
        // Positive OR Even
        assert!(Vel::<Positive, Even>::check(&42i32)); // both
        assert!(Vel::<Positive, Even>::check(&41i32)); // positive only
        assert!(Vel::<Positive, Even>::check(&-2i32)); // even only
        assert!(!Vel::<Positive, Even>::check(&-1i32)); // neither
    }

    #[test]
    fn test_xor() {
        // Positive XOR Even = exactly one
        assert!(!Aut::<Positive, Even>::check(&42i32)); // both - false
        assert!(Aut::<Positive, Even>::check(&41i32)); // positive only - true
        assert!(Aut::<Positive, Even>::check(&-2i32)); // even only - true
        assert!(!Aut::<Positive, Even>::check(&-1i32)); // neither - false
    }

    #[test]
    fn test_implies() {
        // Positive => Even (if positive then even)
        assert!(Implicatio::<Positive, Even>::check(&42i32)); // P=T, Q=T -> T
        assert!(!Implicatio::<Positive, Even>::check(&41i32)); // P=T, Q=F -> F
        assert!(Implicatio::<Positive, Even>::check(&-2i32)); // P=F, Q=T -> T
        assert!(Implicatio::<Positive, Even>::check(&-1i32)); // P=F, Q=F -> T
    }

    #[test]
    fn test_iff() {
        // Positive <=> Even (both same truth value)
        assert!(Aequivalentia::<Positive, Even>::check(&42i32)); // T, T -> T
        assert!(!Aequivalentia::<Positive, Even>::check(&41i32)); // T, F -> F
        assert!(!Aequivalentia::<Positive, Even>::check(&-2i32)); // F, T -> F
        assert!(Aequivalentia::<Positive, Even>::check(&-1i32)); // F, F -> T
    }

    #[test]
    fn test_all() {
        // Positive AND Even AND NonNegative
        assert!(All::<Positive, Even, NonNegative>::check(&42i32));
        assert!(!All::<Positive, Even, NonNegative>::check(&41i32)); // odd
        assert!(!All::<Positive, Even, NonNegative>::check(&0i32)); // not positive
    }

    #[test]
    fn test_any() {
        // Positive OR Even OR Negative
        assert!(Any::<Positive, Even, Negative>::check(&42i32)); // positive and even
        assert!(Any::<Positive, Even, Negative>::check(&41i32)); // positive
        assert!(Any::<Positive, Even, Negative>::check(&-2i32)); // even and negative
        assert!(Any::<Positive, Even, Negative>::check(&-1i32)); // negative
    }

    #[test]
    fn test_combined() {
        // (Positive AND Even) OR Negative
        type PositiveEven = And<Positive, Even>;
        type Combined = Or<PositiveEven, Negative>;

        assert!(Combined::check(&42i32)); // positive and even
        assert!(!Combined::check(&41i32)); // positive but odd
        assert!(Combined::check(&-1i32)); // negative
        assert!(!Combined::check(&0i32)); // neither
    }

    #[test]
    fn test_double_negation() {
        // Not(Not(P)) = P
        type DoubleNot = Not<Not<Positive>>;

        assert!(DoubleNot::check(&42i32));
        assert!(!DoubleNot::check(&-1i32));
    }

    #[test]
    fn test_de_morgan() {
        // Not(P AND Q) = Not(P) OR Not(Q)
        type Lhs = Not<And<Positive, Even>>;
        type Rhs = Or<Not<Positive>, Not<Even>>;

        // Test De Morgan's law
        for val in [-10, -1, 0, 1, 2, 10, 42, 43] {
            assert_eq!(
                Lhs::check(&val),
                Rhs::check(&val),
                "De Morgan failed for {val}"
            );
        }
    }
}
