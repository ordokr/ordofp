//! Tests for `OrdoFP` 4.0 Phase 6: Free Monads & Tagless Final
//!
//! This test suite covers:
//! - Liber (Free Monad) operations and laws
//! - Liberior (Freer Monad) operations
//! - Natural Transformations
//! - Tagless Final algebras and interpreters

use ordofp_core::free::*;
use ordofp_core::typeclasses::hkt::{FunctorHKT, HKT};

// =============================================================================
// Natural Transformation Tests
// =============================================================================

#[test]
fn test_option_functor_witness() {
    let some_val: Option<i32> = Some(42);
    let mapped: Option<i32> = OptionFWitness::map(some_val, |x| x * 2);
    assert_eq!(mapped, Some(84));
}

#[test]
fn test_option_functor_witness_none() {
    let none_val: Option<i32> = None;
    let mapped: Option<i32> = OptionFWitness::map(none_val, |x| x * 2);
    assert_eq!(mapped, None);
}

#[test]
fn test_identity_functor_witness() {
    let value = 42;
    let mapped = IdentitasFWitness::map(value, |x| x * 2);
    assert_eq!(mapped, 84);
}

#[test]
fn test_result_functor_witness_ok() {
    let ok_val: Result<i32, &str> = Ok(42);
    let mapped: Result<i32, &str> = ResultFWitness::<&str>::map(ok_val, |x| x * 2);
    assert_eq!(mapped, Ok(84));
}

#[test]
fn test_result_functor_witness_err() {
    let err_val: Result<i32, &str> = Err("error");
    let mapped: Result<i32, &str> = ResultFWitness::<&str>::map(err_val, |x| x * 2);
    assert_eq!(mapped, Err("error"));
}

#[test]
fn test_identity_natural_transformation() {
    let opt = Some(42);
    let result: Option<i32> = TransformatioIdentitas::<OptionFWitness>::transforma(opt);
    assert_eq!(result, Some(42));
}

#[test]
fn test_identity_natural_transformation_none() {
    let opt: Option<i32> = None;
    let result: Option<i32> = TransformatioIdentitas::<OptionFWitness>::transforma(opt);
    assert_eq!(result, None);
}

// =============================================================================
// Liber (Free Monad) Tests
// =============================================================================

#[test]
fn test_liber_purus() {
    let free: Liber<OptionFWitness, i32> = Liber::purus(42);
    assert!(free.est_purus());
    assert!(!free.est_suspensus());
}

#[test]
fn test_liber_map_purus() {
    let free: Liber<OptionFWitness, i32> = Liber::purus(42);
    let mapped = free.map(|x| x * 2);

    match mapped {
        Liber::Purus(x) => assert_eq!(x, 84),
        _ => panic!("Expected Purus"),
    }
}

#[test]
fn test_liber_flat_map_purus() {
    let free: Liber<OptionFWitness, i32> = Liber::purus(42);
    let chained = free.flat_map(|x| Liber::purus(x + 1));

    match chained {
        Liber::Purus(x) => assert_eq!(x, 43),
        _ => panic!("Expected Purus"),
    }
}

#[test]
fn test_liber_chain_operations() {
    let free: Liber<OptionFWitness, i32> = Liber::purus(10);
    let result = free
        .flat_map(|x| Liber::purus(x + 5))
        .flat_map(|x| Liber::purus(x * 2))
        .map(|x| x - 10);

    match result {
        Liber::Purus(x) => assert_eq!(x, 20), // ((10 + 5) * 2) - 10 = 20
        _ => panic!("Expected Purus"),
    }
}

#[test]
fn test_liber_monad_left_identity() {
    // Left identity: pure a >>= f  ≡  f a
    let a = 42;
    let f = |x: i32| Liber::<OptionFWitness, i32>::purus(x * 2);

    let left: Liber<OptionFWitness, i32> = Liber::purus(a).flat_map(f);
    let right: Liber<OptionFWitness, i32> = f(a);

    match (left, right) {
        (Liber::Purus(l), Liber::Purus(r)) => assert_eq!(l, r),
        _ => panic!("Both should be Purus"),
    }
}

#[test]
fn test_liber_monad_right_identity() {
    // Right identity: m >>= pure  ≡  m
    let m: Liber<OptionFWitness, i32> = Liber::purus(42);
    let result = m.flat_map(Liber::purus);

    match result {
        Liber::Purus(r) => assert_eq!(r, 42),
        _ => panic!("Expected Purus"),
    }
}

