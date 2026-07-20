//! Tests for `OrdoFP` 4.0 Phase 4: Quantitative Types
//!
//! Tests the Multiplicitas system, Qtt wrapper, and linear type abstractions
//! inspired by Idris 2's Quantitative Type Theory.
#![cfg(feature = "quantitative")]

extern crate alloc;

use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use ordofp_core::quantitative::{
    AdditiveChoice,

    // FunctioLinearis types
    FunctioLinearis,
    // ManusLinearis
    ManusLinearis,

    // MonasLinearis types
    MonasLinearis,
    // Multiplicitas types
    Multiplicitas,
    Nihil,
    Omega,
    // ParLinearis types
    ParLinearis,
    // Qtt types
    Qtt,
    QttErasum,
    QttExt,
    QttLiber,

    QttLinearis,
    QttMonad,
    Semel,
    Usage,

    WithLinearis,
    bind_qtt,
    is_subusage,
    linear_apply,
    linear_compose,

    mult_add,
    mult_mul,
    purus_qtt,
    sequence_qtt,
};

// =============================================================================
// MULTIPLICITAS TESTS
// =============================================================================

#[test]
fn test_multiplicitas_variants() {
    let zero = Multiplicitas::Nihil;
    let one = Multiplicitas::Semel;
    let omega = Multiplicitas::Omega;

    assert!(zero.is_nihil());
    assert!(!zero.is_semel());
    assert!(!zero.is_omega());

    assert!(!one.is_nihil());
    assert!(one.is_semel());
    assert!(!one.is_omega());

    assert!(!omega.is_nihil());
    assert!(!omega.is_semel());
    assert!(omega.is_omega());
}

#[test]
fn test_multiplicitas_allows_zero() {
    assert!(Multiplicitas::Nihil.allows_zero());
    assert!(!Multiplicitas::Semel.allows_zero());
    assert!(Multiplicitas::Omega.allows_zero());
}

#[test]
fn test_multiplicitas_requires_use() {
    assert!(!Multiplicitas::Nihil.requires_use());
    assert!(Multiplicitas::Semel.requires_use());
    assert!(!Multiplicitas::Omega.requires_use());
}

#[test]
fn test_multiplicitas_allows_many() {
    assert!(!Multiplicitas::Nihil.allows_many());
    assert!(!Multiplicitas::Semel.allows_many());
    assert!(Multiplicitas::Omega.allows_many());
}

#[test]
fn test_multiplicitas_subusage() {
    // 0 ≤ everything
    assert!(is_subusage(Multiplicitas::Nihil, Multiplicitas::Nihil));
    assert!(is_subusage(Multiplicitas::Nihil, Multiplicitas::Semel));
    assert!(is_subusage(Multiplicitas::Nihil, Multiplicitas::Omega));

    // 1 ≤ 1, ω
    assert!(!is_subusage(Multiplicitas::Semel, Multiplicitas::Nihil));
    assert!(is_subusage(Multiplicitas::Semel, Multiplicitas::Semel));
    assert!(is_subusage(Multiplicitas::Semel, Multiplicitas::Omega));

    // ω ≤ ω only
    assert!(!is_subusage(Multiplicitas::Omega, Multiplicitas::Nihil));
    assert!(!is_subusage(Multiplicitas::Omega, Multiplicitas::Semel));
    assert!(is_subusage(Multiplicitas::Omega, Multiplicitas::Omega));
}

#[test]
fn test_multiplicitas_semiring_addition() {
    // max semantics
    assert_eq!(
        mult_add(Multiplicitas::Nihil, Multiplicitas::Nihil),
        Multiplicitas::Nihil
    );
    assert_eq!(
        mult_add(Multiplicitas::Nihil, Multiplicitas::Semel),
        Multiplicitas::Semel
    );
    assert_eq!(
        mult_add(Multiplicitas::Nihil, Multiplicitas::Omega),
        Multiplicitas::Omega
    );
    assert_eq!(
        mult_add(Multiplicitas::Semel, Multiplicitas::Semel),
        Multiplicitas::Semel
    );
    assert_eq!(
        mult_add(Multiplicitas::Semel, Multiplicitas::Omega),
        Multiplicitas::Omega
    );
    assert_eq!(
        mult_add(Multiplicitas::Omega, Multiplicitas::Omega),
        Multiplicitas::Omega
    );
}

