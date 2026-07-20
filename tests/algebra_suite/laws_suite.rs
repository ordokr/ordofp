//! Root-level consumption of the `ordofp_laws` property-based law modules.
//!
//! `category_laws` is exercised by `tests/category_tests.rs`; this suite
//! exercises the other 15: `functor_laws`, `monad_laws`, `applicative_laws`,
//! `monoid_laws`, `semigroup_laws`, `foldable_laws`, `traversable_laws`,
//! `bifunctor_laws`, `comonad_laws`, `alternative_laws`, `fixpoint_laws`, `is_eq`, and
//! wrapper directly, plus `async_functor_laws` / `async_monad_laws` behind the
//! `async` feature.
//!
//! Invocation:
//! ```text
//! cargo test --test laws_suite
//! cargo test --test laws_suite --features async
//! ```
//! The `async` feature on the root `ordofp` crate forwards to
//! `ordofp_laws/async` (see `Cargo.toml`), which in turn forwards to
//! `ordofp/async` (see `laws/Cargo.toml`).

use ordofp::comonad::{Comonad, Contextus, Identitas};
use ordofp::wrappers::{Max, Min};
use ordofp_laws::wrapper::Wrapper;
use ordofp_laws::{
    alternative_laws, applicative_laws, bifunctor_laws, comonad_laws, fixpoint_laws, foldable_laws,
    functor_laws, monad_laws, monoid_laws, semigroup_laws, traversable_laws,
};
use quickcheck::quickcheck;

// ==================== functor_laws ====================

#[test]
fn functor_option_identity_law() {
    quickcheck(functor_laws::option_identity::<i32> as fn(Option<i32>) -> bool);
}

#[test]
fn functor_vec_composition_law() {
    fn test(fa: Vec<i8>) -> bool {
        functor_laws::vec_composition(fa, |x| x.wrapping_add(1), |x| x.wrapping_mul(2))
    }
    quickcheck(test as fn(Vec<i8>) -> bool);
}

// ==================== monad_laws ====================

#[test]
fn monad_option_left_identity_law() {
    fn test(a: i8) -> bool {
        monad_laws::option_left_identity(a, |x| Some(x.wrapping_mul(2)))
    }
    quickcheck(test as fn(i8) -> bool);
}

#[test]
fn monad_vec_associativity_law() {
    fn test(m: Vec<i8>) -> bool {
        monad_laws::vec_associativity(
            m,
            |x| vec![x, x.wrapping_add(1)],
            |x| vec![x.wrapping_mul(2)],
        )
    }
    quickcheck(test as fn(Vec<i8>) -> bool);
}

// ==================== applicative_laws ====================

#[test]
fn applicative_option_homomorphism_law() {
    fn test(a: i8) -> bool {
        applicative_laws::option_homomorphism(a, |x| x.wrapping_mul(2))
    }
    quickcheck(test as fn(i8) -> bool);
}

#[test]
fn applicative_vec_pure_preservation_law() {
    fn test(a: i32) -> bool {
        applicative_laws::vec_pure_preservation(a, |x| x.to_string())
    }
    quickcheck(test as fn(i32) -> bool);
}

// ==================== monoid_laws ====================

#[test]
fn monoid_string_identity_laws() {
    quickcheck(monoid_laws::left_identity as fn(String) -> bool);
    quickcheck(monoid_laws::right_identity as fn(String) -> bool);
}

#[test]
fn monoid_i32_identity_laws() {
    quickcheck(monoid_laws::left_identity as fn(i32) -> bool);
    quickcheck(monoid_laws::right_identity as fn(i32) -> bool);
}

// ==================== semigroup_laws ====================

#[test]
fn semigroup_vec_associativity_law() {
    quickcheck(semigroup_laws::associativity as fn(Vec<i8>, Vec<i8>, Vec<i8>) -> bool);
}

// Also instantiates `ordofp_laws::wrapper::Wrapper`, the support newtype
// used to implement Compositio/Unitas on external Max/Min types without
// running into the orphan-instance rule.
#[test]
fn semigroup_wrapper_max_min_associativity_law() {
    quickcheck(
        semigroup_laws::associativity
            as fn(Wrapper<Max<i32>>, Wrapper<Max<i32>>, Wrapper<Max<i32>>) -> bool,
    );
    quickcheck(
        semigroup_laws::associativity
            as fn(Wrapper<Min<i32>>, Wrapper<Min<i32>>, Wrapper<Min<i32>>) -> bool,
    );
}

// ==================== foldable_laws ====================

#[test]
fn foldable_vec_length_and_all_any_laws() {
    quickcheck(foldable_laws::vec_length_consistency::<i32> as fn(Vec<i32>) -> bool);
    fn all_any(fa: Vec<i32>) -> bool {
        foldable_laws::vec_all_any_duality(fa, |&x| x > 0)
    }
    quickcheck(all_any as fn(Vec<i32>) -> bool);
}

