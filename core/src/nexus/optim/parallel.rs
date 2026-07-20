//! Type-Safe Parallel Combinators
//!
//! This module provides parallel combinators that use the type system
//! to ensure safety. Users must **explicitly opt-in** by calling these
//! combinators.
//!
//! # Current status: sequential
//!
//! Every combinator in this module executes sequentially on the calling
//! thread. The `par_*` names and `Send`/`Sync` bounds describe what the
//! API *permits*; no thread pool is wired in yet. The [`ParallelStrategy`]
//! parameter is recorded but ignored at runtime. `TODO(parallel-backend)`
//! markers inside each combinator body mark the insertion points for a
//! real executor (e.g. `rayon`) behind a Cargo feature.
//!
//! # Safety guarantee (unchanged by sequential execution)
//!
//! The type system prevents parallel execution of effectful code:
//! - `par_map` only accepts `Eff<Pure, B>` computations
//! - Attempting to parallelize State/Reader/IO effects fails at compile time
//!
//! # Example
//!
//! ```rust
//! use ordofp_core::nexus::prelude::*;
//! use ordofp_core::nexus::optim::*;
//!
//! // This compiles - Pure effects can be parallelized
//! let items = vec![1, 2, 3, 4, 5];
//! let doubled = par_map(&items, |x| pure(x * 2));
//! assert_eq!(doubled, vec![2, 4, 6, 8, 10]);
//!
//! // This does NOT compile - State effects cannot be parallelized
//! // let stateful = par_map(&items, |x| state_modify(|s| s + x));
//! // Error: Row<STATE_BIT> does not implement ParallelSafe
//! ```
//!
//! # Limitations
//!
//! - **Executes sequentially today.** A real parallel backend (rayon or
//!   equivalent) is pending and will be added behind a Cargo feature flag.
//! - Cannot parallelize across effect handlers.
//! - No work-stealing or load balancing yet — [`ParallelStrategy`] is an
//!   API placeholder, not a runtime switch.

use alloc::vec::Vec;
use core::marker::PhantomData;

use crate::nexus::effect::Eff;
use crate::nexus::row::{EffectRow, Pure};

// =============================================================================
// Parallel Marker Trait
// =============================================================================

/// Marker trait for effect rows that are safe to parallelize.
///
/// Only pure effects implement this trait, ensuring type-level
/// safety for parallel execution.
pub trait ParallelSafe: EffectRow {}

/// Pure effects are always safe to parallelize.
impl ParallelSafe for Pure {}

// =============================================================================
// Parallel Execution Strategy
// =============================================================================

/// Strategy for parallel execution.
///
/// **Sequential-only today:** This enum is currently an API placeholder.
/// Every combinator that accepts a `ParallelStrategy` records the value but
/// executes sequentially regardless of which variant is chosen. The variants
/// exist so that call sites can express intent now and pick up a real
/// backend (e.g. `rayon`) once it is wired in behind a Cargo feature, with
/// no source changes required.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ParallelStrategy {
    /// Sequential execution (no parallelism). This is the only variant whose
    /// semantics match its name today.
    Sequential,
    /// Fixed number of parallel tasks.
    ///
    /// *Reserved:* Currently treated as sequential; the task count is
    /// ignored until a parallel backend is wired in.
    Fixed(usize),
    /// Work-stealing with automatic load balancing.
    ///
    /// *Reserved:* Currently treated as sequential. Present as the default
    /// so that once a real backend lands, existing call sites immediately
    /// benefit.
    #[default]
    WorkStealing,
    /// Adaptive based on workload characteristics.
    ///
    /// *Reserved:* Currently treated as sequential; adaptive selection is
    /// not yet implemented.
    Adaptive,
}

// =============================================================================
// Parallel Map
// =============================================================================

