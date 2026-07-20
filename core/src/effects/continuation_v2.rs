//! Multiplicity-Aware Continuations
//!
//! > *"Continuatio cum multiplicitate"*
//! > — Continuation with multiplicity. (Neo-Latin)
//!
//! Extends the one-shot continuation system with multi-shot continuations
//! via explicit multiplicity tracking. Multi-shot continuations are the
//! mechanism behind backtracking and probabilistic choice; the multiplicity
//! parameter keeps one-shot and multi-shot uses distinct at the type level.
//!
//! # Multiplicity Modes
//!
//! | Latin | English | Symbol | Meaning |
//! |-------|---------|--------|---------|
//! | Semel | Once | 1 | Linear: use exactly once |
//! | Affinis | At most once | ≤1 | Affine: use zero or once |
//! | Pluries | Many | ω | Unrestricted: use any times |
//! | Nullus | Never | 0 | Relevant: must not use |
//!
//! # Safety Model
//!
//! Multi-shot continuations require `Clone` on captured state. Only
//! `ContinuatioPluries` implements `Clone`, ensuring type-safe multi-shot
//! semantics at compile time.
//!
//! # Examples
//!
//! ```rust
//! use ordofp_core::effects::continuation_v2::{Continuatio, ContinuatioPluries};
//!
//! // Multi-shot continuation for backtracking
//! let cont: ContinuatioPluries<i32, i32> = Continuatio::pluries(|x: i32| x * 2);
//! let result1 = cont.clone().resume(21);  // 42
//! let result2 = cont.resume(10);          // 20
//! assert_eq!(result1, 42);
//! assert_eq!(result2, 20);
//! ```

use alloc::boxed::Box;
use alloc::sync::Arc;
use core::marker::PhantomData;

use super::Effectus;
use crate::quantitative::{Multiplicitas, Omega, Usage};

// Re-export multiplicity markers so `continuation_v2` is a single
// import surface for continuation-related types.
pub use crate::quantitative::Semel;

// =============================================================================
// Affinis - Affine Multiplicity Marker
// =============================================================================

/// Type-level marker for affine multiplicity (≤1).
///
/// > *"Affinis" - related, akin*
///
/// Affine values can be used at most once (zero or one times).
/// This matches Rust's default ownership semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Affinis;

impl Usage for Affinis {
    const VALUE: Multiplicitas = Multiplicitas::Semel; // Approximated as linear
    const ALLOWS_DISCARD: bool = true; // Can drop without use
    const ALLOWS_DUP: bool = false; // Cannot duplicate
}

/// Type-level marker for relevant multiplicity (must use at least once).
///
/// > *"Nullus" - none*
///
/// Relevant values must be used at least once but cannot be duplicated.
/// The continuation cannot be resumed (it's "dead").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Nullus;

impl Usage for Nullus {
    const VALUE: Multiplicitas = Multiplicitas::Nihil;
    const ALLOWS_DISCARD: bool = false;
    const ALLOWS_DUP: bool = false;
}

/// Type-level marker for unrestricted multiplicity (alias for Omega).
///
/// > *"Pluries" - many times*
///
/// Can be used any number of times including zero.
pub type Pluries = Omega;

// =============================================================================
// Continuatio - Multiplicity-Parameterized Continuation
// =============================================================================

/// A continuation parameterized by its usage multiplicity.
///
/// `Continuatio<A, B, M>` represents a suspended computation that:
/// - Takes a value of type `A` as input
/// - Produces a value of type `B` as output
/// - Has multiplicity `M` determining how many times it can be resumed
///
/// # Type Parameters
///
/// - `A`: Input type for resumption
/// - `B`: Output type after resumption
/// - `M`: Multiplicity marker (`Semel`, `Affinis`, `Pluries`, `Nullus`)
///
/// # Examples
///
/// ```rust
/// use ordofp_core::effects::continuation_v2::{Continuatio, Semel, Pluries};
///
/// // One-shot continuation
/// let linear: Continuatio<i32, i32, Semel> = Continuatio::semel(|x| x * 2);
/// assert_eq!(linear.resume(21), 42);
///
/// // Multi-shot continuation
/// let multi: Continuatio<i32, i32, Pluries> = Continuatio::pluries(|x| x * 2);
/// assert_eq!(multi.resume(21), 42);
/// ```
pub struct Continuatio<A, B, M: Usage> {
    inner: ContinuatioInner<A, B>,
    _multiplicity: PhantomData<M>,
}

