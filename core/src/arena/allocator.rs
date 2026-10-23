//! Arena Allocator
//!
//! A simple bump allocator for arena-based allocation.

use alloc::alloc::{Layout, alloc, dealloc};
use alloc::vec;
use alloc::vec::Vec;
use core::cell::{Cell, RefCell};
use core::marker::PhantomData;
use core::ptr::NonNull;
#[cfg(feature = "std")]
use core::sync::atomic::{AtomicPtr, Ordering};

// =============================================================================
// Arena Allocator
// =============================================================================

/// A bump allocator arena for fast, scoped allocations.
///
/// All allocations are freed when the arena is dropped.
///
/// # No `Drop` for allocated values
///
/// Dropping the arena releases the arena's *memory chunks* only — the `Drop`
/// implementations of values allocated in the arena are **never run**. Types
/// that own non-arena resources (e.g. `Vec`, `String`, file handles) will
/// leak those resources when the arena is dropped. Allocate only `Copy`-like
/// / drop-free data, or take values back out before the arena dies.
///
/// # Example
///
/// ```rust
/// use ordofp_core::arena::Arena;
///
/// let arena = Arena::new();
/// let x = arena.alloc(42);
/// let y = arena.alloc("hello");
/// // The memory of x and y is freed when arena is dropped — but no Drop runs
/// ```
pub struct Arena {
    /// Memory chunks
    chunks: RefCell<Vec<Chunk>>,
    /// Current chunk being allocated from
    current: Cell<usize>,
    /// Pointer to next free byte in current chunk
    ptr: Cell<*mut u8>,
    /// End of current chunk
    end: Cell<*mut u8>,
}

/// A chunk of memory in the arena.
struct Chunk {
    /// Base address pointer of the live chunk allocation.
    ptr: NonNull<u8>,
    /// Layout of the chunk
    layout: Layout,
}

// SAFETY: Chunk owns a raw allocation from `alloc(layout)`. Transferring or sharing
// access to Chunk across threads is sound because access to the memory within
// the chunk is synchronized by the arena.
unsafe impl Send for Chunk {}
unsafe impl Sync for Chunk {}

impl Chunk {
    /// Create a new chunk with the given size.
    fn new(size: usize) -> Option<Self> {
        let layout = Layout::from_size_align(size, 16).ok()?;
        // SAFETY: `layout` has non-zero size (guaranteed by the `MIN_CHUNK_SIZE` lower bound
        // applied before `Chunk::new` is called) and a valid power-of-two alignment (16).
        // These are the only requirements of the global allocator's `alloc`. A null return
        // value (OOM) is filtered out by the `NonNull::new` check immediately below.
        let ptr = unsafe { alloc(layout) };
        NonNull::new(ptr).map(|ptr| Self { ptr, layout })
    }

    /// Get the start pointer of this chunk.
    const fn start(&self) -> *mut u8 {
        self.ptr.as_ptr()
    }

    /// Get the end pointer of this chunk.
    const fn end(&self) -> *mut u8 {
        // `self.ptr` is the base address of a live allocation of exactly
        // `self.layout.size()` bytes, so adding that offset produces a pointer one
        // byte past the end of the allocation — a valid "end" sentinel that is never
        // dereferenced.
        unsafe { self.ptr.as_ptr().add(self.layout.size()) }
    }
}

impl Drop for Chunk {
    fn drop(&mut self) {
        // SAFETY: `self.ptr` is the pointer returned by `alloc(self.layout)` in
        // `Chunk::new` and has not been freed since. `self.layout` is the same
        // layout that was passed to `alloc`, satisfying `dealloc`'s requirement
        // that the pointer and layout match the original allocation.
        unsafe {
            dealloc(self.ptr.as_ptr(), self.layout);
        }
    }
}

/// Default chunk size (64 KB)
pub const DEFAULT_CHUNK_SIZE: usize = 64 * 1024;

/// Minimum chunk size (4 KB)
pub const MIN_CHUNK_SIZE: usize = 4 * 1024;