/// Parallel map over a collection with pure computations.
///
/// This function maps a pure computation over each element of a collection.
/// Because the computation is pure (proven by the type system), the
/// operations can safely execute in parallel.
///
/// # Current Behavior
///
/// **Executes sequentially.** This function runs the mapping function on
/// each element in order on the calling thread. The `Sync`/`Send` bounds
/// and `par_` prefix anticipate a future `rayon`-backed implementation;
/// they do not imply parallel execution today. See the module-level docs
/// for status.
///
/// # Type Safety
///
/// The function only accepts `Eff<Pure, B>` computations, ensuring
/// at compile time that parallel execution is safe.
///
/// # Example
///
/// ```rust
/// use ordofp_core::nexus::pure;
/// use ordofp_core::nexus::optim::par_map;
///
/// let items = vec![1, 2, 3, 4, 5];
/// let results = par_map(&items, |x| pure(x * 2));
/// assert_eq!(results, vec![2, 4, 6, 8, 10]);
/// ```
#[inline]
pub fn par_map<A, B, F>(items: &[A], f: F) -> Vec<B>
where
    A: Sync,
    B: Send,
    F: Fn(&A) -> Eff<Pure, B> + Sync,
{
    // TODO(parallel-backend): swap this for `items.par_iter().map(...)` once
    // the `rayon` Cargo feature is introduced. The `Sync`/`Send` bounds and
    // `Pure` effect row above are already sufficient preconditions for a
    // parallel implementation; only the backend is missing.
    items.iter().map(|a| f(a).run_pure()).collect()
}

/// Parallel map over a collection with an explicit execution strategy.
///
/// Behaves identically to [`par_map`], but allows the caller to choose how
/// work is scheduled via [`ParallelStrategy`].  This is useful when you have
/// domain knowledge about the workload size or want to pin to a specific
/// concurrency model.
///
/// **Note:** The current implementation executes sequentially regardless of
/// the strategy chosen.  The strategy parameter is reserved for a future
/// rayon-backed implementation and has no runtime effect today.
///
/// # Parameters
///
/// * `items`    — Slice of input values to map over.
/// * `f`        — A sync closure that maps each `&A` to a pure `Eff<Pure, B>`.
/// * `_strategy` — Desired execution strategy (currently ignored).
///
/// # Example
///
/// ```rust
/// use ordofp_core::nexus::optim::parallel::{par_map_with, ParallelStrategy};
/// use ordofp_core::nexus::pure;
///
/// let items = vec![1, 2, 3, 4, 5];
/// let results = par_map_with(&items, |x| pure(x * 2), ParallelStrategy::Sequential);
/// assert_eq!(results, vec![2, 4, 6, 8, 10]);
/// ```
pub fn par_map_with<A, B, F>(items: &[A], f: F, _strategy: ParallelStrategy) -> Vec<B>
where
    A: Sync,
    B: Send,
    F: Fn(&A) -> Eff<Pure, B> + Sync,
{
    // TODO(parallel-backend): dispatch on `_strategy` (Fixed / WorkStealing /
    // Adaptive) to a real parallel executor. Until a backend feature exists
    // the strategy is intentionally ignored, and this call degrades to the
    // sequential `par_map` implementation above.
    par_map(items, f)
}

// =============================================================================
// Parallel Traverse
// =============================================================================

/// Parallel traverse - map and collect effects.
///
/// Like `par_map`, but collects the results into an effectful context.
/// This is the parallel version of the standard `traverse` operation.
///
/// # Current Behavior
///
/// **Executes sequentially.** See the module-level docs for status. The
/// `Send` bounds and `par_` prefix anticipate a future parallel backend.
///
/// # Example
///
/// ```rust
/// use ordofp_core::nexus::Eff;
/// use ordofp_core::nexus::pure;
/// use ordofp_core::nexus::optim::par_traverse;
/// use ordofp_core::nexus::Pure;
///
/// let items = vec![1, 2, 3];
/// let result: Eff<Pure, Vec<i32>> = par_traverse(items, |x| pure(x * 2));
/// assert_eq!(result.run_pure(), vec![2, 4, 6]);
/// ```
#[inline]
pub fn par_traverse<A, B, F>(items: Vec<A>, f: F) -> Eff<Pure, Vec<B>>
where
    A: Send,
    B: Send,
    F: Fn(A) -> Eff<Pure, B> + Sync,
{
    // TODO(parallel-backend): once a real backend is wired in, swap this for
    // `items.into_par_iter().map(...).collect()`. For now we execute each
    // pure computation sequentially in input order.
    let results: Vec<B> = items.into_iter().map(|a| f(a).run_pure()).collect();
    Eff::from_value(results)
}