/// Internal representation of continuation, independent of multiplicity.
enum ContinuatioInner<A, B> {
    /// One-shot: uses `FnOnce`
    Once(Box<dyn FnOnce(A) -> B + Send>),
    /// Multi-shot: uses Arc<Fn>
    Multi(Arc<dyn Fn(A) -> B + Send + Sync>),
}

impl<A: 'static, B: 'static> Continuatio<A, B, Semel> {
    /// Create a linear (one-shot) continuation.
    ///
    /// This continuation must be resumed exactly once.
    #[inline]
    pub fn semel<F>(f: F) -> Self
    where
        F: FnOnce(A) -> B + Send + 'static,
    {
        Continuatio {
            inner: ContinuatioInner::Once(Box::new(f)),
            _multiplicity: PhantomData,
        }
    }

    /// Create a linear (one-shot) continuation.
    ///
    /// Compatibility alias for [`Continuatio::semel`], preserving the
    /// constructor name used by the legacy `ContinuatioSemel` API.
    #[inline]
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce(A) -> B + Send + 'static,
    {
        Self::semel(f)
    }

    /// Resume the continuation with a value.
    ///
    /// Consumes the continuation (enforcing one-shot semantics).
    #[inline]
    pub fn resume(self, value: A) -> B {
        match self.inner {
            ContinuatioInner::Once(f) => f(value),
            ContinuatioInner::Multi(f) => f(value),
        }
    }
}

impl<A: 'static, B: 'static> Continuatio<A, B, Affinis> {
    /// Create an affine continuation.
    ///
    /// This continuation can be resumed at most once (may be dropped).
    #[inline]
    pub fn affinis<F>(f: F) -> Self
    where
        F: FnOnce(A) -> B + Send + 'static,
    {
        Continuatio {
            inner: ContinuatioInner::Once(Box::new(f)),
            _multiplicity: PhantomData,
        }
    }

    /// Resume the continuation with a value.
    ///
    /// Consumes the continuation.
    #[inline]
    pub fn resume(self, value: A) -> B {
        match self.inner {
            ContinuatioInner::Once(f) => f(value),
            ContinuatioInner::Multi(f) => f(value),
        }
    }

    /// Discard the continuation without resuming.
    ///
    /// Affine continuations allow this.
    #[inline]
    pub fn discard(self) {
        // Dropping self here is implicit — the method body is intentionally
        // empty. Rust drops `self` at end-of-scope automatically.
    }
}

impl<A: 'static, B: 'static> Continuatio<A, B, Pluries> {
    /// Create an unrestricted (multi-shot) continuation.
    ///
    /// This continuation can be cloned and resumed any number of times.
    #[inline]
    pub fn pluries<F>(f: F) -> Self
    where
        F: Fn(A) -> B + Send + Sync + 'static,
    {
        Continuatio {
            inner: ContinuatioInner::Multi(Arc::new(f)),
            _multiplicity: PhantomData,
        }
    }

    /// Resume the continuation with a value.
    ///
    /// Does not consume the continuation (can resume again after cloning).
    ///
    /// # Panics
    ///
    /// Panics only if a `Pluries`-typed continuation holds a one-shot inner
    /// representation, which the constructors make impossible — such a panic
    /// indicates a bug in this crate.
    #[inline]
    pub fn resume(&self, value: A) -> B {
        match &self.inner {
            ContinuatioInner::Multi(f) => f(value),
            ContinuatioInner::Once(_) => {
                panic!("Pluries continuation must use Multi inner")
            }
        }
    }

    /// Resume and consume (for final use).
    #[inline]
    pub fn resume_final(self, value: A) -> B {
        match self.inner {
            ContinuatioInner::Multi(f) => f(value),
            ContinuatioInner::Once(f) => f(value),
        }
    }
}

