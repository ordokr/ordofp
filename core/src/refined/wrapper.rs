//! Refined Type Wrapper
//!
//! > *"Puritas in typo est"*
//! > — Purity is in the type. (Latin)
//!
//! This module provides the core `Refined<T, P>` wrapper type that
//! holds a value guaranteed to satisfy a predicate.

use core::cmp::Ordering;
use core::hash::{Hash, Hasher};
use core::marker::PhantomData;
use core::ops::Deref;

use super::Praedicatum;

// =============================================================================
// Refined Wrapper
// =============================================================================

/// A refined type wrapper that guarantees a value satisfies a predicate.
///
/// `Refinatus<T, P>` is a newtype that wraps a value of type `T` and
/// provides compile-time evidence that the predicate `P` holds for that value.
///
/// # Latin Etymology
/// *Refinatus* = refined, purified.
///
/// # Example
///
/// ```rust
/// use ordofp_core::refined::{Refinatus, Positive};
///
/// let pos = Refinatus::<i32, Positive>::new(42).unwrap();
/// assert_eq!(*pos, 42);
///
/// // This fails at runtime
/// assert!(Refinatus::<i32, Positive>::new(-1).is_none());
/// ```
pub struct Refinatus<T, P: Praedicatum<T>> {
    value: T,
    _predicate: PhantomData<P>,
}

impl<T, P: Praedicatum<T>> Refinatus<T, P> {
    /// Create a new refined value by checking the predicate.
    ///
    /// Returns `Some` if the predicate holds, `None` otherwise.
    #[inline]
    pub fn new(value: T) -> Option<Self> {
        if P::check(&value) {
            Some(Refinatus {
                value,
                _predicate: PhantomData,
            })
        } else {
            None
        }
    }

    /// Create a refined value or return an error with description.
    ///
    /// # Errors
    ///
    /// Returns a [`RefinementError`] carrying the predicate's name and
    /// description when `P::check` rejects `value`; the rejected value is
    /// dropped.
    #[inline]
    pub fn try_new(value: T) -> Result<Self, RefinementError> {
        if P::check(&value) {
            Ok(Refinatus {
                value,
                _predicate: PhantomData,
            })
        } else {
            Err(RefinementError {
                predicate_name: P::name(),
                description: P::description(),
            })
        }
    }

    /// Create a refined value, panicking if the predicate fails.
    ///
    /// # Panics
    /// Panics if the predicate does not hold.
    #[inline]
    pub fn new_or_panic(value: T) -> Self {
        crate::unlikely_panic!(
            !P::check(&value),
            "Refinement predicate '{}' failed: {}",
            P::name(),
            P::description()
        );

        Refinatus {
            value,
            _predicate: PhantomData,
        }
    }

    /// Create a refined value without checking the predicate.
    ///
    /// # Safety
    /// The caller must ensure the predicate holds for the value.
    #[inline]
    pub unsafe fn new_unchecked(value: T) -> Self {
        Refinatus {
            value,
            _predicate: PhantomData,
        }
    }

    /// Get a reference to the inner value.
    #[inline]
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Consume and return the inner value.
    #[inline]
    pub fn into_inner(self) -> T {
        self.value
    }

    /// Map the inner value, maintaining the refinement if the predicate holds.
    #[inline]
    pub fn map<F, U, Q>(self, f: F) -> Option<Refinatus<U, Q>>
    where
        F: FnOnce(T) -> U,
        Q: Praedicatum<U>,
    {
        Refinatus::new(f(self.value))
    }

    /// Map the inner value, assuming the predicate still holds.
    ///
    /// # Safety
    /// The caller must ensure the predicate holds for the mapped value.
    #[inline]
    pub unsafe fn map_unchecked<F, U, Q>(self, f: F) -> Refinatus<U, Q>
    where
        F: FnOnce(T) -> U,
        Q: Praedicatum<U>,
    {
        // SAFETY: Caller guarantees predicate holds for mapped value
        unsafe { Refinatus::new_unchecked(f(self.value)) }
    }

    /// Apply a function that preserves the predicate.
    #[inline]
    pub fn modify<F>(self, f: F) -> Option<Self>
    where
        F: FnOnce(T) -> T,
    {
        Refinatus::new(f(self.value))
    }

    /// Replace the inner value, checking the predicate.
    #[inline]
    pub fn replace(self, value: T) -> Option<Self> {
        Refinatus::new(value)
    }
}

impl<T: Clone, P: Praedicatum<T>> Refinatus<T, P> {
    /// Clone and modify the value, returning a new refined value if valid.
    #[inline]
    pub fn with<F>(&self, f: F) -> Option<Self>
    where
        F: FnOnce(T) -> T,
    {
        Refinatus::new(f(self.value.clone()))
    }
}