#[test]
fn test_liber_monad_associativity() {
    // Associativity: (m >>= f) >>= g  ≡  m >>= (λx. f x >>= g)
    let f = |x: i32| Liber::<OptionFWitness, i32>::purus(x + 3);
    let g = |x: i32| Liber::<OptionFWitness, i32>::purus(x * 2);

    // Test left side: (m >>= f) >>= g
    let m1: Liber<OptionFWitness, i32> = Liber::purus(5);
    let left = m1.flat_map(f).flat_map(g);

    // Test right side: m >>= (λx. f x >>= g)
    let m2: Liber<OptionFWitness, i32> = Liber::purus(5);
    let right = m2.flat_map(|x| f(x).flat_map(g));

    match (left, right) {
        (Liber::Purus(l), Liber::Purus(r)) => assert_eq!(l, r), // Both should be 16
        _ => panic!("Both should be Purus"),
    }
}

#[test]
fn test_liber_join() {
    let nested: Liber<OptionFWitness, Liber<OptionFWitness, i32>> = Liber::purus(Liber::purus(42));
    let joined = join_liber(nested);

    match joined {
        Liber::Purus(x) => assert_eq!(x, 42),
        _ => panic!("Expected Purus"),
    }
}

#[test]
fn test_purus_liber_helper() {
    let free: Liber<OptionFWitness, i32> = purus_liber(42);
    match free {
        Liber::Purus(x) => assert_eq!(x, 42),
        _ => panic!("Expected Purus"),
    }
}

// =============================================================================
// Liberior (Freer Monad) Tests
// =============================================================================

#[test]
fn test_liberior_purus() {
    let free: Liberior<(), i32> = Liberior::purus(42);
    assert!(free.est_purus());
    assert!(!free.est_impurus());
}

#[test]
fn test_liberior_map_purus() {
    let free: Liberior<(), i32> = Liberior::purus(42);
    let mapped = free.map(|x| x * 2);

    match mapped {
        Liberior::Purus(x) => assert_eq!(x, 84),
        _ => panic!("Expected Purus"),
    }
}

#[test]
fn test_liberior_flat_map_purus() {
    let free: Liberior<(), i32> = Liberior::purus(42);
    let chained = free.flat_map(|x| Liberior::purus(x + 1));

    match chained {
        Liberior::Purus(x) => assert_eq!(x, 43),
        _ => panic!("Expected Purus"),
    }
}

#[test]
fn test_liberior_chain_operations() {
    let free: Liberior<(), i32> = Liberior::purus(10);
    let result = free
        .flat_map(|x| Liberior::purus(x + 5))
        .flat_map(|x| Liberior::purus(x * 2))
        .map(|x| x - 10);

    match result {
        Liberior::Purus(x) => assert_eq!(x, 20), // ((10 + 5) * 2) - 10 = 20
        _ => panic!("Expected Purus"),
    }
}

#[test]
fn test_liberior_monad_left_identity() {
    let a = 42;
    let f = |x: i32| Liberior::<(), i32>::purus(x * 2);

    let left: Liberior<(), i32> = Liberior::purus(a).flat_map(f);
    let right: Liberior<(), i32> = f(a);

    match (left, right) {
        (Liberior::Purus(l), Liberior::Purus(r)) => assert_eq!(l, r),
        _ => panic!("Both should be Purus"),
    }
}

#[test]
fn test_liberior_monad_right_identity() {
    let m: Liberior<(), i32> = Liberior::purus(42);
    let result = m.flat_map(Liberior::purus);

    match result {
        Liberior::Purus(x) => assert_eq!(x, 42),
        _ => panic!("Expected Purus"),
    }
}

#[test]
fn test_liberior_curro_purus() {
    let free: Liberior<(), i32> = Liberior::purus(42);
    let result = curro_purus_liberior(free);
    assert_eq!(result, 42);
}

#[test]
#[should_panic(expected = "Cannot run impure")]
fn test_liberior_curro_purus_panics_on_impure() {
    let free: Liberior<(), i32> = mitto_liberior::<(), i32>(42);
    let _ = curro_purus_liberior(free);
}

