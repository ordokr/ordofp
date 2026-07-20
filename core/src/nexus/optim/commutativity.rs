//! Effect Commutativity
//!
//! This module defines when effects can be safely reordered. Two effects
//! commute if executing them in either order produces the same result.
//!
//! # Commutativity Rules
//!
//! ```text
//! Effect 1    | Effect 2    | Commutes? | Reason
//! ------------|-------------|-----------|--------
//! Reader<A>   | Reader<B>   | Yes       | Both read-only
//! Reader<A>   | Writer<B>   | Yes       | Read doesn't see write
//! Reader<A>   | State<S>    | No        | State may change env
//! Reader<A>   | Error<E>    | Yes       | Error doesn't affect read
//! Writer<A>   | Writer<B>   | Yes*      | If monoid is commutative
//! State<S>    | State<S>    | No        | Order matters for same state
//! State<S>    | State<T>    | Yes       | Different state types
//! Error<E>    | Error<E>    | No        | First error wins
//! IO          | IO          | No        | Side effects don't commute
//! Pure        | Anything    | Yes       | Pure has no effects
//! ```
//!
//! # Usage
//!
//! Effect commutativity is used by **explicit optimization combinators** to:
//! - Validate that operations can be safely reordered
//! - Type-check parallel execution (`par_both`, etc.)
//! - Enforce fusion safety (`fuse_commutative`)
//!
//! **Note:** There is no automatic optimizer. Users must explicitly call
//! combinators that leverage commutativity.

use core::marker::PhantomData;

use crate::nexus::effect::{Error, Reader, State};
use crate::nexus::effects::writer::WriterEffect;
use crate::nexus::row::{EffectRow, Pure};

// =============================================================================
// Commutativity Marker Traits
// =============================================================================

/// Marker for commutative relationships.
pub struct Commutative;

/// Marker for non-commutative relationships.
pub struct NonCommutative;

// =============================================================================
// Effect Commutativity Trait
// =============================================================================

/// Trait indicating that two effects commute.
///
/// If `E1: EffectCommutes<E2>`, then computations with effect E1 can be
/// reordered with computations with effect E2 without changing semantics.
///
/// # Safety
///
/// Incorrectly implementing this trait can lead to incorrect program behavior.
/// Only implement this if you're certain the effects truly commute.
///
/// # Example
///
/// ```rust
/// use ordofp_core::nexus::Reader;
/// use ordofp_core::nexus::optim::commutativity::{Commutative, EffectCommutes};
///
/// // Reader effects commute with each other (already proven by the crate: see
/// // `impl<A, B> EffectCommutes<Reader<B>> for Reader<A>`).
/// fn assert_reader_commutes<A, B>()
/// where
///     Reader<A>: EffectCommutes<Reader<B>, Witness = Commutative>,
/// {
/// }
///
/// // This enables parallel execution via combinators like `par_both`:
/// assert_reader_commutes::<i32, &str>();
/// ```
pub trait EffectCommutes<E2> {
    /// Witness type proving commutativity.
    type Witness;
}

// =============================================================================
// Pure Commutes with Everything
// =============================================================================

/// Pure effects commute with any effect.
impl<E> EffectCommutes<E> for Pure {
    type Witness = Commutative;
}

// =============================================================================
// Reader Commutativity
// =============================================================================

/// Reader effects commute with each other (read-only).
impl<A, B> EffectCommutes<Reader<B>> for Reader<A> {
    type Witness = Commutative;
}

/// Reader commutes with Error (error doesn't affect reading).
impl<A, E> EffectCommutes<Error<E>> for Reader<A> {
    type Witness = Commutative;
}

/// Reader commutes with Writer (read doesn't see concurrent write).
impl<A, W> EffectCommutes<WriterEffect<W>> for Reader<A> {
    type Witness = Commutative;
}

// =============================================================================
// Error Commutativity
// =============================================================================

/// Error commutes with Reader.
impl<E, A> EffectCommutes<Reader<A>> for Error<E> {
    type Witness = Commutative;
}

// Note: Error<E> does NOT commute with Error<E> because the first error wins.

// =============================================================================
// Writer Commutativity
// =============================================================================

