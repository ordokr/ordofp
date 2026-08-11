//! Type-Safe Parallel Combinators
//!
//! This module provides parallel combinators that use the type system
//! to ensure safety. Users must **explicitly opt-in** by calling these
//! combinators.
//!
//! # Current status: backend-aware
//!
//! With the `rayon` Cargo feature enabled, collection-oriented combinators
//! (`par_map`, `par_map_with`, `par_traverse`, `par_traverse_with`,
//! `par_fold`, `par_chunks`, and `ParallelBuilder::map`) execute in parallel.
//! Without `rayon`, all combinators execute sequentially on the calling thread.
//!
//! `ParallelStrategy` is honored for strategy-aware entry points:
//! - `Sequential` runs sequentially.
//! - `Fixed(n)` runs on a dedicated Rayon thread pool with `n` workers.
//! - `WorkStealing` uses Rayon global work-stealing execution.
//! - `Adaptive` uses sequential execution for tiny workloads and Rayon for
//!   larger workloads.
//!
//! # Safety guarantee
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
//! - Actual parallel execution requires the `rayon` Cargo feature.
//! - Cannot parallelize across effect handlers.
//! - `par_race` still returns the first computation in input order; true
//!   first-completion racing needs cancellation-aware runtime support.

use alloc::vec::Vec;
use core::marker::PhantomData;

use crate::nexus::effect::Eff;
use crate::nexus::row::{EffectRow, Pure};
#[cfg(feature = "rayon")]
use rayon::ThreadPoolBuilder;
#[cfg(all(feature = "rayon", feature = "std"))]
use rayon::prelude::IntoParallelRefIterator;
#[cfg(feature = "rayon")]
use rayon::prelude::{IntoParallelIterator, ParallelIterator};

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
/// Honored by strategy-aware combinators when the `rayon` feature is enabled.
/// Without `rayon`, all variants degrade to sequential execution.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ParallelStrategy {
    /// Sequential execution (no parallelism). This is the only variant whose
    /// semantics match its name in all configurations.
    Sequential,
    /// Fixed number of parallel tasks.
    ///
    /// Uses a dedicated Rayon pool when `rayon` is enabled.
    Fixed(usize),
    /// Work-stealing with automatic load balancing.
    ///
    /// Uses Rayon global work-stealing when `rayon` is enabled.
    #[default]
    WorkStealing,
    /// Adaptive based on workload characteristics.
    ///
    /// Uses a small-workload sequential cutoff, otherwise Rayon execution.
    Adaptive,
}

#[cfg(feature = "rayon")]
const ADAPTIVE_MIN_PAR_ITEMS: usize = 128;

#[cfg(feature = "rayon")]
#[inline]
fn adaptive_prefers_sequential(len: usize) -> bool {
    len < ADAPTIVE_MIN_PAR_ITEMS
}

#[cfg(feature = "rayon")]
fn with_fixed_pool<R, F>(threads: usize, f: F) -> R
where
    R: Send,
    F: FnOnce() -> R + Send,
{
    assert!(
        threads != 0,
        "ParallelStrategy::Fixed(0) is invalid; use at least one worker"
    );
    if threads == 1 {
        return f();
    }

    ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .expect("failed to build rayon thread pool for ParallelStrategy::Fixed")
        .install(f)
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
/// Executes in parallel when the `rayon` feature is enabled; otherwise runs
/// sequentially on the calling thread.
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
    #[cfg(feature = "rayon")]
    {
        items.par_iter().map(|a| f(a).run_pure()).collect()
    }
    #[cfg(not(feature = "rayon"))]
    {
        items.iter().map(|a| f(a).run_pure()).collect()
    }
}

