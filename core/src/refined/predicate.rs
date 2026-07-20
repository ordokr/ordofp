//! Predicate Trait for Refinement Types
//!
//! > *"Praedicatum est veritas de subiecto"*
//! > — A predicate is the truth about a subject. (Latin)
//!
//! This module defines the core `Praedicatum` trait that all refinement
//! predicates must implement.

use core::marker::PhantomData;

// =============================================================================
// Predicate Trait
// =============================================================================

/// Core trait for refinement type predicates.
///
/// A predicate defines a boolean condition that a value must satisfy.
/// Predicates are type-level markers with runtime checking capability.
///
/// # Latin Etymology
/// *Praedicatum* = that which is asserted/predicated.
///
/// # Example
///
/// ```rust
/// use ordofp_core::refined::Praedicatum;
///
/// struct IsEven;
///
/// impl Praedicatum<i32> for IsEven {
///     fn check(value: &i32) -> bool {
///         value % 2 == 0
///     }
///
///     fn description() -> &'static str {
///         "value is even"
///     }
/// }
///
/// assert!(IsEven::check(&4));
/// assert!(!IsEven::check(&5));
/// assert_eq!(IsEven::description(), "value is even");
/// ```
pub trait Praedicatum<T>: Sized {
    /// Check if the value satisfies this predicate.
    fn check(value: &T) -> bool;

    /// Human-readable description of this predicate.
    fn description() -> &'static str;

    /// Error message when predicate fails.
    fn error_message() -> &'static str {
        Self::description()
    }

    /// Get the name of this predicate.
    fn name() -> &'static str {
        core::any::type_name::<Self>()
    }
}

/// Alias for the predicate trait.
pub trait Predicate<T>: Praedicatum<T> {}
impl<T, P: Praedicatum<T>> Predicate<T> for P {}

// =============================================================================
// Validatus Trait
// =============================================================================

/// Trait for types that can be validated against a predicate.
///
/// # Latin Etymology
/// *Validatus* = made strong, validated.
pub trait Validatus<P> {
    /// Check if this value is valid according to the predicate.
    fn is_valid(&self) -> bool;

    /// Validate and return an error message if invalid.
    ///
    /// # Errors
    ///
    /// Returns `Err` carrying the predicate's static error message when the
    /// predicate check fails for this value.
    fn validate(&self) -> Result<(), &'static str>;
}

impl<T, P: Praedicatum<T>> Validatus<P> for T {
    #[inline]
    fn is_valid(&self) -> bool {
        P::check(self)
    }

    #[inline]
    fn validate(&self) -> Result<(), &'static str> {
        if P::check(self) {
            Ok(())
        } else {
            Err(P::error_message())
        }
    }
}

// =============================================================================
// Predicate Evidence
// =============================================================================

/// Compile-time evidence that a value satisfies a predicate.
///
/// This is a zero-sized type that proves a predicate holds.
///
/// # Latin Etymology
/// *Evidentia praedicati* = evidence of the predicate.
pub struct EvidentiaPredicati<T, P: Praedicatum<T>> {
    _marker: PhantomData<(T, P)>,
}

impl<T, P: Praedicatum<T>> EvidentiaPredicati<T, P> {
    /// Create evidence by checking the predicate.
    ///
    /// Returns `Some` if the predicate holds, `None` otherwise.
    #[inline]
    pub fn verify(value: &T) -> Option<Self> {
        if P::check(value) {
            Some(EvidentiaPredicati {
                _marker: PhantomData,
            })
        } else {
            None
        }
    }

    /// Create evidence unsafely (caller must ensure predicate holds).
    ///
    /// # Safety
    /// The caller must ensure that the predicate `P` holds for the value.
    #[inline]
    pub unsafe fn assume() -> Self {
        EvidentiaPredicati {
            _marker: PhantomData,
        }
    }
}

impl<T, P: Praedicatum<T>> Clone for EvidentiaPredicati<T, P> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T, P: Praedicatum<T>> Copy for EvidentiaPredicati<T, P> {}

impl<T, P: Praedicatum<T>> core::fmt::Debug for EvidentiaPredicati<T, P> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "EvidentiaPredicati<{}>", P::name())
    }
}

// =============================================================================
// Predicate Result
// =============================================================================

/// Result of checking a predicate.
///
/// # Latin Etymology
/// *Exitus praedicati* = outcome of the predicate.
#[derive(Debug, Clone)]
pub enum ExitusPraedicati {
    /// Predicate passed.
    Verum,
    /// Predicate failed with description.
    Falsum {
        /// The predicate's static human-readable description of what was
        /// required, for error reporting.
        description: &'static str,
    },
}

impl ExitusPraedicati {
    /// Check if the predicate passed.
    #[inline]
    pub fn passed(&self) -> bool {
        matches!(self, ExitusPraedicati::Verum)
    }

    /// Check if the predicate failed.
    #[inline]
    pub fn failed(&self) -> bool {
        matches!(self, ExitusPraedicati::Falsum { .. })
    }

    /// Convert to Result.
    ///
    /// # Errors
    ///
    /// Returns `Err` carrying the failure description exactly when the
    /// outcome is [`ExitusPraedicati::Falsum`]; `Verum` becomes `Ok(())`.
    #[inline]
    pub fn to_result(self) -> Result<(), &'static str> {
        match self {
            ExitusPraedicati::Verum => Ok(()),
            ExitusPraedicati::Falsum { description } => Err(description),
        }
    }
}

/// Check a predicate and get detailed result.
#[inline]
pub fn check_predicate<T, P: Praedicatum<T>>(value: &T) -> ExitusPraedicati {
    if P::check(value) {
        ExitusPraedicati::Verum
    } else {
        ExitusPraedicati::Falsum {
            description: P::description(),
        }
    }
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
    fn test_praedicatum_check() {
        assert!(IsPositive::check(&42));
        assert!(!IsPositive::check(&0));
        assert!(!IsPositive::check(&-1));
    }

    #[test]
    fn test_praedicatum_description() {
        assert_eq!(IsPositive::description(), "value must be positive");
    }

    #[test]
    fn test_validatus() {
        assert!(<i32 as Validatus<IsPositive>>::is_valid(&42i32));
        assert!(!<i32 as Validatus<IsPositive>>::is_valid(&0i32));
        assert!(<i32 as Validatus<IsPositive>>::validate(&42i32).is_ok());
        assert!(<i32 as Validatus<IsPositive>>::validate(&0i32).is_err());
    }

    #[test]
    fn test_evidentia_verify() {
        let evidence = EvidentiaPredicati::<i32, IsPositive>::verify(&42);
        assert!(evidence.is_some());

        let no_evidence = EvidentiaPredicati::<i32, IsPositive>::verify(&-1);
        assert!(no_evidence.is_none());
    }

    #[test]
    fn test_exitus_praedicati() {
        let result = check_predicate::<i32, IsPositive>(&42);
        assert!(result.passed());
        assert!(!result.failed());
        assert!(result.to_result().is_ok());

        let result = check_predicate::<i32, IsPositive>(&-1);
        assert!(!result.passed());
        assert!(result.failed());
        assert!(result.to_result().is_err());
    }
}