#[test]
fn test_multiplicitas_semiring_multiplication() {
    // 0 * x = 0
    assert_eq!(
        mult_mul(Multiplicitas::Nihil, Multiplicitas::Nihil),
        Multiplicitas::Nihil
    );
    assert_eq!(
        mult_mul(Multiplicitas::Nihil, Multiplicitas::Semel),
        Multiplicitas::Nihil
    );
    assert_eq!(
        mult_mul(Multiplicitas::Nihil, Multiplicitas::Omega),
        Multiplicitas::Nihil
    );

    // 1 * x = x
    assert_eq!(
        mult_mul(Multiplicitas::Semel, Multiplicitas::Nihil),
        Multiplicitas::Nihil
    );
    assert_eq!(
        mult_mul(Multiplicitas::Semel, Multiplicitas::Semel),
        Multiplicitas::Semel
    );
    assert_eq!(
        mult_mul(Multiplicitas::Semel, Multiplicitas::Omega),
        Multiplicitas::Omega
    );

    // ω * x
    assert_eq!(
        mult_mul(Multiplicitas::Omega, Multiplicitas::Nihil),
        Multiplicitas::Nihil
    );
    assert_eq!(
        mult_mul(Multiplicitas::Omega, Multiplicitas::Semel),
        Multiplicitas::Omega
    );
    assert_eq!(
        mult_mul(Multiplicitas::Omega, Multiplicitas::Omega),
        Multiplicitas::Omega
    );
}

#[test]
fn test_type_level_usage() {
    assert_eq!(Nihil::VALUE, Multiplicitas::Nihil);
    assert_eq!(Semel::VALUE, Multiplicitas::Semel);
    assert_eq!(Omega::VALUE, Multiplicitas::Omega);

    // Compile-time constants - verify with const assertions
    const _: () = assert!(Nihil::ALLOWS_DISCARD);
    const _: () = assert!(!Semel::ALLOWS_DISCARD);
    const _: () = assert!(Omega::ALLOWS_DISCARD);

    const _: () = assert!(Nihil::ALLOWS_DUP);
    const _: () = assert!(!Semel::ALLOWS_DUP);
    const _: () = assert!(Omega::ALLOWS_DUP);
}

// =============================================================================
// QTT WRAPPER TESTS
// =============================================================================

#[test]
fn test_qtt_linear_creation() {
    let x: Qtt<i32, Semel> = Qtt::linear(42);
    assert_eq!(x.multiplicity(), Multiplicitas::Semel);
    assert!(!x.can_discard());
    assert!(!x.can_dup());
    assert_eq!(x.consume(), 42);
}

#[test]
fn test_qtt_unrestricted_creation() {
    let x: Qtt<i32, Omega> = Qtt::unrestricted(42);
    assert_eq!(x.multiplicity(), Multiplicitas::Omega);
    assert!(x.can_discard());
    assert!(x.can_dup());
}

#[test]
fn test_qtt_erased_creation() {
    let x: Qtt<i32, Nihil> = Qtt::erased(42);
    assert_eq!(x.multiplicity(), Multiplicitas::Nihil);
    assert!(x.can_discard());
}

#[test]
fn test_qtt_fmap() {
    let x: Qtt<i32, Semel> = Qtt::linear(5);
    let y = x.fmap(|n| n * 2);
    assert_eq!(y.consume(), 10);
}

#[test]
fn test_qtt_bind_linear() {
    let x: Qtt<i32, Semel> = Qtt::linear(5);
    let y = x.bind_linear(|n| Qtt::linear(n + 10));
    assert_eq!(y.consume(), 15);
}

#[test]
fn test_qtt_unrestricted_dup() {
    let x: Qtt<i32, Omega> = Qtt::unrestricted(42);
    let y = x.dup();
    let z = x.dup();
    assert_eq!(y.consume(), 42);
    assert_eq!(z.consume(), 42);
    assert_eq!(x.consume(), 42);
}

#[test]
fn test_qtt_unrestricted_discard() {
    let x: Qtt<i32, Omega> = Qtt::unrestricted(42);
    x.discard(); // Should compile and run
}

#[test]
fn test_qtt_linear_relax() {
    let x: Qtt<i32, Semel> = Qtt::linear(42);
    let y: Qtt<i32, Omega> = x.relax();
    assert_eq!(y.multiplicity(), Multiplicitas::Omega);
    assert_eq!(y.consume(), 42);
}

#[test]
fn test_qtt_unrestricted_restrict() {
    let x: Qtt<i32, Omega> = Qtt::unrestricted(42);
    let y: Qtt<i32, Semel> = x.restrict();
    assert_eq!(y.multiplicity(), Multiplicitas::Semel);
    assert_eq!(y.consume(), 42);
}

