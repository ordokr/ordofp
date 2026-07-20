//! Property-based tests for the arena allocator and object pools.
//!
//! `core/src/arena/` pairs a bump allocator with several unsafe pooling
//! strategies. These tests drive them with randomized operation sequences
//! and check global invariants — no value corruption, no leaks, no double
//! drops, size bounds — rather than single-scenario behaviour.

extern crate alloc;

use alloc::vec::Vec;
use core::sync::atomic::{AtomicIsize, Ordering};

use ordofp_core::arena::{Arena, MIN_CHUNK_SIZE, Pool, TypedPool};
use quickcheck::quickcheck;

// =============================================================================
// Arena: value integrity
// =============================================================================

quickcheck! {
    /// Every allocated value must still hold its original contents after
    /// arbitrarily many later allocations, including ones that force chunk
    /// growth. Overlapping bump regions would corrupt earlier values.
    fn arena_values_survive_later_allocations(values: Vec<u32>) -> bool {
        let arena = Arena::with_capacity(0); // MIN_CHUNK_SIZE; growth is easy to hit
        let refs: Vec<&mut u32> = values.iter().map(|&v| arena.alloc(v).into_mut()).collect();

        // Churn: guarantee at least one chunk growth after the tracked values.
        let _ = arena.alloc_slice(&[0xA5u8; 2 * MIN_CHUNK_SIZE]);

        refs.iter().zip(&values).all(|(r, v)| **r == *v)
    }

    /// Interleaved allocations of different shapes (words, strings) must not
    /// corrupt one another, before or after chunk growth.
    fn arena_mixed_allocations_do_not_corrupt(items: Vec<(u64, String)>) -> bool {
        let arena = Arena::with_capacity(0);
        let mut nums: Vec<&u64> = Vec::new();
        let mut strs: Vec<&str> = Vec::new();
        for (n, s) in &items {
            nums.push(arena.alloc(*n).into_mut());
            strs.push(arena.alloc_str(s).into_mut());
        }

        let _ = arena.alloc_slice(&[0x5Au8; 2 * MIN_CHUNK_SIZE]);

        nums.iter().zip(&items).all(|(r, (n, _))| **r == *n)
            && strs.iter().zip(&items).all(|(r, (_, s))| **r == *s)
    }

    /// Slices copied into the arena must round-trip byte-for-byte.
    fn arena_slices_roundtrip(chunks: Vec<Vec<u8>>) -> bool {
        let arena = Arena::with_capacity(0);
        let refs: Vec<&[u8]> = chunks
            .iter()
            .map(|c| &*arena.alloc_slice(c).into_mut())
            .collect();
        refs.iter().zip(&chunks).all(|(r, c)| *r == c.as_slice())
    }

    /// `used()` never exceeds `capacity()` and never decreases as
    /// allocations accumulate (growth counts abandoned chunk tails as used).
    fn arena_used_bounded_and_monotone(sizes: Vec<u8>) -> bool {
        let arena = Arena::with_capacity(0);
        let mut prev_used = arena.used();
        for n in sizes {
            let buf = alloc::vec![0xABu8; usize::from(n) * 8];
            let _ = arena.alloc_slice(&buf);
            let used = arena.used();
            if used > arena.capacity() || used < prev_used {
                return false;
            }
            prev_used = used;
        }
        true
    }
}

// =============================================================================
// Arena: alignment, edge sizes, reset
// =============================================================================

#[repr(align(16))]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Aligned16([u8; 16]);

#[test]
fn arena_respects_alignment() {
    let arena = Arena::new();
    // Interleave 1-byte allocations so the bump pointer is repeatedly left
    // misaligned for the wider types that follow.
    for i in 0..32u8 {
        let byte = arena.alloc(i).into_mut();
        assert_eq!(*byte, i);

        let a16 = arena.alloc(Aligned16([i; 16])).into_mut();
        assert_eq!(a16.0, [i; 16]);
        assert_eq!(core::ptr::from_mut(&mut *a16).addr() % 16, 0);

        let word = arena.alloc(u64::from(i)).into_mut();
        assert_eq!(*word, u64::from(i));
        assert_eq!(
            core::ptr::from_mut(&mut *word).addr() % core::mem::align_of::<u64>(),
            0
        );
    }
}

#[test]
fn arena_zero_sized_allocations() {
    let arena = Arena::new();
    let unit = arena.alloc(());
    let empty_slice = arena.alloc_slice::<u8>(&[]);
    let empty_str = arena.alloc_str("");
    let _: &() = unit.get();
    assert!(empty_slice.is_empty());
    assert!(empty_str.is_empty());

    // The arena must remain usable after zero-sized allocations.
    let x = arena.alloc(7u64);
    assert_eq!(*x, 7);
}

#[test]
fn arena_allocation_larger_than_chunk() {
    let arena = Arena::with_capacity(0); // one MIN_CHUNK_SIZE chunk
    let big = alloc::vec![0x5Au8; 3 * MIN_CHUNK_SIZE];
    let r = arena.alloc_slice(&big);
    assert_eq!(&*r, big.as_slice());
}

