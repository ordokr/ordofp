//! Fast Reader Effect - Stack-Allocated Reader Operations
//!
//! This module provides an optimized reader implementation that avoids
//! heap allocation for common operations by using trait-based defunctionalization.
//!
//! # Performance
//!
//! The standard `ReaderComputation` boxes every closure. `ReaderOp` uses
//! trait-based representation that allows the compiler to inline and
//! monomorphize the entire computation chain.
//!
//! | Operation | ReaderComputation | ReaderOp |
//! |-----------|-------------------|----------|
//! | pure(x)   | 1 heap alloc      | 0 allocs |
//! | ask()     | 1 heap alloc      | 0 allocs |
//! | asks(f)   | 1 heap alloc      | 0 allocs |
//! | map(f)    | 1 heap alloc      | 0 allocs |
//! | and_then  | 1 heap alloc      | 0 allocs |

use core::marker::PhantomData;

// =============================================================================
// Inlined Reader - Zero Allocation
// =============================================================================

/// A trait for reader operations that can be inlined.
pub trait ReaderOp<E> {
    /// The value produced by running this reader against an environment
    /// `&E`; the environment itself is only borrowed, never consumed.
    type Output;
    /// Executes the reader computation against the given environment.
    ///
    /// Consumes `self` and produces the computed `Output` value by reading
    /// from `env`. Implementors should mark this `#[inline(always)]` so the
    /// compiler can monomorphize the entire computation chain with zero heap
    /// allocation.
    fn run_reader(self, env: &E) -> Self::Output;
}

/// Pure value reader operation.
pub struct PureReader<A>(pub A);

impl<E, A> ReaderOp<E> for PureReader<A> {
    type Output = A;
    #[inline(always)]
    fn run_reader(self, _env: &E) -> A {
        self.0
    }
}

/// Ask for the environment.
pub struct AskReader<E>(PhantomData<E>);

impl<E: Clone> ReaderOp<E> for AskReader<E> {
    type Output = E;
    #[inline(always)]
    fn run_reader(self, env: &E) -> E {
        env.clone()
    }
}

/// Extract a value from the environment.
pub struct AsksReader<E, A, F: FnOnce(&E) -> A>(pub F, PhantomData<(E, A)>);

impl<E, A, F: FnOnce(&E) -> A> ReaderOp<E> for AsksReader<E, A, F> {
    type Output = A;
    #[inline(always)]
    fn run_reader(self, env: &E) -> A {
        (self.0)(env)
    }
}

/// Map operation.
pub struct MapReader<Op, F>(pub Op, pub F);

impl<E, Op: ReaderOp<E>, B, F: FnOnce(Op::Output) -> B> ReaderOp<E> for MapReader<Op, F> {
    type Output = B;
    #[inline(always)]
    fn run_reader(self, env: &E) -> B {
        (self.1)(self.0.run_reader(env))
    }
}

/// `AndThen` operation.
pub struct AndThenReader<Op1, F>(pub Op1, pub F);

impl<E, Op1: ReaderOp<E>, Op2: ReaderOp<E>, F: FnOnce(Op1::Output) -> Op2> ReaderOp<E>
    for AndThenReader<Op1, F>
{
    type Output = Op2::Output;
    #[inline(always)]
    fn run_reader(self, env: &E) -> Op2::Output {
        let a = self.0.run_reader(env);
        (self.1)(a).run_reader(env)
    }
}

/// Local operation - run with modified environment.
pub struct LocalReader<Op, F>(pub Op, pub F);

impl<E, Op: ReaderOp<E>, F: FnOnce(&E) -> E> ReaderOp<E> for LocalReader<Op, F> {
    type Output = Op::Output;
    #[inline(always)]
    fn run_reader(self, env: &E) -> Op::Output {
        let new_env = (self.1)(env);
        self.0.run_reader(&new_env)
    }
}

/// Extension trait for chaining reader operations.
pub trait ReaderOpExt<E>: ReaderOp<E> + Sized {
    /// Map over the result.
    #[inline(always)]
    fn map_reader<B, F: FnOnce(Self::Output) -> B>(self, f: F) -> MapReader<Self, F> {
        MapReader(self, f)
    }

    /// Chain with another operation.
    #[inline(always)]
    fn and_then_reader<Op2: ReaderOp<E>, F: FnOnce(Self::Output) -> Op2>(
        self,
        f: F,
    ) -> AndThenReader<Self, F> {
        AndThenReader(self, f)
    }