impl<T, P: Praedicatum<T>> Deref for Refinatus<T, P> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T: Clone, P: Praedicatum<T>> Clone for Refinatus<T, P> {
    #[inline]
    fn clone(&self) -> Self {
        // Safe because we know the predicate holds for cloned value
        Refinatus {
            value: self.value.clone(),
            _predicate: PhantomData,
        }
    }
}

impl<T: Copy, P: Praedicatum<T>> Copy for Refinatus<T, P> {}

impl<T: PartialEq, P: Praedicatum<T>> PartialEq for Refinatus<T, P> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T: Eq, P: Praedicatum<T>> Eq for Refinatus<T, P> {}

impl<T: PartialOrd, P: Praedicatum<T>> PartialOrd for Refinatus<T, P> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl<T: Ord, P: Praedicatum<T>> Ord for Refinatus<T, P> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }
}

impl<T: Hash, P: Praedicatum<T>> Hash for Refinatus<T, P> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl<T: core::fmt::Debug, P: Praedicatum<T>> core::fmt::Debug for Refinatus<T, P> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Refinatus")
            .field("value", &self.value)
            .field("predicate", &P::name())
            .finish()
    }
}

impl<T: core::fmt::Display, P: Praedicatum<T>> core::fmt::Display for Refinatus<T, P> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.value.fmt(f)
    }
}

impl<T: Default, P: Praedicatum<T>> Refinatus<T, P> {
    /// Try to create a refined value from the default.
    #[inline]
    pub fn try_default() -> Option<Self> {
        Refinatus::new(T::default())
    }
}

/// Alias for Refined type.
pub type Refined<T, P> = Refinatus<T, P>;

// =============================================================================
// Refinement Error
// =============================================================================

/// Error returned when a refinement predicate fails.
#[derive(Debug, Clone)]
pub struct RefinementError {
    /// Name of the failed predicate.
    pub predicate_name: &'static str,
    /// Description of the requirement.
    pub description: &'static str,
}

impl core::fmt::Display for RefinementError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Refinement predicate '{}' failed: {}",
            self.predicate_name, self.description
        )
    }
}

// =============================================================================
// Refinement Builder
// =============================================================================

/// Builder for creating refined values with custom validation.
///
/// # Latin Etymology
/// *Aedificator refinati* = builder of refined.
pub struct AedificatorRefinati<T> {
    value: T,
}

impl<T> AedificatorRefinati<T> {
    /// Create a new builder with a value.
    #[inline]
    pub fn new(value: T) -> Self {
        AedificatorRefinati { value }
    }

    /// Refine the value with a predicate.
    #[inline]
    pub fn refine<P: Praedicatum<T>>(self) -> Option<Refinatus<T, P>> {
        Refinatus::new(self.value)
    }

    /// Try to refine the value.
    ///
    /// # Errors
    ///
    /// Returns a [`RefinementError`] carrying the predicate's name and
    /// description when `P::check` rejects the wrapped value.
    #[inline]
    pub fn try_refine<P: Praedicatum<T>>(self) -> Result<Refinatus<T, P>, RefinementError> {
        Refinatus::try_new(self.value)
    }

    /// Refine with a custom predicate function.
    #[inline]
    pub fn refine_with<F>(self, predicate: F) -> Option<T>
    where
        F: FnOnce(&T) -> bool,
    {
        if predicate(&self.value) {
            Some(self.value)
        } else {
            None
        }
    }
}

