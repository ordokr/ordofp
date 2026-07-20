//! Region Effect - Scoped Memory Management
//!
//! This module provides **region-based memory management** as an effect.
//! Regions are memory scopes where allocations live - when the region ends,
//! all allocations are freed together.
//!
//! # Key Concepts
//!
//! - **Region**: A memory scope with a lifetime `'r`
//! - **`RegionRef`**: A reference to data allocated in a region
//! - **`with_region`**: Run a computation with a fresh region
//!
//! # Example
//!
//! ```rust
//! use ordofp_core::nexus::effects::region::*;
//!
//! // Allocate in a region - all memory freed when scope ends
//! let result = with_region(|region| {
//!     let x = region.alloc(42);
//!     let y = region.alloc(vec![1, 2, 3]);
//!     *x + y.len() as i32
//! });
//! assert_eq!(result, 45);
//! // x and y are freed here
//! ```
//!
//! # Use Cases
//!
//! - **Temporary allocations**: Avoid individual deallocations
//! - **Parser scratch space**: Allocate AST nodes in a region
//! - **Request handling**: Per-request allocation pools
//! - **Game frames**: Per-frame temporary data
//!
//! # Verification Tier
//!
//! **Tier 0-1**: Region lifetimes enforced by Rust's borrow checker.
//! Tests verify allocation and deallocation behavior.
//!
//! # Limitations
//!
//! - **No individual deallocation**: Entire region freed at once
//! - **No region inference**: Explicit `with_region` calls required
//! - **References can't escape**: Enforced by Rust's borrow checker
//! - **Not thread-safe**: Single-threaded region access only
//!
//! # Design Philosophy
//!
//! This provides **effect-based region allocation** built on bump allocation.
//! Rust's ownership system already provides region-like guarantees through
//! lifetimes - this module makes regions explicit as an effect.

use alloc::alloc::{Layout, alloc, dealloc};
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::cell::{Cell, RefCell};
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::ptr::NonNull;
use core::slice;

use crate::hints::likely;
use crate::nexus::effect::EffectMarker;
use crate::nexus::row::Row;

// =============================================================================
// Effect Marker
// =============================================================================

/// Bit flag for Region effect.
pub const REGION_BIT: u128 = 1 << 36;

/// The Region effect marker type.
#[derive(Copy, Clone, Debug)]
pub struct RegionEffect;

impl EffectMarker for RegionEffect {
    const BIT: u128 = REGION_BIT;
    const NAME: &'static str = "Region";
}

/// Type alias for a row containing only Region.
pub type RegionRow = Row<REGION_BIT>;

// =============================================================================
// Region Allocator
// =============================================================================

/// A memory chunk in the region.
struct Chunk {
    /// Pointer to allocated memory.
    ptr: NonNull<u8>,
    /// Layout of the allocation.
    layout: Layout,
}

impl Chunk {
    /// Create a new chunk with the given size.
    fn new(size: usize) -> Option<Self> {
        if size == 0 {
            return None;
        }
        let layout = Layout::from_size_align(size, 16).ok()?;
        // SAFETY: `layout` has non-zero size because we explicitly reject
        // zero-sized layouts above, satisfying the precondition of the global allocator.
        // The returned pointer is checked for null via `NonNull::new` before use.
        let ptr = unsafe { alloc(layout) };
        NonNull::new(ptr).map(|ptr| Chunk { ptr, layout })
    }

    /// Get the end of this chunk.
    fn end(&self) -> *mut u8 {
        // SAFETY: `self.ptr` points to an allocation of exactly `self.layout.size()` bytes,
        // so adding `layout.size()` advances the pointer one-past-the-end, which is valid
        // for pointer arithmetic per the Rust reference (the result need not be dereferenced).
        unsafe { self.ptr.as_ptr().add(self.layout.size()) }
    }
}

impl Drop for Chunk {
    fn drop(&mut self) {
        // SAFETY: `self.ptr` was allocated by the global allocator via `alloc(self.layout)`
        // in `Chunk::new`, and `self.layout` is identical to the layout used at allocation
        // time (stored in the struct). No references into this chunk can outlive the `Chunk`
        // because the `Region` lifetime `'r` guarantees all borrows are released before drop.
        unsafe {
            dealloc(self.ptr.as_ptr(), self.layout);
        }
    }
}

#[cfg(feature = "alloc")]
use crate::arena::{DEFAULT_CHUNK_SIZE, MIN_CHUNK_SIZE};