// Multi-shot continuations can be cloned
impl<A, B> Clone for Continuatio<A, B, Pluries> {
    #[inline]
    fn clone(&self) -> Self {
        match &self.inner {
            ContinuatioInner::Multi(arc) => Continuatio {
                inner: ContinuatioInner::Multi(Arc::clone(arc)),
                _multiplicity: PhantomData,
            },
            ContinuatioInner::Once(_) => {
                panic!("Cannot clone Once continuation as Pluries")
            }
        }
    }
}

// =============================================================================
// Convenience Type Aliases
// =============================================================================

/// One-shot continuation (linear).
pub type ContinuatioSemel<A, B> = Continuatio<A, B, Semel>;

/// Affine continuation (at most once).
pub type ContinuatioAffinis<A, B> = Continuatio<A, B, Affinis>;

/// Multi-shot continuation (unrestricted).
pub type ContinuatioPluries<A, B> = Continuatio<A, B, Pluries>;

// =============================================================================
// Multiplicity Conversion
// =============================================================================

impl<A: 'static, B: 'static> Continuatio<A, B, Semel> {
    /// Weaken a linear continuation to affine.
    ///
    /// A linear continuation can always be treated as affine.
    #[inline]
    pub fn to_affinis(self) -> Continuatio<A, B, Affinis> {
        Continuatio {
            inner: self.inner,
            _multiplicity: PhantomData,
        }
    }
}

// =============================================================================
// Continuation Combinators
// =============================================================================

impl<A: 'static, B: 'static, M: Usage> Continuatio<A, B, M> {
    /// Map over the output of the continuation.
    ///
    /// Transforms `Continuatio<A, B, M>` to `Continuatio<A, C, M>`.
    ///
    /// The closure must be `Clone + Sync` so that a multi-shot (`Pluries`)
    /// continuation keeps its repeatedly-callable `Multi` representation:
    /// each resumption clones `f` and consumes the clone.
    #[inline]
    pub fn map<C: 'static, F>(self, f: F) -> Continuatio<A, C, M>
    where
        F: FnOnce(B) -> C + Clone + Send + Sync + 'static,
    {
        let mapped = match self.inner {
            ContinuatioInner::Once(inner) => ContinuatioInner::Once(Box::new(move |a| f(inner(a)))),
            ContinuatioInner::Multi(inner) => {
                // Preserve the multi-shot representation: clone f per call so
                // the wrapper stays callable repeatedly (required for Pluries
                // resume(&self) and clone()).
                ContinuatioInner::Multi(Arc::new(move |a| f.clone()(inner(a))))
            }
        };
        Continuatio {
            inner: mapped,
            _multiplicity: PhantomData,
        }
    }

    /// Pre-compose with a function on the input.
    ///
    /// Transforms `Continuatio<A, B, M>` to `Continuatio<C, B, M>`.
    ///
    /// The closure must be `Clone + Sync` so that a multi-shot (`Pluries`)
    /// continuation keeps its repeatedly-callable `Multi` representation:
    /// each resumption clones `f` and consumes the clone.
    #[inline]
    pub fn contramap<C: 'static, F>(self, f: F) -> Continuatio<C, B, M>
    where
        F: FnOnce(C) -> A + Clone + Send + Sync + 'static,
    {
        let contramapped = match self.inner {
            ContinuatioInner::Once(inner) => ContinuatioInner::Once(Box::new(move |c| inner(f(c)))),
            ContinuatioInner::Multi(inner) => {
                // Preserve the multi-shot representation (see map above).
                ContinuatioInner::Multi(Arc::new(move |c| inner(f.clone()(c))))
            }
        };
        Continuatio {
            inner: contramapped,
            _multiplicity: PhantomData,
        }
    }
}

impl<A: 'static, B: 'static> Continuatio<A, B, Pluries> {
    /// Map with a clonable function (preserves multi-shot).
    ///
    /// # Panics
    ///
    /// Panics only if a `Pluries`-typed continuation holds a one-shot inner
    /// representation, which the constructors make impossible — such a panic
    /// indicates a bug in this crate.
    #[inline]
    pub fn map_multi<C: 'static, F>(self, f: F) -> Continuatio<A, C, Pluries>
    where
        F: Fn(B) -> C + Send + Sync + 'static,
    {
        match self.inner {
            ContinuatioInner::Multi(inner) => Continuatio {
                inner: ContinuatioInner::Multi(Arc::new(move |a| f(inner(a)))),
                _multiplicity: PhantomData,
            },
            ContinuatioInner::Once(_) => panic!("Expected Multi inner"),
        }
    }