#[test]
fn arena_reset_keeps_only_first_chunk() {
    let arena = Arena::with_capacity(0);
    let first_capacity = arena.capacity();
    {
        let big = alloc::vec![1u8; 4 * MIN_CHUNK_SIZE];
        let _ = arena.alloc_slice(&big);
    }
    assert!(arena.capacity() > first_capacity, "growth expected");

    // SAFETY: no references into the arena are live at this point.
    unsafe { arena.reset() };

    assert_eq!(arena.capacity(), first_capacity);
    assert_eq!(arena.used(), 0);

    // Allocations after reset must be intact.
    let vals: Vec<&mut u32> = (0..100u32).map(|i| arena.alloc(i).into_mut()).collect();
    for (i, v) in vals.iter().enumerate() {
        assert_eq!(**v, u32::try_from(i).expect("i < 100"));
    }
}

// =============================================================================
// Pool: size bound and reset invariants
// =============================================================================

quickcheck! {
    /// Under any interleaving of get/return, the pool never retains more
    /// than `max_size` objects, and every handle it gives out has been
    /// through the reset function (an unreset `Vec` would be non-empty).
    fn pool_available_bounded_and_reset_applied(ops: Vec<u8>, max: u8) -> bool {
        let max_size = usize::from(max % 8) + 1;
        let pool: Pool<Vec<i32>> =
            Pool::with_reset(Vec::new, Vec::clear).with_max_size(max_size);

        let mut live = Vec::new();
        for op in ops {
            if op % 2 == 0 {
                let mut handle = pool.get();
                if !handle.is_empty() {
                    return false; // reset was skipped on return
                }
                handle.push(i32::from(op));
                live.push(handle);
            } else if !live.is_empty() {
                let idx = usize::from(op) % live.len();
                live.swap_remove(idx); // drop → return to pool
            }
            if pool.available() > max_size {
                return false;
            }
        }
        drop(live);
        pool.available() <= max_size
    }
}

// =============================================================================
// TypedPool: leak / double-drop accounting
// =============================================================================

/// Net live `Tracked` objects: +1 in the factory, -1 in `Drop`. A leak leaves
/// the counter above its baseline after teardown; a double drop pushes it
/// below.
static TYPED_POOL_LIVE: AtomicIsize = AtomicIsize::new(0);

const TRACKED_PAYLOAD: u64 = 0xDEAD_BEEF;

struct Tracked(u64);

impl Drop for Tracked {
    fn drop(&mut self) {
        TYPED_POOL_LIVE.fetch_sub(1, Ordering::SeqCst);
    }
}

fn make_tracked() -> Tracked {
    TYPED_POOL_LIVE.fetch_add(1, Ordering::SeqCst);
    Tracked(TRACKED_PAYLOAD)
}

quickcheck! {
    /// Any interleaving of get/return — including checking out more handles
    /// than the pool has slots, which produces transient (slotless) objects —
    /// must destroy every object exactly once by the time the pool is gone.
    fn typed_pool_never_leaks_or_double_drops(ops: Vec<u8>) -> bool {
        let baseline = TYPED_POOL_LIVE.load(Ordering::SeqCst);
        {
            let pool = TypedPool::<Tracked, 4>::new(make_tracked);
            let mut live = Vec::new();
            for op in ops {
                if op % 2 == 0 {
                    let handle = pool.get();
                    if handle.0 != TRACKED_PAYLOAD {
                        return false; // uninitialized or corrupted slot handed out
                    }
                    live.push(handle);
                } else if !live.is_empty() {
                    let idx = usize::from(op) % live.len();
                    live.swap_remove(idx);
                }
            }
        } // handles drop, then the pool drops its retained objects
        TYPED_POOL_LIVE.load(Ordering::SeqCst) == baseline
    }
}

// =============================================================================
// SyncArena: concurrent allocation
// =============================================================================

#[cfg(feature = "std")]
mod sync_arena {
    use alloc::vec::Vec;
    use ordofp_core::arena::SyncArena;

    /// Concurrent allocators must receive disjoint regions with intact
    /// contents, across chunk growth under contention.
    #[test]
    fn concurrent_allocations_are_disjoint_and_intact() {
        const THREADS: usize = 4;
        const PER_THREAD: usize = 1_000;

        let arena = SyncArena::with_capacity(0); // small: force growth under contention
        let addrs: Vec<Vec<usize>> = std::thread::scope(|s| {
            let handles: Vec<_> = (0..THREADS)
                .map(|t| {
                    let arena = &arena;
                    s.spawn(move || {
                        let mut addrs = Vec::with_capacity(PER_THREAD);
                        let mut refs = Vec::with_capacity(PER_THREAD);
                        for i in 0..PER_THREAD {
                            let r = arena.alloc(t * PER_THREAD + i).into_mut();
                            addrs.push(core::ptr::from_mut(&mut *r).addr());
                            refs.push(r);
                        }
                        // Values must be intact after all allocations landed.
                        for (i, r) in refs.iter().enumerate() {
                            assert_eq!(**r, t * PER_THREAD + i);
                        }
                        addrs
                    })
                })
                .collect();
            handles
                .into_iter()
                .map(|h| h.join().expect("allocator thread panicked"))
                .collect()
        });

        // No two allocations may overlap.
        let mut all: Vec<usize> = addrs.into_iter().flatten().collect();
        all.sort_unstable();
        assert_eq!(all.len(), THREADS * PER_THREAD);
        for w in all.windows(2) {
            assert!(
                w[1] - w[0] >= core::mem::size_of::<usize>(),
                "overlapping allocations at {:#x} and {:#x}",
                w[0],
                w[1]
            );
        }
    }
}