#[cfg(not(feature = "alloc"))]
/// Default chunk size (64 KB).
const DEFAULT_CHUNK_SIZE: usize = 64 * 1024;

#[cfg(not(feature = "alloc"))]
/// Minimum chunk size (4 KB).
const MIN_CHUNK_SIZE: usize = 4 * 1024;

/// A region allocator using bump allocation.
///
/// All allocations in a region are freed when the region is dropped.
/// This provides fast allocation with no per-object overhead.
pub struct Region<'r> {
    /// Memory chunks (`RefCell` for interior mutability when growing).
    chunks: RefCell<Vec<Chunk>>,
    /// Current allocation pointer.
    ptr: Cell<*mut u8>,
    /// End of current chunk.
    end: Cell<*mut u8>,
    /// Statistics.
    stats: Cell<RegionStats>,
    /// Lifetime marker.
    _marker: PhantomData<&'r ()>,
}

/// Statistics for region allocations.
#[derive(Clone, Copy, Debug, Default)]
pub struct RegionStats {
    /// Number of allocations.
    pub allocations: usize,
    /// Total bytes allocated.
    pub bytes_allocated: usize,
    /// Number of chunks used.
    pub chunks_used: usize,
}

impl<'r> Region<'r> {
    /// Create a new region with default capacity.
    fn new() -> Self {
        Self::with_capacity(DEFAULT_CHUNK_SIZE)
    }

    /// Create a new region with the specified initial capacity.
    fn with_capacity(capacity: usize) -> Self {
        let chunk_size = capacity.max(MIN_CHUNK_SIZE);
        let chunk = Chunk::new(chunk_size).expect("Failed to allocate region chunk");

        let ptr = chunk.ptr.as_ptr();
        let end = chunk.end();

        let stats = RegionStats {
            allocations: 0,
            bytes_allocated: 0,
            chunks_used: 1,
        };

        Region {
            chunks: RefCell::new(alloc::vec![chunk]),
            ptr: Cell::new(ptr),
            end: Cell::new(end),
            stats: Cell::new(stats),
            _marker: PhantomData,
        }
    }