#[test]
fn foldable_option_length_consistency_law() {
    quickcheck(foldable_laws::option_length_consistency::<i32> as fn(Option<i32>) -> bool);
}

// ==================== traversable_laws ====================

#[test]
fn traversable_vec_identity_law() {
    quickcheck(traversable_laws::vec_traverse_identity::<i32> as fn(Vec<i32>) -> bool);
}

#[test]
fn traversable_option_sequence_consistency_law() {
    fn test(fa: Option<Option<i32>>) -> bool {
        traversable_laws::option_sequence_option_consistency(fa)
    }
    quickcheck(test as fn(Option<Option<i32>>) -> bool);
}

// ==================== bifunctor_laws ====================

#[test]
fn bifunctor_result_identity_law() {
    fn test(fa: Result<i32, String>) -> bool {
        bifunctor_laws::result_identity(fa)
    }
    quickcheck(test as fn(Result<i32, String>) -> bool);
}

#[test]
fn bifunctor_tuple_composition_law() {
    fn test(fa: (i8, i8)) -> bool {
        bifunctor_laws::tuple_composition(
            fa,
            |x: i8| x.wrapping_add(1),
            |y: i8| y.wrapping_mul(2),
            |x: i8| x.wrapping_mul(3),
            |y: i8| y.wrapping_add(10),
        )
    }
    quickcheck(test as fn((i8, i8)) -> bool);
}

// ==================== comonad_laws ====================

#[test]
fn comonad_identitas_left_identity_law() {
    fn test(w: i32) -> bool {
        comonad_laws::identitas_left_identity(Identitas(w))
    }
    quickcheck(test as fn(i32) -> bool);
}

#[test]
fn comonad_contextus_right_identity_law() {
    fn test(env: i8, val: i8) -> bool {
        comonad_laws::contextus_right_identity(Contextus::new(env, val), |w| {
            w.extract().wrapping_add(*w.ask())
        })
    }
    quickcheck(test as fn(i8, i8) -> bool);
}

// ==================== alternative_laws ====================

#[test]
fn alternative_option_left_right_identity_laws() {
    quickcheck(alternative_laws::option_left_identity::<i32> as fn(Option<i32>) -> bool);
    quickcheck(alternative_laws::option_right_identity::<i32> as fn(Option<i32>) -> bool);
}

#[test]
fn alternative_vec_associativity_law() {
    fn test(a: Vec<i32>, b: Vec<i32>, c: Vec<i32>) -> bool {
        alternative_laws::vec_associativity(a, b, c)
    }
    quickcheck(test as fn(Vec<i32>, Vec<i32>, Vec<i32>) -> bool);
}

// ==================== fixpoint_laws ====================
//
// `fixpoint_laws` needs a `FunctorHKT` witness type. The laws crate's own
// internal tests define one (`NatF`/`NatHKT`, isomorphic to `Option`), but
// it isn't exported, so a root consumer must define its own.
mod fixpoint_support {
    use ordofp::typeclasses::hkt::{FunctorHKT, HKT};

    #[derive(Clone, Debug, PartialEq)]
    pub enum NatF<A> {
        Zero,
        Succ(A),
    }

    #[derive(Clone, Debug, PartialEq)]
    pub struct NatHKT;

    impl HKT for NatHKT {
        type Target<T> = NatF<T>;
    }

    impl FunctorHKT for NatHKT {
        fn map<A, B, F>(fa: NatF<A>, mut f: F) -> NatF<B>
        where
            F: FnMut(A) -> B,
        {
            match fa {
                NatF::Zero => NatF::Zero,
                NatF::Succ(a) => NatF::Succ(f(a)),
            }
        }
    }

    /// A "constant" functor witness — `Target<T> = i32` regardless of `T` —
    /// used only to exercise Lambek's lemmas. A self-referential witness
    /// like `NatHKT` can express `cata_ana_inverse` fine (its bound is on
    /// the accumulator `A`, not on `Fix<F>` itself), but `lambek_lemma_1`/
    /// `lambek_lemma_2` bound `F::Target<Fix<F>>: Clone` — for `NatHKT`
    /// that's `NatF<Fix<NatHKT>>: Clone`, which (via `NatF`'s derived
    /// `Clone`) requires `Fix<NatHKT>: Clone` again. `Fix`'s own `Clone`
    /// impl has exactly that bound, so the obligation is circular and
    /// rustc's trait solver overflows resolving it (verified: E0275). The
    /// constant functor sidesteps this — `i32: Clone` doesn't mention `Fix`
    /// at all — while still genuinely exercising both lemmas.
    pub struct ConstHKT;

    impl HKT for ConstHKT {
        type Target<T> = i32;
    }

    impl FunctorHKT for ConstHKT {
        fn map<A, B, F>(fa: i32, _f: F) -> i32
        where
            F: FnMut(A) -> B,
        {
            fa
        }
    }
}

