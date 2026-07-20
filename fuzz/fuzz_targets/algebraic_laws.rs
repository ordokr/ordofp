//! Coverage-guided law checking: drives the ordofp_laws properties from
//! fuzzer-generated structured inputs, so law counterexamples are hunted with
//! coverage feedback instead of plain random generation.

#![no_main]
use libfuzzer_sys::fuzz_target;
use ordofp_laws::{functor_laws, monad_laws, monoid_laws, semigroup_laws};

fuzz_target!(|data: (Vec<i8>, Vec<i8>, Vec<i8>, String, String, Option<i64>)| {
    let (a, b, c, s1, s2, opt) = data;

    // Semigroup associativity + monoid identities for the two workhorse
    // instances (Vec concat, String concat).
    assert!(semigroup_laws::associativity(a.clone(), b.clone(), c));
    assert!(monoid_laws::left_identity(a.clone()));
    assert!(monoid_laws::right_identity(a));
    assert!(semigroup_laws::associativity(
        s1.clone(),
        s2.clone(),
        String::new()
    ));
    assert!(monoid_laws::left_identity(s1));
    assert!(monoid_laws::right_identity(s2));

    // Functor identity/composition and monad laws for Option through the
    // ordofp GAT instances, with fixed non-trivial arrows.
    assert!(functor_laws::option_identity(opt));
    assert!(functor_laws::option_composition(
        opt,
        |x: i64| x.wrapping_mul(3),
        |x: i64| x.wrapping_add(7),
    ));
    assert!(monad_laws::option_left_identity(
        b.first().map_or(0, |&x| i64::from(x)),
        |x: i64| x.checked_add(1),
    ));
    assert!(monad_laws::option_associativity(
        opt,
        |x: i64| x.checked_mul(2),
        |x: i64| x.checked_sub(5),
    ));
});