    /// Allocate a value in the region.
    ///
    /// Returns a mutable reference with the region's lifetime.
    #[inline]
    pub fn alloc<T>(&self, value: T) -> &'r mut T {
        let ptr = self.alloc_raw(Layout::new::<T>());
        let typed_ptr = ptr.as_ptr().cast::<T>();
        // SAFETY: `alloc_raw` returns a `NonNull<u8>` pointer that is properly aligned
        // (at least to `Layout::new::<T>()`) and points to at least `size_of::<T>()` bytes
        // of writable, unaliased memory owned by the region. Writing `value` initialises
        // the slot, and the resulting `&mut T` borrows the region lifetime `'r`, which
        // ensures the pointer remains valid and exclusively owned for the lifetime of the
        // reference.
        unsafe {
            typed_ptr.write(value);
            &mut *typed_ptr
        }
    }

    /// Allocate a slice in the region.
    ///
    /// Copies `slice` into region-owned memory and returns a mutable view
    /// with the region's lifetime. An empty slice is returned immediately
    /// without touching the allocator.
    ///
    /// # Panics
    ///
    /// Panics if the slice's total byte size overflows `isize::MAX` (the
    /// `Layout::array` limit) — unreachable for any slice that already
    /// exists in memory, so in practice this cannot fire.
    #[inline]
    pub fn alloc_slice<T: Copy>(&self, slice: &[T]) -> &'r mut [T] {
        if slice.is_empty() {
            return &mut [];
        }

        let layout = Layout::array::<T>(slice.len()).expect("Invalid layout");
        let ptr = self.alloc_raw(layout);
        let typed_ptr = ptr.as_ptr().cast::<T>();

        // SAFETY: `typed_ptr` is non-null, properly aligned for `T`, and points to
        // region-allocated memory large enough for `slice.len()` elements of `T`
        // (guaranteed by `alloc_raw(Layout::array::<T>(slice.len()))`).
        // `slice.as_ptr()` is valid for `slice.len()` reads, and the source and
        // destination ranges do not overlap because `typed_ptr` is freshly allocated.
        // After `copy_nonoverlapping` all `slice.len()` slots are initialized, so
        // constructing a `&mut [T]` with that length is sound. The lifetime `'r`
        // ties the returned reference to the region, preventing use-after-free.
        unsafe {
            core::ptr::copy_nonoverlapping(slice.as_ptr(), typed_ptr, slice.len());
            slice::from_raw_parts_mut(typed_ptr, slice.len())
        }
    }

    /// Allocate a string in the region.
    #[inline]
    pub fn alloc_str(&self, s: &str) -> &'r str {
        let bytes = self.alloc_slice(s.as_bytes());
        // SAFETY: `bytes` is a region-allocated copy of `s.as_bytes()`, which was already
        // valid UTF-8 (it came from a `&str`). Copying valid UTF-8 bytes preserves their
        // validity, so reinterpreting `bytes` as a `str` is sound.
        unsafe { core::str::from_utf8_unchecked(bytes) }
    }

    /// Allocate a single uninitialized slot of type `T` in the region.
    ///
    /// Returns a `&'r mut MaybeUninit<T>` backed by region-owned memory that is
    /// properly aligned for `T` and lives for the region lifetime `'r`.  The
    /// caller **must** write a valid `T` into the slot (via
    /// [`MaybeUninit::write`]) before reading from it; reading an uninitialized
    /// slot is undefined behavior.
    ///
    /// Prefer [`alloc`](Region::alloc) when the value is already available at
    /// call time.  Use this function when initialization must be deferred, for
    /// example to pass the raw pointer to a foreign API that fills the memory.
    ///
    /// # Example
    ///
    /// ```rust
    /// use core::mem::MaybeUninit;
    /// use ordofp_core::nexus::effects::region::with_region;
    ///
    /// with_region(|region| {
    ///     let slot: &mut MaybeUninit<u32> = region.alloc_uninit();
    ///     slot.write(42u32);
    ///     // SAFETY: the slot was just initialized above.
    ///     let value: u32 = unsafe { slot.assume_init_read() };
    ///     assert_eq!(value, 42);
    /// });
    /// ```
    pub fn alloc_uninit<T>(&self) -> &'r mut MaybeUninit<T> {
        let ptr = self.alloc_raw(Layout::new::<T>());
        let typed_ptr = ptr.as_ptr().cast::<MaybeUninit<T>>();
        // SAFETY: `alloc_raw` returns a `NonNull<u8>` pointing to at least
        // `size_of::<T>()` bytes aligned to `align_of::<T>()`, satisfying the
        // requirements of `*mut MaybeUninit<T>`.  The memory lives for the
        // region lifetime `'r` and is exclusively owned (just allocated), so
        // producing a unique `&'r mut MaybeUninit<T>` is sound.
        unsafe { &mut *typed_ptr }
    }

    /// Allocate an uninitialized slice of `len` elements in the region.
    ///
    /// Returns a mutable slice of [`MaybeUninit<T>`] with the region's lifetime
    /// `'r`. The caller is responsible for initializing every element before
    /// reading from the slice. An empty slice (`len == 0`) is returned
    /// immediately without touching the allocator.
    ///
    /// This is the building block for bulk region allocations where the values
    /// are not yet available at allocation time (e.g., output buffers filled by
    /// a subsequent pass).
    ///
    /// # Parameters
    ///
    /// * `len` – Number of `T`-sized slots to allocate. `0` is valid and
    ///   returns an empty slice without hitting the allocator.
    ///
    /// # Returns
    ///
    /// A `&'r mut [MaybeUninit<T>]` pointing to region-owned, properly aligned
    /// memory. The lifetime `'r` prevents the slice from outliving the region.
    ///
    /// # Example
    ///
    /// ```rust
    /// use core::mem::MaybeUninit;
    /// use ordofp_core::nexus::effects::region::with_region;
    ///
    /// with_region(|region| {
    ///     let slots: &mut [MaybeUninit<u32>] = region.alloc_slice_uninit(4);
    ///     for (i, slot) in slots.iter_mut().enumerate() {
    ///         slot.write(i as u32);
    ///     }
    ///     // SAFETY: all elements were just initialized above.
    ///     let inited: &[u32] = unsafe {
    ///         core::mem::transmute::<&[MaybeUninit<u32>], &[u32]>(slots)
    ///     };
    ///     assert_eq!(inited, &[0, 1, 2, 3]);
    /// });
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if `len * size_of::<T>()` overflows `isize::MAX` (the
    /// `Layout::array` limit) — i.e. if the requested slice could not
    /// exist in memory at all.
    pub fn alloc_slice_uninit<T>(&self, len: usize) -> &'r mut [MaybeUninit<T>] {
        if len == 0 {
            return &mut [];
        }

        let layout = Layout::array::<T>(len).expect("Invalid layout");
        let ptr = self.alloc_raw(layout);
        let typed_ptr = ptr.as_ptr().cast::<MaybeUninit<T>>();

        // SAFETY: `alloc_raw` returns a `NonNull<u8>` pointing to at least
        // `Layout::array::<T>(len)` bytes with the correct alignment for `T`.
        // Casting to `*mut MaybeUninit<T>` is valid because `MaybeUninit<T>`
        // has the same size and alignment as `T`. The pointer is valid for
        // `len` elements (guaranteed by the layout), and the memory is
        // exclusively owned (just allocated), so constructing a unique
        // `&'r mut [MaybeUninit<T>]` for the region lifetime `'r` is sound.
        unsafe { slice::from_raw_parts_mut(typed_ptr, len) }
    }

    /// Raw allocation with layout.
    #[inline]
    fn alloc_raw(&self, layout: Layout) -> NonNull<u8> {
        let size = layout.size();
        let align = layout.align();

        // Align the current pointer
        let ptr = self.ptr.get();
        let aligned = align_up(ptr as usize, align) as *mut u8;

        // Check if we have space safely using integer arithmetic
        let new_addr = (aligned as usize)
            .checked_add(size)
            .expect("Address overflow");
        if likely(new_addr <= self.end.get() as usize) {
            self.ptr.set(new_addr as *mut u8);
            self.update_stats(size);
            return NonNull::new(aligned).expect("Aligned pointer should be non-null");
        }

        // Need a new chunk - cold path
        self.alloc_raw_slow(layout, size, align)
    }

    /// Slow path for allocation when current chunk is full.
    #[cold]
    #[inline(never)]
    fn alloc_raw_slow(&self, _layout: Layout, size: usize, align: usize) -> NonNull<u8> {
        // Over-request by `align - 1` so the post-grow alignment adjustment can
        // never push the allocation past the end of the fresh chunk: chunks are
        // only 16-aligned (see `Chunk::new`), so aligning the chunk start for
        // `align > 16` may waste up to `align - 1` bytes.
        let needed = size.checked_add(align - 1).expect("Allocation overflow");

        loop {
            // Need a new chunk
            self.grow(needed.max(MIN_CHUNK_SIZE));

            // Retry allocation
            let ptr = self.ptr.get();
            let aligned = align_up(ptr as usize, align) as *mut u8;
            let new_addr = (aligned as usize)
                .checked_add(size)
                .expect("Address overflow");

            // Hard check (not debug-only): handing out a pointer past the chunk
            // end would be out-of-bounds UB in release builds. The over-request
            // above guarantees this fits on the first iteration; the loop is a
            // defensive retry mirroring `arena::Arena::alloc_layout`.
            if new_addr <= self.end.get() as usize {
                self.ptr.set(new_addr as *mut u8);
                self.update_stats(size);
                return NonNull::new(aligned).expect("Aligned pointer should be non-null");
            }
        }
    }

    /// Grow the region by adding a new chunk.
    fn grow(&self, min_size: usize) {
        let chunk_size = min_size.max(DEFAULT_CHUNK_SIZE);
        let chunk = Chunk::new(chunk_size).expect("Failed to allocate region chunk");

        self.ptr.set(chunk.ptr.as_ptr());
        self.end.set(chunk.end());

        // Update stats
        let mut stats = self.stats.get();
        stats.chunks_used += 1;
        self.stats.set(stats);

        // Use RefCell for interior mutability
        self.chunks.borrow_mut().push(chunk);
    }

    /// Update allocation statistics.
    #[inline(always)]
    fn update_stats(&self, bytes: usize) {
        let mut stats = self.stats.get();
        stats.allocations += 1;
        stats.bytes_allocated += bytes;
        self.stats.set(stats);
    }

    /// Get allocation statistics.
    pub fn stats(&self) -> RegionStats {
        self.stats.get()
    }

    /// Get the number of allocations.
    pub fn allocation_count(&self) -> usize {
        self.stats.get().allocations
    }

    /// Get the total bytes allocated.
    pub fn bytes_allocated(&self) -> usize {
        self.stats.get().bytes_allocated
    }

    /// Reset the region, freeing all allocations.
    ///
    /// This keeps the allocated chunks but resets the allocation pointer.
    /// Useful for reusing a region without reallocating memory.
    ///
    /// # Safety
    ///
    /// All references to region-allocated data become invalid after reset.
    /// This is marked unsafe because the caller must ensure no references
    /// to region data are used after calling this.
    pub unsafe fn reset(&self) {
        let mut chunks = self.chunks.borrow_mut();
        if let Some(chunk) = chunks.first() {
            self.ptr.set(chunk.ptr.as_ptr());
            self.end.set(chunk.end());
        }
        chunks.truncate(1);
        self.stats.set(RegionStats {
            allocations: 0,
            bytes_allocated: 0,
            chunks_used: 1,
        });
    }
}