#[test]
fn fixpoint_cata_ana_inverse_law() {
    use fixpoint_support::NatF;
    use fixpoint_support::NatHKT;

    fn coalg(n: u32) -> NatF<u32> {
        if n == 0 {
            NatF::Zero
        } else {
            NatF::Succ(n - 1)
        }
    }

    fn alg(nf: NatF<u32>) -> u32 {
        match nf {
            NatF::Zero => 0,
            NatF::Succ(n) => n + 1,
        }
    }

    // Keep n small: cata/ana recurse.
    fn prop(n: u8) -> bool {
        fixpoint_laws::cata_ana_inverse::<NatHKT, _, _, _>(u32::from(n), alg, coalg)
    }
    quickcheck(prop as fn(u8) -> bool);
}

#[test]
fn fixpoint_lambek_lemmas() {
    use fixpoint_support::ConstHKT;
    use ordofp::fix::Fix;

    // Lambek's Lemma 1: unfix(new(x)) == x
    assert!(fixpoint_laws::lambek_lemma_1::<ConstHKT>(7));

    // Lambek's Lemma 2: new(unfix(x)) == x
    let fixed: Fix<ConstHKT> = Fix::new(7);
    assert!(fixpoint_laws::lambek_lemma_2::<ConstHKT>(fixed));
}

// ==================== is_eq (support module) ====================
//
// Exercised transitively above wherever a law function's `_eq` variant is
// used, plus directly here so the crate's own equality-assertion API is
// covered by a call site outside its own test module.
#[test]
fn is_eq_direct_and_via_law_eq_variants() {
    use ordofp_laws::is_eq::IsEq;

    assert!(IsEq::new(2 + 2, 4).holds());
    assert!(!IsEq::new(2 + 2, 5).holds());

    assert!(functor_laws::option_identity_eq(Some(42)).holds());
    assert!(monad_laws::option_right_identity_eq(Some(7)).holds());
    assert!(applicative_laws::option_identity_eq(Some(7)).holds());
    assert!(traversable_laws::vec_traverse_identity_eq(vec![1, 2, 3]).holds());
    assert!(bifunctor_laws::result_identity_eq(Ok::<_, String>(9)).holds());
    assert!(comonad_laws::identitas_left_identity_eq(Identitas(9)).holds());
    assert!(alternative_laws::option_left_identity_eq(Some(9)).holds());
    assert!(foldable_laws::vec_length_consistency_eq(vec![1, 2, 3]).holds());
}

#[cfg(feature = "async")]
mod async_laws {
    //! `async_functor_laws` / `async_monad_laws` are gated behind the
    //! `async` feature in `ordofp_laws` (forwarding to `ordofp/async`).
    //! Run with: `cargo test --test laws_suite --features async`.
    use ordofp_laws::{async_functor_laws, async_monad_laws};
    use std::future::Future;
    use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

    // Minimal spin-loop executor (identical pattern used by the laws
    // crate's own internal async law tests) — the futures under test are
    // all immediately-ready, so no real reactor is needed.
    fn block_on<F: Future>(fut: F) -> F::Output {
        fn noop_raw_waker() -> RawWaker {
            fn noop(_: *const ()) {}
            fn clone_waker(_: *const ()) -> RawWaker {
                noop_raw_waker()
            }
            static VTABLE: RawWakerVTable = RawWakerVTable::new(clone_waker, noop, noop, noop);
            RawWaker::new(std::ptr::null(), &VTABLE)
        }

        // SAFETY: `noop_raw_waker()`'s vtable functions are no-ops that
        // never dereference the null data pointer, and `clone_waker`
        // returns a fresh, self-consistent `RawWaker` with the same
        // `'static` vtable — satisfying the `RawWaker`/`Waker` contract.
        let waker = unsafe { Waker::from_raw(noop_raw_waker()) };
        let mut cx = Context::from_waker(&waker);
        let mut fut = std::pin::pin!(fut);

        loop {
            match fut.as_mut().poll(&mut cx) {
                Poll::Ready(result) => return result,
                Poll::Pending => std::thread::yield_now(),
            }
        }
    }

    #[test]
    fn async_functor_option_identity_and_composition_laws() {
        assert!(block_on(async_functor_laws::option_identity_async(Some(
            42
        ))));
        assert!(block_on(async_functor_laws::option_identity_async(
            None::<i32>
        )));
        assert!(block_on(async_functor_laws::option_composition_async(
            Some(5),
            |x| async move { x + 1 },
            |x| async move { x * 2 },
        )));
    }

    #[test]
    fn async_monad_option_left_identity_and_associativity_laws() {
        assert!(block_on(async_monad_laws::option_left_identity_async(
            5,
            |x| async move { Some(x * 2) },
        )));
        assert!(block_on(async_monad_laws::option_associativity_async(
            Some(5),
            |x| async move { Some(x + 1) },
            |x| async move { Some(x * 2) },
        )));
    }
}
