//! Fast IO Effect - Defunctionalized IO Operations
//!
//! This module provides two IO representations with different allocation
//! profiles:
//!
//! - [`FastIO`] — a two-variant enum. `Pure` carries the value inline, but
//!   `perform` / `map` (on the perform path) / `and_then` **still box a
//!   closure** — it is an `IoComputation` alternative, not allocation-free.
//! - The [`IoOp`] trait family (`PureIO`, `PerformIO`, `MapIO`, `AndThenIO`,
//!   `ThenIO`) — genuinely stack-allocated combinator structs that the
//!   compiler can monomorphize and inline; no heap allocation anywhere.
//!
//! Note this `IoOp` trait is unrelated to `nexus::effects::io::IoOp` (an
//! enum of IO operations) — same name, different module, different thing.
//!
//! # Performance
//!
//! For zero allocation use the `IoOp` structs; use `FastIO` when you need a
//! single concrete type and can accept one box per non-pure step.

use alloc::boxed::Box;

// =============================================================================
// Fast IO Representation
// =============================================================================

/// A fast IO computation using enum representation.
pub enum FastIO<A> {
    /// Pure value - no side effect.
    Pure(A),
    /// Perform an action.
    Perform(Box<dyn FnOnce() -> A>),
}

impl<A: 'static> FastIO<A> {
    /// Create a pure value.
    #[inline]
    pub fn pure(value: A) -> Self {
        FastIO::Pure(value)
    }

    /// Perform an IO action.
    #[inline]
    pub fn perform<F: FnOnce() -> A + 'static>(f: F) -> Self {
        FastIO::Perform(Box::new(f))
    }

    /// Run the IO computation.
    #[inline]
    pub fn run(self) -> A {
        match self {
            FastIO::Pure(a) => a,
            FastIO::Perform(f) => f(),
        }
    }

    /// Map over the result.
    #[inline]
    pub fn map<B: 'static, F: FnOnce(A) -> B + 'static>(self, f: F) -> FastIO<B> {
        match self {
            FastIO::Pure(a) => FastIO::Pure(f(a)),
            FastIO::Perform(action) => FastIO::Perform(Box::new(move || f(action()))),
        }
    }

    /// Chain two IO computations.
    #[inline]
    pub fn and_then<B: 'static, F: FnOnce(A) -> FastIO<B> + 'static>(self, f: F) -> FastIO<B> {
        FastIO::Perform(Box::new(move || f(self.run()).run()))
    }
}

// =============================================================================
// Inlined IO - Zero Allocation for Simple Chains
// =============================================================================

/// A trait for IO operations that can be inlined.
pub trait IoOp {
    /// The value produced when the operation runs; composed operations
    /// (map/chain wrappers) surface the final value of the whole chain
    /// here.
    type Output;
    /// Execute the IO operation, consuming `self` and returning the result.
    ///
    /// Implementations are expected to be `#[inline(always)]` so the compiler
    /// can monomorphize and eliminate the trait dispatch entirely, making
    /// zero-allocation chains as fast as hand-written sequential code.
    fn run_io(self) -> Self::Output;
}

/// Pure value IO operation.
pub struct PureIO<A>(pub A);

impl<A> IoOp for PureIO<A> {
    type Output = A;
    #[inline(always)]
    fn run_io(self) -> A {
        self.0
    }
}

/// Perform an action.
pub struct PerformIO<F>(pub F);

impl<A, F: FnOnce() -> A> IoOp for PerformIO<F> {
    type Output = A;
    #[inline(always)]
    fn run_io(self) -> A {
        (self.0)()
    }
}

/// Map operation.
pub struct MapIO<Op, F>(pub Op, pub F);

impl<Op: IoOp, B, F: FnOnce(Op::Output) -> B> IoOp for MapIO<Op, F> {
    type Output = B;
    #[inline(always)]
    fn run_io(self) -> B {
        (self.1)(self.0.run_io())
    }
}

/// `AndThen` operation.
pub struct AndThenIO<Op1, F>(pub Op1, pub F);

impl<Op1: IoOp, Op2: IoOp, F: FnOnce(Op1::Output) -> Op2> IoOp for AndThenIO<Op1, F> {
    type Output = Op2::Output;
    #[inline(always)]
    fn run_io(self) -> Op2::Output {
        (self.1)(self.0.run_io()).run_io()
    }
}

