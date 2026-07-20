//! Effect Purity Analysis
//!
//! This module provides traits and types for analyzing effect properties
//! at the type level. These properties enable **typed optimization combinators**
//! that users explicitly invoke.
//!
//! # Effect Properties
//!
//! | Property    | Definition                           | Enables (via explicit combinator) |
//! |-------------|--------------------------------------|-----------------------------------|
//! | Pure        | No observable effects                | `par_map`, `par_traverse`         |
//! | Idempotent  | Same result on repeated execution    | `memoize`, `Lazy`                 |
//! | Total       | Always terminates with a value       | `speculative`, `race`             |
//! | Monotonic   | Effects can only add, never remove   | Incremental computation           |
//!
//! # Example
//!
//! ```rust
//! use ordofp_core::nexus::Eff;
//! use ordofp_core::nexus::pure;
//! use ordofp_core::nexus::Pure;
//! use rayon::prelude::*;
//!
//! // Pure effects can be parallelized
//! fn parallel_map<A, B, F>(items: Vec<A>, f: F) -> Vec<B>
//! where
//!     F: Fn(A) -> Eff<Pure, B> + Sync + Send,
//!     A: Send,
//!     B: Send,
//! {
//!     // Safe because Pure guarantees no side effects
//!     items.into_par_iter().map(|a| f(a).run_pure()).collect()
//! }
//!
//! let result = parallel_map(vec![1, 2, 3], |x| pure(x * 2));
//! assert_eq!(result, vec![2, 4, 6]);
//! ```

use core::marker::PhantomData;

use crate::nexus::effect::{Reader, State};
use crate::nexus::effects::writer::WriterEffect;
use crate::nexus::row::{ERROR_BIT, IO_BIT, READER_BIT, WRITER_BIT};
use crate::nexus::row::{EffectRow, Pure, Row};

// =============================================================================
// Effect Property Markers
// =============================================================================

/// Marker indicating a property is satisfied.
pub struct Satisfied;

/// Marker indicating a property is not satisfied.
pub struct NotSatisfied;

// =============================================================================
// Purity Trait
// =============================================================================

/// Trait for effects that are pure (no observable side effects).
///
/// Pure effects can be:
/// - Executed in parallel safely
/// - Memoized without concerns
/// - Reordered freely
///
/// # Laws
///
/// 1. **Referential Transparency**: `f(x) == f(x)` for all `x`
/// 2. **No Side Effects**: Execution doesn't affect external state
pub trait IsPure {
    /// Whether this effect row is pure.
    const IS_PURE: bool;
}

impl IsPure for Pure {
    const IS_PURE: bool = true;
}

impl<const BITS: u128> IsPure for Row<BITS> {
    const IS_PURE: bool = BITS == 0;
}

// =============================================================================
// Idempotency Trait
// =============================================================================

/// Trait for effects that are idempotent.
///
/// An effect is idempotent if executing it multiple times has the same
/// observable result as executing it once.
///
/// # Examples of Idempotent Effects
///
/// - `Reader`: Reading the same environment multiple times
/// - `Pure`: No effects at all
///
/// # Examples of Non-Idempotent Effects
///
/// - `Writer`: Each execution appends to the log
/// - `State`: Each execution may modify state
pub trait IsIdempotent {
    /// Whether this effect is idempotent.
    const IS_IDEMPOTENT: bool;
}

impl IsIdempotent for Pure {
    const IS_IDEMPOTENT: bool = true;
}

impl<E> IsIdempotent for Reader<E> {
    const IS_IDEMPOTENT: bool = true;
}

impl<const BITS: u128> IsIdempotent for Row<BITS> {
    // Idempotent if pure or reader-only
    const IS_IDEMPOTENT: bool = BITS == 0 || BITS == READER_BIT;
}

// =============================================================================
// Totality Trait
// =============================================================================

/// Trait for effects that are total (always terminate with a value).
///
/// Total effects can be safely used in speculative execution because
/// they're guaranteed to complete.
///
/// # Total Effects
///
/// - `Pure`: Just returns a value
/// - `Reader`: Reading always succeeds
/// - `State`: State operations always complete
///
/// # Partial Effects
///
/// - `Error`: May fail without producing a value
/// - `IO`: May hang or fail
pub trait IsTotal {
    /// Whether this effect is total.
    const IS_TOTAL: bool;
}

impl IsTotal for Pure {
    const IS_TOTAL: bool = true;
}

impl<E> IsTotal for Reader<E> {
    const IS_TOTAL: bool = true;
}

impl<S> IsTotal for State<S> {
    const IS_TOTAL: bool = true;
}

impl<W> IsTotal for WriterEffect<W> {
    const IS_TOTAL: bool = true;
}

// Error is NOT total - it can fail
// IO is NOT total - it can fail or hang

impl<const BITS: u128> IsTotal for Row<BITS> {
    // Total if no Error or IO effects
    const IS_TOTAL: bool = (BITS & ERROR_BIT) == 0 && (BITS & IO_BIT) == 0;
}

// =============================================================================
// Effect Properties Bundle
// =============================================================================

/// Bundle of all effect properties for a given effect row.
///
/// This provides a convenient way to query all properties at once.
pub struct EffectProperties<R: EffectRow> {
    _marker: PhantomData<R>,
}

impl<R: EffectRow> EffectProperties<R> {
    /// Whether the effect row is pure.
    pub const IS_PURE: bool = R::IS_PURE;

    /// Whether the effect row is idempotent.
    pub const IS_IDEMPOTENT: bool = R::BITS == 0 || R::BITS == READER_BIT;

