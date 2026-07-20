#![cfg(feature = "nexus")]

use ordofp_core::nexus::effects::region::{RegionVec, with_region};

#[test]
fn test_regionvec_leak() {
    let mut dropped = false;
    struct DropTracker<'a>(&'a mut bool);
    impl Drop for DropTracker<'_> {
        fn drop(&mut self) {
            *self.0 = true;
        }
    }

    with_region(|region| {
        let mut vec = RegionVec::with_capacity(region, 10);
        vec.push(DropTracker(&mut dropped));
        // vec dropped here
    });
    assert!(dropped, "RegionVec leaks items on drop!");
}
