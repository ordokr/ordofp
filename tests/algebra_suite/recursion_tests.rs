//! Tests for `OrdoFP` recursion schemes.
//!
//! Tests cata, ana, hylo, para, apo, histo, futu, zygo, dyna, chrono, and
//! the Mendler-style variants (mcata, mana, mhylo).

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

use ordofp_core::recursion::{
    Aut, Cofree, Corecursiva, Free, FunctorBasis, ListF, ListFWitness, NatF, NatFWitness,
    Recursiva, ana, apo, cata, chrono, dyna, futu, histo, hylo, mana, mcata, mhylo, para, zygo,
};

// =============================================================================
// FIXED POINT TYPE FOR TESTING
// =============================================================================

/// Fixed point of a functor.
#[derive(Debug, Clone, PartialEq)]
struct FixNat(Option<Box<FixNat>>);

impl FixNat {
    fn zero() -> Self {
        FixNat(None)
    }

    fn succ(n: Self) -> Self {
        FixNat(Some(Box::new(n)))
    }

    fn from_usize(n: usize) -> Self {
        if n == 0 {
            Self::zero()
        } else {
            Self::succ(Self::from_usize(n - 1))
        }
    }

    fn to_usize(&self) -> usize {
        match &self.0 {
            None => 0,
            Some(n) => 1 + n.to_usize(),
        }
    }
}

impl FunctorBasis for FixNat {
    type Base = NatFWitness;
}

impl Recursiva for FixNat {
    fn project(self) -> NatF<Self> {
        match self.0 {
            None => NatF::ZeroF,
            Some(n) => NatF::SuccF(*n),
        }
    }
}

impl Corecursiva for FixNat {
    fn embed(layer: NatF<Self>) -> Self {
        match layer {
            NatF::ZeroF => FixNat(None),
            NatF::SuccF(n) => FixNat(Some(Box::new(n))),
        }
    }
}

/// Fixed point for List functor.
#[derive(Debug, Clone, PartialEq)]
enum FixList<E> {
    Nil,
    Cons(E, Box<FixList<E>>),
}

impl<E> FixList<E> {
    fn nil() -> Self {
        FixList::Nil
    }

    fn cons(head: E, tail: Self) -> Self {
        FixList::Cons(head, Box::new(tail))
    }
}

impl<E: Clone + 'static> FunctorBasis for FixList<E> {
    type Base = ListFWitness<E>;
}

impl<E: Clone + 'static> Recursiva for FixList<E> {
    fn project(self) -> ListF<E, Self> {
        match self {
            FixList::Nil => ListF::NilF,
            FixList::Cons(h, t) => ListF::ConsF(h, *t),
        }
    }
}

impl<E: Clone + 'static> Corecursiva for FixList<E> {
    fn embed(layer: ListF<E, Self>) -> Self {
        match layer {
            ListF::NilF => FixList::Nil,
            ListF::ConsF(h, t) => FixList::Cons(h, Box::new(t)),
        }
    }
}

// =============================================================================
// CATAMORPHISM TESTS
// =============================================================================

#[test]
fn test_cata_nat_to_usize() {
    let nat5 = FixNat::from_usize(5);

    let result: usize = cata(
        |layer: NatF<usize>| match layer {
            NatF::ZeroF => 0,
            NatF::SuccF(n) => n + 1,
        },
        nat5,
    );

    assert_eq!(result, 5);
}

#[test]
fn test_cata_nat_sum() {
    // Sum 1 + 2 + ... + n
    let nat4 = FixNat::from_usize(4);

    let result: (usize, usize) = cata(
        |layer: NatF<(usize, usize)>| match layer {
            NatF::ZeroF => (0, 0), // (depth, sum)
            NatF::SuccF((depth, sum)) => (depth + 1, sum + depth + 1),
        },
        nat4,
    );

    // Sum 1+2+3+4 = 10
    assert_eq!(result, (4, 10));
}

#[test]
fn test_cata_list_sum() {
    let list: FixList<i32> = FixList::cons(1, FixList::cons(2, FixList::cons(3, FixList::nil())));

    let sum: i32 = cata(
        |layer: ListF<i32, i32>| match layer {
            ListF::NilF => 0,
            ListF::ConsF(x, acc) => x + acc,
        },
        list,
    );

    assert_eq!(sum, 6);
}

