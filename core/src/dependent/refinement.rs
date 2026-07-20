//! Refinement Types - Praecisio
//!
//! > *"Praecisio est virtus perfectae definitionis."*
//! > — Precision is the virtue of perfect definition.
//!
//! This module provides refinement types that add compile-time constraints
//! to base types, inspired by Liquid Haskell and F* refinement types.
//!
//! # Design
//!
//! Refinement types use Rust's type system to encode predicates about values.
//! A `Refined<T, P>` wraps a value of type `T` and witnesses that it satisfies
//! predicate `P`.
//!
//! # Example
//!
//! ```rust
//! use ordofp_core::dependent::refinement::{Refined, Positivus, refine_positive};
//!
//! // Create a positive integer (checked at runtime, tracked in types)
//! let pos: Option<Refined<i32, Positivus>> = refine_positive(42);
//! assert!(pos.is_some());
//!
//! // Negative values don't satisfy the predicate
//! let neg: Option<Refined<i32, Positivus>> = refine_positive(-1);
//! assert!(neg.is_none());
//! ```

use core::marker::PhantomData;
use core::ops::{Add, Mul};

// =============================================================================
// Core Refined Type
// =============================================================================

/// A value with a refinement predicate.
///
/// # Latin Etymology
/// *Praecisio* means "a cutting off, precision" - a precise, refined type.
///
/// `Refined<T, P>` wraps a value of type `T` with a witness that it
/// satisfies predicate `P`. The predicate is checked at construction time.
///
/// # Type Parameters
///
/// - `T`: The base type
/// - `P`: The predicate marker type
///
/// # Example
///
/// ```rust
/// use ordofp_core::dependent::refinement::{Refined, NonNegativum};
///
/// fn process_non_negative(n: Refined<i32, NonNegativum>) -> i32 {
///     // We know n.value() >= 0 by the type
///     n.value() * 2
/// }
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Refined<T, P> {
    value: T,
    _predicate: PhantomData<P>,
}

impl<T: core::fmt::Debug, P> core::fmt::Debug for Refined<T, P> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Refined")
            .field("value", &self.value)
            .finish()
    }
}

impl<T, P> Refined<T, P> {
    /// Create a refined value without checking the predicate.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `value` satisfies predicate `P`.
    ///
    /// # Latin Etymology
    /// *Sine examine* means "without examination".
    #[inline]
    pub unsafe fn sine_examine(value: T) -> Self {
        Refined {
            value,
            _predicate: PhantomData,
        }
    }

    /// Get the underlying value.
    #[inline]
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Consume and return the underlying value.
    #[inline]
    pub fn into_inner(self) -> T {
        self.value
    }

    /// Map over the value while preserving the predicate.
    ///
    /// # Safety
    ///
    /// The mapping function must preserve the predicate.
    #[inline]
    pub unsafe fn map_unchecked<U, F>(self, f: F) -> Refined<U, P>
    where
        F: FnOnce(T) -> U,
    {
        Refined {
            value: f(self.value),
            _predicate: PhantomData,
        }
    }
}

// =============================================================================
// Predicate Traits
// =============================================================================

/// Trait for refinement predicates.
///
/// # Latin Etymology
/// *Praedicatum* means "that which is affirmed".
pub trait Praedicatum<T> {
    /// Check if a value satisfies the predicate.
    fn check(value: &T) -> bool;
}

// =============================================================================
// Common Predicates for Numbers
// =============================================================================

/// Predicate: value > 0
///
/// # Latin Etymology
/// *Positivus* means "positive".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Positivus;

/// Predicate: value >= 0
///
/// # Latin Etymology
/// *Non Negativum* means "not negative".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct NonNegativum;

/// Predicate: value < 0
///
/// # Latin Etymology
/// *Negativus* means "negative".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Negativus;

/// Predicate: value != 0
///
/// # Latin Etymology
/// *Non Nihil* means "not nothing" (non-zero).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct NonNihilum;

