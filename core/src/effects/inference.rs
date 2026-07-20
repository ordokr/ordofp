//! Effect Inference - Utilities for inferring and manipulating effect types
//!
//! > *"Inferentia effectuum"*
//! > — Inference of effects. (Neo-Latin)
//!
//! This module provides utilities for working with effect types at compile time,
//! enabling ergonomic effect composition and type-safe effect manipulation.
//!
//! # Design
//!
//! The inference utilities provide:
//! - Type-level effect row manipulation
//! - Effect row building macros
//! - Constraint helpers for requiring effects
//!
//! # Scholastic Naming
//!
//! | English | Latin | Etymology |
//! |---------|-------|-----------|
//! | Inference | Inferentia | *inferre* = to bring in, infer |
//! | Require | Requirere | *requirere* = to seek, require |
//! | Constraint | Constrictio | *constringere* = to bind together |

use super::row_v2::{EffectId, EffectRow, EffectSet};
use core::marker::PhantomData;

/// Marker trait for inferring effect requirements.
///
/// `InferEffectus<R>` indicates that a type requires effects from row `R`.
/// This trait helps with effect inference in Universalis contexts.
///
/// # Example
///
/// ```rust
/// use ordofp_core::effects::inference::InferEffectus;
/// use ordofp_core::effects::row_v2::IoRow;
///
/// struct MyService;
///
/// impl InferEffectus<IoRow> for MyService {
///     // This service requires IO effects
/// }
/// ```
pub trait InferEffectus<R: EffectRow> {}

/// A witness that an effect is present in a row.
///
/// `EffectusWitness<E, R>` provides a proof that effect `E` is available
/// in effect row `R`. This is useful for passing effect availability
/// as a value rather than just a type constraint.
///
/// # Example
///
/// ```rust
/// use ordofp_core::effects::inference::EffectusWitness;
/// use ordofp_core::effects::IoEffectus;
/// use ordofp_core::effects::row_v2::{EffectRow, IoRow};
///
/// fn with_io_proof<R: EffectRow>(witness: EffectusWitness<IoEffectus, R>) {
///     // We have proof that IO is available
/// }
///
/// let witness = EffectusWitness::<IoEffectus, IoRow>::new();
/// with_io_proof(witness);
/// ```
pub struct EffectusWitness<E: EffectId, R: EffectRow> {
    _effect: PhantomData<E>,
    _row: PhantomData<R>,
}

impl<E: EffectId, R: EffectRow> EffectusWitness<E, R> {
    /// Create a new effect witness.
    ///
    /// Constructing the witness is the proof: a compile-time
    /// (post-monomorphization) assertion checks that `R` contains `E`,
    /// replacing the old `HasEffectType` bound that needed
    /// `generic_const_exprs`.
    #[inline]
    pub const fn new() -> Self {
        const {
            assert!(
                (R::MASK >> E::ID) & 1 == 1,
                "effect row is missing the witnessed effect"
            );
        }
        EffectusWitness {
            _effect: PhantomData,
            _row: PhantomData,
        }
    }
}

impl<E: EffectId, R: EffectRow> Default for EffectusWitness<E, R> {
    fn default() -> Self {
        Self::new()
    }
}

impl<E: EffectId, R: EffectRow> Clone for EffectusWitness<E, R> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<E: EffectId, R: EffectRow> Copy for EffectusWitness<E, R> {}

/// Builder for constructing effect rows.
///
/// `EffectusBuilder` provides a fluent API for building effect rows
/// with compile-time safety.
///
/// # Example
///
/// ```rust
/// use ordofp_core::effects::inference::EffectusBuilder;
/// use ordofp_core::effects::row_v2::{EffectSet, builtin_ids};
/// use ordofp_core::effects::{ErrorEffectus, IoEffectus};
///
/// // Build a row with IO and Error effects
/// let _builder = EffectusBuilder::<EffectSet<0>>::new()
///     .add::<IoEffectus, { 1 << builtin_ids::IO }>()
///     .add::<ErrorEffectus<String>, { (1 << builtin_ids::IO) | (1 << builtin_ids::ERROR) }>();
/// ```
pub struct EffectusBuilder<R: EffectRow> {
    _row: PhantomData<R>,
}

impl Default for EffectusBuilder<EffectSet<0>> {
    fn default() -> Self {
        Self::new()
    }
}