#[test]
fn test_cata_list_length() {
    let list: FixList<&str> =
        FixList::cons("a", FixList::cons("b", FixList::cons("c", FixList::nil())));

    let len: usize = cata(
        |layer: ListF<&str, usize>| match layer {
            ListF::NilF => 0,
            ListF::ConsF(_, acc) => acc + 1,
        },
        list,
    );

    assert_eq!(len, 3);
}

// =============================================================================
// ANAMORPHISM TESTS
// =============================================================================

#[test]
fn test_ana_build_nat() {
    let nat: FixNat = ana(
        |n: usize| {
            if n == 0 {
                NatF::ZeroF
            } else {
                NatF::SuccF(n - 1)
            }
        },
        5,
    );

    assert_eq!(nat.to_usize(), 5);
}

#[test]
fn test_ana_build_list_range() {
    let list: FixList<i32> = ana(
        |n: i32| {
            if n <= 0 {
                ListF::NilF
            } else {
                ListF::ConsF(n, n - 1)
            }
        },
        3,
    );

    // Should be [3, 2, 1]
    let result: Vec<i32> = cata(
        |layer: ListF<i32, Vec<i32>>| match layer {
            ListF::NilF => vec![],
            ListF::ConsF(x, mut xs) => {
                xs.insert(0, x);
                xs
            }
        },
        list,
    );

    assert_eq!(result, vec![3, 2, 1]);
}

// =============================================================================
// HYLOMORPHISM TESTS
// =============================================================================

#[test]
fn test_hylo_identity_nat() {
    // Build a nat and immediately tear it down - should get same value
    let identity: usize = hylo::<NatFWitness, _, _, _, _>(
        |layer: NatF<usize>| match layer {
            NatF::ZeroF => 0,
            NatF::SuccF(acc) => acc + 1,
        },
        |n: usize| {
            if n == 0 {
                NatF::ZeroF
            } else {
                NatF::SuccF(n - 1)
            }
        },
        5,
    );

    assert_eq!(identity, 5);
}

#[test]
fn test_hylo_sum_list() {
    // Build a list [1..n] and sum it, without building intermediate structure
    let sum: i32 = hylo::<ListFWitness<i32>, _, _, _, _>(
        |layer: ListF<i32, i32>| match layer {
            ListF::NilF => 0,
            ListF::ConsF(x, acc) => x + acc,
        },
        |n: i32| {
            if n <= 0 {
                ListF::NilF
            } else {
                ListF::ConsF(n, n - 1)
            }
        },
        5,
    );

    // 5 + 4 + 3 + 2 + 1 = 15
    assert_eq!(sum, 15);
}

// =============================================================================
// PARAMORPHISM TESTS
// =============================================================================

#[test]
fn test_para_tails() {
    // Using para to get access to original structure
    let nat3 = FixNat::from_usize(3);

    // Count and verify we have access to subtrees
    let result: Vec<usize> = para(
        |layer: NatF<(FixNat, Vec<usize>)>| match layer {
            NatF::ZeroF => vec![0],
            NatF::SuccF((orig, mut acc)) => {
                acc.insert(0, orig.to_usize() + 1);
                acc
            }
        },
        nat3,
    );

    // Should be [3, 2, 1, 0]
    assert_eq!(result, vec![3, 2, 1, 0]);
}

#[test]
fn test_para_predecessor() {
    let nat5 = FixNat::from_usize(5);

    // Get the predecessor if it exists
    let pred: Option<FixNat> = para(
        |layer: NatF<(FixNat, Option<FixNat>)>| match layer {
            NatF::ZeroF => None,
            NatF::SuccF((orig, _)) => Some(orig),
        },
        nat5,
    );

    assert!(pred.is_some());
    assert_eq!(
        pred.expect("predecessor of Nat(5) should be Nat(4)")
            .to_usize(),
        4
    );
}

// =============================================================================
// APOMORPHISM TESTS
// =============================================================================