    /// Whether the effect row is total.
    pub const IS_TOTAL: bool = (R::BITS & ERROR_BIT) == 0 && (R::BITS & IO_BIT) == 0;

    /// Whether the effect row is monotonic (can only add effects).
    pub const IS_MONOTONIC: bool = R::BITS == 0 || R::BITS == WRITER_BIT;

    /// Whether `par_map` and related parallel combinators can be used.
    pub const CAN_PARALLELIZE: bool = Self::IS_PURE;

    /// Whether `memoize` and caching combinators can be used.
    pub const CAN_MEMOIZE: bool = Self::IS_IDEMPOTENT;

    /// Whether `speculative` and `race` combinators can be used.
    pub const CAN_SPECULATE: bool = Self::IS_TOTAL;
}

// =============================================================================
// Compile-Time Property Assertions
// =============================================================================

/// Assert at compile time that an effect row is pure.
pub struct AssertPure<R: EffectRow>(PhantomData<R>);

impl<R: EffectRow> AssertPure<R> {
    /// Create the assertion. Only succeeds if R is pure.
    pub const fn new() -> Self
    where
        R: IsPure,
    {
        AssertPure(PhantomData)
    }
}

impl<R: EffectRow + IsPure> Default for AssertPure<R> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// Assert at compile time that an effect row is idempotent.
pub struct AssertIdempotent<R: EffectRow>(PhantomData<R>);

impl<R: EffectRow> AssertIdempotent<R> {
    /// Create the assertion. Only succeeds if R is idempotent.
    pub const fn new() -> Self
    where
        R: IsIdempotent,
    {
        AssertIdempotent(PhantomData)
    }
}

impl<R: EffectRow + IsIdempotent> Default for AssertIdempotent<R> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// Assert at compile time that an effect row is total.
pub struct AssertTotal<R: EffectRow>(PhantomData<R>);

impl<R: EffectRow> AssertTotal<R> {
    /// Create the assertion. Only succeeds if R is total.
    pub const fn new() -> Self
    where
        R: IsTotal,
    {
        AssertTotal(PhantomData)
    }
}

impl<R: EffectRow + IsTotal> Default for AssertTotal<R> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Property-Based Optimization Hints
// =============================================================================

/// Optimization hint that can be attached to computations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OptimizationHint {
    /// No special optimization.
    None,
    /// Safe to parallelize.
    Parallelize,
    /// Safe to memoize.
    Memoize,
    /// Safe to speculate.
    Speculate,
    /// Safe to fuse with adjacent operations.
    Fuse,
}

/// Compute the best optimization hint for an effect row.
pub const fn optimization_hint<R: EffectRow>() -> OptimizationHint {
    if R::IS_PURE {
        OptimizationHint::Parallelize
    } else if R::BITS == READER_BIT {
        // Reader-only can be memoized (most specific case)
        OptimizationHint::Memoize
    } else if (R::BITS & ERROR_BIT) == 0 && (R::BITS & IO_BIT) == 0 {
        // Total effects (no Error or IO) can be speculated
        OptimizationHint::Speculate
    } else {
        OptimizationHint::None
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nexus::row::STATE_BIT;

    #[test]
    fn test_pure_is_pure() {
        assert!(<Pure as IsPure>::IS_PURE);
        assert!(<Row<0> as IsPure>::IS_PURE);
    }

    #[test]
    fn test_state_is_not_pure() {
        assert!(!<Row<STATE_BIT> as IsPure>::IS_PURE);
    }

    #[test]
    fn test_reader_is_idempotent() {
        assert!(<Row<READER_BIT> as IsIdempotent>::IS_IDEMPOTENT);
    }

    #[test]
    fn test_writer_is_not_idempotent() {
        assert!(!<Row<WRITER_BIT> as IsIdempotent>::IS_IDEMPOTENT);
    }

    #[test]
    fn test_error_is_not_total() {
        assert!(!<Row<ERROR_BIT> as IsTotal>::IS_TOTAL);
    }

    #[test]
    fn test_state_is_total() {
        assert!(<Row<STATE_BIT> as IsTotal>::IS_TOTAL);
    }

    #[test]
    fn test_effect_properties_pure() {
        assert!(EffectProperties::<Pure>::IS_PURE);
        assert!(EffectProperties::<Pure>::IS_IDEMPOTENT);
        assert!(EffectProperties::<Pure>::IS_TOTAL);
        assert!(EffectProperties::<Pure>::CAN_PARALLELIZE);
        assert!(EffectProperties::<Pure>::CAN_MEMOIZE);
        assert!(EffectProperties::<Pure>::CAN_SPECULATE);
    }

    #[test]
    fn test_effect_properties_reader() {
        assert!(!EffectProperties::<Row<READER_BIT>>::IS_PURE);
        assert!(EffectProperties::<Row<READER_BIT>>::IS_IDEMPOTENT);
        assert!(EffectProperties::<Row<READER_BIT>>::IS_TOTAL);
        assert!(!EffectProperties::<Row<READER_BIT>>::CAN_PARALLELIZE);
        assert!(EffectProperties::<Row<READER_BIT>>::CAN_MEMOIZE);
        assert!(EffectProperties::<Row<READER_BIT>>::CAN_SPECULATE);
    }

    #[test]
    fn test_optimization_hint() {
        assert_eq!(optimization_hint::<Pure>(), OptimizationHint::Parallelize);
        assert_eq!(
            optimization_hint::<Row<READER_BIT>>(),
            OptimizationHint::Memoize
        );
        assert_eq!(
            optimization_hint::<Row<STATE_BIT>>(),
            OptimizationHint::Speculate
        );
        assert_eq!(optimization_hint::<Row<IO_BIT>>(), OptimizationHint::None);
    }
}
