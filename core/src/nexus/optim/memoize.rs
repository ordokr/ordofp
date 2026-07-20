//! Memoization Infrastructure for Idempotent Effects
//!
//! This module provides **explicit opt-in** memoization combinators for
//! caching results of idempotent computations. Users must explicitly
//! wrap computations with `memoize` or `Lazy` to enable caching.
//!
//! **Note:** There is no automatic memoization insertion. The type system
//! enforces that only idempotent effects can be memoized, but you must
//! explicitly use the memoization combinators.
//!
//! # Idempotent Effects
//!
//! An effect is idempotent if executing it multiple times produces
//! the same observable result. Examples:
//!
//! - `Pure`: No effects, always same result
//! - `Reader`: Reading environment doesn't change it
//!
//! # Non-Idempotent Effects (Cannot Memoize)
//!
//! - `Writer`: Each execution appends to log
//! - `State`: Each execution may modify state
//! - `IO`: External world may change
//!
//! # Example
//!
//! ```rust
//! use ordofp_core::nexus::optim::memoize;
//!
//! // Create a memoized computation
//! let mut expensive = memoize(|key: &i32| {
//!     // Simulated expensive computation
//!     *key * *key
//! });
//!
//! // First call computes and caches
//! let result1 = expensive.call(&7);
//!
//! // Second call returns cached result
//! let result2 = expensive.call(&7); // Cache hit!
//! assert_eq!(result1, result2);
//! assert_eq!(expensive.cache_size(), 1);
//! ```

use alloc::boxed::Box;
use alloc::collections::VecDeque;
use core::marker::PhantomData;

use crate::nexus::row::EffectRow;

// =============================================================================
// Cache Strategy
// =============================================================================

/// Strategy for cache eviction and management.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CacheStrategy {
    /// No caching (useful for testing).
    None,
    /// Keep all cached values forever.
    Unbounded,
    /// Least Recently Used eviction.
    LRU(usize),
    /// Time-to-live based eviction (in seconds).
    ///
    /// **Note:** this crate is `no_std` and has no clock source, so the TTL
    /// value is currently **not enforced**. The strategy degrades to
    /// FIFO-capped behavior (default capacity 1000) pending a clock source.
    TTL(u64),
    /// Bounded size with FIFO eviction.
    FIFO(usize),
}

impl Default for CacheStrategy {
    fn default() -> Self {
        CacheStrategy::LRU(1000)
    }
}

// =============================================================================
// Cache Entry
// =============================================================================

/// A cached value with metadata.
struct CacheEntry<V> {
    /// The cached value.
    value: V,
    /// Monotonic access stamp (from the cache's counter); higher = more recent.
    last_access: u64,
}

impl<V> CacheEntry<V> {
    fn new(value: V, last_access: u64) -> Self {
        CacheEntry { value, last_access }
    }
}

// =============================================================================
// Simple Cache Implementation
// =============================================================================

/// A simple cache for memoized values.
///
/// A deliberately small FIFO cache; swap in a concurrent cache if
/// contention ever matters.
pub struct Cache<K, V> {
    // VecDeque so FIFO eviction is a correct O(1) pop_front. A plain Vec with
    // swap_remove(0) would evict the oldest but move the newest into slot 0,
    // corrupting FIFO order on every subsequent eviction.
    entries: VecDeque<(K, CacheEntry<V>)>,
    strategy: CacheStrategy,
    capacity: usize,
    /// Monotonic counter, incremented on every get/insert to stamp recency.
    clock: u64,
}

impl<K: Eq + Clone, V: Clone> Cache<K, V> {
    /// Create a new cache with the given strategy.
    pub fn new(strategy: CacheStrategy) -> Self {
        let capacity = match strategy {
            CacheStrategy::None => 0,
            CacheStrategy::Unbounded => usize::MAX,
            CacheStrategy::LRU(n) | CacheStrategy::FIFO(n) => n,
            CacheStrategy::TTL(_) => 1000, // Default capacity for TTL
        };
        // Pre-allocate with the known capacity so that early insertions never
        // reallocate. For Unbounded we cap the initial reservation to avoid
        // asking the allocator for usize::MAX bytes.
        let initial = if capacity == usize::MAX { 16 } else { capacity };
        Cache {
            entries: VecDeque::with_capacity(initial),
            strategy,
            capacity,
            clock: 0,
        }
    }

