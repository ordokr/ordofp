#![cfg(feature = "nexus")]

use core::marker::PhantomData;
use ordofp_core::nexus::optim::state_fast::FastState;

#[test]
#[should_panic(expected = "FastState::Put requires A = ()")]
fn test_faststate_ub() {
    // Construct a FastState where A is Box<i32>, but use Put variant
    // Box<T> must be non-null. zeroed Box is UB.
    let bad: FastState<i32, Box<i32>> = FastState::Put(42, PhantomData);

    // Run it. This executes unsafe { core::mem::zeroed() } for Box<i32>
    let (_s, _state) = bad.run(0);

    // Accessing the box will crash or be UB
    // But merely creating it is UB if NonNull is involved.
    // Box uses Unique which uses NonNull.

    // Dropping s will try to free null pointer?
    // Allocator dealloc requires non-null pointer.
}