#[test]
fn test_liberior_mitto_creates_impure() {
    let free: Liberior<(), i32> = mitto_liberior::<(), i32>(42);
    assert!(free.est_impurus());
}

// =============================================================================
// Tagless Final Algebra Tests - Evaluation
// =============================================================================

fn expr_simple<F: HKT, Alg: AlgebraArithmetica<F>>() -> F::Target<i32> {
    Alg::addo(Alg::lit(10), Alg::multiplico(Alg::lit(3), Alg::lit(4)))
}

fn expr_with_neg<F: HKT, Alg: AlgebraArithmetica<F>>() -> F::Target<i32> {
    Alg::addo(Alg::lit(5), Alg::nego(Alg::lit(3)))
}

fn expr_complex<F: HKT, Alg: AlgebraArithmetica<F>>() -> F::Target<i32> {
    // (10 - 3) * (4 + 2) = 7 * 6 = 42
    Alg::multiplico(
        Alg::subtraho(Alg::lit(10), Alg::lit(3)),
        Alg::addo(Alg::lit(4), Alg::lit(2)),
    )
}

#[test]
fn test_eval_simple() {
    let result = expr_simple::<IdentitasFWitness, InterpresAestimationis>();
    assert_eq!(result, 22); // 10 + (3 * 4) = 22
}

#[test]
fn test_eval_with_neg() {
    let result = expr_with_neg::<IdentitasFWitness, InterpresAestimationis>();
    assert_eq!(result, 2); // 5 + (-3) = 2
}

#[test]
fn test_eval_complex() {
    let result = expr_complex::<IdentitasFWitness, InterpresAestimationis>();
    assert_eq!(result, 42); // (10 - 3) * (4 + 2) = 42
}

#[test]
fn test_eval_literal() {
    let result = InterpresAestimationis::lit(42);
    assert_eq!(result, 42);
}

#[test]
fn test_eval_addition() {
    let result = InterpresAestimationis::addo(10, 5);
    assert_eq!(result, 15);
}

#[test]
fn test_eval_multiplication() {
    let result = InterpresAestimationis::multiplico(6, 7);
    assert_eq!(result, 42);
}

#[test]
fn test_eval_subtraction() {
    let result = InterpresAestimationis::subtraho(50, 8);
    assert_eq!(result, 42);
}

#[test]
fn test_eval_negation() {
    let result = InterpresAestimationis::nego(42);
    assert_eq!(result, -42);
}

// =============================================================================
// Tagless Final Algebra Tests - Pretty Printing
// =============================================================================

#[test]
fn test_pretty_simple() {
    let result = expr_simple::<ConstStringFWitness, InterpresPulcher>();
    assert_eq!(result, "(10 + (3 * 4))");
}

#[test]
fn test_pretty_with_neg() {
    let result = expr_with_neg::<ConstStringFWitness, InterpresPulcher>();
    assert_eq!(result, "(5 + (-3))");
}

#[test]
fn test_pretty_complex() {
    let result = expr_complex::<ConstStringFWitness, InterpresPulcher>();
    assert_eq!(result, "((10 - 3) * (4 + 2))");
}

#[test]
fn test_pretty_literal() {
    let result = InterpresPulcher::lit(42);
    assert_eq!(result, "42");
}

#[test]
fn test_pretty_addition() {
    let result = InterpresPulcher::addo("a".to_string(), "b".to_string());
    assert_eq!(result, "(a + b)");
}

#[test]
fn test_pretty_multiplication() {
    let result = InterpresPulcher::multiplico("x".to_string(), "y".to_string());
    assert_eq!(result, "(x * y)");
}

#[test]
fn test_pretty_subtraction() {
    let result = InterpresPulcher::subtraho("m".to_string(), "n".to_string());
    assert_eq!(result, "(m - n)");
}

#[test]
fn test_pretty_negation() {
    let result = InterpresPulcher::nego("z".to_string());
    assert_eq!(result, "(-z)");
}

// =============================================================================
// Tagless Final Algebra Tests - Counting
// =============================================================================

#[test]
fn test_count_simple() {
    let result = expr_simple::<ConstUsizeFWitness, InterpresNumerans>();
    assert_eq!(result, 5); // 1 add + 1 mul + 3 lit = 5 nodes
}

#[test]
fn test_count_with_neg() {
    let result = expr_with_neg::<ConstUsizeFWitness, InterpresNumerans>();
    assert_eq!(result, 4); // 1 add + 1 neg + 2 lit = 4 nodes
}