#[test]
fn test_apo_early_termination() {
    // Build nat but terminate early by returning pre-built structure
    let nat: FixNat = apo(
        |n: usize| {
            if n == 0 {
                NatF::ZeroF
            } else if n <= 2 {
                // Continue normally
                NatF::SuccF(Aut::Dexter(n - 1))
            } else {
                // Terminate early with pre-built nat 2
                NatF::SuccF(Aut::Sinister(FixNat::from_usize(2)))
            }
        },
        5,
    );

    // Result should be 3: succ(succ(succ(2))) where we jumped to 2 directly
    // Actually: succ(Sinister(nat2)) = succ(nat2) = nat3
    assert_eq!(nat.to_usize(), 3);
}

#[test]
fn test_apo_list_replicate() {
    // Replicate an element n times, but with early termination capability
    let list: FixList<i32> = apo(
        |n: i32| {
            if n <= 0 {
                ListF::NilF
            } else {
                ListF::ConsF(42, Aut::Dexter(n - 1))
            }
        },
        3,
    );

    let result: Vec<i32> = cata(
        |layer: ListF<i32, Vec<i32>>| match layer {
            ListF::NilF => vec![],
            ListF::ConsF(x, mut xs) => {
                xs.insert(0, x);
                xs
            }
        },
        list,
    );

    assert_eq!(result, vec![42, 42, 42]);
}

// =============================================================================
// HISTOMORPHISM TESTS
// =============================================================================

#[test]
fn test_histo_fib() {
    // Fibonacci using histo
    // fib(0) = 0
    // fib(1) = 1
    // fib(n) = fib(n-1) + fib(n-2)

    let nat5 = FixNat::from_usize(5);

    let result: usize = histo(
        |layer: NatF<Cofree<NatFWitness, usize>>| match layer {
            NatF::ZeroF => 0,
            NatF::SuccF(prev) => {
                let n_minus_1 = prev.extract();
                // Access n-2 from the history of n-1
                let n_minus_2 = match prev.children_ref() {
                    NatF::ZeroF => 0,
                    NatF::SuccF(prev_prev) => *prev_prev.extract(),
                };

                if *n_minus_1 == 0 {
                    1
                } else {
                    n_minus_1 + n_minus_2
                }
            }
        },
        nat5,
    );

    // 0, 1, 1, 2, 3, 5
    assert_eq!(result, 5);
}

#[test]
fn test_histo_primes() {
    // Determine if a number is prime using histo's access to the FULL
    // history chain: each node's attribute carries (n, n_is_prime), and the
    // primality of n is decided by walking every predecessor (1..n) through
    // the Cofree history and trial-dividing. Not efficient — the point is
    // exercising deep history access, beyond the one-step peek of fib above.
    fn is_prime_via_histo(n: usize) -> bool {
        let nat = FixNat::from_usize(n);
        let (value, prime): (usize, bool) = histo(
            |layer: NatF<Cofree<NatFWitness, (usize, bool)>>| match layer {
                NatF::ZeroF => (0, false), // 0 is not prime
                NatF::SuccF(prev) => {
                    let n = prev.extract().0 + 1;
                    // Walk the entire history: predecessors carry n-1, n-2, ..., 0
                    let mut divisible = false;
                    let mut cur = &prev;
                    loop {
                        let m = cur.extract().0;
                        if m >= 2 && n % m == 0 {
                            divisible = true;
                        }
                        match cur.children_ref() {
                            NatF::ZeroF => break,
                            NatF::SuccF(p) => cur = p,
                        }
                    }
                    (n, n >= 2 && !divisible)
                }
            },
            nat,
        );
        assert_eq!(value, n, "attribute must carry the node's own value");
        prime
    }

    assert!(!is_prime_via_histo(0));
    assert!(!is_prime_via_histo(1));
    assert!(is_prime_via_histo(2));
    assert!(is_prime_via_histo(5));
    assert!(!is_prime_via_histo(6));
    assert!(is_prime_via_histo(7));
    assert!(!is_prime_via_histo(9));
}

// =============================================================================
// FUTUMORPHISM TESTS
// =============================================================================