impl Arena {
    /// Create a new arena with default chunk size.
    #[must_use]
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_CHUNK_SIZE)
    }

    /// Create a new arena with the specified initial capacity.
    ///
    /// The capacity is rounded up to [`MIN_CHUNK_SIZE`] (4 KiB) if smaller.
    ///
    /// # Panics
    ///
    /// Panics if the initial chunk cannot be allocated — either the global
    /// allocator reports out-of-memory, or `capacity` is so large that no
    /// valid `Layout` exists for it (exceeds `isize::MAX` after alignment).
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        let chunk_size = capacity.max(MIN_CHUNK_SIZE);
        let chunk = Chunk::new(chunk_size).expect("Failed to allocate arena chunk");

        let ptr = chunk.start();
        let end = chunk.end();

        Self {
            chunks: RefCell::new(vec![chunk]),
            current: Cell::new(0),
            ptr: Cell::new(ptr),
            end: Cell::new(end),
        }
    }

    /// Allocate a value in the arena.
    ///
    /// Returns an [`ArenaRef`] handle with the arena's lifetime; call
    /// [`ArenaRef::into_mut`] for a plain `&mut T`.
    pub fn alloc<T>(&self, value: T) -> ArenaRef<'_, T> {
        let layout = Layout::new::<T>();
        let ptr = self.alloc_layout(layout);
        // SAFETY: `ptr` is returned by `alloc_layout`, which guarantees:
        //   1. The pointer is non-null (it is a `NonNull<u8>`).
        //   2. It is aligned to `Layout::new::<T>().align()`, so the cast to
        //      `*mut T` is valid.
        //   3. The pointed-to region has exactly `size_of::<T>()` bytes of live
        //      arena-owned memory, making the `write` and the resulting `&mut T`
        //      sound.
        // The returned handle borrows `self` (the arena), so it cannot
        // outlive the allocation, and each call hands out a fresh,
        // non-overlapping region.
        unsafe {
            let typed_ptr = ptr.as_ptr().cast::<T>();
            typed_ptr.write(value);
            ArenaRef::from_mut(&mut *typed_ptr)
        }
    }

    /// Allocate a copy of `slice` in the arena.
    ///
    /// Returns an [`ArenaRef`] handle borrowing the arena, so the copy
    /// cannot outlive it; each call reserves a fresh, non-overlapping
    /// region.
    ///
    /// # Panics
    ///
    /// Panics if the slice's total byte size overflows a valid `Layout`
    /// (i.e. `slice.len() * size_of::<T>()` exceeds `isize::MAX`), or if
    /// growing the arena fails because the global allocator reports
    /// out-of-memory.
    pub fn alloc_slice<T: Copy>(&self, slice: &[T]) -> ArenaRef<'_, [T]> {
        let layout = Layout::array::<T>(slice.len()).expect("Invalid slice layout");
        let ptr = self.alloc_layout(layout);
        // SAFETY: `ptr` comes from `alloc_layout`, which guarantees:
        //   1. The pointer is non-null and aligned to `align_of::<T>()`.
        //   2. The allocation covers exactly `slice.len() * size_of::<T>()` bytes.
        // `copy_nonoverlapping` is sound because `slice` is a valid Rust slice
        // (so `src` is valid for `slice.len()` reads) and the arena allocation
        // is freshly reserved (so `dst` is valid for `slice.len()` writes and
        // cannot alias `src`). `from_raw_parts_mut` is sound because the region
        // is now fully initialized with a copy of `slice`.
        unsafe {
            let typed_ptr = ptr.as_ptr().cast::<T>();
            core::ptr::copy_nonoverlapping(slice.as_ptr(), typed_ptr, slice.len());
            ArenaRef::from_mut(core::slice::from_raw_parts_mut(typed_ptr, slice.len()))
        }
    }

    /// Allocate a string in the arena.
    pub fn alloc_str(&self, s: &str) -> ArenaRef<'_, str> {
        let bytes = self.alloc_slice(s.as_bytes()).into_mut();
        // SAFETY: `bytes` is a copy of `s.as_bytes()`, which are the UTF-8 encoded
        // bytes of a valid `&str`. `alloc_slice` copies them verbatim, so the byte
        // sequence is guaranteed to be valid UTF-8. Converting them back to `&mut str`
        // via `from_utf8_unchecked_mut` is therefore sound.
        ArenaRef::from_mut(unsafe { core::str::from_utf8_unchecked_mut(bytes) })
    }

    /// Allocate memory with a specific layout.
    fn alloc_layout(&self, layout: Layout) -> NonNull<u8> {
        let layout = layout.pad_to_align();

        // Try to allocate from current chunk
        loop {
            let ptr = self.ptr.get();
            let end = self.end.get();

            let ptr_addr = ptr as usize;
            // Safely align the pointer without arithmetic overflow
            let valid_aligned = checked_align_up(ptr_addr, layout.align()).filter(|&aligned| {
                aligned
                    .checked_add(layout.size())
                    .is_some_and(|new_ptr| new_ptr <= end as usize)
            });

            if let Some(aligned) = valid_aligned {
                let offset = aligned - ptr_addr;
                // SAFETY: `offset` advances `ptr` within the bounds of the chunk allocation,
                // preserving pointer provenance. `aligned_ptr` is aligned to `layout.align()`
                // and `new_ptr <= end` confirms the requested size fits in the live chunk.
                let aligned_ptr = unsafe { ptr.add(offset) };
                let new_ptr = unsafe { aligned_ptr.add(layout.size()) };
                self.ptr.set(new_ptr);
                return unsafe { NonNull::new_unchecked(aligned_ptr) };
            }

            // Need a new chunk
            let needed = layout
                .size()
                .checked_add(layout.align() - 1)
                .expect("allocation size overflow");
            self.grow(needed);
        }
    }

    /// Grow the arena by adding a new chunk.
    fn grow(&self, min_size: usize) {
        let mut chunks = self.chunks.borrow_mut();

        // Calculate new chunk size (double previous or fit allocation)
        let last_size = chunks
            .last()
            .map_or(DEFAULT_CHUNK_SIZE, |c| c.layout.size());
        let new_size = last_size
            .saturating_mul(2)
            .max(min_size.saturating_add(256));

        let chunk = Chunk::new(new_size).expect("Failed to allocate arena chunk");
        let ptr = chunk.start();
        let end = chunk.end();

        chunks.push(chunk);
        self.current.set(chunks.len() - 1);
        self.ptr.set(ptr);
        self.end.set(end);
    }

    /// Reset the arena, freeing all allocations but keeping the memory.
    ///
    /// # Safety
    ///
    /// All references to arena-allocated values become invalid after reset.
    pub unsafe fn reset(&self) {
        let mut chunks = self.chunks.borrow_mut();
        if let Some(first) = chunks.first() {
            self.current.set(0);
            self.ptr.set(first.start());
            self.end.set(first.end());
        }
        chunks.truncate(1);
    }

    /// Get the total capacity of all chunks.
    pub fn capacity(&self) -> usize {
        self.chunks.borrow().iter().map(|c| c.layout.size()).sum()
    }

    /// Get the approximate used memory.
    pub fn used(&self) -> usize {
        let chunks = self.chunks.borrow();
        let current = self.current.get();

        let mut used = 0;
        for (i, chunk) in chunks.iter().enumerate() {
            if i < current {
                used += chunk.layout.size();
            } else if i == current {
                used += (self.ptr.get() as usize).saturating_sub(chunk.start() as usize);
            }
        }
        used
    }
}