#[test]
fn test_count_complex() {
    let result = expr_complex::<ConstUsizeFWitness, InterpresNumerans>();
    assert_eq!(result, 7); // 1 mul + 1 sub + 1 add + 4 lit = 7 nodes
}

#[test]
fn test_count_literal() {
    let result = InterpresNumerans::lit(42);
    assert_eq!(result, 1);
}

#[test]
fn test_count_binary_op() {
    let result = InterpresNumerans::addo(1, 1);
    assert_eq!(result, 3); // 1 + 1 + 1 = 3 (op + two children)
}

// =============================================================================
// Tagless Final Boolean Algebra Tests
// =============================================================================

#[test]
fn test_boolean_verum() {
    let result = InterpresAestimationis::verum();
    assert!(result);
}

#[test]
fn test_boolean_falsum() {
    let result = InterpresAestimationis::falsum();
    assert!(!result);
}

#[test]
fn test_boolean_et_true_true() {
    let result = InterpresAestimationis::et(true, true);
    assert!(result);
}

#[test]
fn test_boolean_et_true_false() {
    let result = InterpresAestimationis::et(true, false);
    assert!(!result);
}

#[test]
fn test_boolean_et_false_false() {
    let result = InterpresAestimationis::et(false, false);
    assert!(!result);
}

#[test]
fn test_boolean_vel_false_false() {
    let result = InterpresAestimationis::vel(false, false);
    assert!(!result);
}

#[test]
fn test_boolean_vel_true_false() {
    let result = InterpresAestimationis::vel(true, false);
    assert!(result);
}

#[test]
fn test_boolean_vel_true_true() {
    let result = InterpresAestimationis::vel(true, true);
    assert!(result);
}

#[test]
fn test_boolean_non_true() {
    let result = InterpresAestimationis::non(true);
    assert!(!result);
}

#[test]
fn test_boolean_non_false() {
    let result = InterpresAestimationis::non(false);
    assert!(result);
}

// =============================================================================
// Tagless Final Boolean Pretty Printing
// =============================================================================

#[test]
fn test_pretty_boolean_verum() {
    let result = InterpresPulcher::verum();
    assert_eq!(result, "true");
}

#[test]
fn test_pretty_boolean_falsum() {
    let result = InterpresPulcher::falsum();
    assert_eq!(result, "false");
}

#[test]
fn test_pretty_boolean_et() {
    let result = InterpresPulcher::et("a".to_string(), "b".to_string());
    assert_eq!(result, "(a && b)");
}

#[test]
fn test_pretty_boolean_vel() {
    let result = InterpresPulcher::vel("x".to_string(), "y".to_string());
    assert_eq!(result, "(x || y)");
}

#[test]
fn test_pretty_boolean_non() {
    let result = InterpresPulcher::non("z".to_string());
    assert_eq!(result, "(!z)");
}

// =============================================================================
// Tagless Final Optimization Tests
// =============================================================================

#[test]
fn test_optimization_constant_folding() {
    // Build expression: (2 + 3) * (4 + 1)
    fn expr<F: HKT, Alg: AlgebraArithmetica<F>>() -> F::Target<i32> {
        Alg::multiplico(
            Alg::addo(Alg::lit(2), Alg::lit(3)),
            Alg::addo(Alg::lit(4), Alg::lit(1)),
        )
    }

    let optimized = expr::<OptimumFWitness, InterpresOptimans>();
    assert_eq!(optimized, OptimumExpr::Constans(25)); // Fully folded
}

#[test]
fn test_optimization_zero_addition() {
    // x + 0 = x
    let result = InterpresOptimans::addo(
        OptimumExpr::Incognitus("x".to_string()),
        OptimumExpr::Constans(0),
    );
    assert_eq!(result, OptimumExpr::Incognitus("x".to_string()));
}

#[test]
fn test_optimization_zero_addition_left() {
    // 0 + x = x
    let result = InterpresOptimans::addo(
        OptimumExpr::Constans(0),
        OptimumExpr::Incognitus("x".to_string()),
    );
    assert_eq!(result, OptimumExpr::Incognitus("x".to_string()));
}