impl EffectusBuilder<EffectSet<0>> {
    /// Start building an effect row.
    #[inline]
    pub const fn new() -> Self {
        EffectusBuilder { _row: PhantomData }
    }
}

impl<const MASK: u128> EffectusBuilder<EffectSet<MASK>> {
    /// Add an effect to the row being built, transitioning to a new
    /// builder whose mask is caller-chosen.
    ///
    /// In v1 the next row was `RowExtensio<E, R>` (inferred from `E`).
    /// V2 requires the new mask to be nameable in a type position, and
    /// without `generic_const_exprs` the next mask cannot be derived from
    /// `E::ID | MASK` implicitly. Callers now supply `NEXT` explicitly:
    ///
    /// ```rust
    /// use ordofp_core::effects::inference::EffectusBuilder;
    /// use ordofp_core::effects::row_v2::{EffectSet, builtin_ids};
    /// use ordofp_core::effects::IoEffectus;
    ///
    /// let b = EffectusBuilder::<EffectSet<0>>::new()
    ///     .add::<IoEffectus, { 1 << builtin_ids::IO }>();
    /// ```
    #[inline]
    pub fn add<E: EffectId, const NEXT: u128>(self) -> EffectusBuilder<EffectSet<NEXT>> {
        EffectusBuilder { _row: PhantomData }
    }

    /// Get the type of the built row.
    #[inline]
    pub fn build(self) -> PhantomData<EffectSet<MASK>> {
        PhantomData
    }
}

/// Type alias for the row type from a builder.
pub type BuilderRow<B> = <B as HasRow>::Row;

/// Trait to extract the row type from a builder.
pub trait HasRow {
    /// The effect row this builder has accumulated.
    type Row: EffectRow;
}

impl<R: EffectRow> HasRow for EffectusBuilder<R> {
    type Row = R;
}

/// Constraint that requires multiple effects.
///
/// `RequireEffects` is a helper trait for requiring multiple effects
/// at once, reducing boilerplate in function signatures.
///
/// # Example
///
/// ```rust
/// use ordofp_core::effects::inference::assert_requires_effects;
/// use ordofp_core::effects::row_v2::{EffectRow, IoStateErrorRow};
/// use ordofp_core::effects::{ErrorEffectus, IoEffectus};
///
/// fn my_function<R: EffectRow>() {
///     assert_requires_effects::<R, (IoEffectus, ErrorEffectus<String>)>();
///     // R has both IO and Error effects
/// }
///
/// my_function::<IoStateErrorRow>();
/// ```
pub trait EffectTuple {
    /// Combined bitmask of every effect in the tuple.
    const MASK: u128;
}

impl EffectTuple for () {
    const MASK: u128 = 0;
}

impl<E1: EffectId> EffectTuple for (E1,) {
    const MASK: u128 = 1 << E1::ID;
}

impl<E1: EffectId, E2: EffectId> EffectTuple for (E1, E2) {
    const MASK: u128 = (1 << E1::ID) | (1 << E2::ID);
}

impl<E1: EffectId, E2: EffectId, E3: EffectId> EffectTuple for (E1, E2, E3) {
    const MASK: u128 = (1 << E1::ID) | (1 << E2::ID) | (1 << E3::ID);
}

impl<E1: EffectId, E2: EffectId, E3: EffectId, E4: EffectId> EffectTuple for (E1, E2, E3, E4) {
    const MASK: u128 = (1 << E1::ID) | (1 << E2::ID) | (1 << E3::ID) | (1 << E4::ID);
}

impl<E1: EffectId, E2: EffectId, E3: EffectId, E4: EffectId, E5: EffectId> EffectTuple
    for (E1, E2, E3, E4, E5)
{
    const MASK: u128 =
        (1 << E1::ID) | (1 << E2::ID) | (1 << E3::ID) | (1 << E4::ID) | (1 << E5::ID);
}

/// Compile-time proof that row `R` contains every effect in tuple `Effects`.
///
/// Successor to the old `RequireEffects<Effects>` bound, whose conditional
/// impls needed `generic_const_exprs`.
#[inline(always)]
pub fn assert_requires_effects<R: EffectRow, Effects: EffectTuple>() {
    const {
        assert!(
            (R::MASK & Effects::MASK) == Effects::MASK,
            "effect row is missing required effects"
        );
    }
}

/// Constraint for effect row equality.
///
/// Two rows are equivalent if they contain the same effects
/// (regardless of order).
pub trait RowEquivalent<Other: EffectRow>: EffectRow {}