#[test]
fn test_qtt_ext_trait() {
    let x = 42.into_linear();
    assert_eq!(x.consume(), 42);

    let y = 42.into_unrestricted();
    assert_eq!(y.consume(), 42);
}

#[test]
fn test_qtt_type_aliases() {
    let _linear: QttLinearis<i32> = Qtt::linear(42);
    let _erased: QttErasum<i32> = Qtt::erased(42);
    let _free: QttLiber<i32> = Qtt::unrestricted(42);
}

#[test]
fn test_qtt_chaining() {
    let result = Qtt::linear(5i32)
        .fmap(|x| x * 2)
        .fmap(|x| x + 1)
        .bind_linear(|x: i32| Qtt::linear(x.to_string()))
        .consume();

    assert_eq!(result, "11");
}

// =============================================================================
// MANUS LINEARIS TESTS
// =============================================================================

#[test]
fn test_manus_acquire_release() {
    let handle = ManusLinearis::acquire(42);
    let value = handle.release();
    assert_eq!(value, 42);
}

#[test]
fn test_manus_use_ref() {
    let handle = ManusLinearis::acquire(vec![1, 2, 3]);
    let len = handle.use_ref(std::vec::Vec::len);
    assert_eq!(len, 3);
    let _ = handle.release();
}

#[test]
fn test_manus_use_mut() {
    let mut handle = ManusLinearis::acquire(vec![1, 2, 3]);
    handle.use_mut(|v| v.push(4));
    let vec = handle.release();
    assert_eq!(vec, vec![1, 2, 3, 4]);
}

#[test]
fn test_manus_release_with() {
    let handle = ManusLinearis::acquire(10);
    let result = handle.release_with(|x| x * 2);
    assert_eq!(result, 20);
}

#[test]
fn test_manus_map() {
    let handle = ManusLinearis::acquire(5);
    let mapped = handle.map(|x| x * 2);
    assert_eq!(mapped.release(), 10);
}

#[test]
fn test_manus_and_then() {
    let handle = ManusLinearis::acquire(5);
    let chained = handle.and_then(|x| ManusLinearis::acquire(x + 10));
    assert_eq!(chained.release(), 15);
}

#[test]
fn test_manus_guard() {
    let mut handle = ManusLinearis::acquire(vec![1, 2, 3]);
    {
        let mut guard = handle.guard();
        guard.push(4);
        guard.push(5);
    }
    let vec = handle.release();
    assert_eq!(vec, vec![1, 2, 3, 4, 5]);
}

#[test]
fn test_manus_into_qtt() {
    let handle = ManusLinearis::acquire(42);
    let qtt = handle.into_qtt();
    assert_eq!(qtt.consume(), 42);
}

// =============================================================================
// PAR LINEARIS TESTS
// =============================================================================

#[test]
fn test_par_linearis_new_split() {
    let pair = ParLinearis::new(42, "hello");
    let (n, s) = pair.split();
    assert_eq!(n, 42);
    assert_eq!(s, "hello");
}

#[test]
fn test_par_linearis_consume_with() {
    let pair = ParLinearis::new(5, 10);
    let result = pair.consume_with(|a, b| a + b);
    assert_eq!(result, 15);
}

#[test]
fn test_par_linearis_bimap() {
    let pair = ParLinearis::new(5, 10);
    let mapped = pair.bimap(|x| x * 2, |y| y + 1);
    let (a, b) = mapped.split();
    assert_eq!(a, 10);
    assert_eq!(b, 11);
}

#[test]
fn test_par_linearis_swap() {
    let pair = ParLinearis::new(42, "hello");
    let swapped = pair.swap();
    let (s, n) = swapped.split();
    assert_eq!(s, "hello");
    assert_eq!(n, 42);
}

#[test]
fn test_with_linearis_choose_left() {
    let choice = WithLinearis::new(42, "hello");
    let left = choice.choose_left();
    assert_eq!(left, 42);
}

#[test]
fn test_with_linearis_choose_right() {
    let choice = WithLinearis::new(42, "hello");
    let right = choice.choose_right();
    assert_eq!(right, "hello");
}

#[test]
fn test_additive_choice_fold() {
    let left: AdditiveChoice<i32, &str> = AdditiveChoice::Left(42);
    let result = left.fold(|n| n.to_string(), std::string::ToString::to_string);
    assert_eq!(result, "42");

    let right: AdditiveChoice<i32, &str> = AdditiveChoice::Right("hello");
    let result = right.fold(|n| n.to_string(), std::string::ToString::to_string);
    assert_eq!(result, "hello");
}

