//! Arena Allocator
//!
//! A simple bump allocator for arena-based allocation.

use alloc::alloc::{Layout, alloc, dealloc};
use alloc::vec;
use alloc::vec::Vec;
use core::cell::{Cell, RefCell};
use core::marker::PhantomData;
use core::ptr::NonNull;

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
    /// Pointer to the start of the chunk
    start: NonNull<u8>,
    /// Layout of the chunk
    layout: Layout,
}

impl Chunk {
    /// Create a new chunk with the given size.
    fn new(size: usize) -> Option<Self> {
        let layout = Layout::from_size_align(size, 16).ok()?;
        // SAFETY: `layout` has non-zero size (guaranteed by the `MIN_CHUNK_SIZE` lower bound
        // applied before `Chunk::new` is called) and a valid power-of-two alignment (16).
        // These are the only requirements of the global allocator's `alloc`. A null return
        // value (OOM) is handled by the `NonNull::new` check immediately below.
        let ptr = unsafe { alloc(layout) };
        NonNull::new(ptr).map(|start| Chunk { start, layout })
    }

    /// Get the end pointer of this chunk.
    fn end(&self) -> *mut u8 {
        // SAFETY: `self.start` points to the beginning of a live allocation of exactly
        // `self.layout.size()` bytes, so adding that offset produces a pointer one
        // byte past the end of the allocation — a valid "end" sentinel that is never
        // dereferenced and stays within the bounds required by `pointer::add`.
        unsafe { self.start.as_ptr().add(self.layout.size()) }
    }
}

impl Drop for Chunk {
    fn drop(&mut self) {
        // SAFETY: `self.start` was obtained from `alloc(self.layout)` in `Chunk::new`
        // and has not been freed since. `self.layout` is the same layout that was
        // passed to `alloc`, satisfying `dealloc`'s requirement that the pointer and
        // layout match the original allocation.
        unsafe {
            dealloc(self.start.as_ptr(), self.layout);
        }
    }
}

/// Default chunk size (64 KB)
pub const DEFAULT_CHUNK_SIZE: usize = 64 * 1024;

/// Minimum chunk size (4 KB)
pub const MIN_CHUNK_SIZE: usize = 4 * 1024;

impl Arena {
    /// Create a new arena with default chunk size.
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
    pub fn with_capacity(capacity: usize) -> Self {
        let chunk_size = capacity.max(MIN_CHUNK_SIZE);
        let chunk = Chunk::new(chunk_size).expect("Failed to allocate arena chunk");

        let ptr = chunk.start.as_ptr();
        let end = chunk.end();

        Arena {
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

            // Align the pointer
            let aligned = align_up(ptr as usize, layout.align());
            let new_ptr = aligned + layout.size();

            if new_ptr <= end as usize {
                // Fits in current chunk
                self.ptr.set(new_ptr as *mut u8);
                // SAFETY: `aligned` is non-null because `self.ptr` is always
                // initialised from a successful global-allocator allocation
                // (verified via `NonNull::new` in `Chunk::new`), and `align_up`
                // only rounds the address upward — it never produces zero.
                // The `new_ptr <= end` guard above rules out arithmetic
                // wrap-around, so `aligned` remains strictly within the live
                // chunk allocation and is therefore a valid non-null address.
                return unsafe { NonNull::new_unchecked(aligned as *mut u8) };
            }

            // Need a new chunk
            self.grow(layout.size());
        }
    }

    /// Grow the arena by adding a new chunk.
    fn grow(&self, min_size: usize) {
        let mut chunks = self.chunks.borrow_mut();

        // Calculate new chunk size (double previous or fit allocation)
        let last_size = chunks
            .last()
            .map_or(DEFAULT_CHUNK_SIZE, |c| c.layout.size());
        let new_size = (last_size * 2).max(min_size + 256);

        let chunk = Chunk::new(new_size).expect("Failed to allocate arena chunk");
        let ptr = chunk.start.as_ptr();
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
            self.ptr.set(first.start.as_ptr());
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
                used += self.ptr.get() as usize - chunk.start.as_ptr() as usize;
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
    ptr: *mut u8,
    end: *mut u8,
}

// SAFETY: `ptr` and `end` are only accessed while the `Mutex` is held.
// No reference to the arena memory outlives `&SyncArena`, and allocations
// never overlap because each bump is performed atomically under the lock.
#[cfg(feature = "std")]
unsafe impl Send for SyncArena {}
#[cfg(feature = "std")]
unsafe impl Sync for SyncArena {}

#[cfg(feature = "std")]
impl SyncArena {
    /// Create a new `SyncArena` with the default chunk size (64 KiB).
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
    pub fn with_capacity(capacity: usize) -> Self {
        let chunk_size = capacity.max(MIN_CHUNK_SIZE);
        let chunk = Chunk::new(chunk_size).expect("Failed to allocate SyncArena chunk");
        let ptr = chunk.start.as_ptr();
        let end = chunk.end();
        SyncArena {
            inner: std::sync::Mutex::new(SyncArenaInner {
                chunks: vec![chunk],
                ptr,
                end,
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

    fn alloc_layout(&self, layout: Layout) -> NonNull<u8> {
        let layout = layout.pad_to_align();
        let mut inner = self.inner.lock().expect("SyncArena mutex poisoned");
        loop {
            let aligned = align_up(inner.ptr as usize, layout.align());
            let new_ptr = aligned + layout.size();
            if new_ptr <= inner.end as usize {
                inner.ptr = new_ptr as *mut u8;
                // SAFETY: `aligned` is within the current live chunk allocation
                // (the guard above ensures it) and is non-zero (arena pointers
                // are initialised from a successful global-allocator call).
                return unsafe { NonNull::new_unchecked(aligned as *mut u8) };
            }
            // Current chunk is exhausted — grow.
            let last_size = inner
                .chunks
                .last()
                .map_or(DEFAULT_CHUNK_SIZE, |c| c.layout.size());
            let new_size = (last_size * 2).max(layout.size() + 256);
            let chunk = Chunk::new(new_size).expect("Failed to allocate SyncArena chunk");
            inner.ptr = chunk.start.as_ptr();
            inner.end = chunk.end();
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

/// Align a pointer up to the given alignment.
fn align_up(ptr: usize, align: usize) -> usize {
    (ptr + align - 1) & !(align - 1)
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
    fn from_mut(ptr: &'a mut T) -> Self {
        ArenaRef {
            ptr,
            _marker: PhantomData,
        }
    }

    /// Convert into a mutable reference with the arena's lifetime.
    #[inline]
    pub fn into_mut(self) -> &'a mut T {
        self.ptr
    }

    /// Get a reference to the value.
    pub fn get(&self) -> &T {
        self.ptr
    }

    /// Get a mutable reference to the value.
    pub fn get_mut(&mut self) -> &mut T {
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
        assert_eq!(*y, 2.5);
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
}
