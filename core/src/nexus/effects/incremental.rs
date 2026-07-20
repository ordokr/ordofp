//! Incremental Computation Effect
//!
//! The Incremental effect provides dependency-tracking primitives for
//! self-adjusting computation. When inputs change, only the dependent
//! computations are recomputed.
//!
//! # Key Concepts
//!
//! - **Tracked Inputs**: Values that can change and trigger recomputation
//! - **Memoized Results**: Cached computations with dependency tracking
//! - **Invalidation**: Marking inputs as changed to trigger recomputation
//!
//! # Example
//!
//! ```rust
//! use ordofp_core::nexus::effects::incremental::*;
//!
//! // Create an incremental context
//! let mut ctx = IncrementalContext::new();
//!
//! // Register an input
//! let input_id = ctx.create_input("source", 42_i32);
//!
//! // Create a memoized computation: the closure receives the context
//! let doubled = |ctx: &mut IncrementalContext| {
//!     let value: i32 = ctx.read_input(&input_id).unwrap();
//!     value * 2
//! };
//!
//! // First run computes
//! assert_eq!(ctx.memo("doubled", doubled), 84);
//!
//! // Second run with the same key uses the cache
//! assert_eq!(ctx.memo("doubled", doubled), 84); // Cache hit!
//!
//! // Invalidate input
//! ctx.set_input(&input_id, 100_i32);
//!
//! // Next run recomputes
//! assert_eq!(ctx.memo("doubled", doubled), 200); // Recomputed!
//! ```
//!
//! # Verification Tier
//!
//! **Tier 1**: Tested via unit tests demonstrating invalidation behavior.
//!
//! # Limitations
//!
//! - Single-threaded (no concurrent access to context)
//! - No cross-computation dependency graphs (MVP scope)
//! - No automatic incrementalization (explicit opt-in)

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::any::Any;

use crate::nexus::effect::EffectMarker;
use crate::nexus::row::Row;

// =============================================================================
// Effect Marker
// =============================================================================

/// Bit flag for Incremental effect (using a high bit to avoid conflicts).
pub const INCREMENTAL_BIT: u128 = 1 << 32;

/// The Incremental effect marker type.
#[derive(Copy, Clone, Debug)]
pub struct IncrementalEffect;

impl EffectMarker for IncrementalEffect {
    const BIT: u128 = INCREMENTAL_BIT;
    const NAME: &'static str = "Incremental";
}

/// Type alias for a row containing only Incremental.
pub type IncrementalRow = Row<INCREMENTAL_BIT>;

// =============================================================================
// Input and Memo Identifiers
// =============================================================================

/// Identifier for a tracked input.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct InputId {
    /// The name of this input.
    name: String,
    /// Generation number (reserved for versioned IDs).
    ///
    /// Note: currently **always 0** — live invalidation generations are
    /// tracked separately in `IncrementalContext::generations`, keyed by
    /// this ID. Nothing ever sets a non-zero value here yet.
    generation: u64,
}

impl InputId {
    /// Create a new input ID.
    pub fn new(name: impl Into<String>) -> Self {
        InputId {
            name: name.into(),
            generation: 0,
        }
    }

    /// Get the input name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the generation number.
    pub fn generation(&self) -> u64 {
        self.generation
    }
}

/// Identifier for a memoized computation.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MemoKey {
    /// The name/key of this memoized computation.
    name: String,
}

impl MemoKey {
    /// Create a new memo key.
    pub fn new(name: impl Into<String>) -> Self {
        MemoKey { name: name.into() }
    }

    /// Get the memo name.
    pub fn name(&self) -> &str {
        &self.name
    }
}

// =============================================================================
// Dependency Tracking
// =============================================================================

/// A dependency on a tracked input.
#[derive(Clone, Debug)]
struct Dependency {
    /// The input this depends on.
    input_id: InputId,
    /// The generation when this dependency was recorded.
    recorded_generation: u64,
}

impl Dependency {
    /// Check if this dependency is still valid.
    fn is_valid(&self, current_generation: u64) -> bool {
        self.recorded_generation == current_generation
    }
}