/// Align a value up to the given alignment.
#[inline(always)]
fn align_up(val: usize, align: usize) -> usize {
    (val + align - 1) & !(align - 1)
}

// =============================================================================
// Region Computation
// =============================================================================

/// A computation that allocates in a region.
pub struct RegionComputation<'r, A> {
    /// The computation function.
    run_fn: Box<dyn FnOnce(&Region<'r>) -> A + 'r>,
}

impl<'r, A: 'r> RegionComputation<'r, A> {
    /// Create a new region computation.
    pub fn new<F: FnOnce(&Region<'r>) -> A + 'r>(f: F) -> Self {
        RegionComputation {
            run_fn: Box::new(f),
        }
    }

    /// Run the computation with a region.
    pub fn run(self, region: &Region<'r>) -> A {
        (self.run_fn)(region)
    }

    /// Pure value (no allocation).
    pub fn pure(value: A) -> Self
    where
        A: Clone,
    {
        RegionComputation::new(move |_| value)
    }

    /// Map over the result.
    pub fn map<B: 'r, F: FnOnce(A) -> B + 'r>(self, f: F) -> RegionComputation<'r, B> {
        RegionComputation::new(move |region| {
            let a = (self.run_fn)(region);
            f(a)
        })
    }

    /// Chain region computations.
    pub fn and_then<B: 'r, F: FnOnce(A) -> RegionComputation<'r, B> + 'r>(
        self,
        f: F,
    ) -> RegionComputation<'r, B> {
        RegionComputation::new(move |region| {
            let a = (self.run_fn)(region);
            f(a).run(region)
        })
    }
}

