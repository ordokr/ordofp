//! Tagless Final - DSLs as Traits
//!
//! > *"Forma dat esse rei."*
//! > — Form gives being to a thing. (Scholastic principle)
//!
//! Tagless Final represents domain-specific languages (DSLs) as traits
//! rather than data structures. This enables extensibility and type-safe
//! multiple interpretations.

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::boxed::Box;
#[cfg(feature = "alloc")]
use alloc::format;
#[cfg(feature = "alloc")]
use alloc::string::String;

use crate::typeclasses::hkt::HKT;

// =============================================================================
// Core Tagless Final Traits
// =============================================================================

/// Algebra for arithmetic expressions.
///
/// This is the classic example of tagless final encoding.
/// The trait defines the "syntax" of the DSL, while implementations
/// provide different "semantics" (interpretations).
///
/// # Latin Etymology
///
/// *Algebra Arithmetica* = arithmetic algebra
///
/// # Example
///
/// ```rust
/// use ordofp_core::free::*;
/// use ordofp_core::typeclasses::hkt::HKT;
///
/// // Build an expression
/// fn example<F: HKT, Alg: AlgebraArithmetica<F>>() -> F::Target<i32> {
///     Alg::addo(
///         Alg::lit(10),
///         Alg::multiplico(Alg::lit(3), Alg::lit(4))
///     )
/// }
///
/// // Evaluate it
/// let result = example::<IdentitasFWitness, InterpresAestimationis>();
/// assert_eq!(result, 22);
///
/// // Pretty print it
/// let pretty = example::<ConstStringFWitness, InterpresPulcher>();
/// assert_eq!(pretty, "(10 + (3 * 4))");
/// ```
pub trait AlgebraArithmetica<F: HKT> {
    /// A literal integer value.
    ///
    /// # Latin Etymology
    /// *Lit* = literal, letter
    fn lit(n: i32) -> F::Target<i32>;

    /// Addition of two expressions.
    ///
    /// # Latin Etymology
    /// *Addo* = I add
    fn addo(a: F::Target<i32>, b: F::Target<i32>) -> F::Target<i32>;

    /// Multiplication of two expressions.
    ///
    /// # Latin Etymology
    /// *Multiplico* = I multiply
    fn multiplico(a: F::Target<i32>, b: F::Target<i32>) -> F::Target<i32>;

    /// Subtraction of two expressions.
    ///
    /// # Latin Etymology
    /// *Subtraho* = I subtract
    fn subtraho(a: F::Target<i32>, b: F::Target<i32>) -> F::Target<i32>;

    /// Negation of an expression.
    ///
    /// # Latin Etymology
    /// *Nego* = I deny, negate
    fn nego(a: F::Target<i32>) -> F::Target<i32>;
}

/// Algebra for boolean expressions.
///
/// # Latin Etymology
///
/// *Algebra Booleana* = Boolean algebra
pub trait AlgebraBooleana<F: HKT> {
    /// A literal boolean value.
    fn verum() -> F::Target<bool>;
    /// The literal `false` value (*falsum* = false).
    fn falsum() -> F::Target<bool>;

    /// Logical AND.
    ///
    /// # Latin Etymology
    /// *Et* = and
    fn et(a: F::Target<bool>, b: F::Target<bool>) -> F::Target<bool>;

    /// Logical OR.
    ///
    /// # Latin Etymology
    /// *Vel* = or
    fn vel(a: F::Target<bool>, b: F::Target<bool>) -> F::Target<bool>;

    /// Logical NOT.
    ///
    /// # Latin Etymology
    /// *Non* = not
    fn non(a: F::Target<bool>) -> F::Target<bool>;
}

/// Algebra for comparison operations.
///
/// # Latin Etymology
///
/// *Algebra Comparationis* = comparison algebra
pub trait AlgebraComparationis<F: HKT>: AlgebraArithmetica<F> + AlgebraBooleana<F> {
    /// Equality comparison.
    ///
    /// # Latin Etymology
    /// *Aequalis* = equal
    fn aequalis(a: F::Target<i32>, b: F::Target<i32>) -> F::Target<bool>;