    /// Advance the monotonic clock and return the new stamp.
    #[inline]
    fn tick(&mut self) -> u64 {
        self.clock += 1;
        self.clock
    }

    /// Get a value from the cache.
    #[inline]
    pub fn get(&mut self, key: &K) -> Option<V> {
        if matches!(self.strategy, CacheStrategy::None) {
            return None;
        }

        let now = self.tick();
        for (k, entry) in &mut self.entries {
            if k == key {
                entry.last_access = now;
                return Some(entry.value.clone());
            }
        }
        None
    }

    /// Insert a value into the cache.
    #[inline]
    pub fn insert(&mut self, key: K, value: V) {
        if matches!(self.strategy, CacheStrategy::None) {
            return;
        }

        let now = self.tick();
        // Check if key already exists — update in place to avoid eviction churn.
        for (k, entry) in &mut self.entries {
            if *k == key {
                entry.value = value;
                entry.last_access = now;
                return;
            }
        }

        // Evict if at capacity
        if self.entries.len() >= self.capacity {
            self.evict();
        }

        self.entries.push_back((key, CacheEntry::new(value, now)));
    }

    /// Evict an entry based on strategy.
    #[inline]
    fn evict(&mut self) {
        match self.strategy {
            CacheStrategy::FIFO(_) => {
                // Oldest entry is at the front; pop_front is O(1) and preserves
                // FIFO order for all later evictions.
                self.entries.pop_front();
            }
            CacheStrategy::LRU(_) => {
                // Find the least recently used entry (lowest monotonic access
                // stamp) and remove it. Order is irrelevant for LRU (we always
                // rescan by last_access), so the O(1) swap_remove_back is safe.
                if let Some(idx) = self
                    .entries
                    .iter()
                    .enumerate()
                    .min_by_key(|(_, (_, e))| e.last_access)
                    .map(|(i, _)| i)
                {
                    self.entries.swap_remove_back(idx);
                }
            }
            _ => {
                // TTL (no clock source in no_std — degrades to FIFO-capped
                // behavior) and any other strategy: evict oldest (front) in
                // O(1), preserving insertion order.
                self.entries.pop_front();
            }
        }
    }

    /// Clear all cached entries.
    #[inline]
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Get the number of cached entries.
    #[inline]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the cache is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

// =============================================================================
// Memoized Computation
// =============================================================================

/// A memoized computation wrapper.
///
/// Wraps a function and caches its results based on input keys.
pub struct Memoized<K, V, F> {
    /// The underlying function.
    func: F,
    /// The cache.
    cache: Cache<K, V>,
    /// Phantom data for type safety.
    _marker: PhantomData<(K, V)>,
}

impl<K, V, F> Memoized<K, V, F>
where
    K: Eq + Clone,
    V: Clone,
    F: Fn(&K) -> V,
{
    /// Create a new memoized function with default caching strategy.
    #[inline]
    pub fn new(func: F) -> Self {
        Memoized {
            func,
            cache: Cache::new(CacheStrategy::default()),
            _marker: PhantomData,
        }
    }

    /// Create a new memoized function with specified caching strategy.
    #[inline]
    pub fn with_strategy(func: F, strategy: CacheStrategy) -> Self {
        Memoized {
            func,
            cache: Cache::new(strategy),
            _marker: PhantomData,
        }
    }

    /// Call the memoized function.
    #[inline]
    pub fn call(&mut self, key: &K) -> V {
        // Check cache first
        if let Some(value) = self.cache.get(key) {
            return value;
        }

        // Compute and cache
        let value = (self.func)(key);
        self.cache.insert(key.clone(), value.clone());
        value
    }

    /// Clear the cache.
    #[inline]
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get cache statistics.
    #[inline]
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }
}

// =============================================================================
// Memoization Constructor
// =============================================================================

