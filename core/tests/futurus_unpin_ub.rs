#![cfg(feature = "async")]

use ordofp_core::async_core::Futurus;

// Helper to statically assert Unpin implementation
fn assert_unpin<T: Unpin>() {}

#[test]
fn test_futurus_unpin_correctness() {
    // 1. Valid case: i32 is Unpin, so Futurus<i32> must be Unpin.
    assert_unpin::<Futurus<i32>>();

    // 2. Invalid case: a !Unpin type should not make Futurus<T> Unpin.
    // The soundness fix ensures Futurus<PhantomPinned> is NOT Unpin.
    // Uncommenting the following line should cause a compilation error:
    // assert_unpin::<Futurus<PhantomPinned>>();
}
