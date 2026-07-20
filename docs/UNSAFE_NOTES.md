# Unsafe & UB Audit Notes

Hard-won learnings from the `unsafe`/UB audit of OrdoFP. Each entry:
the bug, the underlying rule, and the prevention.

## TypedPool::drop — drop by `available`, not `allocated` (RESOLVED)

**Issue (historical analysis):** an early note argued `TypedPool::drop` should
iterate the `allocated` bitmap, "or it leaks checked-out objects."

**Resolution (correct, and what the code does):** `TypedPool::drop` must iterate
**`available`**. Checkout (`TypedPool::get`) *moves* the slot's value out of storage
via `storage[slot].as_ptr().read()` and clears the `available` bit, so a checked-out
slot's storage is logically moved-from and the value lives in the `TypedPooled`
handle (which drops it). Dropping by `allocated` would `assume_init_drop` a
moved-from slot → **double-free**. The current implementation uses `available`;
**miri confirms no UB** (19 arena tests pass clean). The earlier "leak" model was
wrong — checkout *moves*, it does not lend.

**Rule:** match the drop condition to the *initialization-and-ownership* state, not
to "ever allocated." When checkout transfers ownership out, the container must not
drop those slots.

## `core::ptr::read` for type-punning in dynamically-dead branches is UB

**Issue:** `unsafe { core::ptr::read(&() as *const () as *const A) }` (and the
`ManuallyDrop::new(())` variant) was used to satisfy monomorphization in branches
proven unreachable at runtime. Reading `size_of::<A>()` bytes from a zero-byte
source is an out-of-bounds read — **UB even if the branch never executes** (miri
flags it during monomorphization).

**Rule:** a dead branch is still monomorphized and analyzable; `ptr::read` past a
source's size is UB regardless of reachability.

**Prevention:** use safe `Any::downcast_*` when `T: 'static`; never `ptr::read` for
type punning unless the source allocation is provably `>= size_of::<Dst>()`.

## `alloc(Layout::from_size_align(0, _))` is UB

**Issue:** `Layout::from_size_align(0, 16)` was passed to the global allocator in the
region allocator (`nexus/effects/region.rs`). `from_size_align` *accepts* size 0 (it
does not reject it), but calling `std::alloc::alloc` with a zero-sized layout is
**UB**.

**Rule:** `Layout::from_size_align` does not guard against zero sizes.

**Prevention:** bounds-check (`if size == 0`) before any dynamically-sized low-level
allocation; do not rely on `Layout` construction to reject zero.