/// Cached result with its dependencies.
struct CachedResult {
    /// The cached value (boxed for type erasure).
    value: Box<dyn Any>,
    /// Dependencies that were read during computation.
    dependencies: Vec<Dependency>,
}

// =============================================================================
// Incremental Context
// =============================================================================

/// Context for incremental computation.
///
/// Tracks inputs, their values, and cached computation results.
pub struct IncrementalContext {
    /// Tracked inputs and their values.
    inputs: BTreeMap<InputId, Box<dyn Any>>,
    /// Current generation for each input (for invalidation).
    generations: BTreeMap<InputId, u64>,
    /// Cached computation results.
    cache: BTreeMap<MemoKey, CachedResult>,
    /// Dependencies being recorded for current computation.
    current_dependencies: Vec<Dependency>,
    /// Whether we're currently recording dependencies.
    recording: bool,
    /// Statistics for debugging.
    stats: IncrementalStats,
}

/// Statistics for incremental computation.
#[derive(Clone, Debug, Default)]
pub struct IncrementalStats {
    /// Number of cache hits.
    pub cache_hits: usize,
    /// Number of cache misses (recomputations).
    pub cache_misses: usize,
    /// Number of invalidations.
    pub invalidations: usize,
}

impl IncrementalContext {
    /// Create a new incremental context.
    pub fn new() -> Self {
        IncrementalContext {
            inputs: BTreeMap::new(),
            generations: BTreeMap::new(),
            cache: BTreeMap::new(),
            current_dependencies: Vec::with_capacity(8),
            recording: false,
            stats: IncrementalStats::default(),
        }
    }

    /// Create a tracked input with an initial value.
    pub fn create_input<T: Clone + 'static>(
        &mut self,
        name: impl Into<String>,
        value: T,
    ) -> InputId {
        let id = InputId::new(name);
        self.inputs.insert(id.clone(), Box::new(value));
        self.generations.insert(id.clone(), 0);
        id
    }

    /// Read a tracked input's value.
    ///
    /// Records a dependency if we're inside a memoized computation.
    pub fn read_input<T: Clone + 'static>(&mut self, id: &InputId) -> Option<T> {
        // Record dependency if we're inside a memo
        if self.recording {
            let generation = self.generations.get(id).copied().unwrap_or(0);
            self.current_dependencies.push(Dependency {
                input_id: id.clone(),
                recorded_generation: generation,
            });
        }

        // Get the value
        self.inputs
            .get(id)
            .and_then(|v| v.downcast_ref::<T>())
            .cloned()
    }

    /// Set a tracked input's value, invalidating dependent computations.
    pub fn set_input<T: Clone + 'static>(&mut self, id: &InputId, value: T) {
        // Update the value
        self.inputs.insert(id.clone(), Box::new(value));

        // Increment generation to invalidate dependents
        let generation = self.generations.entry(id.clone()).or_insert(0);
        *generation += 1;

        self.stats.invalidations += 1;

        // Note: We don't eagerly invalidate cache entries.
        // Instead, we check validity lazily in `memo`.
    }

    /// Execute a computation with memoization.
    ///
    /// If the result is cached and all dependencies are still valid,
    /// returns the cached result. Otherwise, recomputes and caches.
    pub fn memo<T: Clone + 'static, F>(&mut self, key: impl Into<String>, compute: F) -> T
    where
        F: FnOnce(&mut Self) -> T,
    {
        let memo_key = MemoKey::new(key);

        // Check if we have a valid cached result
        if let Some(cached) = self.cache.get(&memo_key) {
            let all_valid = cached.dependencies.iter().all(|dep| {
                self.generations
                    .get(&dep.input_id)
                    .is_some_and(|&g| dep.is_valid(g))
            });

            if all_valid && let Some(value) = cached.value.downcast_ref::<T>() {
                self.stats.cache_hits += 1;
                return value.clone();
            }
        }

        // Cache miss - need to recompute
        self.stats.cache_misses += 1;

        // Start recording dependencies
        let was_recording = self.recording;
        let old_deps = core::mem::take(&mut self.current_dependencies);
        self.recording = true;

        // Execute computation
        let result = compute(self);

        // Stop recording and save dependencies
        self.recording = was_recording;
        let new_deps = core::mem::replace(&mut self.current_dependencies, old_deps);

        // Cache the result
        self.cache.insert(
            memo_key,
            CachedResult {
                value: Box::new(result.clone()),
                dependencies: new_deps,
            },
        );

        result
    }

    /// Invalidate all cached results for a specific input.
    ///
    /// This is an alternative to `set_input` when you want to invalidate
    /// without changing the value.
    pub fn invalidate(&mut self, id: &InputId) {
        if let Some(generation) = self.generations.get_mut(id) {
            *generation += 1;
            self.stats.invalidations += 1;
        }
    }

    /// Clear all cached results.
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get computation statistics.
    pub fn stats(&self) -> &IncrementalStats {
        &self.stats
    }

    /// Reset statistics.
    pub fn reset_stats(&mut self) {
        self.stats = IncrementalStats::default();
    }

    /// Get the number of cached entries.
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }

    /// Get the number of tracked inputs.
    pub fn input_count(&self) -> usize {
        self.inputs.len()
    }
}