/// Parallel map over a collection with an explicit execution strategy.
///
/// Behaves identically to [`par_map`], but allows the caller to choose how
/// work is scheduled via [`ParallelStrategy`].  This is useful when you have
/// domain knowledge about the workload size or want to pin to a specific
/// concurrency model.
///
/// **Note:** Strategy selection is honored only when `rayon` is enabled.
///
/// # Parameters
///
/// * `items`    — Slice of input values to map over.
/// * `f`        — A sync closure that maps each `&A` to a pure `Eff<Pure, B>`.
/// * `strategy` — Desired execution strategy.
///
/// # Examples
///
/// ```rust
/// use ordofp_core::nexus::optim::parallel::{par_map_with, ParallelStrategy};
/// use ordofp_core::nexus::pure;
///
/// let items = vec![1, 2, 3, 4, 5];
/// let results = par_map_with(&items, |x| pure(x * 2), ParallelStrategy::Sequential);
/// assert_eq!(results, vec![2, 4, 6, 8, 10]);
/// ```
pub fn par_map_with<A, B, F>(items: &[A], f: F, strategy: ParallelStrategy) -> Vec<B>
where
    A: Sync,
    B: Send,
    F: Fn(&A) -> Eff<Pure, B> + Sync,
{
    #[cfg(feature = "rayon")]
    {
        match strategy {
            ParallelStrategy::Sequential => items.iter().map(|a| f(a).run_pure()).collect(),
            ParallelStrategy::Fixed(threads) => with_fixed_pool(threads, || {
                items.par_iter().map(|a| f(a).run_pure()).collect()
            }),
            ParallelStrategy::WorkStealing => items.par_iter().map(|a| f(a).run_pure()).collect(),
            ParallelStrategy::Adaptive => {
                if adaptive_prefers_sequential(items.len()) {
                    items.iter().map(|a| f(a).run_pure()).collect()
                } else {
                    items.par_iter().map(|a| f(a).run_pure()).collect()
                }
            }
        }
    }
    #[cfg(not(feature = "rayon"))]
    {
        par_map(items, f)
    }
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
/// Executes in parallel when the `rayon` feature is enabled; otherwise runs
/// sequentially.
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
    #[cfg(feature = "rayon")]
    let results: Vec<B> = items.into_par_iter().map(|a| f(a).run_pure()).collect();
    #[cfg(not(feature = "rayon"))]
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
/// **Note:** Strategy selection is honored only when `rayon` is enabled.
///
/// # Parameters
///
/// * `items`     — Owned collection of input values to map over.
/// * `f`         — A sync closure mapping each `A` to a pure `Eff<Pure, B>`.
/// * `strategy`  — Desired execution strategy.
///
/// # Examples
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
    strategy: ParallelStrategy,
) -> Eff<Pure, Vec<B>>
where
    A: Send,
    B: Send,
    F: Fn(A) -> Eff<Pure, B> + Sync,
{
    #[cfg(feature = "rayon")]
    {
        let len = items.len();
        let results = match strategy {
            ParallelStrategy::Sequential => items.into_iter().map(|a| f(a).run_pure()).collect(),
            ParallelStrategy::Fixed(threads) => with_fixed_pool(threads, || {
                items.into_par_iter().map(|a| f(a).run_pure()).collect()
            }),
            ParallelStrategy::WorkStealing => {
                items.into_par_iter().map(|a| f(a).run_pure()).collect()
            }
            ParallelStrategy::Adaptive => {
                if adaptive_prefers_sequential(len) {
                    items.into_iter().map(|a| f(a).run_pure()).collect()
                } else {
                    items.into_par_iter().map(|a| f(a).run_pure()).collect()
                }
            }
        };
        Eff::from_value(results)
    }
    #[cfg(not(feature = "rayon"))]
    {
        par_traverse(items, f)
    }
}

// =============================================================================
// Parallel Sequence
// =============================================================================