    /// Pre-compose with a clonable function (preserves multi-shot).
    ///
    /// # Panics
    ///
    /// Panics only if a `Pluries`-typed continuation holds a one-shot inner
    /// representation, which the constructors make impossible — such a panic
    /// indicates a bug in this crate.
    #[inline]
    pub fn contramap_multi<C: 'static, F>(self, f: F) -> Continuatio<C, B, Pluries>
    where
        F: Fn(C) -> A + Send + Sync + 'static,
    {
        match self.inner {
            ContinuatioInner::Multi(inner) => Continuatio {
                inner: ContinuatioInner::Multi(Arc::new(move |c| inner(f(c)))),
                _multiplicity: PhantomData,
            },
            ContinuatioInner::Once(_) => panic!("Expected Multi inner"),
        }
    }
}

// =============================================================================
// Effect Handler Result with Multiplicity
// =============================================================================

/// Result of handling an effect, parameterized by continuation multiplicity.
///
/// This extends `TractatorResult` to track whether the continuation
/// is one-shot or multi-shot.
pub enum TractatorResultMulti<E: Effectus, A, B, M: Usage> {
    /// Computation completed with a value.
    Complete(A),

    /// Computation suspended, waiting for effect to be handled.
    Suspended {
        /// The effect that needs to be handled.
        effect: E,
        /// The continuation to resume after handling.
        continuation: Continuatio<B, A, M>,
    },
}

impl<E: Effectus, A, B, M: Usage> TractatorResultMulti<E, A, B, M> {
    /// Create a completed result.
    #[inline]
    pub fn complete(value: A) -> Self {
        TractatorResultMulti::Complete(value)
    }

    /// Check if the result is complete.
    #[inline]
    pub fn is_complete(&self) -> bool {
        matches!(self, TractatorResultMulti::Complete(_))
    }

    /// Check if the result is suspended.
    #[inline]
    pub fn is_suspended(&self) -> bool {
        matches!(self, TractatorResultMulti::Suspended { .. })
    }
}

impl<E: Effectus, A: 'static, B: 'static> TractatorResultMulti<E, A, B, Semel> {
    /// Create a suspended result with a one-shot continuation.
    #[inline]
    pub fn suspended_semel<F>(effect: E, f: F) -> Self
    where
        F: FnOnce(B) -> A + Send + 'static,
    {
        TractatorResultMulti::Suspended {
            effect,
            continuation: Continuatio::semel(f),
        }
    }
}

impl<E: Effectus, A: 'static, B: 'static> TractatorResultMulti<E, A, B, Pluries> {
    /// Create a suspended result with a multi-shot continuation.
    #[inline]
    pub fn suspended_pluries<F>(effect: E, f: F) -> Self
    where
        F: Fn(B) -> A + Send + Sync + 'static,
    {
        TractatorResultMulti::Suspended {
            effect,
            continuation: Continuatio::pluries(f),
        }
    }
}

// =============================================================================
// Backtracking Support
// =============================================================================

/// A choice point for backtracking computations.
///
/// Stores a multi-shot continuation that can be resumed multiple times
/// to explore different branches.
pub struct ChoicePoint<A, B> {
    continuation: ContinuatioPluries<A, B>,
    choices: alloc::vec::Vec<A>,
}

impl<A: Clone + 'static, B: 'static> ChoicePoint<A, B> {
    /// Create a new choice point.
    #[inline]
    pub fn new<F>(choices: alloc::vec::Vec<A>, f: F) -> Self
    where
        F: Fn(A) -> B + Send + Sync + 'static,
    {
        ChoicePoint {
            continuation: Continuatio::pluries(f),
            choices,
        }
    }

    /// Explore all choices, collecting results.
    pub fn explore_all(self) -> alloc::vec::Vec<B> {
        self.choices
            .into_iter()
            .map(|choice| self.continuation.resume(choice))
            .collect()
    }