/// Predicate: value is even
///
/// # Latin Etymology
/// *Par* means "equal, even".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Par;

/// Predicate: value is odd
///
/// # Latin Etymology
/// *Impar* means "unequal, odd".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Impar;

// Implementations for i32
impl Praedicatum<i32> for Positivus {
    #[inline]
    fn check(value: &i32) -> bool {
        *value > 0
    }
}

impl Praedicatum<i32> for NonNegativum {
    #[inline]
    fn check(value: &i32) -> bool {
        *value >= 0
    }
}

impl Praedicatum<i32> for Negativus {
    #[inline]
    fn check(value: &i32) -> bool {
        *value < 0
    }
}

impl Praedicatum<i32> for NonNihilum {
    #[inline]
    fn check(value: &i32) -> bool {
        *value != 0
    }
}

impl Praedicatum<i32> for Par {
    #[inline]
    fn check(value: &i32) -> bool {
        *value % 2 == 0
    }
}

impl Praedicatum<i32> for Impar {
    #[inline]
    fn check(value: &i32) -> bool {
        *value % 2 != 0
    }
}

// Implementations for i64
impl Praedicatum<i64> for Positivus {
    #[inline]
    fn check(value: &i64) -> bool {
        *value > 0
    }
}

impl Praedicatum<i64> for NonNegativum {
    #[inline]
    fn check(value: &i64) -> bool {
        *value >= 0
    }
}

impl Praedicatum<i64> for Negativus {
    #[inline]
    fn check(value: &i64) -> bool {
        *value < 0
    }
}

impl Praedicatum<i64> for NonNihilum {
    #[inline]
    fn check(value: &i64) -> bool {
        *value != 0
    }
}

// Implementations for usize
impl Praedicatum<usize> for Positivus {
    #[inline]
    fn check(value: &usize) -> bool {
        *value > 0
    }
}

impl Praedicatum<usize> for NonNihilum {
    #[inline]
    fn check(value: &usize) -> bool {
        *value != 0
    }
}

impl Praedicatum<usize> for Par {
    #[inline]
    fn check(value: &usize) -> bool {
        (*value).is_multiple_of(2)
    }
}

impl Praedicatum<usize> for Impar {
    #[inline]
    fn check(value: &usize) -> bool {
        !(*value).is_multiple_of(2)
    }
}

// =============================================================================
// Bounded Predicates
// =============================================================================

/// Predicate: value is within a range [Min, Max].
///
/// # Latin Etymology
/// *Intra fines* means "within boundaries".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IntraFines<const MIN: i64, const MAX: i64>;

impl<const MIN: i64, const MAX: i64> Praedicatum<i32> for IntraFines<MIN, MAX> {
    #[inline]
    fn check(value: &i32) -> bool {
        let v = i64::from(*value);
        v >= MIN && v <= MAX
    }
}

impl<const MIN: i64, const MAX: i64> Praedicatum<i64> for IntraFines<MIN, MAX> {
    #[inline]
    fn check(value: &i64) -> bool {
        *value >= MIN && *value <= MAX
    }
}

/// Predicate: value < MAX
///
/// # Latin Etymology
/// *Minor quam* means "less than".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MinorQuam<const MAX: i64>;

impl<const MAX: i64> Praedicatum<i32> for MinorQuam<MAX> {
    #[inline]
    fn check(value: &i32) -> bool {
        i64::from(*value) < MAX
    }
}

/// Predicate: value > MIN
///
/// # Latin Etymology
/// *Maior quam* means "greater than".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MaiorQuam<const MIN: i64>;

impl<const MIN: i64> Praedicatum<i32> for MaiorQuam<MIN> {
    #[inline]
    fn check(value: &i32) -> bool {
        i64::from(*value) > MIN
    }
}

// =============================================================================
// Refinement Constructors
// =============================================================================

