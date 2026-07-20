//! Effect Row Types - Type-Level Effect Sets
//!
//! Effect rows represent sets of effects at the type level. This module provides
//! the foundation for zero-cost effects by enabling compile-time effect tracking
//! without runtime overhead.
//!
//! # Design
//!
//! Effect rows use const-generic bitmasks for O(1) effect membership checking:
//!
//! ```text
//! Row Representation (u128 bitmask):
//! ┌─────────────────────────────────────────────────────────────┐
//! │ Bit 0: IO        │ Bit 1: State    │ Bit 2: Error    │ ... │
//! │ Bit 3: Reader    │ Bit 4: Writer   │ Bit 5: Async    │ ... │
//! │ Bit 6: Choice    │ Bit 7: Nondet   │ Bit 8-127: User │     │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! This allows effect operations (union, intersection, membership) to compile
//! to single CPU instructions.
//!
//! # Relation to `effects::row_v2`
//!
//! This module and `effects::row_v2` are two deliberate, coexisting
//! type-level effect-set encodings: this `Row<const MASK: u128>` serves the
//! `nexus`-gated effect system; `row_v2`'s `EffectSet<const MASK: u128>`
//! serves the `async`-gated `effects` system (inline-const verified). No
//! bridge exists, deliberately — no consumer composes both in one signature.
//! Revisit unification only if such a consumer appears; this module is then
//! the donor encoding and `EffectSet` the target.

// =============================================================================
// Effect Row Trait
// =============================================================================

/// A type-level effect row representing a set of effects.
///
/// This trait is implemented for effect row types and provides const-generic
/// operations for zero-cost effect tracking.
///
/// # Const Parameters
///
/// The row is represented as a u128 bitmask where each bit represents the
/// presence of a specific effect. This allows all row operations to be
/// computed at compile time.
pub trait EffectRow: Copy + Clone {
    /// The bitmask representation of this effect row.
    const BITS: u128;

    /// Whether this row represents pure computation (no effects).
    const IS_PURE: bool = Self::BITS == 0;

    /// Whether this row contains only the State effect.
    const IS_STATE_ONLY: bool = Self::BITS == STATE_BIT;

    /// Whether this row contains only the Reader effect.
    const IS_READER_ONLY: bool = Self::BITS == READER_BIT;

    /// Whether this row contains only the Error effect.
    const IS_ERROR_ONLY: bool = Self::BITS == ERROR_BIT;

    /// Whether this row contains the IO effect.
    const HAS_IO: bool = (Self::BITS & IO_BIT) != 0;

    /// Whether this row contains the State effect.
    const HAS_STATE: bool = (Self::BITS & STATE_BIT) != 0;

    /// Whether this row contains the Reader effect.
    const HAS_READER: bool = (Self::BITS & READER_BIT) != 0;

    /// Whether this row contains the Error effect.
    const HAS_ERROR: bool = (Self::BITS & ERROR_BIT) != 0;

    /// Whether this row contains the Writer effect.
    const HAS_WRITER: bool = (Self::BITS & WRITER_BIT) != 0;

    /// Whether this row contains the Async effect.
    const HAS_ASYNC: bool = (Self::BITS & ASYNC_BIT) != 0;
}

// =============================================================================
// Effect Bit Constants
// =============================================================================

/// Bit position for the IO effect.
pub const IO_BIT: u128 = 1 << 0;

/// Bit position for the State effect.
pub const STATE_BIT: u128 = 1 << 1;

/// Bit position for the Error effect.
pub const ERROR_BIT: u128 = 1 << 2;

/// Bit position for the Reader effect.
pub const READER_BIT: u128 = 1 << 3;

/// Bit position for the Writer effect.
pub const WRITER_BIT: u128 = 1 << 4;

/// Bit position for the Async effect.
pub const ASYNC_BIT: u128 = 1 << 5;

/// Bit position for the Choice effect.
pub const CHOICE_BIT: u128 = 1 << 6;

/// Bit position for the Nondet effect.
pub const NONDET_BIT: u128 = 1 << 7;

/// Start of user-defined effect bits.
pub const USER_EFFECT_START: u128 = 1 << 8;

// =============================================================================
// Pure Effect Row
// =============================================================================