impl Default for Arena {
    fn default() -> Self {
        Self::new()
    }
}

// Arena is not Send or Sync because of interior mutability
// For thread-safe arena, use SyncArena

// =============================================================================
// SyncArena - Thread-safe bump allocator
// =============================================================================

/// Thread-safe bump allocator for concurrent arena allocation.
///
/// Unlike `Arena` (which uses `RefCell`/`Cell`), `SyncArena` serialises all
/// allocation calls through a `Mutex`. This allows it to be shared across
/// threads, making it suitable for parallel effect handler execution.
///
/// The returned references borrow `&self` (the arena lifetime), so they
/// cannot outlive the arena. Concurrent calls to `alloc` are safe because
/// each call acquires the lock before bumping the pointer, ensuring every
/// allocation occupies a distinct, non-overlapping region of memory.
///
/// # Example
///
/// ```rust
/// use ordofp_core::arena::SyncArena;
/// use std::sync::Arc;
///
/// let arena = Arc::new(SyncArena::new());
/// std::thread::scope(|s| {
///     s.spawn(|| { let _ = arena.alloc(1u32); });
///     s.spawn(|| { let _ = arena.alloc(2u32); });
/// });
/// ```
#[cfg(feature = "std")]
pub struct SyncArena {
    inner: std::sync::Mutex<SyncArenaInner>,
}

