//! Characterization tests for `dependent::vect::FromArray`, written ahead of
//! macro-ising the eight copy-paste `FromArray` impls.
//!
//! Pins every existing impl (N = 0..=8): the produced length (runtime and
//! type-level), element order, and first/last accessors. The impl set stops
//! at N = 8 — the macro must reproduce exactly this set.

#![cfg(feature = "dependent")]

use ordofp_core::dependent::peano::{Succ, Zero};
use ordofp_core::dependent::vect::{FromArray, Vectum};

type N0 = Zero;
type N1 = Succ<N0>;
type N2 = Succ<N1>;
type N3 = Succ<N2>;
type N4 = Succ<N3>;
type N5 = Succ<N4>;
type N6 = Succ<N5>;
type N7 = Succ<N6>;
type N8 = Succ<N7>;

#[test]
fn from_array_zero_is_empty() {
    let v: Vectum<i32, N0> = Vectum::from_array([]);
    assert_eq!(v.len(), 0);
    assert!(v.is_empty());
    assert_eq!(Vectum::<i32, N0>::type_len(), 0);
    assert_eq!(v.as_slice(), &[] as &[i32]);
}

#[test]
fn from_array_each_size_preserves_order_and_length() {
    let v1: Vectum<i32, N1> = Vectum::from_array([1]);
    assert_eq!(v1.as_slice(), &[1]);
    assert_eq!((v1.len(), Vectum::<i32, N1>::type_len()), (1, 1));

    let v2: Vectum<i32, N2> = Vectum::from_array([1, 2]);
    assert_eq!(v2.as_slice(), &[1, 2]);
    assert_eq!((v2.len(), Vectum::<i32, N2>::type_len()), (2, 2));

    let v3: Vectum<i32, N3> = Vectum::from_array([1, 2, 3]);
    assert_eq!(v3.as_slice(), &[1, 2, 3]);
    assert_eq!((v3.len(), Vectum::<i32, N3>::type_len()), (3, 3));

    let v4: Vectum<i32, N4> = Vectum::from_array([1, 2, 3, 4]);
    assert_eq!(v4.as_slice(), &[1, 2, 3, 4]);
    assert_eq!((v4.len(), Vectum::<i32, N4>::type_len()), (4, 4));

    let v5: Vectum<i32, N5> = Vectum::from_array([1, 2, 3, 4, 5]);
    assert_eq!(v5.as_slice(), &[1, 2, 3, 4, 5]);
    assert_eq!((v5.len(), Vectum::<i32, N5>::type_len()), (5, 5));

    let v6: Vectum<i32, N6> = Vectum::from_array([1, 2, 3, 4, 5, 6]);
    assert_eq!(v6.as_slice(), &[1, 2, 3, 4, 5, 6]);
    assert_eq!((v6.len(), Vectum::<i32, N6>::type_len()), (6, 6));

    let v7: Vectum<i32, N7> = Vectum::from_array([1, 2, 3, 4, 5, 6, 7]);
    assert_eq!(v7.as_slice(), &[1, 2, 3, 4, 5, 6, 7]);
    assert_eq!((v7.len(), Vectum::<i32, N7>::type_len()), (7, 7));

    let v8: Vectum<i32, N8> = Vectum::from_array([1, 2, 3, 4, 5, 6, 7, 8]);
    assert_eq!(v8.as_slice(), &[1, 2, 3, 4, 5, 6, 7, 8]);
    assert_eq!((v8.len(), Vectum::<i32, N8>::type_len()), (8, 8));
}

#[test]
fn from_array_supports_nonzero_accessors() {
    let v1: Vectum<i32, N1> = Vectum::from_array([10]);
    assert_eq!((v1.caput(), v1.ultimus()), (&10, &10));

    let v3: Vectum<i32, N3> = Vectum::from_array([1, 2, 3]);
    assert_eq!((v3.caput(), v3.ultimus()), (&1, &3));

    let v8: Vectum<i32, N8> = Vectum::from_array([1, 2, 3, 4, 5, 6, 7, 8]);
    assert_eq!((v8.caput(), v8.ultimus()), (&1, &8));
}

#[test]
fn from_array_works_for_non_copy_types() {
    let v2: Vectum<String, N2> = Vectum::from_array([String::from("a"), String::from("b")]);
    assert_eq!(v2.as_slice(), &[String::from("a"), String::from("b")]);
    assert_eq!(v2.ultimus().as_str(), "b");
}