#[test]
fn test_futu_multi_layer() {
    // Build nat producing multiple layers at once
    let nat: FixNat = futu(
        |n: usize| {
            if n == 0 {
                NatF::ZeroF
            } else if n == 1 {
                NatF::SuccF(Free::purus(0)) // One more layer, then done
            } else {
                // Produce two layers at once
                NatF::SuccF(Free::suspensus(NatF::SuccF(Free::purus(n - 2))))
            }
        },
        4,
    );

    assert_eq!(nat.to_usize(), 4);
}

// =============================================================================
// ZYGOMORPHISM TESTS
// =============================================================================

#[test]
fn test_zygo_factorial_nat() {
    // Factorial genuinely needs zygo: the algebra must know the *value* of
    // the predecessor (auxiliary fold) to multiply by, not just its factorial.
    let nat5 = FixNat::from_usize(5);

    let result: usize = zygo(
        // Auxiliary algebra: recover the numeric value of each node.
        |layer: NatF<usize>| match layer {
            NatF::ZeroF => 0,
            NatF::SuccF(n) => n + 1,
        },
        // Main algebra: fact(n+1) = (n+1) * fact(n), using the aux value n.
        |layer: NatF<(usize, usize)>| match layer {
            NatF::ZeroF => 1,
            NatF::SuccF((pred_value, fact_pred)) => (pred_value + 1) * fact_pred,
        },
        nat5,
    );

    assert_eq!(result, 120);
}

#[test]
fn test_zygo_factorial_zero() {
    let result: usize = zygo(
        |layer: NatF<usize>| match layer {
            NatF::ZeroF => 0,
            NatF::SuccF(n) => n + 1,
        },
        |layer: NatF<(usize, usize)>| match layer {
            NatF::ZeroF => 1,
            NatF::SuccF((pred_value, fact_pred)) => (pred_value + 1) * fact_pred,
        },
        FixNat::zero(),
    );

    assert_eq!(result, 1);
}

#[test]
fn test_zygo_sum_where_tail_length_even() {
    // Sum only the elements whose tail has even length. The main algebra
    // needs the tail *length* (auxiliary fold), which the sum alone can't see.
    let list: FixList<i32> = FixList::cons(
        1,
        FixList::cons(2, FixList::cons(3, FixList::cons(4, FixList::nil()))),
    );

    let result: i32 = zygo(
        // Auxiliary algebra: list length.
        |layer: ListF<i32, usize>| match layer {
            ListF::NilF => 0,
            ListF::ConsF(_, n) => n + 1,
        },
        // Main algebra: keep x when its tail length is even.
        |layer: ListF<i32, (usize, i32)>| match layer {
            ListF::NilF => 0,
            ListF::ConsF(x, (tail_len, acc)) => {
                if tail_len % 2 == 0 {
                    acc + x
                } else {
                    acc
                }
            }
        },
        list,
    );

    // Tails: 1 -> [2,3,4] (len 3), 2 -> [3,4] (len 2), 3 -> [4] (len 1),
    // 4 -> [] (len 0). Even tail lengths keep 2 and 4.
    assert_eq!(result, 6);
}

// =============================================================================
// DYNAMORPHISM TESTS
// =============================================================================

/// Course-of-values fibonacci algebra shared by the dyna/chrono tests: reads
/// fib(n-1) from the annotation and fib(n-2) one step deeper in the history.
fn fib_alg(layer: NatF<Cofree<NatFWitness, usize>>) -> usize {
    match layer {
        NatF::ZeroF => 0,
        NatF::SuccF(prev) => {
            let n_minus_1 = *prev.extract();
            let n_minus_2 = match prev.children_ref() {
                NatF::ZeroF => 0,
                NatF::SuccF(prev_prev) => *prev_prev.extract(),
            };
            if n_minus_1 == 0 {
                1
            } else {
                n_minus_1 + n_minus_2
            }
        }
    }
}

fn nat_coalg(n: usize) -> NatF<usize> {
    if n == 0 {
        NatF::ZeroF
    } else {
        NatF::SuccF(n - 1)
    }
}

