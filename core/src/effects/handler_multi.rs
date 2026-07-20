//! Multiplicity-Aware Effect Handlers
//!
//! > *"Tractator cum multiplicitate"*
//! > — Handler with multiplicity. (Neo-Latin)
//!
//! This module provides effect handlers that work with multiplicity-aware
//! continuations, enabling effects like backtracking, probabilistic choice,
//! and non-determinism.
//!
//! # Handler Types
//!
//! | Handler | Multiplicity | Use Case |
//! |---------|--------------|----------|
//! | `TractatorSemel` | Linear (1) | Standard effects |
//! | `TractatorAffinis` | Affine (≤1) | Abortable effects |
//! | `TractatorPluries` | Multi (ω) | Backtracking, choice |
//!
//! # Examples
//!
//! ```rust
//! use ordofp_core::effects::continuation_v2::Continuatio;
//! use ordofp_core::effects::handler_multi::{ElectioEffect, ElectioHandler, TractatorMulti};
//!
//! // Non-deterministic choice handler: explores all choices via a
//! // multi-shot continuation and collects every result.
//! let mut handler = ElectioHandler::<i32>::new();
//! let effect = ElectioEffect::new(vec![1, 2, 3]);
//! let cont = Continuatio::pluries(|x: i32| vec![x * 2]);
//! let results = handler.handle_operation(effect, cont);
//! assert_eq!(results, vec![2, 4, 6]);
//! ```

use alloc::vec::Vec;
use core::marker::PhantomData;

use super::Effectus;
use super::continuation_v2::{
    Affinis, Continuatio, ContinuatioAffinis, ContinuatioPluries, ContinuatioSemel,
};
use crate::quantitative::{Omega, Semel, Usage};

// =============================================================================
// TractatorMulti Trait - Core Multi-Shot Handler
// =============================================================================

/// Trait for multiplicity-aware effect handlers.
///
/// Unlike `TractatorContinuatio` which only supports one-shot continuations,
/// `TractatorMulti` can work with any multiplicity, enabling multi-shot
/// semantics for effects like choice and backtracking.
///
/// # Type Parameters
///
/// - `E`: The effect type being handled
/// - `M`: The multiplicity of the continuation
pub trait TractatorMulti<E: Effectus, M: Usage> {
    /// The type the continuation expects when resumed.
    type Input;

    /// The final output type of the handled computation.
    type Output;

    /// Handle an effect operation with a multiplicity-aware continuation.
    ///
    /// The handler can:
    /// - Resume the continuation with a value (potentially multiple times for `Pluries`)
    /// - Abort by returning without resuming (for `Affinis` or returning early)
    /// - Clone and fork for multi-shot (only for `Pluries`)
    fn handle_operation(
        &mut self,
        effect: E,
        continuation: Continuatio<Self::Input, Self::Output, M>,
    ) -> Self::Output;
}

// =============================================================================
// Standard Handlers for Each Multiplicity
// =============================================================================

/// A handler that always resumes with a default value.
///
/// Works with any multiplicity.
pub struct DefaultMultiHandler<A, M: Usage> {
    default: A,
    _multiplicity: PhantomData<M>,
}

impl<A: Clone + Send + 'static, M: Usage> DefaultMultiHandler<A, M> {
    /// Create a handler that always resumes with the given value.
    pub fn new(default: A) -> Self {
        DefaultMultiHandler {
            default,
            _multiplicity: PhantomData,
        }
    }
}

impl<E: Effectus, A: Clone + Send + 'static> TractatorMulti<E, Semel>
    for DefaultMultiHandler<A, Semel>
{
    type Input = A;
    type Output = A;

    #[inline]
    fn handle_operation(
        &mut self,
        _effect: E,
        continuation: ContinuatioSemel<Self::Input, Self::Output>,
    ) -> Self::Output {
        continuation.resume(self.default.clone())
    }
}

impl<E: Effectus, A: Clone + Send + 'static> TractatorMulti<E, Affinis>
    for DefaultMultiHandler<A, Affinis>
{
    type Input = A;
    type Output = A;

    #[inline]
    fn handle_operation(
        &mut self,
        _effect: E,
        continuation: ContinuatioAffinis<Self::Input, Self::Output>,
    ) -> Self::Output {
        continuation.resume(self.default.clone())
    }
}

impl<E: Effectus, A: Clone + Send + 'static> TractatorMulti<E, Omega>
    for DefaultMultiHandler<A, Omega>
{
    type Input = A;
    type Output = A;

    #[inline]
    fn handle_operation(
        &mut self,
        _effect: E,
        continuation: ContinuatioPluries<Self::Input, Self::Output>,
    ) -> Self::Output {
        continuation.resume(self.default.clone())
    }
}