/// Writer commutes with Reader.
impl<W, A> EffectCommutes<Reader<A>> for WriterEffect<W> {
    type Witness = Commutative;
}

// Note: Writer<W> commutes with Writer<W> only if W's monoid is commutative.
// This requires additional type-level evidence.

// =============================================================================
// Commutativity Proofs
// =============================================================================

/// Type-level proof that two effect rows commute.
pub struct RowsCommute<R1: EffectRow, R2: EffectRow> {
    _marker: PhantomData<(R1, R2)>,
}

impl<R1: EffectRow, R2: EffectRow> RowsCommute<R1, R2> {
    /// Create a commutativity proof.
    ///
    /// # Safety
    ///
    /// This should only be constructed when the rows actually commute.
    #[inline]
    pub const unsafe fn new() -> Self {
        RowsCommute {
            _marker: PhantomData,
        }
    }
}

/// Check if two effect rows commute at compile time.
pub trait RowCommutes<R2: EffectRow>: EffectRow {
    /// Whether the rows commute.
    const COMMUTES: bool;
}

/// Pure row commutes with everything.
impl<R2: EffectRow> RowCommutes<R2> for Pure {
    const COMMUTES: bool = true;
}

// =============================================================================
// Commutativity Combinators
// =============================================================================

/// Swap the order of two computations if they commute.
///
/// This is the foundation for parallel execution and reordering optimizations.
///
/// # Example
///
/// ```rust
/// use ordofp_core::nexus::optim::commutativity::{swap_if_commutes, RowsCommute};
/// use ordofp_core::nexus::Pure;
///
/// // SAFETY: two `Pure` rows trivially commute (no ordering constraints).
/// let proof = unsafe { RowsCommute::<Pure, Pure>::new() };
///
/// // These two values (standing in for two commuting computations) can be swapped
/// let (b, a) = swap_if_commutes(proof, 1, 2);
/// assert_eq!(a, 1);
/// assert_eq!(b, 2);
/// ```
#[inline]
pub fn swap_if_commutes<R1, R2, A, B>(_proof: RowsCommute<R1, R2>, first: A, second: B) -> (B, A)
where
    R1: EffectRow,
    R2: EffectRow,
{
    (second, first)
}

// =============================================================================
// Effect Independence
// =============================================================================

/// Two effects are independent if they access disjoint resources.
///
/// Independent effects can always be executed in parallel, even if they
/// don't strictly commute (e.g., two State effects on different state types).
pub trait EffectIndependent<E2> {
    /// Evidence of independence.
    type Evidence;
}

/// Different state types are independent.
impl<S1, S2> EffectIndependent<State<S2>> for State<S1>
where
    S1: 'static,
    S2: 'static,
{
    type Evidence = ();
}

/// Reader and State with different types are independent.
impl<E, S> EffectIndependent<State<S>> for Reader<E>
where
    E: 'static,
    S: 'static,
{
    type Evidence = ();
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pure_commutes_with_reader() {
        // Type-level test: this should compile
        fn _pure_commutes_with_reader<A>()
        where
            Pure: EffectCommutes<Reader<A>>,
        {
        }
    }

    #[test]
    fn test_reader_commutes_with_reader() {
        // Type-level test: this should compile
        fn _readers_commute<A, B>()
        where
            Reader<A>: EffectCommutes<Reader<B>>,
        {
        }
    }

    #[test]
    fn test_reader_commutes_with_error() {
        // Type-level test: this should compile
        fn _reader_error_commute<A, E>()
        where
            Reader<A>: EffectCommutes<Error<E>>,
            Error<E>: EffectCommutes<Reader<A>>,
        {
        }
    }

    #[test]
    fn test_swap_operation() {
        // SAFETY: `Pure` effects carry no side effects and impose no ordering
        // constraints, so two `Pure` rows trivially commute. The `RowsCommute`
        // invariant ("only constructed when the rows actually commute") is
        // unconditionally satisfied for `Pure × Pure`.
        let proof = unsafe { RowsCommute::<Pure, Pure>::new() };
        let (b, a) = swap_if_commutes(proof, 1, 2);
        assert_eq!(a, 1);
        assert_eq!(b, 2);
    }
}