/// The empty effect row representing pure computation.
///
/// `Pure` is the identity for effect row union and indicates that a computation
/// has no side effects. Computations with `Pure` effect row can be:
/// - Freely reordered
/// - Automatically parallelized
/// - Memoized without concern
///
/// # Example
///
/// ```rust
/// use ordofp_core::nexus::{pure, Eff, Pure};
///
/// // A pure computation can be run directly
/// let result: Eff<Pure, i32> = pure(42);
/// assert_eq!(result.run_pure(), 42);
/// ```
#[derive(Copy, Clone, Debug, Default)]
pub struct Pure;

impl EffectRow for Pure {
    const BITS: u128 = 0;
}

// =============================================================================
// Concrete Row Types
// =============================================================================

/// A concrete effect row with a specific bitmask.
///
/// This type represents effect rows at runtime, though most operations
/// are performed at compile time through const generics.
#[derive(Copy, Clone, Debug, Default)]
pub struct Row<const BITS: u128>;

impl<const BITS: u128> EffectRow for Row<BITS> {
    const BITS: u128 = BITS;
}

// =============================================================================
// Row Operations
// =============================================================================

// Note: Type aliases with const-generic expressions and where clauses
// require the lazy_type_alias feature. For now, we use functions instead.

/// Compute the union of two effect row bitmasks.
///
/// Returns the set of effects present in **either** `R1` or `R2` (or both).
/// An effect bit is set in the result whenever it is set in at least one of
/// the two input rows.
///
/// The result is evaluated at **compile time** when both `R1::BITS` and
/// `R2::BITS` are `const`.
///
/// # Use cases
///
/// - Combining the effects of two computations that will be sequenced
///   together (e.g. via `.and_then`).
/// - Building composite effect rows from individual effect markers.
///
/// # Examples
///
/// ```rust
/// use ordofp_core::nexus::prelude::{IoRow, StateRow};
/// use ordofp_core::nexus::{row_union_bits, EffectRow, Pure};
///
/// // IO ∪ State contains both effect bits.
/// let combined = row_union_bits::<IoRow, StateRow>();
/// assert!(combined & IoRow::BITS != 0);
/// assert!(combined & StateRow::BITS != 0);
///
/// // Any row unioned with Pure equals itself (Pure has no effect bits).
/// assert_eq!(row_union_bits::<IoRow, Pure>(), IoRow::BITS);
///
/// // Idempotent: a row unioned with itself is itself.
/// assert_eq!(row_union_bits::<StateRow, StateRow>(), StateRow::BITS);
/// ```
pub const fn row_union_bits<R1: EffectRow, R2: EffectRow>() -> u128 {
    R1::BITS | R2::BITS
}

/// Compute the intersection of two effect row bitmasks.
///
/// Returns the set of effects that are present in **both** `R1` and `R2`.
/// An effect bit is set in the result only when it is set in both input rows.
///
/// The check is evaluated at **compile time** when both `R1::BITS` and
/// `R2::BITS` are `const`.
///
/// # Use cases
///
/// - Verifying that two computations share a common set of effects before
///   combining them.
/// - Effect analysis passes that need to identify overlapping requirements.
///
/// # Examples
///
/// ```rust
/// use ordofp_core::nexus::prelude::{IoRow, IoStateRow, StateRow};
/// use ordofp_core::nexus::{row_intersect_bits, Pure};
///
/// // IO ∩ State = ∅  (no bits in common)
/// assert_eq!(row_intersect_bits::<IoRow, StateRow>(), 0);
///
/// // (IO + State) ∩ IO = IO
/// assert_eq!(row_intersect_bits::<IoStateRow, IoRow>(), row_intersect_bits::<IoRow, IoRow>());
///
/// // Any row intersected with Pure gives the empty row.
/// assert_eq!(row_intersect_bits::<IoRow, Pure>(), 0);
/// ```
pub const fn row_intersect_bits<R1: EffectRow, R2: EffectRow>() -> u128 {
    R1::BITS & R2::BITS
}

/// Compute the difference of two effect row bitmasks.
pub const fn row_diff_bits<R1: EffectRow, R2: EffectRow>() -> u128 {
    R1::BITS & !R2::BITS
}

