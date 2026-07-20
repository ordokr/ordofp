//! Tests for `OrdoFP` 4.0 Phase 2: Algebraic Effect System
//!
//! Tests the Eff monad, Testimonium (evidence-based handlers),
//! and Sem monad (Polysemy-style extensible effects).

#![cfg(feature = "async")]

extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;

use ordofp_core::effects::{
    Clausula,
    ClausulaGenus,

    // Eff monad types
    Eff,
    // Effect row types (use EffectSetVacuus for pure computations)
    EffectSetVacuus,

    // Algebraic effect types
    EffectusAlgebraicus,
    // Sem monad types
    Sem,
    // Testimonium types
    SignumEffectus,
    Testimonium,
    VectorTestimonium,
    pure_eff,
    pure_sem,
    run_purus,
    run_sem,
    sequence_eff,
    then,
    then_sem,

    traverse_eff,
};

// =============================================================================
// EFF MONAD TESTS
// =============================================================================

#[test]
fn test_eff_purus_computation() {
    let eff: Eff<EffectSetVacuus, i32> = Eff::purus(42);
    let result = run_purus(eff);
    assert_eq!(result, 42);
}

#[test]
fn test_eff_functor_map() {
    let eff: Eff<EffectSetVacuus, i32> = Eff::purus(10);
    let doubled = eff.map(|x| x * 2);
    assert_eq!(run_purus(doubled), 20);
}

#[test]
fn test_eff_monad_flat_map() {
    let eff: Eff<EffectSetVacuus, i32> = Eff::purus(5);
    let result = eff.flat_map(|x| Eff::purus(x * x));
    assert_eq!(run_purus(result), 25);
}

#[test]
fn test_eff_applicative_map2() {
    let eff1: Eff<EffectSetVacuus, i32> = Eff::purus(3);
    let eff2: Eff<EffectSetVacuus, i32> = Eff::purus(4);
    let product = eff1.map2(eff2, |a, b| a * b);
    assert_eq!(run_purus(product), 12);
}

#[test]
fn test_eff_pure_function() {
    let eff: Eff<EffectSetVacuus, String> = pure_eff(String::from("hello"));
    assert_eq!(run_purus(eff), "hello");
}

#[test]
fn test_eff_then_sequencing() {
    let first: Eff<EffectSetVacuus, i32> = Eff::purus(100);
    let second: Eff<EffectSetVacuus, &str> = Eff::purus("done");
    let result = then(first, second);
    assert_eq!(run_purus(result), "done");
}

#[test]
fn test_eff_sequence_empty() {
    let effs: Vec<Eff<EffectSetVacuus, i32>> = vec![];
    let sequenced = sequence_eff(effs);
    assert_eq!(run_purus(sequenced), Vec::<i32>::new());
}

#[test]
fn test_eff_sequence_multiple() {
    let effs: Vec<Eff<EffectSetVacuus, i32>> = vec![Eff::purus(10), Eff::purus(20), Eff::purus(30)];
    let sequenced = sequence_eff(effs);
    assert_eq!(run_purus(sequenced), vec![10, 20, 30]);
}

#[test]
fn test_eff_traverse() {
    let items = vec![1, 2, 3, 4, 5];
    let result = traverse_eff(items, |x| Eff::<EffectSetVacuus, _>::purus(x * 10));
    assert_eq!(run_purus(result), vec![10, 20, 30, 40, 50]);
}

#[test]
fn test_eff_traverse_empty() {
    let items: Vec<i32> = vec![];
    let result = traverse_eff(items, |x| Eff::<EffectSetVacuus, _>::purus(x * 10));
    assert_eq!(run_purus(result), Vec::<i32>::new());
}

#[test]
fn test_eff_monad_laws_left_identity() {
    // Left identity: return a >>= f  ≡  f a
    let a = 5;
    let f = |x: i32| Eff::<EffectSetVacuus, _>::purus(x * 2);

    let left: Eff<EffectSetVacuus, i32> = Eff::purus(a).flat_map(f);
    let right: Eff<EffectSetVacuus, i32> = f(a);

    assert_eq!(run_purus(left), run_purus(right));
}

#[test]
fn test_eff_monad_laws_right_identity() {
    // Right identity: m >>= return  ≡  m
    let m: Eff<EffectSetVacuus, i32> = Eff::purus(42);
    let result = m.flat_map(Eff::purus);

    assert_eq!(run_purus(result), 42);
}

// =============================================================================
// TESTIMONIUM TESTS
// =============================================================================

// Define a simple test effect
struct TestEffect;

impl EffectusAlgebraicus for TestEffect {
    type Result = i32;
}

impl ordofp_core::effects::Effectus for TestEffect {}

#[test]
fn test_signum_effectus_creation() {
    let signum: SignumEffectus<TestEffect> = SignumEffectus::new("TestEffect");
    // SignumEffectus contains effect metadata
    assert_eq!(signum.nomen(), "TestEffect");
}

#[test]
fn test_testimonium_creation() {
    let signum: SignumEffectus<TestEffect> = SignumEffectus::new("TestEffect");
    let handler = |_: TestEffect| 42;
    let testimonium = Testimonium::new(signum, handler);

    assert_eq!(testimonium.depth(), 0);
}