// =============================================================================
// with_region
// =============================================================================

/// Run a computation with a fresh region.
///
/// All allocations in the region are freed when the computation completes.
///
/// # Example
///
/// ```rust
/// use ordofp_core::nexus::effects::region::with_region;
///
/// let result = with_region(|region| {
///     let data = region.alloc(vec![1, 2, 3, 4, 5]);
///     data.iter().sum::<i32>()
/// });
/// assert_eq!(result, 15);
/// ```
pub fn with_region<A, F>(f: F) -> A
where
    F: for<'r> FnOnce(&Region<'r>) -> A,
{
    let region = Region::new();
    f(&region)
}

/// Run a computation with a region pre-allocated to the given byte capacity.
///
/// Like [`with_region`], all allocations made through the region are freed
/// when the computation returns. The difference is that the underlying arena
/// reserves `capacity` bytes upfront, avoiding reallocation for workloads
/// whose total allocation size is known in advance.
///
/// Prefer this over [`with_region`] when you have a rough upper bound on the
/// total number of bytes you will allocate, as it eliminates the cost of
/// chunk-growth during the computation.
///
/// # Parameters
///
/// * `capacity` — Hint for the initial arena size in bytes. The region may
///   still grow beyond this if allocations exceed the reserved space.
/// * `f` — Closure that receives a reference to the region and produces the
///   result `A`.
///
/// # Returns
///
/// The value `A` returned by `f`. All region-allocated memory is released
/// once this function returns.
///
/// # Example
///
/// ```rust
/// use ordofp_core::nexus::effects::region::with_region_capacity;
///
/// // Pre-allocate 4 KB to avoid arena growth while processing a batch.
/// let sum = with_region_capacity(4096, |region| {
///     let nums = region.alloc(vec![1i32; 1000]);
///     nums.iter().sum::<i32>()
/// });
/// assert_eq!(sum, 1000);
/// ```
pub fn with_region_capacity<A, F>(capacity: usize, f: F) -> A
where
    F: for<'r> FnOnce(&Region<'r>) -> A,
{
    let region = Region::with_capacity(capacity);
    f(&region)
}

// =============================================================================
// Region-Allocated Types
// =============================================================================