/// Create a memoized function with default caching.
///
/// # Example
///
/// ```rust
/// use ordofp_core::nexus::optim::memoize;
///
/// fn fib(n: u64) -> u64 {
///     if n <= 1 { n } else { fib(n - 1) + fib(n - 2) }
/// }
///
/// // Note: `memoize` only caches the *outer* call for a given key; unlike a
/// // self-referential memo table, the recursive calls to `fib` inside the
/// // closure are not themselves memoized.
/// let mut memo_fib = memoize(|n: &u64| fib(*n));
/// assert_eq!(memo_fib.call(&10), 55);
/// assert_eq!(memo_fib.call(&10), 55); // cache hit
/// assert_eq!(memo_fib.cache_size(), 1);
/// ```
#[inline]
pub fn memoize<K, V, F>(func: F) -> Memoized<K, V, F>
where
    K: Eq + Clone,
    V: Clone,
    F: Fn(&K) -> V,
{
    Memoized::new(func)
}

/// Create a memoized function with specified strategy.
#[inline]
pub fn memoize_with<K, V, F>(func: F, strategy: CacheStrategy) -> Memoized<K, V, F>
where
    K: Eq + Clone,
    V: Clone,
    F: Fn(&K) -> V,
{
    Memoized::with_strategy(func, strategy)
}

// =============================================================================
// Effect-Aware Memoization
// =============================================================================

/// Marker trait for effects that can be safely memoized.
pub trait MemoizeSafe: EffectRow {}

/// Pure effects are safe to memoize.
impl MemoizeSafe for crate::nexus::row::Pure {}

/// Reader effects are safe to memoize (within same environment).
impl MemoizeSafe for crate::nexus::row::Row<{ crate::nexus::row::READER_BIT }> {}

/// A memoized effectful computation.
///
/// Only available for effects that implement `MemoizeSafe`.
pub struct MemoizedEff<R: MemoizeSafe, K, V, F> {
    inner: Memoized<K, V, F>,
    _effect: PhantomData<R>,
}

impl<R, K, V, F> MemoizedEff<R, K, V, F>
where
    R: MemoizeSafe,
    K: Eq + Clone,
    V: Clone,
    F: Fn(&K) -> V,
{
    /// Create a new memoized effectful computation.
    pub fn new(func: F) -> Self {
        MemoizedEff {
            inner: Memoized::new(func),
            _effect: PhantomData,
        }
    }

    /// Call the memoized computation.
    #[inline]
    pub fn call(&mut self, key: &K) -> V {
        self.inner.call(key)
    }
}

// =============================================================================
// Lazy Memoization
// =============================================================================

/// A lazily-evaluated, memoized value.
///
/// The computation runs at most once, when first accessed.
pub struct Lazy<A, F> {
    /// The computation to run.
    compute: Option<F>,
    /// The cached result.
    value: Option<A>,
}

impl<A, F> Lazy<A, F>
where
    F: FnOnce() -> A,
{
    /// Create a new lazy value.
    pub fn new(compute: F) -> Self {
        Lazy {
            compute: Some(compute),
            value: None,
        }
    }

    /// Force evaluation and get the value.
    ///
    /// Runs the computation on first call and caches the result; later
    /// calls return the cached value without re-running it.
    ///
    /// # Panics
    ///
    /// Panics only if the internal "computed after forcing" invariant is
    /// violated, which would indicate a bug in this crate. If the
    /// computation itself panics on first force, that panic propagates and
    /// leaves the value permanently unset — a subsequent `force` then trips
    /// this invariant instead of re-running the consumed closure.
    #[inline]
    pub fn force(&mut self) -> &A
    where
        A: Clone,
    {
        if self.value.is_none()
            && let Some(f) = self.compute.take()
        {
            self.value = Some(f());
        }
        self.value.as_ref().expect("Lazy value should be computed")
    }

    /// Check if the value has been computed.
    #[inline]
    pub fn is_computed(&self) -> bool {
        self.value.is_some()
    }
}

/// Create a lazy value.
#[inline]
pub fn lazy<A, F: FnOnce() -> A>(compute: F) -> Lazy<A, F> {
    Lazy::new(compute)
}

// =============================================================================
// Thunk (Boxed Lazy)
// =============================================================================

/// A boxed lazy computation that can be stored and passed around.
pub struct Thunk<A> {
    inner: Box<dyn FnOnce() -> A>,
}