#[cfg(feature = "std")]
struct SyncArenaInner {
    chunks: Vec<Chunk>,
    /// Bump pointer into the live tail chunk. Shared across threads
    /// under the `Mutex` with `Relaxed` ordering — the lock provides all
    /// synchronization; the atomic pointer type documents the sharing and keeps
    /// the struct auto-`Send + Sync`. Always a non-null live pointer with valid
    /// allocation provenance: every store comes from a live `Chunk` allocation
    /// or an aligned, bounds-checked bump (see `alloc_layout`).
    ptr: AtomicPtr<u8>,
    /// One-past-the-end pointer of the live tail chunk; same sharing and
    /// validity argument as `ptr`.
    end: AtomicPtr<u8>,
}

// `SyncArena` is automatically `Send + Sync`: every field is `Send + Sync`
// (`Mutex<T>` is `Send`/`Sync` when `T: Send`; `Vec<Chunk>`, `Layout` and
// `AtomicUsize` are all `Send + Sync`), and the bump addresses are only
// loaded, stored and dereferenced while the `Mutex` is held, so no reference
// to arena memory outlives `&SyncArena` and no two allocations overlap.

#[cfg(feature = "std")]
impl SyncArena {
    /// Create a new `SyncArena` with the default chunk size (64 KiB).
    #[must_use]
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_CHUNK_SIZE)
    }

    /// Create a new `SyncArena` with the given initial capacity.
    ///
    /// The capacity is rounded up to [`MIN_CHUNK_SIZE`] (4 KiB) if smaller.
    ///
    /// # Panics
    ///
    /// Panics if the initial chunk cannot be allocated — either the global
    /// allocator reports out-of-memory, or `capacity` is so large that no
    /// valid `Layout` exists for it (exceeds `isize::MAX` after alignment).
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        let chunk_size = capacity.max(MIN_CHUNK_SIZE);
        let chunk = Chunk::new(chunk_size).expect("Failed to allocate SyncArena chunk");
        let ptr = chunk.start();
        let end = chunk.end();
        Self {
            inner: std::sync::Mutex::new(SyncArenaInner {
                chunks: vec![chunk],
                ptr: AtomicPtr::new(ptr),
                end: AtomicPtr::new(end),
            }),
        }
    }

    /// Allocate a value in the arena.
    ///
    /// Returns an [`ArenaRef`] handle with the arena's lifetime. Concurrent
    /// calls are safe: each call holds the lock only while bumping the
    /// pointer.
    pub fn alloc<T>(&self, value: T) -> ArenaRef<'_, T> {
        let layout = Layout::new::<T>();
        let ptr = self.alloc_layout(layout);
        // SAFETY: `ptr` is freshly bumped from the arena's live allocation and
        // aligned for `T`. Writing `value` initialises it. The returned handle
        // borrows `self`, so it cannot outlive the arena. No two calls return
        // overlapping regions because the bump is performed under the Mutex.
        unsafe {
            let typed_ptr = ptr.as_ptr().cast::<T>();
            typed_ptr.write(value);
            ArenaRef::from_mut(&mut *typed_ptr)
        }
    }

    /// Allocate a copy of `slice` in the thread-safe arena.
    ///
    /// Returns an [`ArenaRef`] handle borrowing the arena, so the copy
    /// cannot outlive it. Concurrent calls are safe because memory reservation
    /// is synchronized under the Mutex.
    ///
    /// # Panics
    ///
    /// Panics if the slice's total byte size overflows a valid `Layout`
    /// (i.e. `slice.len() * size_of::<T>()` exceeds `isize::MAX`), or if
    /// growing the arena fails because the global allocator reports
    /// out-of-memory.
    pub fn alloc_slice<T: Copy>(&self, slice: &[T]) -> ArenaRef<'_, [T]> {
        let layout = Layout::array::<T>(slice.len()).expect("Invalid slice layout");
        let ptr = self.alloc_layout(layout);
        // SAFETY: `ptr` comes from `alloc_layout`, which guarantees:
        //   1. The pointer is non-null and aligned to `align_of::<T>()`.
        //   2. The allocation covers exactly `slice.len() * size_of::<T>()` bytes.
        // `copy_nonoverlapping` is sound because `slice` is a valid Rust slice
        // (so `src` is valid for `slice.len()` reads) and the arena allocation
        // is freshly reserved (so `dst` is valid for `slice.len()` writes and
        // cannot alias `src`). `from_raw_parts_mut` is sound because the region
        // is now fully initialized with a copy of `slice`.
        unsafe {
            let typed_ptr = ptr.as_ptr().cast::<T>();
            core::ptr::copy_nonoverlapping(slice.as_ptr(), typed_ptr, slice.len());
            ArenaRef::from_mut(core::slice::from_raw_parts_mut(typed_ptr, slice.len()))
        }
    }

    /// Allocate a string in the thread-safe arena.
    pub fn alloc_str(&self, s: &str) -> ArenaRef<'_, str> {
        let bytes = self.alloc_slice(s.as_bytes()).into_mut();
        // SAFETY: `bytes` is a copy of `s.as_bytes()`, which are the UTF-8 encoded
        // bytes of a valid `&str`. `alloc_slice` copies them verbatim, so the byte
        // sequence is guaranteed to be valid UTF-8. Converting them back to `&mut str`
        // via `from_utf8_unchecked_mut` is therefore sound.
        ArenaRef::from_mut(unsafe { core::str::from_utf8_unchecked_mut(bytes) })
    }

    fn alloc_layout(&self, layout: Layout) -> NonNull<u8> {
        let layout = layout.pad_to_align();
        let mut inner = self.inner.lock().expect("SyncArena mutex poisoned");
        loop {
            let ptr = inner.ptr.load(Ordering::Relaxed);
            let end = inner.end.load(Ordering::Relaxed);

            let ptr_addr = ptr as usize;
            let valid_aligned = checked_align_up(ptr_addr, layout.align()).filter(|&aligned| {
                aligned
                    .checked_add(layout.size())
                    .is_some_and(|new_ptr| new_ptr <= end as usize)
            });

            if let Some(aligned) = valid_aligned {
                let offset = aligned - ptr_addr;
                // SAFETY: `offset` advances `ptr` within the bounds of the live chunk allocation,
                // preserving pointer provenance. `aligned_ptr` is aligned to `layout.align()`
                // and `new_ptr <= end` confirms the requested size fits in the live chunk.
                let aligned_ptr = unsafe { ptr.add(offset) };
                let new_ptr = unsafe { aligned_ptr.add(layout.size()) };
                inner.ptr.store(new_ptr, Ordering::Relaxed);
                return unsafe { NonNull::new_unchecked(aligned_ptr) };
            }
            // Current chunk is exhausted — grow.
            let needed = layout
                .size()
                .checked_add(layout.align() - 1)
                .expect("allocation size overflow");
            let last_size = inner
                .chunks
                .last()
                .map_or(DEFAULT_CHUNK_SIZE, |c| c.layout.size());
            let new_size = last_size.saturating_mul(2).max(needed.saturating_add(256));
            let chunk = Chunk::new(new_size).expect("Failed to allocate SyncArena chunk");
            inner.ptr.store(chunk.start(), Ordering::Relaxed);
            inner.end.store(chunk.end(), Ordering::Relaxed);
            inner.chunks.push(chunk);
        }
    }

    /// Total capacity across all allocated chunks.
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned, i.e. another thread
    /// panicked while holding the allocation lock.
    pub fn capacity(&self) -> usize {
        self.inner
            .lock()
            .expect("SyncArena mutex poisoned")
            .chunks
            .iter()
            .map(|c| c.layout.size())
            .sum()
    }
}

