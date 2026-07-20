//! Regression tests for the `nonempty!` macro as seen by a downstream crate.
//!
//! This integration crate deliberately does NOT declare `extern crate alloc`:
//! the multi-element arm of `nonempty!` used a bare `alloc::vec!`, which only
//! resolved inside `ordofp_core` itself (`CLEANUP_REPORT` §6a item 1).

use ordofp::nonempty;

#[test]
fn nonempty_macro_multi_element_works_without_extern_alloc() {
    let nel = nonempty![1, 2, 3];
    assert_eq!(nel.head(), &1);
    assert_eq!(nel.as_vec(), vec![1, 2, 3]);
}

#[test]
fn nonempty_macro_singleton_works_without_extern_alloc() {
    let single = nonempty![42];
    assert_eq!(single.head(), &42);
}

#[test]
fn nonempty_macro_trailing_comma() {
    let nel = nonempty![1, 2,];
    assert_eq!(nel.as_vec(), vec![1, 2]);
}