    /// Explore choices until one succeeds (returns Some).
    pub fn find_first<F>(self, mut predicate: F) -> Option<B>
    where
        F: FnMut(&B) -> bool,
    {
        for choice in self.choices {
            let result = self.continuation.resume(choice);
            if predicate(&result) {
                return Some(result);
            }
        }
        None
    }
}

// =============================================================================
// Legacy v1 API (preserved for compatibility after continuation.rs removal)
// =============================================================================

/// Trait for effect handlers with one-shot continuation support.
///
/// `TractatorContinuatio<E>` handles effects of type `E` with access to
/// a one-shot continuation, allowing the handler to resume computation
/// or abort.
///
/// This trait is preserved from the former `effects::continuation` (v1)
/// module and now delegates to `Continuatio<_, _, Semel>`.
pub trait TractatorContinuatio<E: Effectus> {
    /// The type passed to the continuation when resuming.
    type Input;

    /// The final output type of the handled computation.
    type Output;

    /// Handle an effect with access to a one-shot continuation.
    fn handle_with_continuation(
        &self,
        effect: E,
        cont: ContinuatioSemel<Self::Input, Self::Output>,
    ) -> Self::Output;
}

/// Result of handling an effect with a one-shot continuation.
///
/// Either the computation is complete, or it's suspended waiting for an
/// effect to be handled. Preserved from the v1 `effects::continuation`
/// module for backward compatibility.
pub enum TractatorResult<E: Effectus, A, B> {
    /// Computation completed with a value.
    Complete(A),

    /// Computation suspended, waiting for effect to be handled.
    Suspended {
        /// The effect that needs to be handled.
        effect: E,
        /// The continuation to resume after handling.
        continuation: ContinuatioSemel<B, A>,
    },
}

impl<E: Effectus, A, B> TractatorResult<E, A, B> {
    /// Create a completed result.
    #[inline]
    pub fn complete(value: A) -> Self {
        TractatorResult::Complete(value)
    }

    /// Create a suspended result.
    #[inline]
    pub fn suspended(effect: E, continuation: ContinuatioSemel<B, A>) -> Self {
        TractatorResult::Suspended {
            effect,
            continuation,
        }
    }

    /// Check if the result is complete.
    #[inline]
    pub fn is_complete(&self) -> bool {
        matches!(self, TractatorResult::Complete(_))
    }

    /// Check if the result is suspended.
    #[inline]
    pub fn is_suspended(&self) -> bool {
        matches!(self, TractatorResult::Suspended { .. })
    }