impl Default for IncrementalContext {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Incremental Computation Type
// =============================================================================

/// An incremental computation that tracks dependencies.
///
/// This wraps a computation that reads from an `IncrementalContext`
/// and can be memoized with automatic dependency tracking.
pub struct IncrementalComputation<A> {
    /// The computation function.
    compute: Box<dyn FnOnce(&mut IncrementalContext) -> A>,
}

impl<A: 'static> IncrementalComputation<A> {
    /// Create a new incremental computation.
    #[inline]
    pub fn new<F: FnOnce(&mut IncrementalContext) -> A + 'static>(f: F) -> Self {
        IncrementalComputation {
            compute: Box::new(f),
        }
    }

    /// Run the computation with a context.
    #[inline]
    pub fn run(self, ctx: &mut IncrementalContext) -> A {
        (self.compute)(ctx)
    }

    /// Pure value in incremental context.
    pub fn pure(value: A) -> Self
    where
        A: Clone,
    {
        IncrementalComputation::new(move |_| value)
    }

    /// Map over the result.
    pub fn map<B: 'static, F: FnOnce(A) -> B + 'static>(self, f: F) -> IncrementalComputation<B> {
        IncrementalComputation::new(move |ctx| {
            let a = (self.compute)(ctx);
            f(a)
        })
    }

    /// Chain two incremental computations.
    pub fn and_then<B: 'static, F: FnOnce(A) -> IncrementalComputation<B> + 'static>(
        self,
        f: F,
    ) -> IncrementalComputation<B> {
        IncrementalComputation::new(move |ctx| {
            let a = (self.compute)(ctx);
            f(a).run(ctx)
        })
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Create an incremental computation that reads an input.
pub fn read_input<T: Clone + 'static>(id: InputId) -> IncrementalComputation<Option<T>> {
    IncrementalComputation::new(move |ctx| ctx.read_input::<T>(&id))
}

