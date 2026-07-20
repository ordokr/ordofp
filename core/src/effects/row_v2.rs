//! Row V2 - Const-Generic Effect Rows
//!
//! > *"Ordo effectuum in numeris latet"*
//! > — The order of effects lies hidden in numbers. (Neo-Latin)
//!
//! This module provides a revolutionary approach to effect row tracking using
//! const-generic bitsets instead of type-level lists. This enables O(1) effect
//! membership checks and efficient effect set operations at compile time.
//!
//! # Architecture
//!
//! Effects are assigned unique IDs (0-127) via the `EffectId` trait. Effect sets
//! are represented as `EffectSet<const MASK: u128>` where each bit position
//! corresponds to an effect ID.
//!
//! # Relation to `nexus::row`
//!
//! `nexus::row` is a second, deliberate type-level effect-set encoding
//! (`Row<const MASK: u128>`) serving the `nexus`-gated system. The two
//! coexist on a documented boundary; no bridge exists because no consumer
//! composes both in one signature. Revisit unification only if such a
//! consumer appears; this module's `EffectSet` is then the target encoding
//! and `nexus::row` the donor.
//!
//! # Example
//!
//! ```rust
//! use core::marker::PhantomData;
//! use ordofp_core::effects::row_v2::*;
//!
//! // Define effects with unique IDs
//! struct IoEffect;
//! impl EffectId for IoEffect {
//!     const ID: u64 = 0;
//!     const NAME: &'static str = "IO";
//! }
//!
//! struct StateEffect<S>(PhantomData<S>);
//! impl<S> EffectId for StateEffect<S> {
//!     const ID: u64 = 1;
//!     const NAME: &'static str = "State";
//! }
//!
//! // Create effect sets
//! type IoOnly = EffectSet<{ 1 << 0 }>;           // Just IO
//! type IoAndState = EffectSet<{ (1 << 0) | (1 << 1) }>; // IO + State
//!
//! // Effect constraints (post-monomorphization compile-time checks)
//! fn requires_io<R: EffectRow>() { assert_has_effect::<R, 0>(); }
//! fn requires_state<R: EffectRow>() { assert_has_effect::<R, 1>(); }
//!
//! requires_io::<IoOnly>();
//! requires_io::<IoAndState>();
//! requires_state::<IoAndState>();
//! ```
//!
//! # Comparison to Row V1
//!
//! | Feature | Row V1 (RowExtensio) | Row V2 (EffectSet) |
//! |---------|---------------------|-------------------|
//! | Membership check | O(n) type recursion | O(1) const arithmetic |
//! | Union | Nested types | Const OR |
//! | Intersection | Complex | Const AND |
//! | Max effects | Unlimited (but slow) | 128 (fast) |
//! | Error messages | Deeply nested | Clear bitset display |

use core::marker::PhantomData;

// =============================================================================
// Effect Identity
// =============================================================================

/// Trait for effects with unique compile-time identifiers.
///
/// Every effect type must implement this trait with a unique `ID` value.
/// The ID is used as the bit position in `EffectSet<MASK>`.
///
/// # Scholastic Etymology
///
/// *Effectus Identitas* — Effect Identity
///
/// # Safety
///
/// IDs must be unique across all effects in a program. Duplicate IDs will
/// cause incorrect effect tracking. There is no derive macro for this trait;
/// assign IDs manually from a single registry in your program (the built-in
/// effects reserve the IDs in [`builtin_ids`]).
///
/// # Example
///
/// ```rust
/// use ordofp_core::effects::row_v2::EffectId;
///
/// struct MyEffect;
///
/// impl EffectId for MyEffect {
///     const ID: u64 = 42;
///     const NAME: &'static str = "MyEffect";
/// }
///
/// assert_eq!(MyEffect::ID, 42);
/// assert_eq!(MyEffect::NAME, "MyEffect");
/// assert_eq!(MyEffect::mask(), 1u128 << 42);
/// ```
pub trait EffectId {
    /// Unique identifier for this effect (0-127).
    ///
    /// Must be unique across all effects in the program.
    const ID: u64;