/// Sequence operations, discarding first result.
pub struct ThenIO<Op1, Op2>(pub Op1, pub Op2);

impl<Op1: IoOp, Op2: IoOp> IoOp for ThenIO<Op1, Op2> {
    type Output = Op2::Output;
    #[inline(always)]
    fn run_io(self) -> Op2::Output {
        let _ = self.0.run_io();
        self.1.run_io()
    }
}

/// Extension trait for chaining IO operations.
pub trait IoOpExt: IoOp + Sized {
    /// Map over the result.
    #[inline(always)]
    fn map_io<B, F: FnOnce(Self::Output) -> B>(self, f: F) -> MapIO<Self, F> {
        MapIO(self, f)
    }

    /// Chain with another operation.
    #[inline(always)]
    fn and_then_io<Op2: IoOp, F: FnOnce(Self::Output) -> Op2>(self, f: F) -> AndThenIO<Self, F> {
        AndThenIO(self, f)
    }

    /// Sequence, discarding first result.
    #[inline(always)]
    fn then_io<Op2: IoOp>(self, next: Op2) -> ThenIO<Self, Op2> {
        ThenIO(self, next)
    }
}

impl<Op: IoOp> IoOpExt for Op {}

// =============================================================================
// Convenience Functions
// =============================================================================

/// Create a pure IO operation.
#[inline(always)]
pub fn pure_io<A>(a: A) -> PureIO<A> {
    PureIO(a)
}

/// Create a perform IO operation.
#[inline(always)]
pub fn perform_io<A, F: FnOnce() -> A>(f: F) -> PerformIO<F> {
    PerformIO(f)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fast_io_pure() {
        let io = FastIO::pure(42);
        assert_eq!(io.run(), 42);
    }

    #[test]
    fn test_fast_io_perform() {
        let io = FastIO::perform(|| 42);
        assert_eq!(io.run(), 42);
    }

    #[test]
    fn test_fast_io_map_pure() {
        let io = FastIO::pure(21).map(|x| x * 2);
        assert_eq!(io.run(), 42);
    }

    #[test]
    fn test_io_op_pure() {
        let op = pure_io(42);
        assert_eq!(op.run_io(), 42);
    }

    #[test]
    fn test_io_op_perform() {
        let op = perform_io(|| 42);
        assert_eq!(op.run_io(), 42);
    }

    #[test]
    fn test_io_op_map() {
        let op = pure_io(21).map_io(|x| x * 2);
        assert_eq!(op.run_io(), 42);
    }

    #[test]
    fn test_io_op_chain() {
        let op = pure_io(20)
            .and_then_io(|x| pure_io(x + 1))
            .map_io(|x| x * 2);
        assert_eq!(op.run_io(), 42);
    }

    #[test]
    fn test_io_op_then() {
        let op = pure_io(1).then_io(pure_io(42));
        assert_eq!(op.run_io(), 42);
    }

    #[test]
    fn test_io_op_long_chain() {
        // Chain of 10 operations - fully inlined, no heap allocation
        let op = pure_io(0i32)
            .and_then_io(|x| pure_io(x + 1))
            .and_then_io(|x| pure_io(x + 1))
            .and_then_io(|x| pure_io(x + 1))
            .and_then_io(|x| pure_io(x + 1))
            .and_then_io(|x| pure_io(x + 1))
            .and_then_io(|x| pure_io(x + 1))
            .and_then_io(|x| pure_io(x + 1))
            .and_then_io(|x| pure_io(x + 1))
            .and_then_io(|x| pure_io(x + 1))
            .and_then_io(|x| pure_io(x + 1));
        assert_eq!(op.run_io(), 10);
    }

    #[test]
    fn test_io_op_with_side_effects() {
        use alloc::rc::Rc;
        use core::cell::Cell;

        let counter = Rc::new(Cell::new(0));
        let c1 = counter.clone();
        let c2 = counter.clone();
        let c3 = counter.clone();

        let op = perform_io(move || {
            c1.set(c1.get() + 1);
            10
        })
        .and_then_io(move |x| {
            perform_io(move || {
                c2.set(c2.get() + x);
                x + 5
            })
        })
        .map_io(move |x| {
            c3.set(c3.get() + 100);
            x * 2
        });

        let result = op.run_io();
        assert_eq!(result, 30); // (10 + 5) * 2
        assert_eq!(counter.get(), 111); // 1 + 10 + 100
    }
}