/// A boxed value allocated in a region.
///
/// Unlike `Box<T>`, this doesn't own its memory - the region does.
/// The value is freed when the region is dropped.
#[derive(Debug)]
pub struct RegionBox<'r, T: ?Sized> {
    ptr: &'r mut T,
}

impl<'r, T> RegionBox<'r, T> {
    /// Create a new region-boxed value.
    pub fn new(region: &Region<'r>, value: T) -> Self {
        RegionBox {
            ptr: region.alloc(value),
        }
    }
}

impl<T: ?Sized> core::ops::Deref for RegionBox<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.ptr
    }
}

impl<T: ?Sized> core::ops::DerefMut for RegionBox<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.ptr
    }
}

/// A vector allocated in a region.
///
/// This is a simple vector that stores its elements in the region.
/// Unlike `Vec<T>`, it cannot grow - capacity is fixed at creation.
pub struct RegionVec<'r, T> {
    /// Pointer to elements.
    ptr: *mut T,
    /// Number of elements.
    len: usize,
    /// Capacity.
    capacity: usize,
    /// Lifetime marker.
    _marker: PhantomData<&'r mut T>,
}

impl<'r, T> RegionVec<'r, T> {
    /// Create a new empty region vector with capacity.
    #[inline]
    pub fn with_capacity(region: &Region<'r>, capacity: usize) -> Self {
        if capacity == 0 {
            return RegionVec {
                ptr: core::ptr::NonNull::dangling().as_ptr(),
                len: 0,
                capacity: 0,
                _marker: PhantomData,
            };
        }

        let uninit = region.alloc_slice_uninit::<T>(capacity);
        RegionVec {
            ptr: uninit.as_mut_ptr().cast::<T>(),
            len: 0,
            capacity,
            _marker: PhantomData,
        }
    }

    /// Push a value onto the vector.
    ///
    /// # Panics
    ///
    /// Panics if the vector is at capacity.
    #[inline]
    pub fn push(&mut self, value: T) {
        assert!(self.len < self.capacity, "RegionVec at capacity");
        // SAFETY: The assert above guarantees `self.len < self.capacity`, so
        // `self.ptr.add(self.len)` is within the allocated region. The slot at
        // `self.len` is not yet initialized (it is beyond the live prefix), so
        // writing to it does not drop an existing value and does not alias any
        // live reference into the vector.
        unsafe {
            self.ptr.add(self.len).write(value);
        }
        self.len += 1;
    }

    /// Pop a value from the vector.
    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            // SAFETY: `self.len` was just decremented from a non-zero value, so
            // the new `self.len` is in the range `0..old_len <= capacity`. The
            // slot at that index was part of the initialized live prefix before
            // the decrement, so the `read()` transfers a valid, fully-initialized
            // `T` out of the allocation. The decrement ensures no future access
            // (via `push`, `as_slice`, etc.) will treat this slot as initialized,
            // preventing any double-read of the now-moved value.
            Some(unsafe { self.ptr.add(self.len).read() })
        }
    }

    /// Get the length.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Check if empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Get the capacity.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Get a slice of the elements.
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        // SAFETY: `self.ptr` is a non-null, properly aligned pointer into a live
        // region allocation of at least `self.capacity` elements of type `T`.
        // `self.len <= self.capacity` is maintained as an invariant, so the
        // first `self.len` elements are fully initialized. The `PhantomData<&'r mut T>`
        // marker ensures the returned reference cannot outlive the region, and
        // `&self` prevents any concurrent mutable access for the duration of the borrow.
        unsafe { slice::from_raw_parts(self.ptr, self.len) }
    }

    /// Get a mutable slice of the elements.
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        // SAFETY: Same allocation and initialization invariants as `as_slice`.
        // The exclusive `&mut self` borrow guarantees no other reference to the
        // elements can exist simultaneously, satisfying the aliasing requirement
        // for `from_raw_parts_mut`.
        unsafe { slice::from_raw_parts_mut(self.ptr, self.len) }
    }
}

impl<T> core::ops::Deref for RegionVec<'_, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T> core::ops::DerefMut for RegionVec<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