/// A handler that aborts with a fixed value (never resumes).
pub struct AbortMultiHandler<A, B, M: Usage> {
    value: A,
    _input: PhantomData<B>,
    _multiplicity: PhantomData<M>,
}

impl<A: Clone, B, M: Usage> AbortMultiHandler<A, B, M> {
    /// Create a handler that always aborts with the given value.
    pub fn new(value: A) -> Self {
        AbortMultiHandler {
            value,
            _input: PhantomData,
            _multiplicity: PhantomData,
        }
    }
}

impl<E: Effectus, A: Clone + 'static, B: 'static> TractatorMulti<E, Affinis>
    for AbortMultiHandler<A, B, Affinis>
{
    type Input = B;
    type Output = A;

    #[inline]
    fn handle_operation(
        &mut self,
        _effect: E,
        continuation: ContinuatioAffinis<Self::Input, Self::Output>,
    ) -> Self::Output {
        continuation.discard();
        self.value.clone()
    }
}

// =============================================================================
// Choice Effect - Non-Determinism
// =============================================================================

/// The choice effect for non-deterministic computation.
///
/// > *"Electio" - choice*
///
/// This effect allows a computation to non-deterministically choose
/// from a set of alternatives.
#[derive(Debug, Clone)]
pub struct ElectioEffect<T> {
    /// Available choices.
    pub choices: Vec<T>,
}

impl<T> ElectioEffect<T> {
    /// Create a choice effect with the given alternatives.
    pub fn new(choices: Vec<T>) -> Self {
        ElectioEffect { choices }
    }

    /// Binary choice (true or false).
    pub fn boolean() -> ElectioEffect<bool> {
        ElectioEffect {
            choices: alloc::vec![true, false],
        }
    }

    /// Choice from a range.
    pub fn range(start: i32, end: i32) -> ElectioEffect<i32> {
        ElectioEffect {
            choices: (start..end).collect(),
        }
    }
}

impl<T: Send + Sync + 'static> Effectus for ElectioEffect<T> {}

/// Handler for the choice effect that explores all possibilities.
///
/// Uses multi-shot continuations to backtrack and explore all choices,
/// collecting all possible results.
pub struct ElectioHandler<A> {
    _output: PhantomData<A>,
}

impl<A> ElectioHandler<A> {
    /// Create a new choice handler.
    pub fn new() -> Self {
        ElectioHandler {
            _output: PhantomData,
        }
    }
}

impl<A> Default for ElectioHandler<A> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone + Send + Sync + 'static, A: 'static> TractatorMulti<ElectioEffect<T>, Omega>
    for ElectioHandler<A>
{
    type Input = T;
    type Output = Vec<A>;

    fn handle_operation(
        &mut self,
        effect: ElectioEffect<T>,
        continuation: ContinuatioPluries<Self::Input, Self::Output>,
    ) -> Self::Output {
        // For each choice, resume the continuation and collect results.
        // Pre-allocate based on choice count to avoid repeated reallocation.
        let mut all_results = Vec::with_capacity(effect.choices.len());
        for choice in effect.choices {
            let results = continuation.resume(choice);
            all_results.extend(results);
        }
        all_results
    }
}

/// Handler for choice effect that finds the first successful result.
pub struct ElectioFirstHandler<A, P> {
    predicate: P,
    _output: PhantomData<A>,
}

impl<A, P> ElectioFirstHandler<A, P>
where
    P: Fn(&A) -> bool,
{
    /// Create a handler that finds the first result satisfying the predicate.
    pub fn new(predicate: P) -> Self {
        ElectioFirstHandler {
            predicate,
            _output: PhantomData,
        }
    }
}

impl<T: Clone + Send + Sync + 'static, A: 'static, P> TractatorMulti<ElectioEffect<T>, Omega>
    for ElectioFirstHandler<Option<A>, P>
where
    P: Fn(&A) -> bool,
{
    type Input = T;
    type Output = Option<A>;

    fn handle_operation(
        &mut self,
        effect: ElectioEffect<T>,
        continuation: ContinuatioPluries<Self::Input, Self::Output>,
    ) -> Self::Output {
        for choice in effect.choices {
            if let Some(result) = continuation.resume(choice)
                && (self.predicate)(&result)
            {
                return Some(result);
            }
        }
        None
    }
}

// =============================================================================
// Failure Effect - Backtracking
// =============================================================================

/// The failure effect for backtracking.
///
/// > *"Defectio" - failure, giving up*
///
/// This effect signals that the current computation path has failed
/// and should backtrack to try another alternative.
#[derive(Debug, Clone, Copy)]
pub struct DefectioEffect;