    /// Less than comparison.
    ///
    /// # Latin Etymology
    /// *Minor* = less
    fn minor(a: F::Target<i32>, b: F::Target<i32>) -> F::Target<bool>;

    /// Greater than comparison.
    ///
    /// # Latin Etymology
    /// *Maior* = greater
    fn maior(a: F::Target<i32>, b: F::Target<i32>) -> F::Target<bool>;
}

/// Algebra for conditional expressions.
///
/// # Latin Etymology
///
/// *Algebra Conditionalis* = conditional algebra
pub trait AlgebraConditionalis<F: HKT>: AlgebraBooleana<F> {
    /// Conditional expression (if-then-else).
    ///
    /// # Latin Etymology
    /// *Si* = if
    fn si<A>(cond: F::Target<bool>, then_: F::Target<A>, else_: F::Target<A>) -> F::Target<A>;
}

// =============================================================================
// Evaluation Interpreter
// =============================================================================

/// Evaluation interpreter - computes the actual value.
///
/// # Latin Etymology
///
/// *Interpres Aestimationis* = evaluation interpreter
pub struct InterpresAestimationis;

impl AlgebraArithmetica<super::nat::IdentitasFWitness> for InterpresAestimationis {
    #[inline]
    fn lit(n: i32) -> i32 {
        n
    }

    #[inline]
    fn addo(a: i32, b: i32) -> i32 {
        a + b
    }

    #[inline]
    fn multiplico(a: i32, b: i32) -> i32 {
        a * b
    }

    #[inline]
    fn subtraho(a: i32, b: i32) -> i32 {
        a - b
    }

    #[inline]
    fn nego(a: i32) -> i32 {
        -a
    }
}

impl AlgebraBooleana<super::nat::IdentitasFWitness> for InterpresAestimationis {
    #[inline]
    fn verum() -> bool {
        true
    }

    #[inline]
    fn falsum() -> bool {
        false
    }

    #[inline]
    fn et(a: bool, b: bool) -> bool {
        a && b
    }

    #[inline]
    fn vel(a: bool, b: bool) -> bool {
        a || b
    }

    #[inline]
    fn non(a: bool) -> bool {
        !a
    }
}

// =============================================================================
// Pretty Printing Interpreter
// =============================================================================

/// Pretty printing interpreter - produces a string representation.
///
/// # Latin Etymology
///
/// *Interpres Pulcher* = pretty interpreter
#[cfg(feature = "alloc")]
pub struct InterpresPulcher;

/// Const functor witness for String output.
#[cfg(feature = "alloc")]
pub struct ConstStringFWitness;

#[cfg(feature = "alloc")]
impl HKT for ConstStringFWitness {
    type Target<T> = String;
}

#[cfg(feature = "alloc")]
impl AlgebraArithmetica<ConstStringFWitness> for InterpresPulcher {
    #[inline]
    fn lit(n: i32) -> String {
        format!("{n}")
    }

    #[inline]
    fn addo(a: String, b: String) -> String {
        format!("({a} + {b})")
    }

    #[inline]
    fn multiplico(a: String, b: String) -> String {
        format!("({a} * {b})")
    }

    #[inline]
    fn subtraho(a: String, b: String) -> String {
        format!("({a} - {b})")
    }

    #[inline]
    fn nego(a: String) -> String {
        format!("(-{a})")
    }
}

#[cfg(feature = "alloc")]
impl AlgebraBooleana<ConstStringFWitness> for InterpresPulcher {
    #[inline]
    fn verum() -> String {
        String::from("true")
    }

    #[inline]
    fn falsum() -> String {
        String::from("false")
    }

    #[inline]
    fn et(a: String, b: String) -> String {
        format!("({a} && {b})")
    }

    #[inline]
    fn vel(a: String, b: String) -> String {
        format!("({a} || {b})")
    }

    #[inline]
    fn non(a: String) -> String {
        format!("(!{a})")
    }
}

// =============================================================================
// Counting Interpreter
// =============================================================================

/// Counting interpreter - counts the number of operations.
///
/// # Latin Etymology
///
/// *Interpres Numerans* = counting interpreter
pub struct InterpresNumerans;

/// Const functor witness for usize output (counting).
pub struct ConstUsizeFWitness;