#[test]
fn test_dyna_fib() {
    // Fibonacci straight from a usize seed: dyna = ana + histo without
    // materializing the intermediate FixNat.
    let result: usize = dyna::<NatFWitness, usize, usize, _, _>(fib_alg, nat_coalg, 10);

    // 0, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55
    assert_eq!(result, 55);
}

#[test]
fn test_dyna_zero_seed() {
    let result: usize = dyna::<NatFWitness, usize, usize, _, _>(fib_alg, nat_coalg, 0);
    assert_eq!(result, 0);
}

#[test]
fn test_dyna_matches_ana_then_histo() {
    // dyna(alg, coalg, n) must agree with histo(alg, ana(coalg, n)).
    for n in 0..10 {
        let direct: usize = dyna::<NatFWitness, usize, usize, _, _>(fib_alg, nat_coalg, n);

        let nat: FixNat = ana(nat_coalg, n);
        let via_histo: usize = histo(fib_alg, nat);

        assert_eq!(direct, via_histo, "dyna and ana;histo disagree at n={n}");
    }
}

// =============================================================================
// CHRONOMORPHISM TESTS
// =============================================================================

/// Futu-style coalgebra that emits two `SuccF` layers per step (like the
/// futu test), so the chrono tests exercise the `Free::Suspensus` branch.
fn two_layer_coalg(n: usize) -> NatF<Free<NatFWitness, usize>> {
    if n == 0 {
        NatF::ZeroF
    } else if n == 1 {
        NatF::SuccF(Free::purus(0))
    } else {
        NatF::SuccF(Free::suspensus(NatF::SuccF(Free::purus(n - 2))))
    }
}

#[test]
fn test_chrono_depth_multi_layer() {
    // Depth of the structure built by a two-layers-at-a-time unfold must
    // still be the seed value, whichever unfold path produced each node.
    for n in 0..8 {
        let depth: usize = chrono::<NatFWitness, usize, usize, _, _>(
            |layer: NatF<Cofree<NatFWitness, usize>>| match layer {
                NatF::ZeroF => 0,
                NatF::SuccF(prev) => prev.extract() + 1,
            },
            two_layer_coalg,
            n,
        );
        assert_eq!(depth, n);
    }
}

#[test]
fn test_chrono_fib_multi_layer() {
    // Course-of-values algebra over a multi-layer unfold: the history seen
    // by the algebra must be identical whether a node came from the direct
    // unfold or from inside a suspended layer.
    let result: usize = chrono::<NatFWitness, usize, usize, _, _>(fib_alg, two_layer_coalg, 10);
    assert_eq!(result, 55);

    let zero: usize = chrono::<NatFWitness, usize, usize, _, _>(fib_alg, two_layer_coalg, 0);
    assert_eq!(zero, 0);
}

// =============================================================================
// MENDLER-STYLE TESTS
// =============================================================================

#[test]
fn test_mcata_nat_to_usize() {
    let nat5 = FixNat::from_usize(5);

    let result: usize = mcata(
        |recurse: &dyn Fn(FixNat) -> usize, layer: NatF<FixNat>| match layer {
            NatF::ZeroF => 0,
            NatF::SuccF(n) => 1 + recurse(n),
        },
        nat5,
    );

    assert_eq!(result, 5);
}

#[test]
fn test_mana_build_nat() {
    let nat: FixNat = mana(
        |embed: &dyn Fn(usize) -> FixNat, n: usize| {
            if n == 0 {
                NatF::ZeroF
            } else {
                NatF::SuccF(embed(n - 1))
            }
        },
        5,
    );

    assert_eq!(nat.to_usize(), 5);
}

#[test]
fn test_mhylo_sum_list() {
    // Sum 1..=n without materializing the list, Mendler-style: the algebra
    // drives recursion itself through the supplied `recurse` handle.
    let sum: i32 = mhylo::<ListFWitness<i32>, i32, i32, _, _>(
        |recurse: &dyn Fn(i32) -> i32, layer: ListF<i32, i32>| match layer {
            ListF::NilF => 0,
            ListF::ConsF(x, rest_seed) => x + recurse(rest_seed),
        },
        |n: i32| {
            if n <= 0 {
                ListF::NilF
            } else {
                ListF::ConsF(n, n - 1)
            }
        },
        5,
    );

    assert_eq!(sum, 15);
}