    /// Human-readable name for error messages and debugging.
    const NAME: &'static str;

    /// Get the bitmask for this effect.
    #[inline]
    fn mask() -> u128 {
        1u128 << Self::ID
    }
}

// =============================================================================
// Effect Set
// =============================================================================

/// A compile-time set of effects represented as a bitmask.
///
/// `EffectSet<MASK>` is the core type for Row V2 effect tracking. Each bit
/// in `MASK` represents the presence (1) or absence (0) of an effect.
///
/// # Scholastic Etymology
///
/// *Coniunctio Effectuum* — Conjunction of Effects
///
/// # Type Parameters
///
/// * `MASK` - A 128-bit bitmask where bit `i` indicates effect with ID `i` is present.
///
/// # Example
///
/// ```rust
/// use ordofp_core::effects::row_v2::EffectSet;
///
/// // Empty effect set (pure computation)
/// type MyPure = EffectSet<0>;
///
/// // Single effect (bit 0 set)
/// type IoOnly = EffectSet<1>;
///
/// // Multiple effects (bits 0 and 1 set)
/// type IoAndState = EffectSet<3>;  // 0b11
///
/// assert!(MyPure::is_empty());
/// assert!(!IoOnly::is_empty());
/// assert_eq!(IoAndState::count(), 2);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EffectSet<const MASK: u128>;

impl<const MASK: u128> EffectSet<MASK> {
    /// The bitmask value for this effect set.
    pub const MASK: u128 = MASK;

    /// Create a new effect set instance.
    #[inline]
    pub const fn new() -> Self {
        EffectSet
    }

    /// Check if this set is empty (pure computation).
    #[inline]
    pub const fn is_empty() -> bool {
        MASK == 0
    }

    /// Count the number of effects in this set.
    #[inline]
    pub const fn count() -> u32 {
        MASK.count_ones()
    }

    /// Check if a specific effect ID is present.
    #[inline]
    pub const fn contains(effect_id: u64) -> bool {
        (MASK >> effect_id) & 1 == 1
    }

    /// Get an iterator over effect IDs in this set.
    #[inline]
    pub fn iter() -> EffectSetIter {
        EffectSetIter {
            mask: MASK,
            position: 0,
        }
    }
}

impl<const MASK: u128> Default for EffectSet<MASK> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// Iterator over effect IDs in an effect set.
pub struct EffectSetIter {
    mask: u128,
    position: u64,
}

impl Iterator for EffectSetIter {
    type Item = u64;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        while self.position < 128 {
            let pos = self.position;
            self.position += 1;
            if (self.mask >> pos) & 1 == 1 {
                return Some(pos);
            }
        }
        None
    }
}

// =============================================================================
// Effect Set Type Aliases
// =============================================================================

/// Empty effect set - pure computation.
///
/// *Vacuus* — Empty, void.
pub type EffectSetVacuus = EffectSet<0>;

/// Alias for pure effect set.
pub type Pure = EffectSetVacuus;

// =============================================================================
// Effect Membership (compile-time assertions)
// =============================================================================
//
// These were previously expressed as trait bounds (`HasEffect<ID>`,
// `SubRow<SUPER>`, …) whose impls were conditioned on
// `Assert<{ const expr }>: IsTrue` — a pattern that required the incomplete
// `generic_const_exprs` feature. They are now inline-const assertions that
// evaluate post-monomorphization (stable Rust): violations are still
// compile-time errors at the offending instantiation, with zero runtime cost.

/// Compile-time proof that row `R` contains the effect with `EFFECT_ID`.
///
/// Call at the top of any function that requires the effect:
///
/// ```rust
/// use ordofp_core::effects::row_v2::{EffectRow, EffectSet, assert_has_effect};
///
/// fn requires_io<R: EffectRow>() {
///     assert_has_effect::<R, 0>();
/// }
///
/// requires_io::<EffectSet<1>>(); // compiles: IO present
/// // requires_io::<EffectSet<2>>(); // compile error: only State, no IO
/// ```
///
/// # Scholastic Etymology
///
/// *Habet Effectum* — Has the effect.
#[inline(always)]
pub fn assert_has_effect<R: EffectRow, const EFFECT_ID: u64>() {
    const {
        assert!(
            (R::MASK >> EFFECT_ID) & 1 == 1,
            "effect row is missing a required effect"
        );
    }
}

