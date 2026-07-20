//! Law tests for OrdoFP's own datatypes.
//!
//! `tests/laws_suite.rs` runs the `ordofp_laws` machinery against std types
//! (`Option`, `Vec`, `Result`, tuples). This suite points the same laws at
//! the library's own instances — the semigroup/monoid wrappers, HList
//! monoids, `NonEmpty`'s functor/applicative/monad, and `Probatum`'s
//! error-accumulating applicative — which previously had no law coverage.

#![cfg(feature = "Probatum")]

use ordofp::nonempty::NonEmpty;
use ordofp::typeclasses::{Applicatio, Apply, Functor};
use ordofp::validated::Probatum;
use ordofp::wrappers::{Aggregatio, Aliquid, Multiplicatio, Omnis, Primus, Ultimus};
use ordofp::{HList, hlist};
use ordofp_laws::{monoid_laws, semigroup_laws};
use quickcheck::quickcheck;

// =============================================================================
// Semigroup / monoid laws for the wrapper newtypes
// =============================================================================

quickcheck! {
    // Wrapping arithmetic isn't available through the wrappers, so drive the
    // numeric ones with i8 widened to i64: |i8|^3 products can't overflow.
    fn multiplicatio_associativity(a: i8, b: i8, c: i8) -> bool {
        semigroup_laws::associativity(
            Multiplicatio(i64::from(a)),
            Multiplicatio(i64::from(b)),
            Multiplicatio(i64::from(c)),
        )
    }

    fn aggregatio_associativity(a: i8, b: i8, c: i8) -> bool {
        semigroup_laws::associativity(
            Aggregatio(i64::from(a)),
            Aggregatio(i64::from(b)),
            Aggregatio(i64::from(c)),
        )
    }

    fn multiplicatio_identity(a: i8) -> bool {
        let a = Multiplicatio(i64::from(a));
        monoid_laws::left_identity(a) && monoid_laws::right_identity(a)
    }

    fn aggregatio_identity(a: i8) -> bool {
        let a = Aggregatio(i64::from(a));
        monoid_laws::left_identity(a) && monoid_laws::right_identity(a)
    }

    fn omnis_bool_monoid_laws(a: bool, b: bool, c: bool) -> bool {
        semigroup_laws::associativity(Omnis(a), Omnis(b), Omnis(c))
            && monoid_laws::left_identity(Omnis(a))
            && monoid_laws::right_identity(Omnis(a))
    }

    fn aliquid_bool_monoid_laws(a: bool, b: bool, c: bool) -> bool {
        semigroup_laws::associativity(Aliquid(a), Aliquid(b), Aliquid(c))
            && monoid_laws::left_identity(Aliquid(a))
            && monoid_laws::right_identity(Aliquid(a))
    }

    fn primus_ultimus_associativity(a: i32, b: i32, c: i32) -> bool {
        semigroup_laws::associativity(Primus(a), Primus(b), Primus(c))
            && semigroup_laws::associativity(Ultimus(a), Ultimus(b), Ultimus(c))
    }

    /// `Option<T: Compositio>` lifts a semigroup to a monoid with `None` as
    /// identity.
    fn option_aggregatio_monoid_laws(a: Option<i8>, b: Option<i8>, c: Option<i8>) -> bool {
        let lift = |o: Option<i8>| o.map(|x| Aggregatio(i64::from(x)));
        let (a, b, c) = (lift(a), lift(b), lift(c));
        semigroup_laws::associativity(a, b, c)
            && monoid_laws::left_identity(a)
            && monoid_laws::right_identity(a)
    }

    /// HLists of semigroups combine component-wise. (Coniunctio implements
    /// Compositio but not Unitas, so only associativity is checkable.)
    fn hlist_semigroup_associativity(s1: String, v1: Vec<u8>, s2: String, v2: Vec<u8>) -> bool {
        let a: HList![String, Vec<u8>] = hlist![s1, v1];
        let b: HList![String, Vec<u8>] = hlist![s2, v2];
        let c: HList![String, Vec<u8>] = hlist![String::from("c"), vec![9]];
        semigroup_laws::associativity(a, b, c)
    }
}

// =============================================================================
// NonEmpty: functor / applicative / monad laws
// =============================================================================

fn ne(head: i8, tail: Vec<i8>) -> NonEmpty<i8> {
    NonEmpty::new(head, tail)
}