impl<T> Drop for RegionVec<'_, T> {
    fn drop(&mut self) {
        // Drop all initialized elements. The memory itself is part of the Region
        // and will be reclaimed when the Region drops.
        let slice = core::ptr::slice_from_raw_parts_mut(self.ptr, self.len);
        // SAFETY: `self.ptr` points to a region-allocated buffer whose first
        // `self.len` elements are fully initialized — this invariant is upheld
        // by every method that modifies `len` (`push` initialises before
        // incrementing, `pop` decrements before reading).  `drop_in_place`
        // requires a valid, non-null, properly-aligned pointer to an
        // initialised value of the correct type; all three properties follow
        // from the above invariant.  Because this is the `Drop` impl, the
        // `&mut self` receiver guarantees exclusive access: no other live
        // reference to the elements can exist at this point.
        unsafe {
            core::ptr::drop_in_place(slice);
        }
    }
}

// =============================================================================
// Scoped Allocation Pattern
// =============================================================================

/// A scoped allocator that provides region-like allocation with a scope guard.
///
/// This is useful when you need to pass allocation capability around
/// without exposing the region lifetime in types.
pub struct ScopedAllocator {
    /// Current allocation count (for testing/debugging).
    allocation_count: Cell<usize>,
}

impl ScopedAllocator {
    /// Create a new scoped allocator.
    pub fn new() -> Self {
        ScopedAllocator {
            allocation_count: Cell::new(0),
        }
    }

    /// Run a computation with a fresh region.
    pub fn scope<A, F>(&self, f: F) -> A
    where
        F: for<'r> FnOnce(&Region<'r>) -> A,
    {
        let count_before = self.allocation_count.get();
        with_region(|region| {
            let r = f(region);
            self.allocation_count
                .set(count_before + region.allocation_count());
            r
        })
    }

    /// Get total allocation count across all scopes.
    pub fn total_allocations(&self) -> usize {
        self.allocation_count.get()
    }
}

impl Default for ScopedAllocator {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_region_alloc() {
        with_region(|region| {
            let x = region.alloc(42i32);
            let y = region.alloc(100i32);

            assert_eq!(*x, 42);
            assert_eq!(*y, 100);

            *x = 99;
            assert_eq!(*x, 99);
        });
    }

    #[test]
    fn test_region_alloc_slice() {
        with_region(|region| {
            let slice = region.alloc_slice(&[1, 2, 3, 4, 5]);
            assert_eq!(slice, &[1, 2, 3, 4, 5]);

            slice[0] = 10;
            assert_eq!(slice[0], 10);
        });
    }

    #[test]
    fn test_region_alloc_str() {
        with_region(|region| {
            let s = region.alloc_str("hello world");
            assert_eq!(s, "hello world");
        });
    }

    #[test]
    #[allow(clippy::large_stack_arrays)] // >chunk-size value is what the regression exercises
    fn test_region_grow_respects_alignment() {
        // Regression: the grow path must account for alignment. Chunks are
        // only 16-aligned, so an align-128 value sized just over
        // DEFAULT_CHUNK_SIZE forces grow() to allocate a chunk where the
        // post-grow alignment adjustment can push the allocation past the
        // chunk end unless grow over-requests for the alignment slack.
        #[repr(align(128))]
        struct Aligned([u8; 65664]); // > DEFAULT_CHUNK_SIZE (64 KiB)

        for _ in 0..32 {
            with_region(|region| {
                let v = region.alloc(Aligned([7u8; 65664]));
                assert_eq!((core::ptr::from_ref(&*v) as usize) % 128, 0);
                assert_eq!(v.0[0], 7);
                assert_eq!(v.0[65663], 7);
            });
        }
    }

    #[test]
    fn test_region_stats() {
        with_region(|region| {
            assert_eq!(region.allocation_count(), 0);

            region.alloc(1i32);
            region.alloc(2i32);
            region.alloc(3i32);

            assert_eq!(region.allocation_count(), 3);
            assert!(region.bytes_allocated() >= 12); // At least 3 * 4 bytes
        });
    }

    #[test]
    fn test_region_multiple_types() {
        with_region(|region| {
            let i = region.alloc(42i32);
            let f = region.alloc(2.5f64);
            let s = region.alloc_str("test");
            let arr = region.alloc_slice(&[1u8, 2, 3]);

            assert_eq!(*i, 42);
            assert_eq!(*f, 2.5);
            assert_eq!(s, "test");
            assert_eq!(arr, &[1, 2, 3]);
        });
    }

    #[test]
    fn test_with_region_capacity() {
        let result = with_region_capacity(1024, |region| {
            let x = region.alloc(42);
            *x * 2
        });
        assert_eq!(result, 84);
    }

    #[test]
    fn test_region_box() {
        with_region(|region| {
            let boxed = RegionBox::new(region, 42);
            assert_eq!(*boxed, 42);
        });
    }