// =============================================================================
// FUNCTIO LINEARIS TESTS
// =============================================================================

#[test]
fn test_functio_linearis_new_apply() {
    let f = FunctioLinearis::new(|x: i32| x * 2);
    let result = f.apply(21);
    assert_eq!(result, 42);
}

#[test]
fn test_functio_linearis_compose() {
    let f = FunctioLinearis::new(|x: i32| x + 1);
    let g = FunctioLinearis::new(|x: i32| x * 2);

    let composed = f.compose(g);
    let result = composed.apply(5);
    assert_eq!(result, 12); // (5 + 1) * 2
}

#[test]
fn test_functio_linearis_identity() {
    let id: FunctioLinearis<i32, i32> = FunctioLinearis::identity();
    assert_eq!(id.apply(42), 42);
}

#[test]
fn test_linear_apply_fn() {
    let f = FunctioLinearis::new(|x: i32| x * 2);
    let result = linear_apply(f, 21);
    assert_eq!(result, 42);
}

#[test]
fn test_linear_compose_fn() {
    let f = FunctioLinearis::new(|x: i32| x + 1);
    let g = FunctioLinearis::new(|x: i32| x * 2);

    let h = linear_compose(f, g);
    assert_eq!(h.apply(5), 12);
}

// =============================================================================
// MONAS LINEARIS TESTS
// =============================================================================

#[test]
fn test_monas_linearis_purus() {
    let q: Qtt<i32, Semel> = MonasLinearis::purus(42);
    assert_eq!(q.consume(), 42);
}

#[test]
fn test_monas_linearis_bind() {
    let q: Qtt<i32, Semel> = Qtt::linear(5);
    let result = q.bind(|x| Qtt::new(x * 2));
    assert_eq!(result.consume(), 10);
}

#[test]
fn test_qtt_monad_chaining() {
    let result = QttMonad::<_, Semel>::purus(5i32)
        .map(|x| x * 2)
        .flat_map(|x| QttMonad::purus(x + 1))
        .map(|x: i32| x.to_string())
        .run();

    assert_eq!(result, "11");
}

#[test]
fn test_purus_qtt_fn() {
    let q: Qtt<i32, Semel> = purus_qtt(42);
    assert_eq!(q.consume(), 42);
}

#[test]
fn test_bind_qtt_fn() {
    let q: Qtt<i32, Semel> = Qtt::linear(5);
    let result = bind_qtt(q, |x| Qtt::new(x * 2));
    assert_eq!(result.consume(), 10);
}

#[test]
fn test_sequence_qtt() {
    let qs: Vec<Qtt<i32, Semel>> = vec![Qtt::linear(1), Qtt::linear(2), Qtt::linear(3)];
    let result = sequence_qtt(qs);
    assert_eq!(result.consume(), vec![1, 2, 3]);
}

// =============================================================================
// MONAD LAW TESTS
// =============================================================================

#[test]
fn test_qtt_left_identity() {
    // pure(a).bind(f) ≡ f(a)
    let a = 5;
    let f = |x: i32| Qtt::<_, Semel>::linear(x * 2);

    let left: Qtt<i32, Semel> = MonasLinearis::purus(a);
    let left_result = left.bind(f);

    let right = f(a);

    assert_eq!(left_result.consume(), right.consume());
}

#[test]
fn test_qtt_right_identity() {
    // m.bind(pure) ≡ m
    let m: Qtt<i32, Semel> = Qtt::linear(42);
    let result = m.bind(Qtt::<_, Semel>::linear);

    assert_eq!(result.consume(), 42);
}

#[test]
fn test_qtt_associativity() {
    // m.bind(f).bind(g) ≡ m.bind(|x| f(x).bind(g))
    let f = |x: i32| Qtt::<_, Semel>::linear(x + 1);
    let g = |x: i32| Qtt::<_, Semel>::linear(x * 2);

    let m1: Qtt<i32, Semel> = Qtt::linear(5);
    let left = m1.bind(f).bind(g);

    let m2: Qtt<i32, Semel> = Qtt::linear(5);
    let right = m2.bind(|x| {
        let fx: Qtt<i32, Semel> = Qtt::linear(x + 1);
        fx.bind(g)
    });

    assert_eq!(left.consume(), right.consume());
}