#[test]
fn test_testimonium_with_depth() {
    let signum: SignumEffectus<TestEffect> = SignumEffectus::new("TestEffect");
    let handler = |_: TestEffect| 42;
    let testimonium = Testimonium::with_depth(signum, handler, 5);

    assert_eq!(testimonium.depth(), 5);
}

#[test]
fn test_vector_testimonium_empty() {
    let vector: VectorTestimonium = VectorTestimonium::new();
    assert!(vector.is_empty());
    assert_eq!(vector.len(), 0);
}

#[test]
fn test_clausula_fun_genus() {
    let clausula: Clausula<i32, i32, TestEffect, i32> =
        Clausula::Fun(alloc::boxed::Box::new(|x| x * 2));
    assert_eq!(clausula.genus(), ClausulaGenus::Fun);
}

#[test]
fn test_clausula_ctl_genus() {
    let clausula: Clausula<i32, i32, TestEffect, i32> =
        Clausula::Ctl(alloc::boxed::Box::new(|x, _resume| x * 2));
    assert_eq!(clausula.genus(), ClausulaGenus::Ctl);
}

#[test]
fn test_clausula_final_genus() {
    let clausula: Clausula<i32, i32, TestEffect, i32> =
        Clausula::Final(alloc::boxed::Box::new(|x| x * 2));
    assert_eq!(clausula.genus(), ClausulaGenus::Final);
}

// =============================================================================
// SEM MONAD TESTS
// =============================================================================

#[test]
fn test_sem_purus_computation() {
    let sem: Sem<EffectSetVacuus, i32> = Sem::purus(42);
    let result = run_sem(sem);
    assert_eq!(result, 42);
}

#[test]
fn test_sem_functor_map() {
    let sem: Sem<EffectSetVacuus, i32> = Sem::purus(10);
    let doubled = sem.map(|x| x * 2);
    assert_eq!(run_sem(doubled), 20);
}

#[test]
fn test_sem_monad_flat_map() {
    let sem: Sem<EffectSetVacuus, i32> = Sem::purus(7);
    let result = sem.flat_map(|x| Sem::purus(x * x));
    assert_eq!(run_sem(result), 49);
}

#[test]
fn test_sem_applicative_map2() {
    let sem1: Sem<EffectSetVacuus, i32> = Sem::purus(6);
    let sem2: Sem<EffectSetVacuus, i32> = Sem::purus(7);
    let product = sem1.map2(sem2, |a, b| a * b);
    assert_eq!(run_sem(product), 42);
}

#[test]
fn test_sem_pure_function() {
    let sem: Sem<EffectSetVacuus, String> = pure_sem(String::from("world"));
    assert_eq!(run_sem(sem), "world");
}

#[test]
fn test_sem_then_sequencing() {
    let first: Sem<EffectSetVacuus, i32> = Sem::purus(999);
    let second: Sem<EffectSetVacuus, &str> = Sem::purus("finished");
    let result = then_sem(first, second);
    assert_eq!(run_sem(result), "finished");
}

#[test]
fn test_sem_monad_laws_left_identity() {
    // Left identity: return a >>= f  ≡  f a
    let a = 10;
    let f = |x: i32| Sem::<EffectSetVacuus, _>::purus(x + 5);

    let left: Sem<EffectSetVacuus, i32> = Sem::purus(a).flat_map(f);
    let right: Sem<EffectSetVacuus, i32> = f(a);

    assert_eq!(run_sem(left), run_sem(right));
}

#[test]
fn test_sem_monad_laws_right_identity() {
    // Right identity: m >>= return  ≡  m
    let m: Sem<EffectSetVacuus, i32> = Sem::purus(100);
    let result = m.flat_map(Sem::purus);

    assert_eq!(run_sem(result), 100);
}

// =============================================================================
// INTEGRATION TESTS
// =============================================================================

#[test]
fn test_eff_chain_computation() {
    // Test a chain of computations
    let result = run_purus(
        Eff::<EffectSetVacuus, _>::purus(1)
            .flat_map(|a| Eff::purus(a + 1))
            .flat_map(|b| Eff::purus(b * 2))
            .flat_map(|c| Eff::purus(c + 10)),
    );

    // 1 + 1 = 2, 2 * 2 = 4, 4 + 10 = 14
    assert_eq!(result, 14);
}

#[test]
fn test_sem_chain_computation() {
    // Test a chain of computations
    let result = run_sem(
        Sem::<EffectSetVacuus, _>::purus(5)
            .flat_map(|a| Sem::purus(a * 2))
            .flat_map(|b| Sem::purus(b + 3))
            .flat_map(|c| Sem::purus(c * c)),
    );

    // 5 * 2 = 10, 10 + 3 = 13, 13 * 13 = 169
    assert_eq!(result, 169);
}

#[test]
fn test_eff_and_sem_equivalent_for_pure() {
    // For pure computations, Eff and Sem should produce the same result
    let eff_result = run_purus(
        Eff::<EffectSetVacuus, _>::purus(42)
            .map(|x| x * 2)
            .flat_map(|x| Eff::purus(x + 1)),
    );

    let sem_result = run_sem(
        Sem::<EffectSetVacuus, _>::purus(42)
            .map(|x| x * 2)
            .flat_map(|x| Sem::purus(x + 1)),
    );

    assert_eq!(eff_result, sem_result);
    assert_eq!(eff_result, 85); // 42 * 2 + 1 = 85
}