/// Parallel traverse with an explicit execution strategy.
///
/// Applies `f` to each element of `items`, collecting the pure results into a
/// single `Eff<Pure, Vec<B>>`.  Behaves identically to [`par_traverse`], but
/// allows the caller to specify the desired [`ParallelStrategy`] for workload
/// scheduling.
///
/// **Note:** The current implementation executes sequentially regardless of the
/// chosen strategy.  The `_strategy` parameter is reserved for a future
/// rayon-backed implementation and has no runtime effect today.  Existing call
/// sites will automatically gain parallel execution once that backend lands,
/// without any code changes.
///
/// # Parameters
///
/// * `items`     — Owned collection of input values to map over.
/// * `f`         — A sync closure mapping each `A` to a pure `Eff<Pure, B>`.
/// * `_strategy` — Desired execution strategy (currently ignored).
///
/// # Example
///
/// ```rust
/// use ordofp_core::nexus::optim::parallel::{par_traverse_with, ParallelStrategy};
/// use ordofp_core::nexus::pure;
///
/// let items = vec![1, 2, 3];
/// let result = par_traverse_with(items, |x| pure(x * 2), ParallelStrategy::WorkStealing);
/// assert_eq!(result.run_pure(), vec![2, 4, 6]);
/// ```
pub fn par_traverse_with<A, B, F>(
    items: Vec<A>,
    f: F,
    _strategy: ParallelStrategy,
) -> Eff<Pure, Vec<B>>
where
    A: Send,
    B: Send,
    F: Fn(A) -> Eff<Pure, B> + Sync,
{
    // TODO(parallel-backend): dispatch on `_strategy` once a real executor
    // is available. Today the strategy is recorded-but-ignored and we fall
    // through to the sequential `par_traverse` implementation.
    par_traverse(items, f)
}

// =============================================================================
// Parallel Sequence
// =============================================================================

/// Parallel sequence - convert a collection of effects to an effect of collection.
///
/// All effects must be pure, allowing parallel execution.
///
/// # Current Behavior
///
/// **Executes sequentially.** See the module-level docs for status. A real
/// parallel backend (rayon or equivalent) is pending.
///
/// # Example
///
/// ```rust
/// use ordofp_core::nexus::Eff;
/// use ordofp_core::nexus::pure;
/// use ordofp_core::nexus::optim::par_sequence;
/// use ordofp_core::nexus::Pure;
///
/// let effects: Vec<Eff<Pure, i32>> = vec![pure(1), pure(2), pure(3)];
/// let result: Eff<Pure, Vec<i32>> = par_sequence(effects);
/// assert_eq!(result.run_pure(), vec![1, 2, 3]);
/// ```
#[inline]
pub fn par_sequence<A>(effects: Vec<Eff<Pure, A>>) -> Eff<Pure, Vec<A>>
where
    A: Send,
{
    // TODO(parallel-backend): run each pure effect on a worker pool once a
    // parallel executor feature is available. Currently sequential.
    let results: Vec<A> = effects
        .into_iter()
        .map(super::super::effect::Eff::run_pure)
        .collect();
    Eff::from_value(results)
}

// =============================================================================
// Parallel Fold
// =============================================================================

/// Parallel fold with associative operation.
///
/// Reduces a collection in parallel using an associative combining function.
/// The function must be associative for correct parallel execution.
///
/// # Current Behavior
///
/// **Executes sequentially** as a left fold. Associativity of `combine` is
/// still required so that this call site is drop-in ready for a future
/// divide-and-conquer parallel implementation. See the module-level docs
/// for status.
///
/// # Example
///
/// ```rust
/// use ordofp_core::nexus::optim::parallel::par_fold;
///
/// let items = vec![1, 2, 3, 4, 5];
/// let sum = par_fold(&items, 0, |x| *x, |a, b| a + b);
/// assert_eq!(sum, 15);
/// ```
pub fn par_fold<A, B, Map, Combine>(items: &[A], initial: B, map: Map, combine: Combine) -> B
where
    A: Sync,
    B: Send + Clone,
    Map: Fn(&A) -> B + Sync,
    Combine: Fn(B, B) -> B + Sync,
{
    if items.is_empty() {
        return initial;
    }

    // TODO(parallel-backend): replace with a divide-and-conquer / tree
    // reduction using a work-stealing executor. Associativity of `combine`
    // is required for that refactor and is documented above, so no API
    // change will be needed when the backend lands.
    items.iter().map(map).fold(initial, combine)
}

// =============================================================================
// Parallel Both
// =============================================================================