#[cfg(feature = "std")]
impl Default for SyncArena {
    fn default() -> Self {
        Self::new()
    }
}

/// Safely align a pointer address up to `align` (which must be a power of two),
/// returning `None` if arithmetic overflow occurs.
const fn checked_align_up(ptr: usize, align: usize) -> Option<usize> {
    if align == 0 {
        return None;
    }
    match ptr.checked_add(align - 1) {
        Some(sum) => Some(sum & !(align - 1)),
        None => None,
    }
}

// =============================================================================
// Scoped Arena Execution
// =============================================================================

/// Run a computation with a temporary arena.
///
/// The arena is created, used for the computation, and then dropped.
/// All arena *memory* is freed when the computation completes — but, as with
/// [`Arena`] generally, `Drop` of allocated values never runs.
///
/// # Example
///
/// ```rust
/// use ordofp_core::arena::with_arena;
///
/// let result = with_arena(|arena| {
///     // NOTE: the Vec's heap buffer is leaked — the arena never runs Drop
///     // for allocated values. Prefer drop-free data in arenas.
///     let data = arena.alloc(vec![1, 2, 3]);
///     data.iter().sum::<i32>()
/// });
/// ```
pub fn with_arena<F, R>(f: F) -> R
where
    F: FnOnce(&Arena) -> R,
{
    let arena = Arena::new();
    f(&arena)
}