/// Compile-time proof that row `R` does NOT contain the effect with
/// `EFFECT_ID`.
///
/// # Scholastic Etymology
///
/// *Sine Effectu* — Without the effect.
#[inline(always)]
pub fn assert_without_effect<R: EffectRow, const EFFECT_ID: u64>() {
    const {
        assert!(
            (R::MASK >> EFFECT_ID) & 1 == 0,
            "effect row unexpectedly contains an excluded effect"
        );
    }
}

// =============================================================================
// Effect Set Operations
// =============================================================================
//
// Mask algebra is provided as `const fn`s; build the resulting set with an
// anonymous constant over concrete masks, e.g.
// `EffectSet<{ union(A::MASK, B::MASK) }>`.

/// Union of two effect-set masks (*Unio* — union, joining).
#[inline]
#[must_use]
pub const fn union(a: u128, b: u128) -> u128 {
    a | b
}

/// Intersection of two effect-set masks (*Intersectio* — crossing).
#[inline]
#[must_use]
pub const fn intersection(a: u128, b: u128) -> u128 {
    a & b
}

/// Difference of two effect-set masks, `a - b` (*Differentia* — distinction).
#[inline]
#[must_use]
pub const fn difference(a: u128, b: u128) -> u128 {
    a & !b
}

/// Add an effect to a set mask (*Extensio* — extension).
#[inline]
#[must_use]
pub const fn extend(set: u128, effect_id: u64) -> u128 {
    set | (1 << effect_id)
}

/// Remove an effect from a set mask (*Contractio* — contraction).
#[inline]
#[must_use]
pub const fn contract(set: u128, effect_id: u64) -> u128 {
    set & !(1 << effect_id)
}

// =============================================================================
// Subrow Relationship
// =============================================================================

/// Compile-time proof that every effect in `R` is also in `SUPER` — effect
/// polymorphism: a computation with fewer effects can be used where more
/// effects are expected.
///
/// # Scholastic Etymology
///
/// *Sub Ordine* — Under the order.
#[inline(always)]
pub fn assert_subrow<R: EffectRow, const SUPER: u128>() {
    const {
        assert!(
            (R::MASK & SUPER) == R::MASK,
            "effect row is not a subset of the target row"
        );
    }
}

/// Compile-time proof that `R` and `OTHER` share no effects.
///
/// # Scholastic Etymology
///
/// *Disiunctio* — Disjunction, separation.
#[inline(always)]
pub fn assert_disjoint<R: EffectRow, const OTHER: u128>() {
    const { assert!((R::MASK & OTHER) == 0, "effect rows are not disjoint") }
}

// =============================================================================
// Effect Row Marker
// =============================================================================

/// Marker trait for all effect row types.
///
/// This enables generic programming over effect rows. The `Send + Sync +
/// 'static` supertrait bounds mirror the row-v1 `EffectusRow` contract so
/// consumers that previously held effect rows across thread boundaries (e.g.
/// `Box<dyn FnOnce() + Send>` closures in `Eff`/`Sem`/`Computatio`) keep
/// working after the v1 → v2 consolidation.
pub trait EffectRow: Send + Sync + 'static {
    /// The bitmask representation of this row.
    const MASK: u128;
}

impl<const MASK: u128> EffectRow for EffectSet<MASK> {
    const MASK: u128 = MASK;
}

// =============================================================================
// Typed Effect Membership
// =============================================================================