impl Effectus for DefectioEffect {}

/// Handler that converts failure into an empty result.
pub struct DefectioHandler<A> {
    _output: PhantomData<A>,
}

impl<A> DefectioHandler<A> {
    /// Create a new failure handler.
    pub fn new() -> Self {
        DefectioHandler {
            _output: PhantomData,
        }
    }
}

impl<A> Default for DefectioHandler<A> {
    fn default() -> Self {
        Self::new()
    }
}

impl<A: 'static> TractatorMulti<DefectioEffect, Affinis> for DefectioHandler<Vec<A>> {
    type Input = core::convert::Infallible;
    type Output = Vec<A>;

    fn handle_operation(
        &mut self,
        _effect: DefectioEffect,
        continuation: ContinuatioAffinis<Self::Input, Self::Output>,
    ) -> Self::Output {
        continuation.discard();
        Vec::new()
    }
}

// =============================================================================
// Amb Effect - McCarthy's Amb Operator
// =============================================================================

/// The amb (ambiguous) effect for logic programming.
///
/// > *"Ambiguitas" - ambiguity*
///
/// This is `McCarthy`'s amb operator, which non-deterministically chooses
/// a value from alternatives and backtracks on failure.
#[derive(Debug, Clone)]
pub struct AmbiguitasEffect<T> {
    /// Available alternatives.
    pub alternatives: Vec<T>,
}

impl<T> AmbiguitasEffect<T> {
    /// Create an amb effect with alternatives.
    pub fn new(alternatives: Vec<T>) -> Self {
        AmbiguitasEffect { alternatives }
    }

    /// Require a condition to hold (fail if false).
    pub fn require(condition: bool) -> AmbiguitasEffect<()> {
        if condition {
            AmbiguitasEffect {
                alternatives: alloc::vec![()],
            }
        } else {
            AmbiguitasEffect {
                alternatives: Vec::new(),
            }
        }
    }
}

impl<T: Send + Sync + 'static> Effectus for AmbiguitasEffect<T> {}

/// Handler for amb that explores all paths.
pub struct AmbiguitasHandler<A> {
    _output: PhantomData<A>,
}

impl<A> AmbiguitasHandler<A> {
    /// Create a new amb handler.
    pub fn new() -> Self {
        AmbiguitasHandler {
            _output: PhantomData,
        }
    }
}

impl<A> Default for AmbiguitasHandler<A> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone + Send + Sync + 'static, A: 'static> TractatorMulti<AmbiguitasEffect<T>, Omega>
    for AmbiguitasHandler<A>
{
    type Input = T;
    type Output = Vec<A>;

    fn handle_operation(
        &mut self,
        effect: AmbiguitasEffect<T>,
        continuation: ContinuatioPluries<Self::Input, Self::Output>,
    ) -> Self::Output {
        // Pre-allocate based on alternative count to avoid repeated reallocation.
        let mut all_results = Vec::with_capacity(effect.alternatives.len());
        for alt in effect.alternatives {
            let results = continuation.resume(alt);
            all_results.extend(results);
        }
        all_results
    }
}

// =============================================================================
// Probabilistic Effect
// =============================================================================

/// Effect for probabilistic choice.
///
/// > *"Probabilitas" - probability*
///
/// Associates probabilities with alternatives for probabilistic programming.
#[derive(Debug, Clone)]
pub struct ProbabilitasEffect<T> {
    /// Weighted alternatives: (value, probability weight).
    pub weighted: Vec<(T, f64)>,
}

impl<T> ProbabilitasEffect<T> {
    /// Create a probabilistic choice with weights.
    pub fn weighted(weighted: Vec<(T, f64)>) -> Self {
        ProbabilitasEffect { weighted }
    }

    /// Uniform distribution over alternatives.
    pub fn uniform(alternatives: Vec<T>) -> Self {
        let weight = 1.0 / alternatives.len() as f64;
        ProbabilitasEffect {
            weighted: alternatives.into_iter().map(|a| (a, weight)).collect(),
        }
    }

    /// Bernoulli distribution (coin flip).
    pub fn bernoulli(p: f64) -> ProbabilitasEffect<bool> {
        ProbabilitasEffect {
            weighted: alloc::vec![(true, p), (false, 1.0 - p)],
        }
    }
}

impl<T: Send + Sync + 'static> Effectus for ProbabilitasEffect<T> {}

/// Weighted result from probabilistic computation.
#[derive(Debug, Clone)]
pub struct WeightedResult<A> {
    /// The result value.
    pub value: A,
    /// The probability weight of this result.
    pub weight: f64,
}

