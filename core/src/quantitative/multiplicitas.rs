//! Multiplicitas - The core multiplicity type for QTT
//!
//! > *"Quot sunt modi utendi"*
//! > — How many are the modes of use. (Neo-Latin)
//!
//! This module defines the `Multiplicitas` enum representing usage multiplicities
//! in Quantitative Type Theory, inspired by Idris 2's QTT implementation.

use core::fmt;

// =============================================================================
// Multiplicitas - Runtime Multiplicity Value
// =============================================================================

/// Multiplicity values in Quantitative Type Theory.
///
/// > *"Multiplicitas est modus utendi"*
/// > — Multiplicity is the mode of use.
///
/// Multiplicities describe how many times a value can or must be used:
///
/// - `Nihil` (0): The value is erased at runtime (compile-time only)
/// - `Semel` (1): The value must be used exactly once (linear)
/// - `Omega` (ω): The value can be used any number of times (unrestricted)
///
/// # Semiring Structure
///
/// Multiplicities form a semiring with:
/// - Addition: sequential composition (max)
/// - Multiplication: context composition
///
/// # Example
///
/// ```rust
/// use ordofp_core::quantitative::{Multiplicitas, MultiplicitasSemiring};
///
/// let linear = Multiplicitas::Semel;
/// let unrestricted = Multiplicitas::Omega;
///
/// // Multiplication composes usage contexts: a value used `m` times within
/// // a context used `n` times totals `m * n`. A linear (1) context composed
/// // with an unrestricted (ω) value stays ω.
/// let composed = linear.mul(unrestricted);
/// assert_eq!(composed, Multiplicitas::Omega);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Multiplicitas {
    /// Zero multiplicity - value is erased at runtime.
    ///
    /// > *"Nihil in executione"*
    /// > — Nothing in execution.
    ///
    /// Values with zero multiplicity exist only for type checking
    /// and are completely erased during compilation.
    Nihil,

    /// One multiplicity - value must be used exactly once.
    ///
    /// > *"Semel et non amplius"*
    /// > — Once and no more.
    ///
    /// Linear values must be consumed exactly once. They cannot be
    /// dropped without use or duplicated.
    Semel,

    /// Omega multiplicity - value can be used any number of times.
    ///
    /// > *"Infinities utibile"*
    /// > — Infinitely usable.
    ///
    /// Unrestricted values can be used zero, one, or many times.
    /// This is the default in most programming languages.
    Omega,
}

impl Multiplicitas {
    /// Check if this is the zero (erased) multiplicity.
    #[inline]
    pub const fn is_nihil(&self) -> bool {
        matches!(self, Multiplicitas::Nihil)
    }

    /// Check if this is the one (linear) multiplicity.
    #[inline]
    pub const fn is_semel(&self) -> bool {
        matches!(self, Multiplicitas::Semel)
    }

    /// Check if this is the omega (unrestricted) multiplicity.
    #[inline]
    pub const fn is_omega(&self) -> bool {
        matches!(self, Multiplicitas::Omega)
    }

    /// Check if this multiplicity allows zero uses.
    ///
    /// Returns true for `Nihil` and `Omega`.
    #[inline]
    pub const fn allows_zero(&self) -> bool {
        matches!(self, Multiplicitas::Nihil | Multiplicitas::Omega)
    }

    /// Check if this multiplicity requires at least one use.
    ///
    /// Returns true for `Semel`.
    #[inline]
    pub const fn requires_use(&self) -> bool {
        matches!(self, Multiplicitas::Semel)
    }

    /// Check if this multiplicity allows multiple uses.
    ///
    /// Returns true for `Omega`.
    #[inline]
    pub const fn allows_many(&self) -> bool {
        matches!(self, Multiplicitas::Omega)
    }

    /// Check if `self` is a subusage of `other`.
    ///
    /// Subusage relation: 0 ≤ 1 ≤ ω
    #[inline]
    pub const fn is_subusage_of(&self, other: &Multiplicitas) -> bool {
        match (self, other) {
            (Multiplicitas::Nihil, _) => true,
            (Multiplicitas::Semel, Multiplicitas::Nihil) => false,
            (Multiplicitas::Semel, _) => true,
            (Multiplicitas::Omega, Multiplicitas::Omega) => true,
            (Multiplicitas::Omega, _) => false,
        }
    }

    /// Convert to a numeric representation for display.
    #[inline]
    pub const fn to_symbol(&self) -> &'static str {
        match self {
            Multiplicitas::Nihil => "0",
            Multiplicitas::Semel => "1",
            Multiplicitas::Omega => "ω",
        }
    }
}

impl Default for Multiplicitas {
    /// Default multiplicity is unrestricted (ω).
    #[inline]
    fn default() -> Self {
        Multiplicitas::Omega
    }
}

impl fmt::Display for Multiplicitas {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_symbol())
    }
}

// =============================================================================
// Multiplicity Semiring Operations
// =============================================================================