impl<A: 'static> Thunk<A> {
    /// Create a new thunk.
    pub fn new<F: FnOnce() -> A + 'static>(compute: F) -> Self {
        Thunk {
            inner: Box::new(compute),
        }
    }

    /// Force evaluation and take the value.
    #[inline]
    pub fn force(self) -> A {
        (self.inner)()
    }
}

/// Create a thunk.
#[inline]
pub fn thunk<A: 'static, F: FnOnce() -> A + 'static>(compute: F) -> Thunk<A> {
    Thunk::new(compute)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::String;

    #[test]
    fn test_cache_insert_get() {
        let mut cache: Cache<i32, String> = Cache::new(CacheStrategy::Unbounded);
        cache.insert(1, "one".into());
        cache.insert(2, "two".into());

        assert_eq!(cache.get(&1), Some("one".into()));
        assert_eq!(cache.get(&2), Some("two".into()));
        assert_eq!(cache.get(&3), None);
    }

    #[test]
    fn test_cache_lru_eviction() {
        let mut cache: Cache<i32, i32> = Cache::new(CacheStrategy::LRU(2));
        cache.insert(1, 10);
        cache.insert(2, 20);

        // Access 1 to make it more recently used
        let _ = cache.get(&1);

        // Insert 3, should evict 2 (least hits)
        cache.insert(3, 30);

        assert_eq!(cache.get(&1), Some(10));
        assert_eq!(cache.get(&2), None); // Evicted
        assert_eq!(cache.get(&3), Some(30));
    }

    #[test]
    fn test_cache_lru_evicts_least_recent_not_least_hit() {
        // Regression: LRU must evict by recency, not frequency (LFU).
        // Key 1 is hit often but LONG AGO; key 2 is hit once but LAST.
        // True LRU evicts 1; an LFU implementation would evict 2.
        let mut cache: Cache<i32, i32> = Cache::new(CacheStrategy::LRU(2));
        cache.insert(1, 10);
        cache.insert(2, 20);

        let _ = cache.get(&1);
        let _ = cache.get(&1);
        let _ = cache.get(&2); // 2 is now the most recently used

        cache.insert(3, 30);

        assert_eq!(cache.get(&1), None, "least recently used must be evicted");
        assert_eq!(cache.get(&2), Some(20), "most recently used must survive");
        assert_eq!(cache.get(&3), Some(30));
    }

    #[test]
    fn test_memoized_caching() {
        let mut memo = Memoized::new(|x: &i32| *x * 2);

        // First calls compute
        assert_eq!(memo.call(&5), 10);
        assert_eq!(memo.call(&3), 6);

        // Repeated calls use cache
        assert_eq!(memo.call(&5), 10);
        assert_eq!(memo.call(&3), 6);

        assert_eq!(memo.cache_size(), 2);
    }

    #[test]
    fn test_lazy_evaluation() {
        let mut lazy_val = Lazy::new(|| 42);

        // Not computed yet
        assert!(!lazy_val.is_computed());

        // Force evaluation
        let value = lazy_val.force();
        assert_eq!(*value, 42);
        assert!(lazy_val.is_computed());
    }

    #[test]
    fn test_thunk() {
        let t = thunk(|| 100 + 23);
        assert_eq!(t.force(), 123);
    }

    #[test]
    fn test_cache_none_strategy() {
        let mut cache: Cache<i32, i32> = Cache::new(CacheStrategy::None);
        cache.insert(1, 10);
        assert_eq!(cache.get(&1), None); // Nothing cached
    }

    #[test]
    fn test_memoize_helper() {
        let mut memo = memoize(|x: &i32| x * x);
        assert_eq!(memo.call(&4), 16);
        assert_eq!(memo.call(&5), 25);
        assert_eq!(memo.call(&4), 16); // Cached
    }

    #[test]
    fn test_memoize_with_strategy() {
        let mut memo = memoize_with(|x: &i32| *x + 1, CacheStrategy::FIFO(2));
        assert_eq!(memo.call(&1), 2);
        assert_eq!(memo.call(&2), 3);
        assert_eq!(memo.call(&3), 4); // Evicts 1
        assert_eq!(memo.cache_size(), 2);
    }
}