impl HKT for ConstUsizeFWitness {
    type Target<T> = usize;
}

impl AlgebraArithmetica<ConstUsizeFWitness> for InterpresNumerans {
    #[inline]
    fn lit(_n: i32) -> usize {
        1 // One node
    }

    #[inline]
    fn addo(a: usize, b: usize) -> usize {
        1 + a + b // One operation plus children
    }

    #[inline]
    fn multiplico(a: usize, b: usize) -> usize {
        1 + a + b
    }

    #[inline]
    fn subtraho(a: usize, b: usize) -> usize {
        1 + a + b
    }

    #[inline]
    fn nego(a: usize) -> usize {
        1 + a
    }
}

impl AlgebraBooleana<ConstUsizeFWitness> for InterpresNumerans {
    #[inline]
    fn verum() -> usize {
        1
    }

    #[inline]
    fn falsum() -> usize {
        1
    }

    #[inline]
    fn et(a: usize, b: usize) -> usize {
        1 + a + b
    }

    #[inline]
    fn vel(a: usize, b: usize) -> usize {
        1 + a + b
    }

    #[inline]
    fn non(a: usize) -> usize {
        1 + a
    }
}

// =============================================================================
// Optimization Interpreter
// =============================================================================

/// Optimization interpreter - performs constant folding.
///
/// This interpreter tries to evaluate constant expressions at "compile time".
///
/// # Latin Etymology
///
/// *Interpres Optimans* = optimizing interpreter
pub struct InterpresOptimans;

/// Either a known constant or an unknown expression (represented as string).
#[cfg(feature = "alloc")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OptimumExpr<T> {
    /// A known constant value.
    Constans(T),
    /// An unknown expression (stored as string for debugging).
    Incognitus(String),
}

/// HKT witness for `OptimumExpr`.
#[cfg(feature = "alloc")]
pub struct OptimumFWitness;

#[cfg(feature = "alloc")]
impl HKT for OptimumFWitness {
    type Target<T> = OptimumExpr<T>;
}

#[cfg(feature = "alloc")]
impl AlgebraArithmetica<OptimumFWitness> for InterpresOptimans {
    #[inline]
    fn lit(n: i32) -> OptimumExpr<i32> {
        OptimumExpr::Constans(n)
    }

    fn addo(a: OptimumExpr<i32>, b: OptimumExpr<i32>) -> OptimumExpr<i32> {
        match (a, b) {
            (OptimumExpr::Constans(x), OptimumExpr::Constans(y)) => OptimumExpr::Constans(x + y),
            (OptimumExpr::Constans(0), other) | (other, OptimumExpr::Constans(0)) => other,
            (a, b) => OptimumExpr::Incognitus(format!("{a:?} + {b:?}")),
        }
    }

    fn multiplico(a: OptimumExpr<i32>, b: OptimumExpr<i32>) -> OptimumExpr<i32> {
        match (a, b) {
            (OptimumExpr::Constans(x), OptimumExpr::Constans(y)) => OptimumExpr::Constans(x * y),
            (OptimumExpr::Constans(0), _) | (_, OptimumExpr::Constans(0)) => {
                OptimumExpr::Constans(0)
            }
            (OptimumExpr::Constans(1), other) | (other, OptimumExpr::Constans(1)) => other,
            (a, b) => OptimumExpr::Incognitus(format!("{a:?} * {b:?}")),
        }
    }

    fn subtraho(a: OptimumExpr<i32>, b: OptimumExpr<i32>) -> OptimumExpr<i32> {
        match (a, b) {
            (OptimumExpr::Constans(x), OptimumExpr::Constans(y)) => OptimumExpr::Constans(x - y),
            (other, OptimumExpr::Constans(0)) => other,
            (a, b) => OptimumExpr::Incognitus(format!("{a:?} - {b:?}")),
        }
    }

    fn nego(a: OptimumExpr<i32>) -> OptimumExpr<i32> {
        match a {
            OptimumExpr::Constans(x) => OptimumExpr::Constans(-x),
            a @ OptimumExpr::Incognitus(_) => OptimumExpr::Incognitus(format!("-{a:?}")),
        }
    }
}

// =============================================================================
// Higher-Order Tagless Final
// =============================================================================