/// Semiring operations for multiplicities.
///
/// Multiplicities form a semiring with:
/// - Zero element: `Nihil` (for multiplication)
/// - One element: `Semel` (for multiplication)
/// - Addition: `max` operation
/// - Multiplication: context composition
pub trait MultiplicitasSemiring {
    /// Additive identity (zero for addition).
    fn zero() -> Self;

    /// Multiplicative identity (one for multiplication).
    fn one() -> Self;

    /// Semiring addition (sequential composition).
    fn add(self, other: Self) -> Self;

    /// Semiring multiplication (context composition).
    fn mul(self, other: Self) -> Self;
}

impl MultiplicitasSemiring for Multiplicitas {
    /// Zero for addition is `Nihil`.
    #[inline]
    fn zero() -> Self {
        Multiplicitas::Nihil
    }

    /// One for multiplication is `Semel`.
    #[inline]
    fn one() -> Self {
        Multiplicitas::Semel
    }

    /// Addition is the maximum operation.
    ///
    /// This captures that if a value is used in two branches,
    /// the overall usage is the maximum of both branches.
    #[inline]
    fn add(self, other: Self) -> Self {
        match (self, other) {
            (Multiplicitas::Omega, _) | (_, Multiplicitas::Omega) => Multiplicitas::Omega,
            (Multiplicitas::Semel, _) | (_, Multiplicitas::Semel) => Multiplicitas::Semel,
            (Multiplicitas::Nihil, Multiplicitas::Nihil) => Multiplicitas::Nihil,
        }
    }

    /// Multiplication composes contexts.
    ///
    /// If we use a value `m` times in a context that is used `n` times,
    /// the total usage is `m * n`.
    #[inline]
    fn mul(self, other: Self) -> Self {
        match (self, other) {
            (Multiplicitas::Nihil, _) | (_, Multiplicitas::Nihil) => Multiplicitas::Nihil,
            (Multiplicitas::Semel, x) | (x, Multiplicitas::Semel) => x,
            (Multiplicitas::Omega, Multiplicitas::Omega) => Multiplicitas::Omega,
        }
    }
}

/// Convenience function for multiplicity addition.
#[inline]
pub fn mult_add(a: Multiplicitas, b: Multiplicitas) -> Multiplicitas {
    a.add(b)
}

/// Convenience function for multiplicity multiplication.
#[inline]
pub fn mult_mul(a: Multiplicitas, b: Multiplicitas) -> Multiplicitas {
    a.mul(b)
}

/// Check if one multiplicity is a subusage of another.
#[inline]
pub fn is_subusage(a: Multiplicitas, b: Multiplicitas) -> bool {
    a.is_subusage_of(&b)
}

// =============================================================================
// Type-Level Multiplicity Markers
// =============================================================================

/// Type-level marker for zero/erased multiplicity (0).
///
/// > *"Nihil" - nothing*
///
/// Values at this multiplicity exist only at compile time
/// and are completely erased at runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Nihil;

/// Type-level marker for linear multiplicity (1).
///
/// > *"Semel" - once*
///
/// Values at this multiplicity must be used exactly once.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Semel;

/// Type-level marker for unrestricted multiplicity (ω).
///
/// > *"Omega" - unlimited*
///
/// Values at this multiplicity can be used any number of times.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Omega;

/// Trait for type-level multiplicities.
///
/// This trait is implemented by the phantom type markers `Nihil`, `Semel`,
/// and `Omega` to enable type-level multiplicity reasoning.
pub trait Usage: Copy + Default + 'static {
    /// The runtime multiplicity value.
    const VALUE: Multiplicitas;

    /// Check if this usage allows discarding without use.
    const ALLOWS_DISCARD: bool;

    /// Check if this usage allows duplication.
    const ALLOWS_DUP: bool;
}

impl Usage for Nihil {
    const VALUE: Multiplicitas = Multiplicitas::Nihil;
    const ALLOWS_DISCARD: bool = true;
    const ALLOWS_DUP: bool = true; // Can be "duplicated" since it's erased
}

impl Usage for Semel {
    const VALUE: Multiplicitas = Multiplicitas::Semel;
    const ALLOWS_DISCARD: bool = false;
    const ALLOWS_DUP: bool = false;
}

impl Usage for Omega {
    const VALUE: Multiplicitas = Multiplicitas::Omega;
    const ALLOWS_DISCARD: bool = true;
    const ALLOWS_DUP: bool = true;
}

// =============================================================================
// Display Implementations
// =============================================================================

impl fmt::Display for Nihil {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0")
    }
}

impl fmt::Display for Semel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "1")
    }
}

impl fmt::Display for Omega {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ω")
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::format;

    #[test]
    fn test_multiplicitas_predicates() {
        assert!(Multiplicitas::Nihil.is_nihil());
        assert!(!Multiplicitas::Nihil.is_semel());
        assert!(!Multiplicitas::Nihil.is_omega());

        assert!(!Multiplicitas::Semel.is_nihil());
        assert!(Multiplicitas::Semel.is_semel());
        assert!(!Multiplicitas::Semel.is_omega());

        assert!(!Multiplicitas::Omega.is_nihil());
        assert!(!Multiplicitas::Omega.is_semel());
        assert!(Multiplicitas::Omega.is_omega());
    }

