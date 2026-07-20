extern crate alloc;

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
pub struct Pool<T> {
    available: RefCell<Vec<T>>,
    factory: fn() -> T,
    reset: fn(&mut T),
    max_size: usize,
}

impl<T> Pool<T> {
    pub fn new(factory: fn() -> T) -> Self {
        Self::with_reset(factory, |_| {})
    }

    pub fn with_reset(factory: fn() -> T, reset: fn(&mut T)) -> Self {
        Pool {
            available: RefCell::new(Vec::new()),
            factory,
            reset,
            max_size: DEFAULT_MAX_POOL_SIZE,
        }
    }

    pub fn with_max_size(mut self, max_size: usize) -> Self {
        self.max_size = max_size;
        self
    }

    pub fn with_initial(self, count: usize) -> Self {
        let mut available = self.available.borrow_mut();
        for _ in 0..count.min(self.max_size) {
            available.push((self.factory)());
        }
        drop(available);
        self
    }

    /// Borrow a pooled object, creating one via the factory if the pool is empty.
    ///
    /// The returned [`Pooled`] guard automatically returns the object to the pool
    /// (after invoking the reset function) when it is dropped.
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

    fn return_object(&self, mut obj: T) {
        (self.reset)(&mut obj);

        let mut available = self.available.borrow_mut();
        if available.len() < self.max_size {
            available.push(obj);
        }
    }

    pub fn available(&self) -> usize {
        self.available.borrow().len()
    }

    pub fn clear(&self) {
        self.available.borrow_mut().clear();
    }
}

pub struct Pooled<'a, T> {
    pool: &'a Pool<T>,
    value: Option<T>,
}

impl<T> Deref for Pooled<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
            .as_ref()
            .expect("pooled value is present until returned to pool in Drop")
    }
}

impl<T> DerefMut for Pooled<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value
            .as_mut()
            .expect("pooled value is present until returned to pool in Drop")
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

pub struct TypedPool<T, const N: usize> {
    storage: RefCell<[MaybeUninit<T>; N]>,
    available: RefCell<u64>,
    factory: fn() -> T,
}

impl<T, const N: usize> TypedPool<T, N> {
    pub fn new(factory: fn() -> T) -> Self {
        assert!(N <= 64, "TypedPool size must be <= 64");

        let mut storage: [MaybeUninit<T>; N] = [const { MaybeUninit::uninit() }; N];

        // Guard: drop already-initialized elements if `factory` panics mid-loop.
        struct InitGuard<'a, T, const M: usize> {
            storage: &'a mut [MaybeUninit<T>; M],
            count: usize,
        }
        impl<T, const M: usize> Drop for InitGuard<'_, T, M> {
            fn drop(&mut self) {
                for i in 0..self.count {
                    // SAFETY: `self.count` is incremented only after a slot is
                    // initialized via `MaybeUninit::new(factory())`, so every
                    // index in `0..self.count` holds a fully initialized `T`.
                    // `assume_init_drop` is therefore sound and transfers
                    // ownership so the value is dropped exactly once.
                    unsafe {
                        self.storage[i].assume_init_drop();
                    }
                }
            }
        }

        let mut guard = InitGuard {
            storage: &mut storage,
            count: 0,
        };
        for i in 0..N {
            guard.storage[i] = MaybeUninit::new(factory());
            guard.count += 1;
        }
        core::mem::forget(guard);

        let mask = if N == 64 { !0 } else { (1u64 << N) - 1 };

        TypedPool {
            storage: RefCell::new(storage),
            available: RefCell::new(mask),
            factory,
        }
    }

    pub fn get(&self) -> TypedPooled<'_, T, N> {
        let mut available = self.available.borrow_mut();

        if *available != 0 {
            let slot = available.trailing_zeros() as usize;
            *available &= !(1 << slot);

            let storage = self.storage.borrow();
            // SAFETY: `slot` is derived from `available.trailing_zeros()` on a
            // non-zero bitmap, so `slot < N` is a valid array index. The
            // available bitmap only marks slots that were previously initialized
            // (written by the factory or returned via `return_object`). We
            // cleared the bit above before this read, so the value is uniquely
            // owned by this call and the slot is logically uninitialized until
            // the value is returned to the pool again.
            let value = unsafe { storage[slot].as_ptr().read() };

            return TypedPooled {
                pool: self,
                slot: Some(slot),
                value: Some(value),
            };
        }

        TypedPooled {
            pool: self,
            slot: None,
            value: Some((self.factory)()),
        }
    }

    fn return_object(&self, slot: Option<usize>, obj: T) {
        if let Some(idx) = slot {
            let mut storage = self.storage.borrow_mut();
            storage[idx] = MaybeUninit::new(obj);

            let mut available = self.available.borrow_mut();
            *available |= 1 << idx;
        } else {
            drop(obj);
        }
    }
}

impl<T, const N: usize> Drop for TypedPool<T, N> {
    fn drop(&mut self) {
        let available = self.available.get_mut();
        let storage = self.storage.get_mut();

        for (i, item) in storage.iter_mut().enumerate() {
            if (*available & (1 << i)) != 0 {
                // SAFETY: Bit `i` is set in `available`, which means this slot
                // was initialized by the pool factory and has not yet been read
                // out (reads clear the bit). `get_mut` gives exclusive access,
                // so no other code path can observe or drop this slot
                // concurrently, making `assume_init_drop` sound.
                unsafe {
                    item.assume_init_drop();
                }
            }
        }
    }
}

pub struct TypedPooled<'a, T, const N: usize> {
    pool: &'a TypedPool<T, N>,
    slot: Option<usize>,
    value: Option<T>,
}

impl<T, const N: usize> Deref for TypedPooled<'_, T, N> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.value
            .as_ref()
            .expect("TypedPooled value is None; deref called after Drop")
    }
}

impl<T, const N: usize> DerefMut for TypedPooled<'_, T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value
            .as_mut()
            .expect("TypedPooled value is None; deref_mut called after Drop")
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
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use core::sync::atomic::{AtomicUsize, Ordering};
    use std::panic::{AssertUnwindSafe, catch_unwind};

    #[test]
    fn test_typed_pool_panic_during_init() {
        static DROPPED: AtomicUsize = AtomicUsize::new(0);
        static CREATED: AtomicUsize = AtomicUsize::new(0);

        struct Tracked;
        impl Drop for Tracked {
            fn drop(&mut self) {
                DROPPED.fetch_add(1, Ordering::SeqCst);
            }
        }

        // We want to test N=3
        // 1. Success
        // 2. Success
        // 3. Panic
        // Expected: 1 and 2 are dropped.

        let res = catch_unwind(AssertUnwindSafe(|| {
            TypedPool::<Tracked, 3>::new(|| {
                let count = CREATED.fetch_add(1, Ordering::SeqCst);
                assert!(count != 2, "Simulated panic during init");
                Tracked
            })
        }));

        assert!(res.is_err());

        // We created 2 successfully before panic.
        // So we expect 2 drops.
        assert_eq!(
            DROPPED.load(Ordering::SeqCst),
            2,
            "Memory leak on panic during initialization!"
        );
    }
}