/// Run a computation with an arena of specific capacity.
pub fn with_arena_capacity<F, R>(capacity: usize, f: F) -> R
where
    F: FnOnce(&Arena) -> R,
{
    let arena = Arena::with_capacity(capacity);
    f(&arena)
}

// =============================================================================
// Arena Reference
// =============================================================================

/// A uniquely-owned reference to a value allocated in an arena.
///
/// Returned by the `alloc*` methods (cf. `bumpalo::boxed::Box`): it derefs to
/// the value, and [`ArenaRef::into_mut`] converts it into a plain `&mut`
/// with the arena's lifetime. The wrapper expresses that each allocation is
/// a fresh, non-overlapping region handed out exactly once.
pub struct ArenaRef<'a, T: ?Sized> {
    ptr: &'a mut T,
    _marker: PhantomData<&'a ()>,
}

impl<'a, T: ?Sized> ArenaRef<'a, T> {
    /// Wrap a freshly allocated, uniquely-borrowed value.
    #[inline]
    const fn from_mut(ptr: &'a mut T) -> Self {
        ArenaRef {
            ptr,
            _marker: PhantomData,
        }
    }

    /// Convert into a mutable reference with the arena's lifetime.
    #[inline]
    #[must_use]
    pub const fn into_mut(self) -> &'a mut T {
        self.ptr
    }

    /// Get a reference to the value.
    #[must_use]
    pub const fn get(&self) -> &T {
        self.ptr
    }

    /// Get a mutable reference to the value.
    pub const fn get_mut(&mut self) -> &mut T {
        self.ptr
    }
}

impl<'a, T> ArenaRef<'a, T> {
    /// Create a new arena reference.
    pub fn new(arena: &'a Arena, value: T) -> Self {
        arena.alloc(value)
    }
}

impl<T: ?Sized> core::ops::Deref for ArenaRef<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.ptr
    }
}