/// Returns `true` if every effect in `R1` is also present in `R2`.
///
/// A computation typed as `Eff<R1, A>` can be safely widened to `Eff<R2, A>`
/// whenever `row_subset::<R1, R2>()` holds — the handler for `R2` knows how
/// to discharge all of `R1`'s effects because `R2` is a superset of them.
///
/// The check is a single bitmask test and is evaluated at **compile time**
/// when both `R1::BITS` and `R2::BITS` are `const`.
///
/// # Examples
///
/// ```rust
/// use ordofp_core::nexus::prelude::{row_subset, IoRow, IoStateRow, Pure, StateRow};
///
/// // A pure row is a subset of every row (no effects to satisfy).
/// assert!(row_subset::<Pure, IoRow>());
///
/// // IO ⊆ IO + State, but IO + State ⊄ IO.
/// assert!( row_subset::<IoRow, IoStateRow>());
/// assert!(!row_subset::<IoStateRow, IoRow>());
///
/// // Any row is a subset of itself.
/// assert!(row_subset::<StateRow, StateRow>());
/// ```
pub const fn row_subset<R1: EffectRow, R2: EffectRow>() -> bool {
    (R1::BITS & !R2::BITS) == 0
}

/// Check if two rows are equal.
pub const fn row_eq<R1: EffectRow, R2: EffectRow>() -> bool {
    R1::BITS == R2::BITS
}

// =============================================================================
// Effect Row Macros
// =============================================================================

/// Macro for creating effect row types.
///
/// # Example
///
/// ```rust
/// use ordofp_core::effect_row;
/// use ordofp_core::nexus::{EffectRow, Error, State, IO};
///
/// struct Config;
/// struct AppError;
///
/// type MyEffects = effect_row![IO, State<Config>, Error<AppError>];
///
/// assert!(MyEffects::HAS_IO);
/// assert!(MyEffects::HAS_STATE);
/// assert!(MyEffects::HAS_ERROR);
/// ```
#[macro_export]
macro_rules! effect_row {
    () => { $crate::nexus::prelude::Pure };
    ($e:ty) => { $crate::nexus::prelude::Row<{ <$e as $crate::nexus::EffectMarker>::BIT }> };
    ($e:ty, $($rest:ty),+ $(,)?) => {
        $crate::nexus::prelude::Row<{
            <$e as $crate::nexus::EffectMarker>::BIT
            $(| <$rest as $crate::nexus::EffectMarker>::BIT)+
        }>
    };
}

// =============================================================================
// Row Type Aliases for Common Patterns
// =============================================================================

/// IO-only effect row.
pub type IoRow = Row<IO_BIT>;

/// State-only effect row.
pub type StateRow = Row<STATE_BIT>;

/// Error-only effect row.
pub type ErrorRow = Row<ERROR_BIT>;

/// Reader-only effect row.
pub type ReaderRow = Row<READER_BIT>;

/// Writer-only effect row.
pub type WriterRow = Row<WRITER_BIT>;

/// Async-only effect row.
pub type AsyncRow = Row<ASYNC_BIT>;

/// IO + Error effects (common pattern).
pub type IoErrorRow = Row<{ IO_BIT | ERROR_BIT }>;

/// IO + State effects.
pub type IoStateRow = Row<{ IO_BIT | STATE_BIT }>;

/// Full synchronous effects (IO + State + Error + Reader + Writer).
pub type FullSyncRow = Row<{ IO_BIT | STATE_BIT | ERROR_BIT | READER_BIT | WRITER_BIT }>;

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pure_row() {
        assert!(Pure::IS_PURE);
        assert!(!Pure::HAS_IO);
        assert!(!Pure::HAS_STATE);
        assert_eq!(Pure::BITS, 0);
    }

    #[test]
    fn test_io_row() {
        assert!(!IoRow::IS_PURE);
        assert!(IoRow::HAS_IO);
        assert!(!IoRow::HAS_STATE);
        assert_eq!(IoRow::BITS, IO_BIT);
    }

    #[test]
    fn test_state_row() {
        assert!(StateRow::IS_STATE_ONLY);
        assert!(StateRow::HAS_STATE);
        assert!(!StateRow::HAS_IO);
    }

    #[test]
    fn test_row_union() {
        // Test that union of IO and State has both bits set
        const COMBINED_BITS: u128 = row_union_bits::<IoRow, StateRow>();
        type Combined = Row<COMBINED_BITS>;
        assert!(Combined::HAS_IO);
        assert!(Combined::HAS_STATE);
        assert!(!Combined::HAS_ERROR);
    }

    #[test]
    fn test_row_subset() {
        assert!(row_subset::<Pure, IoRow>());
        assert!(row_subset::<IoRow, IoStateRow>());
        assert!(!row_subset::<IoStateRow, IoRow>());
    }

    #[test]
    fn test_row_eq() {
        assert!(row_eq::<Pure, Pure>());
        assert!(row_eq::<IoRow, IoRow>());
        assert!(!row_eq::<IoRow, StateRow>());
    }
}
