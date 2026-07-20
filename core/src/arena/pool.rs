//! Object Pool for Reusable Allocations
//!
//! Provides object pooling for frequently allocated/deallocated objects.

use alloc::vec::Vec;
use core::cell::RefCell;
use core::mem::MaybeUninit;
use core::ops::{Deref, DerefMut};

/// Default maximum number of objects retained in a pool.
const DEFAULT_MAX_POOL_SIZE: usize = 1024;

// =============================================================================
// Object Pool
// =============================================================================

/// A pool of reusable objects.
///
/// Objects are borrowed from the pool and automatically returned when dropped.
/// This reduces allocation overhead for frequently used objects.
///
/// # Example
///
/// ```rust
/// use ordofp_core::arena::Pool;
///
/// let pool: Pool<Vec<i32>> = Pool::new(|| Vec::with_capacity(100));
///
/// {
///     let mut vec = pool.get();
///     vec.push(1);
///     vec.push(2);
///     // vec is returned to pool when dropped
/// }
///
/// {
///     let vec = pool.get();
///     // Reuses the same Vec (but cleared)
/// }
/// ```
pub struct Pool<T> {
    /// Available objects
    available: RefCell<Vec<T>>,
    /// Factory for creating new objects
    factory: fn() -> T,
    /// Reset function called when object is returned
    reset: fn(&mut T),
    /// Maximum pool size
    max_size: usize,
}

impl<T> Pool<T> {
    /// Create a new pool with the given factory.
    pub fn new(factory: fn() -> T) -> Self {
        Self::with_reset(factory, |_| {})
    }

    /// Create a pool with a custom reset function.
    pub fn with_reset(factory: fn() -> T, reset: fn(&mut T)) -> Self {
        Pool {
            available: RefCell::new(Vec::new()),
            factory,
            reset,
            max_size: DEFAULT_MAX_POOL_SIZE,
        }
    }

    /// Set the maximum pool size.
    pub fn with_max_size(mut self, max_size: usize) -> Self {
        self.max_size = max_size;
        self
    }

    /// Pre-populate the pool with objects.
    pub fn with_initial(self, count: usize) -> Self {
        let mut available = self.available.borrow_mut();
        for _ in 0..count.min(self.max_size) {
            available.push((self.factory)());
        }
        drop(available);
        self
    }

    /// Get an object from the pool.
    pub fn get(&self) -> Pooled<'_, T> {
        let obj = self
            .available
            .borrow_mut()
            .pop()
            .unwrap_or_else(|| (self.factory)());
        Pooled {
            pool: self,
            value: Some(obj),
        }
    }

    /// Return an object to the pool.
    fn return_object(&self, mut obj: T) {
        (self.reset)(&mut obj);

        let mut available = self.available.borrow_mut();
        if available.len() < self.max_size {
            available.push(obj);
        }
        // Otherwise, object is dropped
    }

    /// Get the number of available objects.
    pub fn available(&self) -> usize {
        self.available.borrow().len()
    }

    /// Clear the pool.
    pub fn clear(&self) {
        self.available.borrow_mut().clear();
    }
}

/// A pooled object that returns to the pool when dropped.
pub struct Pooled<'a, T> {
    pool: &'a Pool<T>,
    value: Option<T>,
}

impl<T> Deref for Pooled<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value.as_ref().unwrap()
    }
}

impl<T> DerefMut for Pooled<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value.as_mut().unwrap()
    }
}

impl<T> Drop for Pooled<'_, T> {
    fn drop(&mut self) {
        if let Some(obj) = self.value.take() {
            self.pool.return_object(obj);
        }
    }
}

// =============================================================================
// Typed Pool
// =============================================================================

/// A pool for a specific type with type-safe operations.
pub struct TypedPool<T, const N: usize> {
    /// Storage for pooled objects
    storage: RefCell<[MaybeUninit<T>; N]>,
    /// Bitmap of available slots (bit set = slot holds an object ready to give out)
    available: RefCell<u64>,
    /// Bitmap of claimed slots (bit set = slot has been assigned, either available or checked out)
    allocated: RefCell<u64>,
    /// Factory function
    factory: fn() -> T,
}

impl<T, const N: usize> TypedPool<T, N> {
    /// Create a new typed pool.
    ///
    /// Note: N must be <= 64 for the bitmap to work.
    ///
    /// # Panics
    ///
    /// Panics if the const parameter `N` exceeds 64, since slot occupancy is
    /// tracked in a single `u64` bitmap.
    pub fn new(factory: fn() -> T) -> Self {
        assert!(N <= 64, "TypedPool size must be <= 64");

        // Lazy initialization: slots are claimed and filled on first use.
        TypedPool {
            storage: RefCell::new([const { MaybeUninit::uninit() }; N]),
            available: RefCell::new(0),
            allocated: RefCell::new(0),
            factory,
        }
    }