impl<T: ?Sized> core::ops::DerefMut for ArenaRef<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.ptr
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arena_alloc() {
        let arena = Arena::new();
        let x = arena.alloc(42);
        let y = arena.alloc(100);
        assert_eq!(*x, 42);
        assert_eq!(*y, 100);
    }

    #[test]
    fn test_arena_alloc_slice() {
        let arena = Arena::new();
        let slice = arena.alloc_slice(&[1, 2, 3, 4, 5]);
        assert_eq!(&*slice, &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_arena_alloc_str() {
        let arena = Arena::new();
        let s = arena.alloc_str("hello world");
        assert_eq!(&*s, "hello world");
    }

    #[test]
    fn test_arena_multiple_types() {
        let arena = Arena::new();
        let x: &mut i32 = arena.alloc(42).into_mut();
        let y: &mut f64 = arena.alloc(2.5).into_mut();
        let z: &mut bool = arena.alloc(true).into_mut();

        assert_eq!(*x, 42);
        assert_eq!(y.to_bits(), 2.5f64.to_bits());
        assert!(*z);
    }

    #[test]
    fn test_arena_capacity() {
        let arena = Arena::with_capacity(1024);
        assert!(arena.capacity() >= 1024);

        // Allocate more than initial capacity to trigger growth
        for i in 0..1000 {
            let _ = arena.alloc(i);
        }

        assert!(arena.capacity() > 1024);
    }

    #[test]
    fn test_with_arena() {
        let result = with_arena(|arena| {
            let x = arena.alloc(10);
            let y = arena.alloc(20);
            *x + *y
        });
        assert_eq!(result, 30);
    }

    #[test]
    fn test_arena_ref() {
        let arena = Arena::new();
        let mut r = ArenaRef::new(&arena, 42);
        assert_eq!(*r, 42);
        *r.get_mut() = 100;
        assert_eq!(*r, 100);
    }

    #[test]
    #[allow(
        clippy::large_stack_arrays,
        reason = ">chunk-size value is what the test exercises"
    )]
    fn test_arena_grow_respects_alignment() {
        #[repr(align(128))]
        struct Aligned([u8; 65664]);

        for _ in 0..10 {
            let arena = Arena::new();
            let v = arena.alloc(Aligned([7u8; 65664]));
            assert_eq!((core::ptr::from_ref(&*v) as usize) % 128, 0);
            assert_eq!(v.0[0], 7);
            assert_eq!(v.0[65663], 7);
        }
    }

    #[cfg(feature = "std")]
    #[test]
    #[allow(
        clippy::large_stack_arrays,
        reason = ">chunk-size value is what the test exercises"
    )]
    fn test_sync_arena_grow_respects_alignment() {
        #[repr(align(128))]
        struct Aligned([u8; 65664]);

        for _ in 0..10 {
            let arena = SyncArena::new();
            let v = arena.alloc(Aligned([7u8; 65664]));
            assert_eq!((core::ptr::from_ref(&*v) as usize) % 128, 0);
            assert_eq!(v.0[0], 7);
            assert_eq!(v.0[65663], 7);
        }
    }

    #[test]
    fn test_checked_align_up_overflow() {
        assert_eq!(checked_align_up(0, 8), Some(0));
        assert_eq!(checked_align_up(1, 8), Some(8));
        assert_eq!(checked_align_up(8, 8), Some(8));
        assert_eq!(checked_align_up(usize::MAX - 7, 8), Some(usize::MAX - 7));
        assert_eq!(checked_align_up(usize::MAX - 3, 8), None);
        assert_eq!(checked_align_up(usize::MAX, 8), None);
        assert_eq!(checked_align_up(100, 0), None);
    }

    #[test]
    #[should_panic(expected = "Failed to allocate arena chunk")]
    fn test_arena_grow_overflow_protection() {
        let arena = Arena::new();
        // Requesting an allocation with size near isize::MAX triggers grow().
        // Saturating arithmetic prevents integer overflow panic/wrap during new_size calculation.
        let layout = Layout::from_size_align(isize::MAX as usize - 16, 1).unwrap();
        let _ = arena.alloc_layout(layout);
    }

    #[cfg(feature = "std")]
    #[test]
    #[should_panic(expected = "Failed to allocate SyncArena chunk")]
    fn test_sync_arena_grow_overflow_protection() {
        let arena = SyncArena::new();
        let layout = Layout::from_size_align(isize::MAX as usize - 16, 1).unwrap();
        let _ = arena.alloc_layout(layout);
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_sync_arena_concurrent_alloc() {
        use alloc::sync::Arc;

        let arena = Arc::new(SyncArena::new());
        std::thread::scope(|s| {
            for i in 0..4 {
                let arena_ref = Arc::clone(&arena);
                s.spawn(move || {
                    let val = arena_ref.alloc(i);
                    assert_eq!(*val, i);
                });
            }
        });

        assert!(arena.capacity() >= DEFAULT_CHUNK_SIZE);
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_sync_arena_alloc_slice_and_str() {
        let arena = SyncArena::new();
        let slice = arena.alloc_slice(&[10, 20, 30]);
        assert_eq!(&*slice, &[10, 20, 30]);

        let str_ref = arena.alloc_str("concurrent arena");
        assert_eq!(&*str_ref, "concurrent arena");
    }

    #[test]
    fn test_arena_reset() {
        let arena = Arena::new();

        // Perform multiple allocations across chunks
        let _x = arena.alloc(42i32);
        let _s = arena.alloc_str("hello world");
        let _slice = arena.alloc_slice(&[1u8, 2, 3, 4]);

        assert!(arena.used() > 0);

        // SAFETY: No references to the previously allocated data are held or accessed after reset.
        unsafe { arena.reset() };

        // Post-reset allocation works cleanly and reuses the initial chunk
        let y = arena.alloc(99i32);
        assert_eq!(*y, 99);
    }
}