/// Compile-time proof that a typed effect is in a row.
///
/// Unlike [`assert_has_effect`], this takes the effect type directly:
///
/// ```rust
/// use ordofp_core::effects::row_v2::{EffectRow, EffectSet, IoEffectV2, assert_has_effect_type};
///
/// fn requires<R: EffectRow>() {
///     assert_has_effect_type::<R, IoEffectV2>();
/// }
///
/// requires::<EffectSet<1>>(); // compiles: IO (bit 0) present
/// ```
#[inline(always)]
pub fn assert_has_effect_type<R: EffectRow, E: EffectId>() {
    const {
        assert!(
            (R::MASK >> E::ID) & 1 == 1,
            "effect row is missing a required typed effect"
        );
    }
}

// =============================================================================
// Common Effect IDs
// =============================================================================

/// Reserved effect ID namespace.
///
/// IDs 0-31 are reserved for built-in effects.
/// IDs 32-127 are available for user-defined effects.
pub mod builtin_ids {
    /// IO effect (console, file, network).
    pub const IO: u64 = 0;

    /// State effect (mutable state).
    pub const STATE: u64 = 1;

    /// Reader effect (environment access).
    pub const READER: u64 = 2;

    /// Writer effect (output accumulation).
    pub const WRITER: u64 = 3;

    /// Error effect (recoverable errors).
    pub const ERROR: u64 = 4;

    /// Async effect (asynchronous operations).
    pub const ASYNC: u64 = 5;

    /// Random effect (random number generation).
    pub const RANDOM: u64 = 6;

    /// Time effect (clock, timers).
    pub const TIME: u64 = 7;

    /// Resource effect (managed resources).
    pub const RESOURCE: u64 = 8;

    /// Concurrency effect (forking, joining).
    pub const CONCURRENCY: u64 = 9;

    /// First user-defined effect ID.
    pub const USER_START: u64 = 32;
}

// =============================================================================
// Built-in Effects with IDs
// =============================================================================

/// IO effect marker.
pub struct IoEffectV2;

impl EffectId for IoEffectV2 {
    const ID: u64 = builtin_ids::IO;
    const NAME: &'static str = "IO";
}

/// State effect marker.
pub struct StateEffectV2<S>(PhantomData<S>);

impl<S> EffectId for StateEffectV2<S> {
    const ID: u64 = builtin_ids::STATE;
    const NAME: &'static str = "State";
}

/// Reader effect marker.
pub struct ReaderEffectV2<R>(PhantomData<R>);

impl<R> EffectId for ReaderEffectV2<R> {
    const ID: u64 = builtin_ids::READER;
    const NAME: &'static str = "Reader";
}

/// Writer effect marker.
pub struct WriterEffectV2<W>(PhantomData<W>);

impl<W> EffectId for WriterEffectV2<W> {
    const ID: u64 = builtin_ids::WRITER;
    const NAME: &'static str = "Writer";
}

/// Error effect marker.
pub struct ErrorEffectV2<E>(PhantomData<E>);

impl<E> EffectId for ErrorEffectV2<E> {
    const ID: u64 = builtin_ids::ERROR;
    const NAME: &'static str = "Error";
}

/// Async effect marker.
pub struct AsyncEffectV2;

impl EffectId for AsyncEffectV2 {
    const ID: u64 = builtin_ids::ASYNC;
    const NAME: &'static str = "Async";
}

// =============================================================================
// EffectId impls for v1-era common effect types
// =============================================================================
//
// The row v1 migration (v1 → v2 consolidation) requires v1-era common effect
// types such as `IoEffectus` / `StatusEffectus<S>` / etc. to participate in
// the v2 bit-indexed row scheme. Each impl maps the v1 marker to the matching
// built-in effect ID; no additional state or behavior is introduced.

impl EffectId for super::common::IoEffectus {
    const ID: u64 = builtin_ids::IO;
    const NAME: &'static str = "IO";
}

impl<S: Send + Sync + 'static> EffectId for super::common::StatusEffectus<S> {
    const ID: u64 = builtin_ids::STATE;
    const NAME: &'static str = "State";
}

impl<R: Send + Sync + 'static> EffectId for super::common::ReaderEffectus<R> {
    const ID: u64 = builtin_ids::READER;
    const NAME: &'static str = "Reader";
}