/// Try to refine a value with predicate P.
///
/// Returns `Some(refined)` if the predicate holds, `None` otherwise.
///
/// # Latin Etymology
/// *Praecidere* means "to cut off, refine".
#[inline]
pub fn praecidere<T, P>(value: T) -> Option<Refined<T, P>>
where
    P: Praedicatum<T>,
{
    if P::check(&value) {
        Some(Refined {
            value,
            _predicate: PhantomData,
        })
    } else {
        None
    }
}

/// Refine a value, panicking if the predicate doesn't hold.
///
/// # Panics
///
/// Panics if `P::check(&value)` returns false.
pub fn praecidere_vel_panico<T, P>(value: T) -> Refined<T, P>
where
    P: Praedicatum<T>,
    T: core::fmt::Debug,
{
    praecidere(value).expect("Refinement predicate not satisfied")
}

/// Try to refine as positive.
#[inline]
pub fn refine_positive<T>(value: T) -> Option<Refined<T, Positivus>>
where
    Positivus: Praedicatum<T>,
{
    praecidere(value)
}

/// Try to refine as non-negative.
#[inline]
pub fn refine_non_negative<T>(value: T) -> Option<Refined<T, NonNegativum>>
where
    NonNegativum: Praedicatum<T>,
{
    praecidere(value)
}

/// Try to refine as non-zero.
#[inline]
pub fn refine_non_zero<T>(value: T) -> Option<Refined<T, NonNihilum>>
where
    NonNihilum: Praedicatum<T>,
{
    praecidere(value)
}

/// Try to refine as even.
#[inline]
pub fn refine_even<T>(value: T) -> Option<Refined<T, Par>>
where
    Par: Praedicatum<T>,
{
    praecidere(value)
}

/// Try to refine as odd.
#[inline]
pub fn refine_odd<T>(value: T) -> Option<Refined<T, Impar>>
where
    Impar: Praedicatum<T>,
{
    praecidere(value)
}

// =============================================================================
// Operations that Preserve Refinements
// =============================================================================

// Addition of two positive numbers is positive
impl<T: Add<Output = T> + Clone> Add for Refined<T, Positivus>
where
    Positivus: Praedicatum<T>,
{
    type Output = Refined<T, Positivus>;

    /// # Panics
    ///
    /// Panics if the sum no longer satisfies `Positivus` — e.g. on signed
    /// integer wrap-around in release mode (overflow producing a negative).
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        let result = self.value + rhs.value;
        assert!(
            Positivus::check(&result),
            "Overflow in Refined addition: result does not satisfy predicate"
        );
        // SAFETY: We explicitly checked the predicate above.
        unsafe { Refined::sine_examine(result) }
    }
}

/// Overflow-aware multiplication for the numeric types Refined supports.
/// Private: exists so `Refined<T, Positivus>` multiplication can reject
/// integer wrap-around instead of trusting the wrapped sign.
trait MultiplicatioTuta: Sized {
    /// None = arithmetic overflow (integers). Floats never wrap; they
    /// saturate to ±inf, which the predicate re-check handles, so they
    /// return Some unconditionally.
    fn mul_tuta(self, rhs: Self) -> Option<Self>;
}

macro_rules! impl_mul_tuta_checked {
    ($($t:ty),* $(,)?) => {
        $(
            impl MultiplicatioTuta for $t {
                #[inline]
                fn mul_tuta(self, rhs: Self) -> Option<Self> {
                    self.checked_mul(rhs)
                }
            }
        )*
    };
}

macro_rules! impl_mul_tuta_float {
    ($($t:ty),* $(,)?) => {
        $(
            impl MultiplicatioTuta for $t {
                #[inline]
                fn mul_tuta(self, rhs: Self) -> Option<Self> {
                    Some(self * rhs)
                }
            }
        )*
    };
}

impl_mul_tuta_checked!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize
);
impl_mul_tuta_float!(f32, f64);