/// Algebra with lambda abstractions (higher-order abstract syntax).
///
/// This enables representing functions in the DSL.
///
/// # Latin Etymology
///
/// *Algebra Superior* = higher algebra
pub trait AlgebraSuperior<F: HKT>: AlgebraArithmetica<F> {
    /// Lambda abstraction.
    ///
    /// # Latin Etymology
    /// *Lam* = lambda (λ)
    fn lam<A, B, Func>(f: Func) -> F::Target<Box<dyn Fn(A) -> B>>
    where
        Func: Fn(F::Target<A>) -> F::Target<B> + 'static;

    /// Function application.
    ///
    /// # Latin Etymology
    /// *App* = apply
    fn app<A, B>(f: F::Target<Box<dyn Fn(A) -> B>>, a: F::Target<A>) -> F::Target<B>;
}

// =============================================================================
// Symantics - Combined Semantics
// =============================================================================

/// Combined semantics trait for full expression language.
///
/// This combines multiple algebras into a single DSL.
///
/// # Latin Etymology
///
/// *Symantica* = semantics (syntax + meaning)
pub trait Symantica<F: HKT>: AlgebraArithmetica<F> + AlgebraBooleana<F> {}

// Blanket implementation
impl<F: HKT, T> Symantica<F> for T where T: AlgebraArithmetica<F> + AlgebraBooleana<F> {}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::super::nat::IdentitasFWitness;
    use super::*;
    use alloc::string::ToString;

    // Helper function to build an expression polymorphically
    fn expr_simple<F: HKT, Alg: AlgebraArithmetica<F>>() -> F::Target<i32> {
        Alg::addo(Alg::lit(10), Alg::multiplico(Alg::lit(3), Alg::lit(4)))
    }

    fn expr_with_neg<F: HKT, Alg: AlgebraArithmetica<F>>() -> F::Target<i32> {
        Alg::addo(Alg::lit(5), Alg::nego(Alg::lit(3)))
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

    #[cfg(feature = "alloc")]
    #[test]
    fn test_pretty_simple() {
        let result = expr_simple::<ConstStringFWitness, InterpresPulcher>();
        assert_eq!(result, "(10 + (3 * 4))");
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn test_pretty_with_neg() {
        let result = expr_with_neg::<ConstStringFWitness, InterpresPulcher>();
        assert_eq!(result, "(5 + (-3))");
    }

    #[test]
    fn test_count_simple() {
        let result = expr_simple::<ConstUsizeFWitness, InterpresNumerans>();
        assert_eq!(result, 5); // 1 add + 1 mul + 3 lit = 5 nodes
    }

    #[test]
    fn test_boolean_eval() {
        let t = InterpresAestimationis::verum();
        let f = InterpresAestimationis::falsum();

        assert!(t);
        assert!(!f);

        let and_tt = InterpresAestimationis::et(true, true);
        let and_tf = InterpresAestimationis::et(true, false);
        let or_ff = InterpresAestimationis::vel(false, false);
        let or_tf = InterpresAestimationis::vel(true, false);
        let not_t = InterpresAestimationis::non(true);

        assert!(and_tt);
        assert!(!and_tf);
        assert!(!or_ff);
        assert!(or_tf);
        assert!(!not_t);
    }

    #[cfg(feature = "alloc")]
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

    #[cfg(feature = "alloc")]
    #[test]
    fn test_optimization_identity_rules() {
        // x + 0 = x
        let zero_add = InterpresOptimans::addo(
            OptimumExpr::Incognitus("x".to_string()),
            OptimumExpr::Constans(0),
        );
        assert_eq!(zero_add, OptimumExpr::Incognitus("x".to_string()));

        // x * 1 = x
        let one_mul = InterpresOptimans::multiplico(
            OptimumExpr::Incognitus("x".to_string()),
            OptimumExpr::Constans(1),
        );
        assert_eq!(one_mul, OptimumExpr::Incognitus("x".to_string()));

        // x * 0 = 0
        let zero_mul = InterpresOptimans::multiplico(
            OptimumExpr::Incognitus("x".to_string()),
            OptimumExpr::Constans(0),
        );
        assert_eq!(zero_mul, OptimumExpr::Constans(0));
    }
}