impl<W: Send + Sync + 'static> EffectId for super::common::ScriptorEffectus<W> {
    const ID: u64 = builtin_ids::WRITER;
    const NAME: &'static str = "Writer";
}

impl<E: Send + Sync + 'static> EffectId for super::common::ErrorEffectus<E> {
    const ID: u64 = builtin_ids::ERROR;
    const NAME: &'static str = "Error";
}

impl EffectId for super::common::AsyncEffectus {
    const ID: u64 = builtin_ids::ASYNC;
    const NAME: &'static str = "Async";
}

impl EffectId for super::common::RandomEffectus {
    const ID: u64 = builtin_ids::RANDOM;
    const NAME: &'static str = "Random";
}

impl EffectId for super::common::TempusEffectus {
    const ID: u64 = builtin_ids::TIME;
    const NAME: &'static str = "Time";
}

impl<R: Send + Sync + 'static> EffectId for super::common::ResourceEffectus<R> {
    const ID: u64 = builtin_ids::RESOURCE;
    const NAME: &'static str = "Resource";
}

// =============================================================================
// Common Effect Sets
// =============================================================================

/// Effect set containing only IO.
pub type IoRow = EffectSet<{ 1 << builtin_ids::IO }>;

/// Effect set containing only State.
pub type StateRow = EffectSet<{ 1 << builtin_ids::STATE }>;

/// Effect set containing only Reader.
pub type ReaderRow = EffectSet<{ 1 << builtin_ids::READER }>;

/// Effect set containing only Error.
pub type ErrorRow = EffectSet<{ 1 << builtin_ids::ERROR }>;

/// Effect set containing IO and State.
pub type IoStateRow = EffectSet<{ (1 << builtin_ids::IO) | (1 << builtin_ids::STATE) }>;

/// Effect set containing IO, State, and Error.
pub type IoStateErrorRow =
    EffectSet<{ (1 << builtin_ids::IO) | (1 << builtin_ids::STATE) | (1 << builtin_ids::ERROR) }>;

// =============================================================================
// Display Implementation
// =============================================================================