// Multiplication of two positive numbers, using checked arithmetic to
// prevent silent overflow wrap-around from producing a false-positive
// `Positivus`.
//
// Bounded on the private `MultiplicatioTuta` trait rather than plain `Mul`:
// a custom numeric type that only implements `Mul` (and relies on silent
// wrap-around) no longer gets a `Mul` impl here. That's a deliberate
// tradeoff, not an oversight — silent wrap was the audited bug (1ab5143
// narrowed this impl to i32-only instead of fixing it, which broke
// `PositiveI64` and `PositiveUsize` multiplication in the process).
impl<T: MultiplicatioTuta + Clone> Mul for Refined<T, Positivus>
where
    Positivus: Praedicatum<T>,
{
    type Output = Refined<T, Positivus>;

    /// # Panics
    ///
    /// Panics if the product overflows the underlying integer type (floats
    /// never overflow via `mul_tuta`, see `MultiplicatioTuta`), or if the
    /// product no longer satisfies `Positivus` (e.g. multiplying by zero).
    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        let result = self
            .value
            .mul_tuta(rhs.value)
            .expect("Refined multiplication overflowed");
        assert!(
            Positivus::check(&result),
            "Refined multiplication: result does not satisfy predicate"
        );
        // SAFETY: We explicitly checked the predicate above.
        unsafe { Refined::sine_examine(result) }
    }
}

// Division of positive by positive is positive (for integers, may truncate to 0)
// This is actually not always safe for integers, so we don't provide it

// Non-zero divided by non-zero is non-zero (for floats only, not integers)

// =============================================================================
// Type Aliases for Common Refinements
// =============================================================================

/// A positive integer.
pub type PositiveI32 = Refined<i32, Positivus>;
/// A positive i64.
pub type PositiveI64 = Refined<i64, Positivus>;
/// A non-negative integer.
pub type NonNegativeI32 = Refined<i32, NonNegativum>;
/// A non-zero integer.
pub type NonZeroI32 = Refined<i32, NonNihilum>;
/// A positive usize.
pub type PositiveUsize = Refined<usize, Positivus>;
/// A non-zero usize.
pub type NonZeroUsize = Refined<usize, NonNihilum>;

/// A percentage (0-100).
pub type Percentage = Refined<i32, IntraFines<0, 100>>;
/// A probability (0-100 as integer percentage).
pub type ProbabilityPercent = Refined<i32, IntraFines<0, 100>>;

// =============================================================================
// And/Or Predicates
// =============================================================================

/// Conjunction of two predicates.
///
/// # Latin Etymology
/// *Et* means "and".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Et<P, Q>(PhantomData<(P, Q)>);

impl<T, P: Praedicatum<T>, Q: Praedicatum<T>> Praedicatum<T> for Et<P, Q> {
    #[inline]
    fn check(value: &T) -> bool {
        P::check(value) && Q::check(value)
    }
}

/// Disjunction of two predicates.
///
/// # Latin Etymology
/// *Vel* means "or".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Vel<P, Q>(PhantomData<(P, Q)>);

impl<T, P: Praedicatum<T>, Q: Praedicatum<T>> Praedicatum<T> for Vel<P, Q> {
    #[inline]
    fn check(value: &T) -> bool {
        P::check(value) || Q::check(value)
    }
}

/// Negation of a predicate.
///
/// # Latin Etymology
/// *Non* means "not".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Non<P>(PhantomData<P>);

impl<T, P: Praedicatum<T>> Praedicatum<T> for Non<P> {
    #[inline]
    fn check(value: &T) -> bool {
        !P::check(value)
    }
}

// =============================================================================
// String Predicates
// =============================================================================

/// Predicate: string is non-empty.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct NonVacuus;

#[cfg(feature = "alloc")]
impl Praedicatum<alloc::string::String> for NonVacuus {
    #[inline]
    fn check(value: &alloc::string::String) -> bool {
        !value.is_empty()
    }
}