/// Handler for probabilistic effects that enumerates all outcomes with weights.
pub struct ProbabilitasHandler<A> {
    _output: PhantomData<A>,
}

impl<A> ProbabilitasHandler<A> {
    /// Create a new probabilistic handler.
    pub fn new() -> Self {
        ProbabilitasHandler {
            _output: PhantomData,
        }
    }
}

impl<A> Default for ProbabilitasHandler<A> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone + Send + Sync + 'static, A: 'static> TractatorMulti<ProbabilitasEffect<T>, Omega>
    for ProbabilitasHandler<A>
{
    type Input = (T, f64);
    type Output = Vec<WeightedResult<A>>;

    fn handle_operation(
        &mut self,
        effect: ProbabilitasEffect<T>,
        continuation: ContinuatioPluries<Self::Input, Self::Output>,
    ) -> Self::Output {
        let mut all_results = Vec::new();
        for (value, weight) in effect.weighted {
            let results = continuation.resume((value, weight));
            // Scale weights by the probability of this branch
            for mut weighted_result in results {
                weighted_result.weight *= weight;
                all_results.push(weighted_result);
            }
        }
        all_results
    }
}

// =============================================================================
// Handler Composition
// =============================================================================

/// Compose two multi-shot handlers.
pub struct ComposedMultiHandler<H1, H2> {
    /// First handler.
    pub handler1: H1,
    /// Second handler.
    pub handler2: H2,
}

impl<H1, H2> ComposedMultiHandler<H1, H2> {
    /// Create a composed handler.
    pub fn new(handler1: H1, handler2: H2) -> Self {
        ComposedMultiHandler { handler1, handler2 }
    }
}

/// Extension trait for composing handlers.
pub trait TractatorMultiExt: Sized {
    /// Compose this handler with another.
    fn compose_multi<H2>(self, other: H2) -> ComposedMultiHandler<Self, H2> {
        ComposedMultiHandler::new(self, other)
    }
}

impl<H> TractatorMultiExt for H {}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_default_multi_handler() {
        struct TestEffect;
        impl Effectus for TestEffect {}

        let mut handler = DefaultMultiHandler::<i32, Semel>::new(42);
        let cont = Continuatio::semel(|x: i32| x + 1);
        let result = handler.handle_operation(TestEffect, cont);
        assert_eq!(result, 43);
    }

    #[test]
    fn test_electio_effect_boolean() {
        let effect = ElectioEffect::<()>::boolean();
        assert_eq!(effect.choices.len(), 2);
        assert!(effect.choices.contains(&true));
        assert!(effect.choices.contains(&false));
    }

    #[test]
    fn test_electio_effect_range() {
        let effect = ElectioEffect::<()>::range(1, 4);
        assert_eq!(effect.choices, vec![1, 2, 3]);
    }

    #[test]
    fn test_electio_handler() {
        let mut handler = ElectioHandler::<i32>::new();
        let effect = ElectioEffect::new(vec![1, 2, 3]);
        let cont = Continuatio::pluries(|x: i32| vec![x * 2]);
        let results = handler.handle_operation(effect, cont);
        assert_eq!(results, vec![2, 4, 6]);
    }

    #[test]
    fn test_ambiguitas_effect() {
        let effect = AmbiguitasEffect::new(vec![1, 2, 3]);
        assert_eq!(effect.alternatives.len(), 3);
    }

    #[test]
    fn test_ambiguitas_require_true() {
        let effect = AmbiguitasEffect::<()>::require(true);
        assert_eq!(effect.alternatives.len(), 1);
    }

    #[test]
    fn test_ambiguitas_require_false() {
        let effect = AmbiguitasEffect::<()>::require(false);
        assert_eq!(effect.alternatives.len(), 0);
    }

    #[test]
    fn test_probabilitas_effect_uniform() {
        let effect = ProbabilitasEffect::uniform(vec![1, 2, 3, 4]);
        assert_eq!(effect.weighted.len(), 4);
        for (_, w) in &effect.weighted {
            assert!((w - 0.25).abs() < 0.0001);
        }
    }

    #[test]
    fn test_probabilitas_effect_bernoulli() {
        let effect = ProbabilitasEffect::<()>::bernoulli(0.7);
        assert_eq!(effect.weighted.len(), 2);
        assert!(effect.weighted[0].0);
        assert!((effect.weighted[0].1 - 0.7).abs() < 0.0001);
        assert!(!effect.weighted[1].0);
        assert!((effect.weighted[1].1 - 0.3).abs() < 0.0001);
    }
}