quickcheck! {
    fn nonempty_functor_identity(head: i8, tail: Vec<i8>) -> bool {
        let fa = ne(head, tail);
        fa.clone().map(|x| x) == fa
    }

    fn nonempty_functor_composition(head: i8, tail: Vec<i8>) -> bool {
        let fa = ne(head, tail);
        let f = |x: i8| x.wrapping_add(1);
        let g = |x: i8| x.wrapping_mul(3);
        fa.clone().map(f).map(g) == fa.map(|x| g(f(x)))
    }

    fn nonempty_monad_left_identity(a: i8) -> bool {
        let f = |x: i8| NonEmpty::new(x, vec![x.wrapping_add(1)]);
        NonEmpty::pure(a).flat_map(f) == f(a)
    }

    fn nonempty_monad_right_identity(head: i8, tail: Vec<i8>) -> bool {
        let m = ne(head, tail);
        m.clone().flat_map(NonEmpty::pure) == m
    }

    fn nonempty_monad_associativity(head: i8, tail: Vec<i8>) -> bool {
        let m = ne(head, tail);
        let f = |x: i8| NonEmpty::new(x, vec![x.wrapping_add(1)]);
        let g = |x: i8| NonEmpty::new(x.wrapping_mul(2), vec![]);
        m.clone().flat_map(f).flat_map(g) == m.flat_map(|x| f(x).flat_map(g))
    }

    fn nonempty_applicative_homomorphism(a: i8) -> bool {
        let f = |x: i8| x.wrapping_mul(2);
        NonEmpty::pure(a).apply(NonEmpty::pure(f)) == NonEmpty::<i8>::pure(f(a))
    }
}

// =============================================================================
// Probatum: functor laws and error-accumulating applicative
// =============================================================================

/// Build a `Probatum` from quickcheck-generated parts: `Ok` → Valid,
/// `Err(msgs)` → Invalid with one error per message (at least one).
fn probatum(input: Result<i32, (u8, Vec<u8>)>) -> Probatum<u8, i32> {
    match input {
        Ok(v) => Probatum::valid(v),
        Err((first, rest)) => {
            Probatum::invalid_many(core::iter::once(first).chain(rest.into_iter().take(3)))
        }
    }
}

quickcheck! {
    fn probatum_functor_identity(input: Result<i32, (u8, Vec<u8>)>) -> bool {
        let fa = probatum(input);
        fa.clone().map(|x| x) == fa
    }

    fn probatum_functor_composition(input: Result<i32, (u8, Vec<u8>)>) -> bool {
        let fa = probatum(input);
        let f = |x: i32| x.wrapping_add(1);
        let g = |x: i32| x.wrapping_mul(3);
        fa.clone().map(f).map(g) == fa.map(|x| g(f(x)))
    }

    /// The whole point of Probatum over Result: two invalids ACCUMULATE
    /// (left errors first), they don't short-circuit.
    fn probatum_map2_accumulates_all_errors(
        a: Result<i32, (u8, Vec<u8>)>,
        b: Result<i32, (u8, Vec<u8>)>
    ) -> bool {
        let (va, vb) = (probatum(a), probatum(b));
        let ea = va.errors().map(<[u8]>::to_vec);
        let eb = vb.errors().map(<[u8]>::to_vec);

        let combined = va.map2(vb, |x, y| (x, y));
        match (ea, eb) {
            (None, None) => combined.is_valid(),
            (Some(e), None) | (None, Some(e)) => combined.errors() == Some(&e[..]),
            (Some(mut e1), Some(e2)) => {
                e1.extend(e2);
                combined.errors() == Some(&e1[..])
            }
        }
    }

    /// `collect` is all-or-nothing: every error from every invalid entry, in
    /// entry order; valid values only when nothing failed.
    fn probatum_collect_gathers_errors_in_order(
        entries: Vec<Result<i32, (u8, Vec<u8>)>>
    ) -> bool {
        let items: Vec<Probatum<u8, i32>> = entries.into_iter().map(probatum).collect();
        let expected_errors: Vec<u8> = items
            .iter()
            .filter_map(|p| p.errors())
            .flatten()
            .copied()
            .collect();
        let expected_values: Vec<i32> = items.iter().filter_map(|p| p.value()).copied().collect();

        let collected = Probatum::<u8, i32>::collect(items.clone());
        if expected_errors.is_empty() {
            collected == Probatum::valid(expected_values)
        } else {
            collected.errors() == Some(&expected_errors[..])
        }
    }

    /// Applicative identity: `v.ap(pure(id)) == v`.
    fn probatum_ap_identity(input: Result<i32, (u8, Vec<u8>)>) -> bool {
        let v = probatum(input);
        v.clone().ap(Probatum::valid(|x: i32| x)) == v
    }
}

#[test]
fn probatum_round_trips_through_result() {
    let valid: Probatum<&str, i32> = Ok::<_, &str>(7).into();
    assert_eq!(valid, Probatum::valid(7));
    assert_eq!(valid.into_result(), Ok(7));

    let invalid: Probatum<&str, i32> = Err::<i32, _>("boom").into();
    assert_eq!(invalid.errors(), Some(&["boom"][..]));
    assert_eq!(
        invalid.into_result().expect_err("must stay invalid")[..],
        ["boom"][..]
    );
}