#[test]
fn test_optimization_one_multiplication() {
    // x * 1 = x
    let result = InterpresOptimans::multiplico(
        OptimumExpr::Incognitus("x".to_string()),
        OptimumExpr::Constans(1),
    );
    assert_eq!(result, OptimumExpr::Incognitus("x".to_string()));
}

#[test]
fn test_optimization_one_multiplication_left() {
    // 1 * x = x
    let result = InterpresOptimans::multiplico(
        OptimumExpr::Constans(1),
        OptimumExpr::Incognitus("x".to_string()),
    );
    assert_eq!(result, OptimumExpr::Incognitus("x".to_string()));
}

#[test]
fn test_optimization_zero_multiplication() {
    // x * 0 = 0
    let result = InterpresOptimans::multiplico(
        OptimumExpr::Incognitus("x".to_string()),
        OptimumExpr::Constans(0),
    );
    assert_eq!(result, OptimumExpr::Constans(0));
}

#[test]
fn test_optimization_zero_multiplication_left() {
    // 0 * x = 0
    let result = InterpresOptimans::multiplico(
        OptimumExpr::Constans(0),
        OptimumExpr::Incognitus("x".to_string()),
    );
    assert_eq!(result, OptimumExpr::Constans(0));
}

#[test]
fn test_optimization_subtraction_zero() {
    // x - 0 = x
    let result = InterpresOptimans::subtraho(
        OptimumExpr::Incognitus("x".to_string()),
        OptimumExpr::Constans(0),
    );
    assert_eq!(result, OptimumExpr::Incognitus("x".to_string()));
}

#[test]
fn test_optimization_negation_constant() {
    let result = InterpresOptimans::nego(OptimumExpr::Constans(42));
    assert_eq!(result, OptimumExpr::Constans(-42));
}

// =============================================================================
// Polymorphic Expression Tests
// =============================================================================

/// A polymorphic expression that works with any interpreter
fn fibonacci_expr<F: HKT, Alg: AlgebraArithmetica<F>>(n: i32) -> F::Target<i32> {
    // Just a simple expression for testing: fib(5) = (5 * (5 - 1)) / 2 ≈ 10
    // We'll compute n * (n - 1) / 2 using available ops
    Alg::addo(Alg::multiplico(Alg::lit(n), Alg::lit(n - 1)), Alg::lit(0))
}

#[test]
fn test_polymorphic_expr_eval() {
    let result = fibonacci_expr::<IdentitasFWitness, InterpresAestimationis>(5);
    assert_eq!(result, 20); // 5 * 4 + 0 = 20
}

#[test]
fn test_polymorphic_expr_pretty() {
    let result = fibonacci_expr::<ConstStringFWitness, InterpresPulcher>(5);
    assert_eq!(result, "((5 * 4) + 0)");
}

#[test]
fn test_polymorphic_expr_count() {
    let result = fibonacci_expr::<ConstUsizeFWitness, InterpresNumerans>(5);
    assert_eq!(result, 5); // 1 add + 1 mul + 3 lit = 5
}

// =============================================================================
// Multiple Interpreter Tests (same expression, different interpretations)
// =============================================================================

fn quadratic<F: HKT, Alg: AlgebraArithmetica<F>>(a: i32, b: i32, c: i32, x: i32) -> F::Target<i32> {
    // a*x^2 + b*x + c
    Alg::addo(
        Alg::addo(
            Alg::multiplico(Alg::lit(a), Alg::multiplico(Alg::lit(x), Alg::lit(x))),
            Alg::multiplico(Alg::lit(b), Alg::lit(x)),
        ),
        Alg::lit(c),
    )
}

#[test]
fn test_quadratic_eval() {
    // 2x^2 + 3x + 1 where x = 2
    // = 2*4 + 3*2 + 1 = 8 + 6 + 1 = 15
    let result = quadratic::<IdentitasFWitness, InterpresAestimationis>(2, 3, 1, 2);
    assert_eq!(result, 15);
}

#[test]
fn test_quadratic_pretty() {
    let result = quadratic::<ConstStringFWitness, InterpresPulcher>(2, 3, 1, 2);
    assert_eq!(result, "(((2 * (2 * 2)) + (3 * 2)) + 1)");
}

#[test]
fn test_quadratic_count() {
    let result = quadratic::<ConstUsizeFWitness, InterpresNumerans>(2, 3, 1, 2);
    // 2 adds + 3 muls + 6 lits = 11 nodes
    assert_eq!(result, 11);
}