impl Praedicatum<&str> for NonVacuus {
    #[inline]
    fn check(value: &&str) -> bool {
        !value.is_empty()
    }
}

/// Predicate: string length is at most N.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MaxLength<const N: usize>;

#[cfg(feature = "alloc")]
impl<const N: usize> Praedicatum<alloc::string::String> for MaxLength<N> {
    #[inline]
    fn check(value: &alloc::string::String) -> bool {
        value.len() <= N
    }
}

impl<const N: usize> Praedicatum<&str> for MaxLength<N> {
    #[inline]
    fn check(value: &&str) -> bool {
        value.len() <= N
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_positive_refine() {
        let pos = refine_positive(42i32);
        assert!(pos.is_some());
        assert_eq!(
            *pos.expect("refine_positive(42) should succeed for a positive value")
                .value(),
            42
        );

        let zero = refine_positive(0i32);
        assert!(zero.is_none());

        let neg = refine_positive(-1i32);
        assert!(neg.is_none());
    }

    #[test]
    fn test_non_negative_refine() {
        let pos = refine_non_negative(42i32);
        assert!(pos.is_some());

        let zero = refine_non_negative(0i32);
        assert!(zero.is_some());

        let neg = refine_non_negative(-1i32);
        assert!(neg.is_none());
    }

    #[test]
    fn test_non_zero_refine() {
        let pos = refine_non_zero(42i32);
        assert!(pos.is_some());

        let zero = refine_non_zero(0i32);
        assert!(zero.is_none());

        let neg = refine_non_zero(-1i32);
        assert!(neg.is_some());
    }

    #[test]
    fn test_even_odd_refine() {
        assert!(refine_even(42i32).is_some());
        assert!(refine_even(43i32).is_none());

        assert!(refine_odd(43i32).is_some());
        assert!(refine_odd(42i32).is_none());
    }

    #[test]
    fn test_bounded_refine() {
        // Percentage: 0-100
        let valid: Option<Percentage> = praecidere(50);
        assert!(valid.is_some());

        let too_low: Option<Percentage> = praecidere(-1);
        assert!(too_low.is_none());

        let too_high: Option<Percentage> = praecidere(101);
        assert!(too_high.is_none());

        let edge_low: Option<Percentage> = praecidere(0);
        assert!(edge_low.is_some());

        let edge_high: Option<Percentage> = praecidere(100);
        assert!(edge_high.is_some());
    }

    #[test]
    fn test_positive_add() {
        let a: PositiveI32 = refine_positive(10i32).expect("10 satisfies the Positivus predicate");
        let b: PositiveI32 = refine_positive(20i32).expect("20 satisfies the Positivus predicate");
        let sum: PositiveI32 = a + b;
        assert_eq!(*sum.value(), 30);
    }

    #[test]
    fn test_positive_mul() {
        let a: PositiveI32 = refine_positive(5i32).expect("5 satisfies the Positivus predicate");
        let b: PositiveI32 = refine_positive(6i32).expect("6 satisfies the Positivus predicate");
        let prod: PositiveI32 = a * b;
        assert_eq!(*prod.value(), 30);
    }

    #[test]
    #[should_panic(expected = "Refined multiplication overflowed")]
    fn positive_mul_overflow_is_rejected_not_wrapped() {
        let a: PositiveI32 =
            refine_positive(65_536i32).expect("65_536 satisfies the Positivus predicate");
        let b: PositiveI32 =
            refine_positive(65_537i32).expect("65_537 satisfies the Positivus predicate");
        // Pre-fix: wraps to +65_536 and passes. Post-fix: uses checked_mul
        // and panics when the result would overflow.
        let _ = a * b;
    }

    #[test]
    fn positive_i64_mul_smoke() {
        let a: PositiveI64 = refine_positive(5i64).expect("5 satisfies the Positivus predicate");
        let b: PositiveI64 = refine_positive(6i64).expect("6 satisfies the Positivus predicate");
        let prod: PositiveI64 = a * b;
        assert_eq!(*prod.value(), 30);
    }

    #[test]
    fn positive_usize_mul_smoke() {
        let a: PositiveUsize =
            refine_positive(5usize).expect("5 satisfies the Positivus predicate");
        let b: PositiveUsize =
            refine_positive(6usize).expect("6 satisfies the Positivus predicate");
        let prod: PositiveUsize = a * b;
        assert_eq!(*prod.value(), 30);
    }

    #[test]
    #[should_panic(expected = "Refined multiplication overflowed")]
    fn positive_i64_mul_overflow_is_rejected_not_wrapped() {
        let a: PositiveI64 =
            refine_positive(i64::MAX).expect("i64::MAX satisfies the Positivus predicate");
        let b: PositiveI64 = refine_positive(2i64).expect("2 satisfies the Positivus predicate");
        let _ = a * b;
    }

    #[test]
    fn test_into_inner() {
        let refined: PositiveI32 =
            refine_positive(42i32).expect("42 satisfies the Positivus predicate");
        let value: i32 = refined.into_inner();
        assert_eq!(value, 42);
    }

    #[test]
    fn test_conjunction_predicate() {
        // Positive and even
        type PositiveEven = Et<Positivus, Par>;

        let valid: Option<Refined<i32, PositiveEven>> = praecidere(42);
        assert!(valid.is_some());

        let positive_odd: Option<Refined<i32, PositiveEven>> = praecidere(43);
        assert!(positive_odd.is_none());

        let negative_even: Option<Refined<i32, PositiveEven>> = praecidere(-2);
        assert!(negative_even.is_none());
    }

    #[test]
    fn test_disjunction_predicate() {
        type PositiveOrEven = Vel<Positivus, Par>;

        let positive_odd: Option<Refined<i32, PositiveOrEven>> = praecidere(3);
        assert!(positive_odd.is_some());

        let negative_even: Option<Refined<i32, PositiveOrEven>> = praecidere(-2);
        assert!(negative_even.is_some());

        let negative_odd: Option<Refined<i32, PositiveOrEven>> = praecidere(-3);
        assert!(negative_odd.is_none());
    }

    #[test]
    fn test_negation_predicate() {
        // Not positive (i.e., zero or negative)
        type NotPositive = Non<Positivus>;

        let zero: Option<Refined<i32, NotPositive>> = praecidere(0);
        assert!(zero.is_some());

        let negative: Option<Refined<i32, NotPositive>> = praecidere(-5);
        assert!(negative.is_some());

        let positive: Option<Refined<i32, NotPositive>> = praecidere(5);
        assert!(positive.is_none());
    }

    #[test]
    fn test_non_vacuus_str() {
        let valid: Option<Refined<&str, NonVacuus>> = praecidere("hello");
        assert!(valid.is_some());

        let empty: Option<Refined<&str, NonVacuus>> = praecidere("");
        assert!(empty.is_none());
    }

    #[test]
    fn test_max_length() {
        let short: Option<Refined<&str, MaxLength<10>>> = praecidere("hello");
        assert!(short.is_some());

        let exact: Option<Refined<&str, MaxLength<5>>> = praecidere("hello");
        assert!(exact.is_some());

        let too_long: Option<Refined<&str, MaxLength<3>>> = praecidere("hello");
        assert!(too_long.is_none());
    }

    #[test]
    fn test_usize_predicates() {
        let pos: Option<PositiveUsize> = refine_positive(42usize);
        assert!(pos.is_some());

        let zero: Option<PositiveUsize> = refine_positive(0usize);
        assert!(zero.is_none());

        let non_zero: Option<NonZeroUsize> = refine_non_zero(42usize);
        assert!(non_zero.is_some());
    }

    #[test]
    #[should_panic(expected = "Refinement predicate not satisfied")]
    fn test_praecidere_vel_panico() {
        let _: Refined<i32, Positivus> = praecidere_vel_panico(-1);
    }
}