    /// Get an object from the pool.
    pub fn get(&self) -> TypedPooled<'_, T, N> {
        let mut available = self.available.borrow_mut();

        // Fast path: reuse an object already in the pool.
        if *available != 0 {
            let slot = available.trailing_zeros() as usize;
            *available &= !(1 << slot);
            drop(available);

            let storage = self.storage.borrow();
            // SAFETY: `slot` was derived from `available.trailing_zeros()` and we
            // verified `*available != 0`, so `slot < N <= 64` is a valid index.
            // The `available` bitmap only has a bit set for slots that were
            // previously initialized (written by the factory and returned to the
            // pool via `release`).  We cleared the bit above before this read,
            // so no other code path can observe the same slot as initialized until
            // the value is returned to the pool again.
            let value = unsafe { storage[slot].as_ptr().read() };

            return TypedPooled {
                pool: self,
                slot: Some(slot),
                value: Some(value),
            };
        }
        drop(available);

        // Slow path: claim a new slot (lazy init) or create a transient.
        let mut allocated = self.allocated.borrow_mut();
        let all_mask: u64 = if N == 64 { !0 } else { (1u64 << N) - 1 };
        let free = all_mask & !*allocated;
        if free != 0 {
            let slot = free.trailing_zeros() as usize;
            *allocated |= 1 << slot;
            drop(allocated);
            // Storage at this slot is uninit; value comes from the factory.
            TypedPooled {
                pool: self,
                slot: Some(slot),
                value: Some((self.factory)()),
            }
        } else {
            drop(allocated);
            // All N slots are currently checked out. Create a transient with no slot.
            TypedPooled {
                pool: self,
                slot: None,
                value: Some((self.factory)()),
            }
        }
    }

    /// Return an object to the pool.
    fn return_object(&self, slot: Option<usize>, obj: T) {
        if let Some(idx) = slot {
            // Return to the reserved slot.
            // We know this slot is reserved for us (bit is 0), so we can safely write to it.
            // Note: idx comes from get(), which ensures idx < N.
            let mut storage = self.storage.borrow_mut();
            storage[idx] = MaybeUninit::new(obj);

            let mut available = self.available.borrow_mut();
            *available |= 1 << idx;
        } else {
            // Overflow object.
            // Since the pool has fixed size N, and all N slots are either:
            // 1. Occupied by a value in the pool (bit 1)
            // 2. Reserved by a TypedPooled handle (bit 0)
            // There is no space for an overflow object. We must drop it.
            drop(obj);
        }
    }
}

impl<T, const N: usize> Drop for TypedPool<T, N> {
    fn drop(&mut self) {
        let storage = self.storage.get_mut();
        let available = self.available.get_mut();

        // Drop all available (initialized and returned) objects
        for (i, item) in storage.iter_mut().enumerate() {
            if (*available & (1 << i)) != 0 {
                // SAFETY: We checked the `available` bitmap. If bit `i` is set,
                // the slot at index `i` is guaranteed to contain a fully initialized
                // object that has been returned to the pool and not currently checked out.
                unsafe {
                    item.assume_init_drop();
                }
            }
        }
    }
}

/// A value from a typed pool.
pub struct TypedPooled<'a, T, const N: usize> {
    pool: &'a TypedPool<T, N>,
    slot: Option<usize>,
    value: Option<T>,
}

impl<T, const N: usize> Deref for TypedPooled<'_, T, N> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value.as_ref().unwrap()
    }
}

impl<T, const N: usize> DerefMut for TypedPooled<'_, T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value.as_mut().unwrap()
    }
}

impl<T, const N: usize> Drop for TypedPooled<'_, T, N> {
    fn drop(&mut self) {
        if let Some(obj) = self.value.take() {
            self.pool.return_object(self.slot, obj);
        }
    }
}

// =============================================================================
// Continuation Pool
// =============================================================================

/// A specialized pool for continuation-sized objects.
///
/// This is optimized for the common case of small closures.
pub struct ContinuationPool {
    /// Pool for small continuations (up to 64 bytes)
    small: Pool<SmallContinuation>,
    /// Pool for medium continuations (up to 256 bytes)
    medium: Pool<MediumContinuation>,
}

/// Small continuation storage (64 bytes)
#[repr(align(8))]
pub struct SmallContinuation {
    data: [u8; 64],
}

impl SmallContinuation {
    /// Get the raw storage bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Get mutable access to the raw storage bytes.
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }
}

/// Medium continuation storage (256 bytes)
#[repr(align(8))]
pub struct MediumContinuation {
    data: [u8; 256],
}

impl MediumContinuation {
    /// Get the raw storage bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Get mutable access to the raw storage bytes.
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }
}

impl Default for SmallContinuation {
    fn default() -> Self {
        SmallContinuation { data: [0; 64] }
    }
}

impl Default for MediumContinuation {
    fn default() -> Self {
        MediumContinuation { data: [0; 256] }
    }
}