/// Execute two pure computations in parallel.
///
/// Returns a tuple of both results.
///
/// # Current Behavior
///
/// **Executes sequentially:** `left` is fully evaluated, then `right`. See
/// the module-level docs for status.
///
/// # Example
///
/// ```rust
/// use ordofp_core::nexus::pure;
/// use ordofp_core::nexus::optim::parallel::par_both;
///
/// let (a, b) = par_both(
///     pure(1), // e.g. expensive_computation_1()
///     pure(2), // e.g. expensive_computation_2()
/// );
/// assert_eq!(a, 1);
/// assert_eq!(b, 2);
/// ```
#[inline]
pub fn par_both<A, B>(left: Eff<Pure, A>, right: Eff<Pure, B>) -> (A, B)
where
    A: Send,
    B: Send,
{
    // TODO(parallel-backend): run `left` and `right` on separate worker
    // threads once a parallel executor feature is available. For now the
    // two effects are evaluated sequentially on the calling thread.
    let a = left.run_pure();
    let b = right.run_pure();
    (a, b)
}

/// Execute three pure computations in parallel.
///
/// Returns a triple of all three results. All three result types must
/// implement [`Send`] to allow future execution on separate threads or via a
/// work-stealing scheduler.
///
/// # Current Behavior
///
/// **Executes sequentially** in the order `first`, `second`, `third`. See
/// the module-level docs for status.
///
/// # Example
///
/// ```rust
/// use ordofp_core::nexus::pure;
/// use ordofp_core::nexus::optim::parallel::par_triple;
///
/// let (a, b, c) = par_triple(
///     pure(1), // e.g. compute_x()
///     pure(2), // e.g. compute_y()
///     pure(3), // e.g. compute_z()
/// );
/// assert_eq!(a, 1);
/// assert_eq!(b, 2);
/// assert_eq!(c, 3);
/// ```
#[inline]
pub fn par_triple<A, B, C>(
    first: Eff<Pure, A>,
    second: Eff<Pure, B>,
    third: Eff<Pure, C>,
) -> (A, B, C)
where
    A: Send,
    B: Send,
    C: Send,
{
    // TODO(parallel-backend): run all three effects on separate workers
    // once a parallel executor feature is available. For now the three
    // effects are evaluated sequentially on the calling thread.
    let a = first.run_pure();
    let b = second.run_pure();
    let c = third.run_pure();
    (a, b, c)
}

// =============================================================================
// Parallel Racing
// =============================================================================

/// Race multiple pure computations, returning the first to complete.
///
/// Since computations are pure, it doesn't matter which one "wins" -
/// they all produce the same result for the same input.
///
/// This is mainly useful for computations that may have different
/// performance characteristics.
///
/// # Current Behavior
///
/// **No actual race occurs.** This function simply returns the result of
/// the first computation in the input vector (or `None` if empty); the
/// remaining computations are dropped without being run. A true racing
/// implementation requires a thread/async backend. See the module-level
/// docs for status.
pub fn par_race<A>(computations: Vec<Eff<Pure, A>>) -> Option<A>
where
    A: Send,
{
    // TODO(parallel-backend): spawn all computations and return whichever
    // finishes first, cancelling the others. Until a real executor is
    // wired in we just run the first computation in input order.
    computations
        .into_iter()
        .next()
        .map(super::super::effect::Eff::run_pure)
}

// =============================================================================
// Chunk-Based Parallelism
// =============================================================================

/// Process items in parallel chunks.
///
/// Divides the input into chunks and processes each chunk in parallel.
/// Useful for controlling the granularity of parallelism.
///
/// # Current Behavior
///
/// **Executes sequentially.** Chunks are processed one at a time on the
/// calling thread in input order. See the module-level docs for status.
///
/// # Example
///
/// ```rust
/// use ordofp_core::nexus::pure;
/// use ordofp_core::nexus::optim::parallel::par_chunks;
///
/// let items: Vec<i32> = (0..1000).collect();
/// let results = par_chunks(&items, 100, |chunk| {
///     pure(chunk.iter().sum::<i32>())
/// });
/// assert_eq!(results.len(), 10);
/// assert_eq!(results[0], (0..100).sum::<i32>());
/// ```
pub fn par_chunks<A, B, F>(items: &[A], chunk_size: usize, f: F) -> Vec<B>
where
    A: Sync,
    B: Send,
    F: Fn(&[A]) -> Eff<Pure, B> + Sync,
{
    // TODO(parallel-backend): distribute chunks across a worker pool once a
    // parallel executor feature is available. Currently chunks are
    // processed sequentially in input order.
    items
        .chunks(chunk_size)
        .map(|chunk| f(chunk).run_pure())
        .collect()
}

// =============================================================================
// Parallel Computation Builder
// =============================================================================