/// Create a refinement builder for a value.
#[inline]
pub fn refine<T>(value: T) -> AedificatorRefinati<T> {
    AedificatorRefinati::new(value)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    struct IsPositive;

    impl Praedicatum<i32> for IsPositive {
        fn check(value: &i32) -> bool {
            *value > 0
        }

        fn description() -> &'static str {
            "value must be positive"
        }
    }

    #[test]
    fn test_refinatus_new() {
        let pos = Refinatus::<i32, IsPositive>::new(42);
        assert!(pos.is_some());
        assert_eq!(*pos.expect("42 satisfies the IsPositive predicate"), 42);

        let neg = Refinatus::<i32, IsPositive>::new(-1);
        assert!(neg.is_none());
    }

    #[test]
    fn test_refinatus_try_new() {
        let pos = Refinatus::<i32, IsPositive>::try_new(42);
        assert!(pos.is_ok());

        let neg = Refinatus::<i32, IsPositive>::try_new(-1);
        assert!(neg.is_err());
    }

    #[test]
    #[should_panic(expected = "Refinement predicate")]
    fn test_refinatus_new_or_panic() {
        let _: Refinatus<i32, IsPositive> = Refinatus::new_or_panic(-1);
    }

    #[test]
    fn test_refinatus_value() {
        let pos = Refinatus::<i32, IsPositive>::new(42).expect("42 satisfies IsPositive predicate");
        assert_eq!(pos.value(), &42);
        assert_eq!(pos.into_inner(), 42);
    }

    #[test]
    fn test_refinatus_deref() {
        let pos = Refinatus::<i32, IsPositive>::new(42).expect("42 satisfies IsPositive predicate");
        let val: i32 = *pos;
        assert_eq!(val, 42);
    }

    #[test]
    fn test_refinatus_clone() {
        let pos = Refinatus::<i32, IsPositive>::new(42).expect("42 satisfies IsPositive predicate");
        let cloned = pos;
        assert_eq!(*cloned, 42);
    }

    #[test]
    fn test_refinatus_eq() {
        let a = Refinatus::<i32, IsPositive>::new(42).expect("42 satisfies IsPositive predicate");
        let b = Refinatus::<i32, IsPositive>::new(42).expect("42 satisfies IsPositive predicate");
        let c = Refinatus::<i32, IsPositive>::new(10).expect("10 satisfies IsPositive predicate");

        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn test_refinatus_ord() {
        let a = Refinatus::<i32, IsPositive>::new(10).expect("10 satisfies IsPositive predicate");
        let b = Refinatus::<i32, IsPositive>::new(42).expect("42 satisfies IsPositive predicate");

        assert!(a < b);
        assert!(b > a);
    }

    #[test]
    fn test_refinatus_modify() {
        let pos = Refinatus::<i32, IsPositive>::new(42).expect("42 satisfies IsPositive predicate");

        // Valid modification
        let doubled = pos.modify(|x| x * 2);
        assert!(doubled.is_some());
        assert_eq!(
            *doubled.expect("modify with *2 on 42 yields 84, which satisfies IsPositive"),
            84
        );
    }

    #[test]
    fn test_refinatus_modify_invalid() {
        let pos = Refinatus::<i32, IsPositive>::new(42).expect("42 satisfies IsPositive predicate");

        // Invalid modification (negation)
        let negated = pos.modify(|x| -x);
        assert!(negated.is_none());
    }

    #[test]
    fn test_refinatus_with() {
        let pos = Refinatus::<i32, IsPositive>::new(42).expect("42 satisfies IsPositive predicate");

        let doubled = pos.with(|x| x * 2);
        assert!(doubled.is_some());

        // Original unchanged
        assert_eq!(*pos, 42);
    }

    #[test]
    fn test_refine_builder() {
        let pos: Option<Refinatus<i32, IsPositive>> = refine(42).refine();
        assert!(pos.is_some());

        let neg: Option<Refinatus<i32, IsPositive>> = refine(-1).refine();
        assert!(neg.is_none());
    }

    #[test]
    fn test_refine_with() {
        let even = refine(42).refine_with(|x| x % 2 == 0);
        assert!(even.is_some());
        assert_eq!(
            even.expect("42 satisfies the even predicate (42 % 2 == 0)"),
            42
        );

        let odd = refine(42).refine_with(|x| x % 2 == 1);
        assert!(odd.is_none());
    }

    #[test]
    fn test_refinement_error_display() {
        let err = RefinementError {
            predicate_name: "IsPositive",
            description: "value must be positive",
        };
        let msg = alloc::format!("{err}");
        assert!(msg.contains("IsPositive"));
        assert!(msg.contains("positive"));
    }

    #[test]
    fn test_refinatus_debug() {
        let pos = Refinatus::<i32, IsPositive>::new(42).expect("42 satisfies IsPositive predicate");
        let debug = alloc::format!("{pos:?}");
        assert!(debug.contains("42"));
    }

    #[test]
    fn test_refinatus_display() {
        let pos = Refinatus::<i32, IsPositive>::new(42).expect("42 satisfies IsPositive predicate");
        let display = alloc::format!("{pos}");
        assert_eq!(display, "42");
    }

    #[test]
    fn test_refinatus_map_revalidates_predicate_on_result() {
        // `map` must re-check predicate Q on the mapped value, not blindly wrap it.
        // Valid path: mapping a positive value in a way that stays positive.
        let pos = Refinatus::<i32, IsPositive>::new(21).expect("21 satisfies IsPositive");
        let doubled: Option<Refinatus<i32, IsPositive>> = pos.map(|x| x * 2);
        assert!(
            doubled.is_some(),
            "map that keeps value positive must return Some"
        );
        assert_eq!(*doubled.expect("42 satisfies IsPositive"), 42);

        // Edge-case path: mapping a positive value to a non-positive one must yield None,
        // not silently produce a Refinatus whose invariant is violated.
        let pos2 = Refinatus::<i32, IsPositive>::new(21).expect("21 satisfies IsPositive");
        let negated: Option<Refinatus<i32, IsPositive>> = pos2.map(|x| -x);
        assert!(
            negated.is_none(),
            "map that violates predicate must return None, not an invalid Refinatus"
        );
    }
}
