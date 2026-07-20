use ordofp_core::arena::TypedPool;
use std::cell::RefCell;

thread_local! {
    static DROP_COUNT: RefCell<usize> = const { RefCell::new(0) };
    static ALLOC_COUNT: RefCell<usize> = const { RefCell::new(0) };
}

struct Dropper {
    pub id: usize,
}

impl Drop for Dropper {
    fn drop(&mut self) {
        DROP_COUNT.with(|c| *c.borrow_mut() += 1);
    }
}

fn factory() -> Dropper {
    ALLOC_COUNT.with(|c| *c.borrow_mut() += 1);
    let id = ALLOC_COUNT.with(|c| *c.borrow());
    Dropper { id }
}

#[test]
fn test_typed_pool_reuse_and_leak() {
    // Reset counters
    DROP_COUNT.with(|c| *c.borrow_mut() = 0);
    ALLOC_COUNT.with(|c| *c.borrow_mut() = 0);

    {
        let pool = TypedPool::<Dropper, 4>::new(factory);

        // 1. Get an object (allocates id=1)
        {
            let d1 = pool.get();
            assert_eq!(d1.id, 1);
            // d1 drops here, calls return_object
        }

        // 2. Get another object.
        {
            let d2 = pool.get();
            // This assertion fails if pooling is broken (factory called again -> id=2)
            assert_eq!(d2.id, 1, "Should reuse object with id=1, got {}", d2.id);
        }
        // d2 returns to pool
    } // pool drops here. d2 inside pool should be dropped.

    // 3. Pool dropped.
    // d2 was in pool. Pool drop should drop d2.
    // d1 was reused as d2. So total drops should be 1.
    // Total allocs = 1. Total drops = 1.

    // Wait, if d1 is reused as d2.
    // 1. get d1 (alloc 1).
    // 2. drop d1 -> into pool.
    // 3. get d2 -> from pool (same object).
    // 4. drop d2 -> into pool.
    // 5. drop pool -> drop contents (d2).

    // So one object created, dropped once (at end).
    // But `Dropper` impl drops on every drop?
    // `TypedPooled` calls `return_object`. Does it `drop` the inner value?
    // `TypedPooled` takes value out (`take()`).
    // `return_object` puts value into storage.
    // So `Dropper::drop` is NOT called when returning to pool!
    // Correct.

    // So `Dropper::drop` is only called when:
    // a) Pool is full and we drop (overflow).
    // b) Pool is dropped (cleanup).

    // So DROP_COUNT should match ALLOC_COUNT eventually.

    let allocs = ALLOC_COUNT.with(|c| *c.borrow());
    let drops = DROP_COUNT.with(|c| *c.borrow());

    assert_eq!(allocs, 1, "Should only allocate once");
    assert_eq!(drops, 1, "Should drop once (when pool is dropped)");
}