/// Builder for configuring parallel computations.
///
/// # Current Behavior
///
/// **All configuration is currently advisory.** The strategy and chunk
/// size are stored on the builder but ignored when [`ParallelBuilder::map`]
/// runs, which dispatches sequentially. The builder exists so call sites
/// can declare intent now and pick up a real parallel backend (e.g.
/// `rayon`) later without API changes. See the module-level docs.
pub struct ParallelBuilder<A> {
    items: Vec<A>,
    strategy: ParallelStrategy,
    chunk_size: Option<usize>,
}

impl<A> ParallelBuilder<A> {
    /// Create a new parallel builder.
    #[inline]
    pub fn new(items: Vec<A>) -> Self {
        ParallelBuilder {
            items,
            strategy: ParallelStrategy::default(),
            chunk_size: None,
        }
    }

    /// Set the parallel strategy.
    #[inline]
    pub fn with_strategy(mut self, strategy: ParallelStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Set the chunk size for chunked processing.
    #[inline]
    pub fn with_chunk_size(mut self, size: usize) -> Self {
        self.chunk_size = Some(size);
        self
    }

    /// Map a pure function over the items.
    ///
    /// # Current Behavior
    ///
    /// **Executes sequentially.** The configured `strategy` and
    /// `chunk_size` are ignored — see the struct-level note and the
    /// module-level docs.
    pub fn map<B, F>(self, f: F) -> Vec<B>
    where
        A: Send,
        B: Send,
        F: Fn(A) -> Eff<Pure, B> + Sync,
    {
        // TODO(parallel-backend): honor `self.strategy` and
        // `self.chunk_size` by dispatching through a real executor. For
        // now both fields are accepted for forward compatibility but have
        // no runtime effect.
        self.items.into_iter().map(|a| f(a).run_pure()).collect()
    }
}

// =============================================================================
// Type-Level Parallel Safety Proof
// =============================================================================

/// Proof that a computation is safe to parallelize.
pub struct ParallelProof<R: ParallelSafe> {
    _marker: PhantomData<R>,
}

impl ParallelProof<Pure> {
    /// Create a proof of parallel safety for pure computations.
    pub const fn pure() -> Self {
        ParallelProof {
            _marker: PhantomData,
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nexus::ops::pure;

    #[test]
    fn test_par_map() {
        let items = alloc::vec![1, 2, 3, 4, 5];
        let results = par_map(&items, |x| pure(x * 2));
        assert_eq!(results, alloc::vec![2, 4, 6, 8, 10]);
    }

    #[test]
    fn test_par_traverse() {
        let items = alloc::vec![1, 2, 3];
        let result = par_traverse(items, |x| pure(x + 10));
        assert_eq!(result.run_pure(), alloc::vec![11, 12, 13]);
    }

    #[test]
    fn test_par_sequence() {
        let effects = alloc::vec![pure(1), pure(2), pure(3)];
        let result = par_sequence(effects);
        assert_eq!(result.run_pure(), alloc::vec![1, 2, 3]);
    }

    #[test]
    fn test_par_fold() {
        let items = alloc::vec![1, 2, 3, 4, 5];
        let sum = par_fold(&items, 0, |x| *x, |a, b| a + b);
        assert_eq!(sum, 15);
    }

    #[test]
    fn test_par_both() {
        let (a, b) = par_both(pure(1), pure(2));
        assert_eq!(a, 1);
        assert_eq!(b, 2);
    }

    #[test]
    fn test_par_triple() {
        let (a, b, c) = par_triple(pure(1), pure(2), pure(3));
        assert_eq!(a, 1);
        assert_eq!(b, 2);
        assert_eq!(c, 3);
    }

    #[test]
    fn test_par_chunks() {
        let items: alloc::vec::Vec<i32> = (1..=10).collect();
        let sums = par_chunks(&items, 5, |chunk| pure(chunk.iter().sum::<i32>()));
        assert_eq!(sums, alloc::vec![15, 40]); // 1+2+3+4+5=15, 6+7+8+9+10=40
    }

    #[test]
    fn test_par_race() {
        let comps = alloc::vec![pure(42), pure(43)];
        let result = par_race(comps);
        assert_eq!(result, Some(42));
    }

    #[test]
    fn test_parallel_builder() {
        let items = alloc::vec![1, 2, 3];
        let results = ParallelBuilder::new(items)
            .with_strategy(ParallelStrategy::Sequential)
            .map(|x| pure(x * 3));
        assert_eq!(results, alloc::vec![3, 6, 9]);
    }
}