impl ContinuationPool {
    /// Create a new continuation pool.
    pub fn new() -> Self {
        ContinuationPool {
            small: Pool::new(SmallContinuation::default)
                .with_max_size(256)
                .with_initial(32),
            medium: Pool::new(MediumContinuation::default)
                .with_max_size(64)
                .with_initial(8),
        }
    }

    /// Get storage for a small continuation.
    pub fn get_small(&self) -> Pooled<'_, SmallContinuation> {
        self.small.get()
    }

    /// Get storage for a medium continuation.
    pub fn get_medium(&self) -> Pooled<'_, MediumContinuation> {
        self.medium.get()
    }

    /// Get pool statistics.
    pub fn stats(&self) -> PoolStats {
        PoolStats {
            small_available: self.small.available(),
            medium_available: self.medium.available(),
        }
    }
}

impl Default for ContinuationPool {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics for continuation pool.
#[derive(Debug, Clone)]
pub struct PoolStats {
    /// Available small slots.
    pub small_available: usize,
    /// Available medium slots.
    pub medium_available: usize,
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use core::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_pool_basic() {
        let pool: Pool<Vec<i32>> =
            Pool::with_reset(|| Vec::with_capacity(10), std::vec::Vec::clear);

        {
            let mut v = pool.get();
            v.push(1);
            v.push(2);
            v.push(3);
            assert_eq!(v.len(), 3);
        }

        assert_eq!(pool.available(), 1);

        {
            let v = pool.get();
            // Should be empty after reset
            assert_eq!(v.len(), 0);
            // But capacity preserved
            assert!(v.capacity() >= 10);
        }
    }

    #[test]
    fn test_pool_max_size() {
        let pool: Pool<i32> = Pool::new(|| 0).with_max_size(2);

        let a = pool.get();
        let b = pool.get();
        let c = pool.get();

        drop(a);
        drop(b);
        drop(c);

        // Only 2 should be kept
        assert_eq!(pool.available(), 2);
    }

    #[test]
    fn test_pool_initial() {
        let pool: Pool<i32> = Pool::new(|| 42).with_initial(5);
        assert_eq!(pool.available(), 5);
    }

    #[test]
    fn test_continuation_pool() {
        let pool = ContinuationPool::new();

        let small = pool.get_small();
        assert_eq!(small.data.len(), 64);

        let medium = pool.get_medium();
        assert_eq!(medium.data.len(), 256);

        let stats = pool.stats();
        assert!(stats.small_available > 0 || stats.medium_available > 0);
    }

    #[test]
    fn test_typed_pool_reuse() {
        static CREATED: AtomicUsize = AtomicUsize::new(0);
        CREATED.store(0, Ordering::SeqCst);

        // N=2 means max 2 items in pool.
        // Lazy initialization: new() does NOT call the factory.
        let pool = TypedPool::<i32, 2>::new(|| {
            CREATED.fetch_add(1, Ordering::SeqCst);
            42
        });

        // Lazy: nothing created yet
        assert_eq!(CREATED.load(Ordering::SeqCst), 0);

        // First get: factory is called once (slot 0 claimed)
        let a = pool.get();
        assert_eq!(*a, 42);
        assert_eq!(CREATED.load(Ordering::SeqCst), 1);

        // Drop a, returns the value to slot 0
        drop(a);

        // Second get: should reuse slot 0; no new factory call
        let b = pool.get();
        assert_eq!(*b, 42);

        // Should not create new objects
        assert_eq!(
            CREATED.load(Ordering::SeqCst),
            1,
            "Expected pool to reuse object, but it created a new one"
        );
    }

    #[test]
    fn test_typed_pool_overflow_leak() {
        use core::sync::atomic::{AtomicUsize, Ordering};
        static DROPPED: AtomicUsize = AtomicUsize::new(0);

        struct Tracked;

        impl Drop for Tracked {
            fn drop(&mut self) {
                DROPPED.fetch_add(1, Ordering::SeqCst);
            }
        }

        // Pool size 1.
        let pool = TypedPool::<Tracked, 1>::new(|| Tracked);

        // Get A (takes slot 0)
        let a = pool.get();
        assert!(a.slot.is_some());

        // Get B (overflow, takes slot None)
        let b = pool.get();
        assert!(b.slot.is_none());

        // Reset drop counter (ignore initial drops if any)
        DROPPED.store(0, Ordering::SeqCst);

        // B overflowed (slot None), so dropping it destroys the object
        // instead of returning it to the pool — slot 0 still belongs to A.
        drop(b);

        // A returns to slot 0 on drop (object kept alive in the pool).
        drop(a);

        // Dropping the pool drops the pooled object (A's). Total drops:
        // B + A = 2. The old overflow bug let B overwrite A's reservation
        // and leak one object, which showed up here as a count of 1.
        drop(pool);

        assert_eq!(DROPPED.load(Ordering::SeqCst), 2, "Memory leak detected!");
    }
}