    /// Map over a completed result.
    pub fn map<C: 'static, F>(self, f: F) -> TractatorResult<E, C, B>
    where
        F: FnOnce(A) -> C + Clone + Send + Sync + 'static,
        A: 'static,
        B: 'static,
    {
        match self {
            TractatorResult::Complete(a) => TractatorResult::Complete(f(a)),
            TractatorResult::Suspended {
                effect,
                continuation,
            } => TractatorResult::Suspended {
                effect,
                continuation: continuation.map(f),
            },
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_continuatio_semel() {
        let cont: ContinuatioSemel<i32, i32> = Continuatio::semel(|x| x * 2);
        let result = cont.resume(21);
        assert_eq!(result, 42);
    }

    #[test]
    fn test_continuatio_affinis() {
        let cont: ContinuatioAffinis<i32, i32> = Continuatio::affinis(|x| x * 2);
        let result = cont.resume(21);
        assert_eq!(result, 42);
    }

    #[test]
    fn test_continuatio_affinis_discard() {
        let cont: ContinuatioAffinis<i32, i32> = Continuatio::affinis(|x| x * 2);
        cont.discard(); // Should not panic
    }

    #[test]
    fn test_continuatio_pluries() {
        let cont: ContinuatioPluries<i32, i32> = Continuatio::pluries(|x| x * 2);
        let result1 = cont.resume(21);
        let result2 = cont.resume(10);
        assert_eq!(result1, 42);
        assert_eq!(result2, 20);
    }

    #[test]
    fn test_continuatio_pluries_clone() {
        let cont1: ContinuatioPluries<i32, i32> = Continuatio::pluries(|x| x * 2);
        let cont2 = cont1.clone();

        let result1 = cont1.resume(21);
        let result2 = cont2.resume(10);

        assert_eq!(result1, 42);
        assert_eq!(result2, 20);
    }

    #[test]
    fn test_semel_to_affinis() {
        let semel: ContinuatioSemel<i32, i32> = Continuatio::semel(|x| x * 2);
        let affinis: ContinuatioAffinis<i32, i32> = semel.to_affinis();
        let result = affinis.resume(21);
        assert_eq!(result, 42);
    }

    #[test]
    fn test_map() {
        let cont: ContinuatioSemel<i32, i32> = Continuatio::semel(|x| x * 2);
        let mapped = cont.map(|y| y + 1);
        let result = mapped.resume(21);
        assert_eq!(result, 43); // (21 * 2) + 1
    }

    #[test]
    fn test_contramap() {
        let cont: ContinuatioSemel<i32, i32> = Continuatio::semel(|x| x * 2);
        let contramapped = cont.contramap(|s: &str| {
            s.parse::<i32>()
                .expect("test input should be a valid integer")
        });
        let result = contramapped.resume("21");
        assert_eq!(result, 42);
    }

    #[test]
    fn test_pluries_map_preserves_multi() {
        // Regression: map on a Pluries continuation must preserve the
        // multi-shot (Multi) representation, so resume(&self) and clone()
        // keep working instead of panicking.
        let cont: ContinuatioPluries<i32, i32> = Continuatio::pluries(|x| x * 2);
        let mapped = cont.map(|y| y + 1);

        let result1 = mapped.resume(21);
        let result2 = mapped.resume(10);
        assert_eq!(result1, 43); // (21 * 2) + 1
        assert_eq!(result2, 21); // (10 * 2) + 1

        let cloned = mapped.clone();
        assert_eq!(cloned.resume(0), 1);
    }

    #[test]
    fn test_pluries_contramap_preserves_multi() {
        // Regression: contramap on a Pluries continuation must preserve the
        // multi-shot (Multi) representation.
        let cont: ContinuatioPluries<i32, i32> = Continuatio::pluries(|x| x * 2);
        let pre = cont.contramap(|c: i32| c + 1);

        assert_eq!(pre.resume(20), 42); // (20 + 1) * 2
        assert_eq!(pre.resume(0), 2); // (0 + 1) * 2

        let cloned = pre.clone();
        assert_eq!(cloned.resume(4), 10);
    }

    #[test]
    fn test_pluries_map_multi() {
        let cont: ContinuatioPluries<i32, i32> = Continuatio::pluries(|x| x * 2);
        let mapped = cont.map_multi(|y| y + 1);

        let result1 = mapped.resume(21);
        let result2 = mapped.resume(10);

        assert_eq!(result1, 43); // (21 * 2) + 1
        assert_eq!(result2, 21); // (10 * 2) + 1
    }

    #[test]
    fn test_choice_point_explore_all() {
        let cp = ChoicePoint::new(vec![1, 2, 3], |x| x * 2);
        let results = cp.explore_all();
        assert_eq!(results, vec![2, 4, 6]);
    }

    #[test]
    fn test_choice_point_find_first() {
        let cp = ChoicePoint::new(vec![1, 2, 3, 4, 5], |x| x * 2);
        let result = cp.find_first(|&x| x > 5);
        assert_eq!(result, Some(6)); // First value > 5 is 3*2=6
    }

    #[test]
    fn test_tractator_result_multi_complete() {
        struct TestEffect;
        impl Effectus for TestEffect {}

        let result: TractatorResultMulti<TestEffect, i32, i32, Semel> =
            TractatorResultMulti::complete(42);
        assert!(result.is_complete());
        assert!(!result.is_suspended());
    }

    #[test]
    fn test_tractator_result_multi_suspended() {
        struct TestEffect;
        impl Effectus for TestEffect {}

        let result: TractatorResultMulti<TestEffect, i32, i32, Semel> =
            TractatorResultMulti::suspended_semel(TestEffect, |x| x * 2);

        assert!(result.is_suspended());
        match result {
            TractatorResultMulti::Suspended { continuation, .. } => {
                assert_eq!(continuation.resume(21), 42);
            }
            _ => panic!("Expected Suspended"),
        }
    }
}