#[test]
fn test_mhylo_matches_hylo() {
    // mhylo must agree with hylo when the Mendler algebra just recurses
    // into every seed position.
    for n in 0..10usize {
        let via_mhylo: usize = mhylo::<NatFWitness, usize, usize, _, _>(
            |recurse: &dyn Fn(usize) -> usize, layer: NatF<usize>| match layer {
                NatF::ZeroF => 0,
                NatF::SuccF(seed) => recurse(seed) + 1,
            },
            nat_coalg,
            n,
        );

        let via_hylo: usize = hylo::<NatFWitness, usize, usize, _, _>(
            |layer: NatF<usize>| match layer {
                NatF::ZeroF => 0,
                NatF::SuccF(acc) => acc + 1,
            },
            nat_coalg,
            n,
        );

        assert_eq!(via_mhylo, via_hylo, "mhylo and hylo disagree at n={n}");
    }
}

#[test]
fn test_mhylo_early_stop() {
    // The Mendler algebra controls recursion: it can decline to call
    // `recurse`, cutting off an otherwise infinite unfold.
    let result: usize = mhylo::<NatFWitness, usize, usize, _, _>(
        |recurse: &dyn Fn(usize) -> usize, layer: NatF<usize>| match layer {
            NatF::ZeroF => 0,
            // Stop after 3 levels regardless of what the coalgebra produces.
            NatF::SuccF(seed) => {
                if seed > 3 {
                    3
                } else {
                    recurse(seed) + 1
                }
            }
        },
        // Coalgebra that never reaches ZeroF on its own for large seeds:
        // it counts down forever from usize::MAX.
        |n: usize| {
            if n == 0 {
                NatF::ZeroF
            } else {
                NatF::SuccF(n - 1)
            }
        },
        usize::MAX,
    );

    assert_eq!(result, 3);
}

// =============================================================================
// EDGE CASES
// =============================================================================

#[test]
fn test_cata_empty_list() {
    let list: FixList<i32> = FixList::nil();

    let sum: i32 = cata(
        |layer: ListF<i32, i32>| match layer {
            ListF::NilF => 0,
            ListF::ConsF(x, acc) => x + acc,
        },
        list,
    );

    assert_eq!(sum, 0);
}

#[test]
fn test_cata_zero_nat() {
    let nat = FixNat::zero();

    let result: usize = cata(
        |layer: NatF<usize>| match layer {
            NatF::ZeroF => 0,
            NatF::SuccF(n) => n + 1,
        },
        nat,
    );

    assert_eq!(result, 0);
}

#[test]
fn test_ana_zero() {
    let nat: FixNat = ana(|(): ()| NatF::ZeroF, ());

    assert_eq!(nat.to_usize(), 0);
}

// =============================================================================
// ROUNDTRIP TESTS
// =============================================================================

#[test]
fn test_nat_roundtrip() {
    // Build with ana, tear down with cata, should get same value
    for n in 0..10 {
        let nat: FixNat = ana(
            |m: usize| {
                if m == 0 {
                    NatF::ZeroF
                } else {
                    NatF::SuccF(m - 1)
                }
            },
            n,
        );

        let result: usize = cata(
            |layer: NatF<usize>| match layer {
                NatF::ZeroF => 0,
                NatF::SuccF(m) => m + 1,
            },
            nat,
        );

        assert_eq!(result, n);
    }
}

#[test]
fn test_list_roundtrip() {
    let original = vec![1, 2, 3, 4, 5];

    // Build list from vec using ana
    let list: FixList<i32> = ana(
        |v: Vec<i32>| {
            if v.is_empty() {
                ListF::NilF
            } else {
                let mut rest = v;
                let head = rest.remove(0);
                ListF::ConsF(head, rest)
            }
        },
        original.clone(),
    );

    // Tear down back to vec using cata
    let result: Vec<i32> = cata(
        |layer: ListF<i32, Vec<i32>>| match layer {
            ListF::NilF => vec![],
            ListF::ConsF(x, mut xs) => {
                xs.insert(0, x);
                xs
            }
        },
        list,
    );

    assert_eq!(result, original);
}