    /// Run with a locally modified environment.
    #[inline(always)]
    fn local_reader<F: FnOnce(&E) -> E>(self, f: F) -> LocalReader<Self, F> {
        LocalReader(self, f)
    }
}

impl<E, Op: ReaderOp<E>> ReaderOpExt<E> for Op {}

// =============================================================================
// Convenience Functions
// =============================================================================

/// Create a pure reader operation.
#[inline(always)]
pub fn pure_reader<A>(a: A) -> PureReader<A> {
    PureReader(a)
}

/// Create an ask reader operation.
#[inline(always)]
pub fn ask_reader<E>() -> AskReader<E> {
    AskReader(PhantomData)
}

/// Create an asks reader operation.
#[inline(always)]
pub fn asks_reader<E, A, F: FnOnce(&E) -> A>(f: F) -> AsksReader<E, A, F> {
    AsksReader(f, PhantomData)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct TestEnv {
        value: i32,
        multiplier: i32,
    }

    #[test]
    fn test_reader_op_pure() {
        let op = pure_reader(42);
        let env = TestEnv {
            value: 0,
            multiplier: 1,
        };
        assert_eq!(op.run_reader(&env), 42);
    }

    #[test]
    fn test_reader_op_ask() {
        let op = ask_reader::<TestEnv>();
        let env = TestEnv {
            value: 42,
            multiplier: 2,
        };
        let result = op.run_reader(&env);
        assert_eq!(result.value, 42);
    }

    #[test]
    fn test_reader_op_asks() {
        let op = asks_reader(|e: &TestEnv| e.value);
        let env = TestEnv {
            value: 42,
            multiplier: 2,
        };
        assert_eq!(op.run_reader(&env), 42);
    }

    #[test]
    fn test_reader_op_map() {
        let op = asks_reader(|e: &TestEnv| e.value).map_reader(|x| x * 2);
        let env = TestEnv {
            value: 21,
            multiplier: 2,
        };
        assert_eq!(op.run_reader(&env), 42);
    }

    #[test]
    fn test_reader_op_and_then() {
        let op = asks_reader(|e: &TestEnv| e.value)
            .and_then_reader(|v| asks_reader(move |e: &TestEnv| v * e.multiplier));
        let env = TestEnv {
            value: 21,
            multiplier: 2,
        };
        assert_eq!(op.run_reader(&env), 42);
    }

    #[test]
    fn test_reader_op_local() {
        let op = asks_reader(|e: &TestEnv| e.value).local_reader(|e| TestEnv {
            value: e.value + 10,
            multiplier: e.multiplier,
        });
        let env = TestEnv {
            value: 32,
            multiplier: 1,
        };
        assert_eq!(op.run_reader(&env), 42);
    }

    #[test]
    fn test_reader_op_chain() {
        // Chain of operations - fully inlined
        let op = asks_reader(|e: &TestEnv| e.value)
            .and_then_reader(|v| asks_reader(move |e: &TestEnv| v + e.multiplier))
            .and_then_reader(|v| asks_reader(move |e: &TestEnv| v * e.multiplier))
            .map_reader(|x| x + 1);
        let env = TestEnv {
            value: 10,
            multiplier: 2,
        };
        // (10 + 2) * 2 + 1 = 25
        assert_eq!(op.run_reader(&env), 25);
    }

    #[test]
    fn test_reader_op_long_chain() {
        // Chain of 10 asks - fully inlined, no heap allocation
        let op = asks_reader(|e: &TestEnv| e.value)
            .and_then_reader(|acc| asks_reader(move |e: &TestEnv| acc + e.value))
            .and_then_reader(|acc| asks_reader(move |e: &TestEnv| acc + e.value))
            .and_then_reader(|acc| asks_reader(move |e: &TestEnv| acc + e.value))
            .and_then_reader(|acc| asks_reader(move |e: &TestEnv| acc + e.value))
            .and_then_reader(|acc| asks_reader(move |e: &TestEnv| acc + e.value))
            .and_then_reader(|acc| asks_reader(move |e: &TestEnv| acc + e.value))
            .and_then_reader(|acc| asks_reader(move |e: &TestEnv| acc + e.value))
            .and_then_reader(|acc| asks_reader(move |e: &TestEnv| acc + e.value))
            .and_then_reader(|acc| asks_reader(move |e: &TestEnv| acc + e.value));
        let env = TestEnv {
            value: 1,
            multiplier: 0,
        };
        assert_eq!(op.run_reader(&env), 10);
    }
}