    #[test]
    fn test_region_vec() {
        with_region(|region| {
            let mut vec = RegionVec::<i32>::with_capacity(region, 10);

            vec.push(1);
            vec.push(2);
            vec.push(3);

            assert_eq!(vec.len(), 3);
            assert_eq!(vec.as_slice(), &[1, 2, 3]);

            assert_eq!(vec.pop(), Some(3));
            assert_eq!(vec.len(), 2);
        });
    }

    #[test]
    fn test_region_vec_iteration() {
        with_region(|region| {
            let mut vec = RegionVec::<i32>::with_capacity(region, 5);
            vec.push(10);
            vec.push(20);
            vec.push(30);

            let sum: i32 = vec.iter().sum();
            assert_eq!(sum, 60);
        });
    }

    #[test]
    fn test_region_computation() {
        let result = with_region(|region| {
            let comp = RegionComputation::new(|r| {
                let x = r.alloc(10);
                let y = r.alloc(20);
                *x + *y
            });
            comp.run(region)
        });
        assert_eq!(result, 30);
    }

    #[test]
    fn test_region_computation_map() {
        let result = with_region(|region| {
            let comp = RegionComputation::new(|r| r.alloc(10)).map(|x| *x * 2);
            comp.run(region)
        });
        assert_eq!(result, 20);
    }

    #[test]
    fn test_scoped_allocator() {
        let allocator = ScopedAllocator::new();

        allocator.scope(|region| {
            region.alloc(1);
            region.alloc(2);
        });

        allocator.scope(|region| {
            region.alloc(3);
        });

        assert_eq!(allocator.total_allocations(), 3);
    }

    #[test]
    #[allow(clippy::large_stack_arrays)] // >chunk-size slice is what the test exercises
    fn test_large_allocation() {
        with_region(|region| {
            // Allocate more than a chunk size
            let large = region.alloc_slice(&[0u8; 100_000]);
            assert_eq!(large.len(), 100_000);
            assert!(region.stats().chunks_used >= 2);
        });
    }

    #[test]
    fn test_alignment() {
        with_region(|region| {
            // Allocate values with different alignments
            let b = region.alloc(1u8);
            let i = region.alloc(42i32);
            let d = region.alloc(2.5f64);

            // Check that addresses are properly aligned
            assert_eq!(core::ptr::from_ref::<u8>(b) as usize % align_of::<u8>(), 0);
            assert_eq!(
                core::ptr::from_ref::<i32>(i) as usize % align_of::<i32>(),
                0
            );
            assert_eq!(
                core::ptr::from_ref::<f64>(d) as usize % align_of::<f64>(),
                0
            );
        });
    }

    #[test]
    fn test_region_vec_empty() {
        with_region(|region| {
            let vec = RegionVec::<i32>::with_capacity(region, 0);
            assert!(vec.is_empty());
            assert_eq!(vec.capacity(), 0);
        });
    }

    #[test]
    fn test_region_computation_pure() {
        let result = with_region(|region| {
            let comp = RegionComputation::pure(42);
            comp.run(region)
        });
        assert_eq!(result, 42);
    }

    #[test]
    fn test_nested_regions() {
        let outer_result = with_region(|outer| {
            let x = outer.alloc(10);

            let inner_result = with_region(|inner| {
                let y = inner.alloc(20);
                *y
            });

            *x + inner_result
        });
        assert_eq!(outer_result, 30);
    }

    /// Test demonstrating a realistic region usage pattern.
    #[test]
    fn test_realistic_usage() {
        // Simulating parsing or tree construction
        struct Node<'r> {
            value: i32,
            children: &'r [&'r Node<'r>],
        }

        let sum = with_region(|region| {
            // Build a simple tree
            let leaf1 = region.alloc(Node {
                value: 1,
                children: &[],
            });
            let leaf2 = region.alloc(Node {
                value: 2,
                children: &[],
            });
            let _leaf3 = region.alloc(Node {
                value: 3,
                children: &[],
            });

            let children = region.alloc_slice(&[leaf1 as &Node, leaf2 as &Node]);
            let parent = region.alloc(Node {
                value: 10,
                children,
            });

            // Calculate sum of all values
            fn sum_tree(node: &Node) -> i32 {
                node.value + node.children.iter().map(|c| sum_tree(c)).sum::<i32>()
            }

            sum_tree(parent)
        });

        assert_eq!(sum, 13); // 10 + 1 + 2
    }
}