/// Create a memoized incremental computation.
pub fn memo<T: Clone + 'static, F>(
    key: impl Into<String> + 'static,
    compute: F,
) -> IncrementalComputation<T>
where
    F: FnOnce(&mut IncrementalContext) -> T + 'static,
{
    let key_string = key.into();
    IncrementalComputation::new(move |ctx| ctx.memo(key_string, compute))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_read_input() {
        let mut ctx = IncrementalContext::new();
        let id = ctx.create_input("x", 42);

        let value = ctx.read_input::<i32>(&id);
        assert_eq!(value, Some(42));
    }

    #[test]
    fn test_set_input() {
        let mut ctx = IncrementalContext::new();
        let id = ctx.create_input("x", 42);

        ctx.set_input(&id, 100);
        let value = ctx.read_input::<i32>(&id);
        assert_eq!(value, Some(100));
    }

    #[test]
    fn test_memo_caches_result() {
        let mut ctx = IncrementalContext::new();
        let id = ctx.create_input("x", 10);

        // First call computes
        let result1 = ctx.memo("doubled", |ctx| {
            let x = ctx
                .read_input::<i32>(&id)
                .expect("input 'x' was just registered in this context");
            x * 2
        });
        assert_eq!(result1, 20);
        assert_eq!(ctx.stats().cache_misses, 1);
        assert_eq!(ctx.stats().cache_hits, 0);

        // Second call uses cache
        let result2 = ctx.memo("doubled", |ctx| {
            let x = ctx
                .read_input::<i32>(&id)
                .expect("input 'x' was just registered in this context");
            x * 2
        });
        assert_eq!(result2, 20);
        assert_eq!(ctx.stats().cache_misses, 1);
        assert_eq!(ctx.stats().cache_hits, 1);
    }

    #[test]
    fn test_invalidation_triggers_recompute() {
        let mut ctx = IncrementalContext::new();
        let id = ctx.create_input("x", 10);

        // First computation
        let result1 = ctx.memo("doubled", |ctx| {
            let x = ctx
                .read_input::<i32>(&id)
                .expect("input 'x' should exist and be i32");
            x * 2
        });
        assert_eq!(result1, 20);

        // Change input
        ctx.set_input(&id, 50);

        // Should recompute
        let result2 = ctx.memo("doubled", |ctx| {
            let x = ctx
                .read_input::<i32>(&id)
                .expect("input 'x' should exist and be i32 after update");
            x * 2
        });
        assert_eq!(result2, 100);
        assert_eq!(ctx.stats().cache_misses, 2); // Both computed
    }

    #[test]
    fn test_independent_inputs() {
        let mut ctx = IncrementalContext::new();
        let x = ctx.create_input("x", 10);
        let y = ctx.create_input("y", 5);

        // Computation depends only on x
        let result1 = ctx.memo("x_doubled", |ctx| {
            let val = ctx
                .read_input::<i32>(&x)
                .expect("input 'x' should exist and be i32");
            val * 2
        });
        assert_eq!(result1, 20);

        // Changing y shouldn't invalidate x_doubled
        ctx.set_input(&y, 100);

        let result2 = ctx.memo("x_doubled", |ctx| {
            let val = ctx
                .read_input::<i32>(&x)
                .expect("input 'x' should still exist after updating y");
            val * 2
        });
        assert_eq!(result2, 20);
        assert_eq!(ctx.stats().cache_hits, 1); // Should be cache hit
    }

    #[test]
    fn test_chained_dependencies() {
        let mut ctx = IncrementalContext::new();
        let x = ctx.create_input("x", 10);

        // Computation A depends on x
        let a = ctx.memo("a", |ctx| {
            let val = ctx
                .read_input::<i32>(&x)
                .expect("input 'x' should be registered in context");
            val + 1
        });
        assert_eq!(a, 11);

        // Computation B depends on A (indirectly on x)
        // Note: In this MVP, B must re-read x to establish dependency
        let b = ctx.memo("b", |ctx| {
            let val = ctx
                .read_input::<i32>(&x)
                .expect("input 'x' should be registered in context");
            (val + 1) * 2 // Same as a * 2
        });
        assert_eq!(b, 22);

        // Change x
        ctx.set_input(&x, 20);

        // Both should recompute
        let a2 = ctx.memo("a", |ctx| {
            let val = ctx
                .read_input::<i32>(&x)
                .expect("input 'x' should remain registered after set_input");
            val + 1
        });
        assert_eq!(a2, 21);

        let b2 = ctx.memo("b", |ctx| {
            let val = ctx
                .read_input::<i32>(&x)
                .expect("input 'x' should remain registered after set_input");
            (val + 1) * 2
        });
        assert_eq!(b2, 42);
    }

    #[test]
    fn test_clear_cache() {
        let mut ctx = IncrementalContext::new();
        let id = ctx.create_input("x", 42);

        ctx.memo("test", |ctx| {
            ctx.read_input::<i32>(&id)
                .expect("input 'x' should be registered in test_clear_cache")
        });
        assert_eq!(ctx.cache_size(), 1);

        ctx.clear_cache();
        assert_eq!(ctx.cache_size(), 0);
    }

    #[test]
    fn test_incremental_computation_type() {
        let mut ctx = IncrementalContext::new();
        let id = ctx.create_input("x", 5);

        let comp = IncrementalComputation::new(move |ctx| {
            ctx.read_input::<i32>(&id)
                .expect("input 'x' was registered in this context")
                * 3
        });

        let result = comp.run(&mut ctx);
        assert_eq!(result, 15);
    }

    #[test]
    fn test_incremental_computation_map() {
        let mut ctx = IncrementalContext::new();
        let id = ctx.create_input("x", 10);

        let comp = IncrementalComputation::new(move |ctx| {
            ctx.read_input::<i32>(&id)
                .expect("input 'x' was registered and must be readable as i32")
        })
        .map(|x| x * 2)
        .map(|x| x + 1);

        let result = comp.run(&mut ctx);
        assert_eq!(result, 21); // (10 * 2) + 1
    }

    #[test]
    fn test_stats() {
        let mut ctx = IncrementalContext::new();
        let id = ctx.create_input("x", 1);

        assert_eq!(ctx.stats().cache_hits, 0);
        assert_eq!(ctx.stats().cache_misses, 0);
        assert_eq!(ctx.stats().invalidations, 0);

        ctx.memo("test", |ctx| ctx.read_input::<i32>(&id));
        assert_eq!(ctx.stats().cache_misses, 1);

        ctx.memo("test", |ctx| ctx.read_input::<i32>(&id));
        assert_eq!(ctx.stats().cache_hits, 1);

        ctx.set_input(&id, 2);
        assert_eq!(ctx.stats().invalidations, 1);

        ctx.reset_stats();
        assert_eq!(ctx.stats().cache_hits, 0);
    }

    /// Benchmark demonstrating recomputation avoidance.
    ///
    /// This test shows that cached computations are reused,
    /// avoiding redundant work.
    #[test]
    fn test_recomputation_avoidance_benchmark() {
        let mut ctx = IncrementalContext::new();

        // Create multiple inputs
        let x = ctx.create_input("x", 10);
        let y = ctx.create_input("y", 20);
        let z = ctx.create_input("z", 30);

        // Track computation count
        let mut compute_count = 0;

        // First computation
        let result1 = ctx.memo("expensive", |ctx| {
            compute_count += 1;
            let vx = ctx
                .read_input::<i32>(&x)
                .expect("input 'x' was created in this context");
            let vy = ctx
                .read_input::<i32>(&y)
                .expect("input 'y' was created in this context");
            let vz = ctx
                .read_input::<i32>(&z)
                .expect("input 'z' was created in this context");
            vx * 100 + vy * 10 + vz
        });
        assert_eq!(result1, 1230); // 10*100 + 20*10 + 30
        assert_eq!(compute_count, 1);

        // Run 10 more times - all should be cache hits
        for _ in 0..10 {
            let _ = ctx.memo("expensive", |ctx| {
                compute_count += 1;
                let vx = ctx
                    .read_input::<i32>(&x)
                    .expect("input 'x' was created in this context");
                let vy = ctx
                    .read_input::<i32>(&y)
                    .expect("input 'y' was created in this context");
                let vz = ctx
                    .read_input::<i32>(&z)
                    .expect("input 'z' was created in this context");
                vx * 100 + vy * 10 + vz
            });
        }
        // compute_count should still be 1 (all cache hits)
        assert_eq!(compute_count, 1);
        assert_eq!(ctx.stats().cache_hits, 10);

        // Change only z
        ctx.set_input(&z, 99);

        // Now should recompute
        let result2 = ctx.memo("expensive", |ctx| {
            compute_count += 1;
            let vx = ctx
                .read_input::<i32>(&x)
                .expect("input 'x' was created in this context");
            let vy = ctx
                .read_input::<i32>(&y)
                .expect("input 'y' was created in this context");
            let vz = ctx
                .read_input::<i32>(&z)
                .expect("input 'z' was created in this context");
            vx * 100 + vy * 10 + vz
        });
        assert_eq!(result2, 1299); // 10*100 + 20*10 + 99
        assert_eq!(compute_count, 2); // Only one recomputation

        // Verify we avoided 10 redundant computations
        assert!(ctx.stats().cache_hits >= 10);
    }
}