    #[test]
    fn test_allows_zero() {
        assert!(Multiplicitas::Nihil.allows_zero());
        assert!(!Multiplicitas::Semel.allows_zero());
        assert!(Multiplicitas::Omega.allows_zero());
    }

    #[test]
    fn test_requires_use() {
        assert!(!Multiplicitas::Nihil.requires_use());
        assert!(Multiplicitas::Semel.requires_use());
        assert!(!Multiplicitas::Omega.requires_use());
    }

    #[test]
    fn test_allows_many() {
        assert!(!Multiplicitas::Nihil.allows_many());
        assert!(!Multiplicitas::Semel.allows_many());
        assert!(Multiplicitas::Omega.allows_many());
    }

    #[test]
    fn test_subusage() {
        // 0 ≤ everything
        assert!(Multiplicitas::Nihil.is_subusage_of(&Multiplicitas::Nihil));
        assert!(Multiplicitas::Nihil.is_subusage_of(&Multiplicitas::Semel));
        assert!(Multiplicitas::Nihil.is_subusage_of(&Multiplicitas::Omega));

        // 1 ≤ 1, ω
        assert!(!Multiplicitas::Semel.is_subusage_of(&Multiplicitas::Nihil));
        assert!(Multiplicitas::Semel.is_subusage_of(&Multiplicitas::Semel));
        assert!(Multiplicitas::Semel.is_subusage_of(&Multiplicitas::Omega));

        // ω ≤ ω only
        assert!(!Multiplicitas::Omega.is_subusage_of(&Multiplicitas::Nihil));
        assert!(!Multiplicitas::Omega.is_subusage_of(&Multiplicitas::Semel));
        assert!(Multiplicitas::Omega.is_subusage_of(&Multiplicitas::Omega));
    }

    #[test]
    fn test_semiring_addition() {
        // max semantics
        assert_eq!(
            mult_add(Multiplicitas::Nihil, Multiplicitas::Nihil),
            Multiplicitas::Nihil
        );
        assert_eq!(
            mult_add(Multiplicitas::Nihil, Multiplicitas::Semel),
            Multiplicitas::Semel
        );
        assert_eq!(
            mult_add(Multiplicitas::Nihil, Multiplicitas::Omega),
            Multiplicitas::Omega
        );
        assert_eq!(
            mult_add(Multiplicitas::Semel, Multiplicitas::Semel),
            Multiplicitas::Semel
        );
        assert_eq!(
            mult_add(Multiplicitas::Semel, Multiplicitas::Omega),
            Multiplicitas::Omega
        );
        assert_eq!(
            mult_add(Multiplicitas::Omega, Multiplicitas::Omega),
            Multiplicitas::Omega
        );
    }

    #[test]
    fn test_semiring_multiplication() {
        // 0 * x = 0
        assert_eq!(
            mult_mul(Multiplicitas::Nihil, Multiplicitas::Nihil),
            Multiplicitas::Nihil
        );
        assert_eq!(
            mult_mul(Multiplicitas::Nihil, Multiplicitas::Semel),
            Multiplicitas::Nihil
        );
        assert_eq!(
            mult_mul(Multiplicitas::Nihil, Multiplicitas::Omega),
            Multiplicitas::Nihil
        );

        // 1 * x = x
        assert_eq!(
            mult_mul(Multiplicitas::Semel, Multiplicitas::Nihil),
            Multiplicitas::Nihil
        );
        assert_eq!(
            mult_mul(Multiplicitas::Semel, Multiplicitas::Semel),
            Multiplicitas::Semel
        );
        assert_eq!(
            mult_mul(Multiplicitas::Semel, Multiplicitas::Omega),
            Multiplicitas::Omega
        );

        // ω * x
        assert_eq!(
            mult_mul(Multiplicitas::Omega, Multiplicitas::Nihil),
            Multiplicitas::Nihil
        );
        assert_eq!(
            mult_mul(Multiplicitas::Omega, Multiplicitas::Semel),
            Multiplicitas::Omega
        );
        assert_eq!(
            mult_mul(Multiplicitas::Omega, Multiplicitas::Omega),
            Multiplicitas::Omega
        );
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", Multiplicitas::Nihil), "0");
        assert_eq!(format!("{}", Multiplicitas::Semel), "1");
        assert_eq!(format!("{}", Multiplicitas::Omega), "ω");
    }

    #[test]
    fn test_type_level_usage() {
        assert_eq!(Nihil::VALUE, Multiplicitas::Nihil);
        assert_eq!(Semel::VALUE, Multiplicitas::Semel);
        assert_eq!(Omega::VALUE, Multiplicitas::Omega);

        assert!(Nihil::ALLOWS_DISCARD);
        assert!(!Semel::ALLOWS_DISCARD);
        assert!(Omega::ALLOWS_DISCARD);

        assert!(Nihil::ALLOWS_DUP);
        assert!(!Semel::ALLOWS_DUP);
        assert!(Omega::ALLOWS_DUP);
    }

    #[test]
    fn test_default() {
        assert_eq!(Multiplicitas::default(), Multiplicitas::Omega);
    }
}