/// Sequence a collection of pure effects into an effect of collection.
///
/// # Current Behavior
///
/// Executes sequentially today. The `Eff` representation is not currently
/// transferable across worker threads, so this combinator remains
/// single-threaded even when `rayon` is enabled.
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
/// Uses parallel reduction when `rayon` is enabled; otherwise executes as a
/// sequential left fold. Associativity of `combine` is required.
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
    B: Send + Sync + Clone,
    Map: Fn(&A) -> B + Sync + Send,
    Combine: Fn(B, B) -> B + Sync + Send,
{
    if items.is_empty() {
        return initial;
    }

    #[cfg(feature = "rayon")]
    {
        items
            .par_iter()
            .map(map)
            .reduce(|| initial.clone(), combine)
    }
    #[cfg(not(feature = "rayon"))]
    {
        items.iter().map(map).fold(initial, combine)
    }
}

// =============================================================================
// Parallel Both
// =============================================================================

/// Execute two pure computations and return both results.
///
/// Returns a tuple of both results.
///
/// # Current Behavior
///
/// Executes sequentially (`left` then `right`) today.
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
    let a = left.run_pure();
    let b = right.run_pure();
    (a, b)
}

/// Execute three pure computations and return all results.
///
/// Returns a triple of all three results. All three result types must
/// implement [`Send`] to keep this API forward-compatible with future worker
/// execution.
///
/// # Current Behavior
///
/// Executes sequentially in `first`, `second`, `third` order today.
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
/// Executes chunk workers in parallel when `rayon` is enabled; otherwise
/// processes chunks sequentially in input order.
///
/// # Panics
///
/// Panics if `chunk_size` is 0.
///
/// # Examples
///
/// ```rust
/// use ordofp_core::nexus::optim::parallel::par_chunks;
/// use ordofp_core::nexus::pure;
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
    assert!(chunk_size > 0, "chunk_size must be > 0");
    #[cfg(all(feature = "rayon", feature = "std"))]
    {
        use rayon::slice::ParallelSlice;
        items
            .par_chunks(chunk_size)
            .map(|chunk| f(chunk).run_pure())
            .collect()
    }
    #[cfg(not(all(feature = "rayon", feature = "std")))]
    {
        items
            .chunks(chunk_size)
            .map(|chunk| f(chunk).run_pure())
            .collect()
    }
}

// =============================================================================
// Parallel Computation Builder
// =============================================================================

/// Builder for configuring parallel computations.
///
/// # Current Behavior
///
/// `strategy` is honored by [`ParallelBuilder::map`] when `rayon` is enabled.
/// `chunk_size` is currently advisory and reserved for chunk-aware map
/// scheduling.
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
    /// Uses `strategy` when `rayon` is enabled; otherwise runs sequentially.
    /// `chunk_size` is currently advisory for `map` and reserved for future
    /// chunk-aware scheduling.
    pub fn map<B, F>(self, f: F) -> Vec<B>
    where
        A: Send,
        B: Send,
        F: Fn(A) -> Eff<Pure, B> + Sync,
    {
        let strategy = self.strategy;
        let _chunk_size = self.chunk_size;
        #[cfg(feature = "rayon")]
        {
            let len = self.items.len();
            match strategy {
                ParallelStrategy::Sequential => {
                    self.items.into_iter().map(|a| f(a).run_pure()).collect()
                }
                ParallelStrategy::Fixed(threads) => with_fixed_pool(threads, || {
                    self.items
                        .into_par_iter()
                        .map(|a| f(a).run_pure())
                        .collect()
                }),
                ParallelStrategy::WorkStealing => self
                    .items
                    .into_par_iter()
                    .map(|a| f(a).run_pure())
                    .collect(),
                ParallelStrategy::Adaptive => {
                    if adaptive_prefers_sequential(len) {
                        self.items.into_iter().map(|a| f(a).run_pure()).collect()
                    } else {
                        self.items
                            .into_par_iter()
                            .map(|a| f(a).run_pure())
                            .collect()
                    }
                }
            }
        }
        #[cfg(not(feature = "rayon"))]
        {
            let _ = strategy;
            self.items.into_iter().map(|a| f(a).run_pure()).collect()
        }
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