impl<const MASK: u128> core::fmt::Display for EffectSet<MASK> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if MASK == 0 {
            return write!(f, "Pure");
        }

        write!(f, "{{")?;
        let mut first = true;
        for id in Self::iter() {
            if !first {
                write!(f, ", ")?;
            }
            first = false;

            // Try to match known effect names
            match id {
                0 => write!(f, "IO")?,
                1 => write!(f, "State")?,
                2 => write!(f, "Reader")?,
                3 => write!(f, "Writer")?,
                4 => write!(f, "Error")?,
                5 => write!(f, "Async")?,
                6 => write!(f, "Random")?,
                7 => write!(f, "Time")?,
                8 => write!(f, "Resource")?,
                9 => write!(f, "Concurrency")?,
                n => write!(f, "Effect#{n}")?,
            }
        }
        write!(f, "}}")
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effect_set_empty() {
        type Empty = EffectSet<0>;
        assert!(Empty::is_empty());
        assert_eq!(Empty::count(), 0);
    }

    #[test]
    fn test_effect_set_single() {
        type IoOnly = EffectSet<1>;
        assert!(!IoOnly::is_empty());
        assert_eq!(IoOnly::count(), 1);
        assert!(IoOnly::contains(0));
        assert!(!IoOnly::contains(1));
    }

    #[test]
    fn test_effect_set_multiple() {
        type IoAndState = EffectSet<3>;
        assert_eq!(IoAndState::count(), 2);
        assert!(IoAndState::contains(0));
        assert!(IoAndState::contains(1));
        assert!(!IoAndState::contains(2));
    }

    #[test]
    fn test_effect_set_iter() {
        type Effects = EffectSet<{ (1 << 0) | (1 << 2) | (1 << 5) }>;
        let ids: alloc::vec::Vec<_> = Effects::iter().collect();
        assert_eq!(ids, alloc::vec![0, 2, 5]);
    }

    #[test]
    fn test_effect_id_trait() {
        assert_eq!(IoEffectV2::ID, 0);
        assert_eq!(IoEffectV2::NAME, "IO");
        assert_eq!(IoEffectV2::mask(), 1);
    }

    #[test]
    fn test_union() {
        type A = EffectSet<0b0011>; // Effects 0, 1
        type B = EffectSet<0b0110>; // Effects 1, 2
        type Ab = EffectSet<{ union(A::MASK, B::MASK) }>;

        assert_eq!(Ab::MASK, 0b0111); // Effects 0, 1, 2
        assert!(Ab::contains(0));
        assert!(Ab::contains(1));
        assert!(Ab::contains(2));
    }

    #[test]
    fn test_intersection() {
        type A = EffectSet<0b0011>;
        type B = EffectSet<0b0110>;
        type Ab = EffectSet<{ intersection(A::MASK, B::MASK) }>;

        assert_eq!(Ab::MASK, 0b0010); // Only effect 1
        assert!(!Ab::contains(0));
        assert!(Ab::contains(1));
        assert!(!Ab::contains(2));
    }

    #[test]
    fn test_difference() {
        type A = EffectSet<0b0111>; // Effects 0, 1, 2
        type B = EffectSet<0b0011>; // Effects 0, 1
        type Diff = EffectSet<{ difference(A::MASK, B::MASK) }>;

        assert_eq!(Diff::MASK, 0b0100); // Only effect 2
    }

    #[test]
    fn test_extend() {
        type Base = EffectSet<0b0001>;
        type Extended = EffectSet<{ extend(Base::MASK, 2) }>;

        assert_eq!(Extended::MASK, 0b0101);
        assert!(Extended::contains(0));
        assert!(Extended::contains(2));
    }

    #[test]
    fn test_contract() {
        type Base = EffectSet<0b0111>;
        type Contracted = EffectSet<{ contract(Base::MASK, 1) }>;

        assert_eq!(Contracted::MASK, 0b0101);
        assert!(Contracted::contains(0));
        assert!(!Contracted::contains(1));
        assert!(Contracted::contains(2));
    }

    #[test]
    fn test_display() {
        use alloc::format;

        assert_eq!(format!("{}", EffectSet::<0>::new()), "Pure");
        assert_eq!(format!("{}", EffectSet::<1>::new()), "{IO}");
        assert_eq!(format!("{}", EffectSet::<3>::new()), "{IO, State}");
    }

    // Compile-time constraint tests (these just need to compile: the
    // assertions are evaluated post-monomorphization)

    fn requires_io<R: EffectRow>() {
        assert_has_effect::<R, 0>();
    }
    fn requires_state<R: EffectRow>() {
        assert_has_effect::<R, 1>();
    }
    fn requires_io_and_state<R: EffectRow>() {
        assert_has_effect::<R, 0>();
        assert_has_effect::<R, 1>();
    }

    #[test]
    fn test_has_effect_compiles() {
        // These should compile (and assert at monomorphization time)
        requires_io::<EffectSet<1>>();
        requires_io::<EffectSet<3>>();
        requires_state::<EffectSet<2>>();
        requires_state::<EffectSet<3>>();
        requires_io_and_state::<EffectSet<3>>();
    }

    fn requires_subrow<R: EffectRow>() {
        assert_subrow::<R, 0b111>();
    }

    #[test]
    fn test_subrow_compiles() {
        requires_subrow::<EffectSet<0b000>>(); // Empty is subrow of anything
        requires_subrow::<EffectSet<0b001>>(); // Subset
        requires_subrow::<EffectSet<0b011>>(); // Subset
        requires_subrow::<EffectSet<0b111>>(); // Equal
    }

    #[test]
    fn test_without_effect_and_disjoint_compile() {
        assert_without_effect::<EffectSet<0b010>, 0>();
        assert_disjoint::<EffectSet<0b001>, 0b110>();
        assert_has_effect_type::<EffectSet<1>, IoEffectV2>();
    }
}