// Empty rows are equivalent
impl RowEquivalent<EffectSet<0>> for EffectSet<0> {}

/// Type alias for merged rows.
///
/// In v1 this was `EffectusUnio<R1, R2>`. In v2 merging two masks by name
/// requires `generic_const_exprs`, so this alias now takes the merged mask
/// directly as a const generic. Callers that previously wrote
/// `Merged<R1, R2>` should now write `Merged<{ R1::MASK | R2::MASK }>` (in
/// a context where that const-expression is valid) or name the combined
/// mask explicitly.
pub type Merged<const MASK: u128> = EffectSet<MASK>;

/// Type-level function to check if a row is pure (has no effects).
pub trait IsPure: EffectRow {
    /// `true` exactly when the row's effect mask is empty (`EffectSet<0>`).
    const IS_PURE: bool;
}

impl IsPure for EffectSet<0> {
    const IS_PURE: bool = true;
}

// The blanket impl below covers all non-empty masks (MASK != 0). We cannot
// directly write a const-comparison in a where-clause without
// `generic_const_exprs`, so we rely on the specialization between the
// explicit `EffectSet<0>` impl above and this blanket default. Because
// specialization is currently not stable for associated consts, we express
// "non-pure" by providing a fallback trait and let callers use `Self::MASK
// != 0` at the value level. Consumers of `IS_PURE` now see `true` only for
// the literal empty row; other rows must call `EffectSet::is_empty()` or
// check `Self::MASK`.
//
// This is a deliberate narrowing of the API relative to v1, which could
// pattern-match the row's structural shape.

/// Marker for effect polymorphism.
///
/// `EffectVar<N>` represents an effect type variable, useful for
/// expressing row polymorphism in function signatures.
///
/// # Example
///
/// ```rust
/// use ordofp_core::effects::inference::EffectVar;
/// use ordofp_core::effects::row_v2::EffectId;
///
/// // A function polymorphic over "any" effect-like type parameter.
/// fn accepts_effect_id<E: EffectId>() {}
///
/// accepts_effect_id::<EffectVar<0>>();
/// ```
pub struct EffectVar<const N: usize>;

impl<const N: usize> EffectId for EffectVar<N> {
    const ID: u64 = N as u64;
    const NAME: &'static str = "EffectVar";
}

/// Marker for "any additional effects".
///
/// Used in type signatures to indicate row polymorphism.
pub type AnyEffects = EffectVar<0>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effects::row_v2::{IoRow, builtin_ids};
    use crate::effects::{ErrorEffectus, IoEffectus};

    #[test]
    fn test_effectus_witness() {
        let _witness: EffectusWitness<IoEffectus, IoRow> = EffectusWitness::new();
    }

    #[test]
    fn test_effectus_builder() {
        // V2 add() needs an explicit target mask for each step.
        let _builder = EffectusBuilder::<EffectSet<0>>::new()
            .add::<IoEffectus, { 1 << builtin_ids::IO }>()
            .add::<ErrorEffectus<&str>, { (1 << builtin_ids::IO) | (1 << builtin_ids::ERROR) }>();
    }

    fn requires_single<R: EffectRow>() {
        assert_requires_effects::<R, (IoEffectus,)>();
    }

    #[test]
    fn test_require_effects() {
        // Single-effect requirement, checked at monomorphization time.
        requires_single::<IoRow>();
    }

    #[test]
    fn test_is_pure() {
        assert!(<EffectSet<0> as IsPure>::IS_PURE);
        // Non-empty rows no longer implement IsPure; use the runtime helper.
        assert!(!IoRow::is_empty());
    }

    #[test]
    fn test_effect_var() {
        fn accepts_effect_id<E: EffectId>() {}
        accepts_effect_id::<EffectVar<0>>();
        accepts_effect_id::<AnyEffects>();
    }

    #[test]
    fn test_merged_rows() {
        type Combined = Merged<{ (1 << builtin_ids::IO) | (1 << builtin_ids::ERROR) }>;
        fn requires_row<R: EffectRow>() {}
        requires_row::<Combined>();
    }

    fn with_proof<R: EffectRow>(_witness: EffectusWitness<IoEffectus, R>) {}

    #[test]
    fn test_witness_as_proof() {
        let witness = EffectusWitness::<IoEffectus, IoRow>::new();
        with_proof(witness);
    }
}
