//! Fast Writer Effect - Stack-Allocated Writer Operations
//!
//! This module provides an optimized writer implementation that avoids
//! heap allocation for common operations by using trait-based defunctionalization.
//!
//! # Performance
//!
//! The standard `WriterComputation` boxes every closure. `WriterOp` uses
//! trait-based representation that allows the compiler to inline and
//! monomorphize the entire computation chain.

use core::marker::PhantomData;

// =============================================================================
// Monoid Trait (re-export or define locally)
// =============================================================================

/// A monoid has an identity element and an associative binary operation.
pub trait FastMonoid: Clone {
    /// The identity element.
    fn empty() -> Self;

    /// Combine two values (consuming self for efficiency).
    fn combine(self, other: Self) -> Self;
}

impl FastMonoid for alloc::string::String {
    #[inline(always)]
    fn empty() -> Self {
        alloc::string::String::new()
    }

    #[inline(always)]
    fn combine(mut self, other: Self) -> Self {
        self.push_str(&other);
        self
    }
}

impl<T: Clone> FastMonoid for alloc::vec::Vec<T> {
    #[inline(always)]
    fn empty() -> Self {
        alloc::vec::Vec::new()
    }

    #[inline(always)]
    fn combine(mut self, other: Self) -> Self {
        self.extend(other);
        self
    }
}

// =============================================================================
// Inlined Writer - Zero Allocation
// =============================================================================

/// A trait for writer operations that can be inlined.
///
/// Writer operations produce a value and accumulate a log.
pub trait WriterOp<W: FastMonoid> {
    /// The value produced alongside the accumulated log `W` — the first
    /// component of the `(result, log)` pair `run_writer` returns.
    type Output;
    /// Run the writer operation, returning (result, log).
    fn run_writer(self) -> (Self::Output, W);
}

/// Pure value writer operation - no log output.
pub struct PureWriter<W, A>(pub A, PhantomData<W>);

impl<W: FastMonoid, A> WriterOp<W> for PureWriter<W, A> {
    type Output = A;
    #[inline(always)]
    fn run_writer(self) -> (A, W) {
        (self.0, W::empty())
    }
}

/// Tell operation - write to the log.
pub struct TellWriter<W>(pub W);

impl<W: FastMonoid> WriterOp<W> for TellWriter<W> {
    type Output = ();
    #[inline(always)]
    fn run_writer(self) -> ((), W) {
        ((), self.0)
    }
}

/// Map operation.
pub struct MapWriter<Op, F>(pub Op, pub F);

impl<W: FastMonoid, Op: WriterOp<W>, B, F: FnOnce(Op::Output) -> B> WriterOp<W>
    for MapWriter<Op, F>
{
    type Output = B;
    #[inline(always)]
    fn run_writer(self) -> (B, W) {
        let (a, w) = self.0.run_writer();
        ((self.1)(a), w)
    }
}

/// `AndThen` operation.
pub struct AndThenWriter<Op1, F>(pub Op1, pub F);

impl<W: FastMonoid, Op1: WriterOp<W>, Op2: WriterOp<W>, F: FnOnce(Op1::Output) -> Op2> WriterOp<W>
    for AndThenWriter<Op1, F>
{
    type Output = Op2::Output;
    #[inline(always)]
    fn run_writer(self) -> (Op2::Output, W) {
        let (a, w1) = self.0.run_writer();
        let (b, w2) = (self.1)(a).run_writer();
        (b, w1.combine(w2))
    }
}

/// Sequence operation - run two writers, keep second result.
pub struct ThenWriter<Op1, Op2>(pub Op1, pub Op2);

impl<W: FastMonoid, Op1: WriterOp<W>, Op2: WriterOp<W>> WriterOp<W> for ThenWriter<Op1, Op2> {
    type Output = Op2::Output;
    #[inline(always)]
    fn run_writer(self) -> (Op2::Output, W) {
        let (_, w1) = self.0.run_writer();
        let (b, w2) = self.1.run_writer();
        (b, w1.combine(w2))
    }
}

/// Listen operation - get the log along with the result.
pub struct ListenWriter<Op>(pub Op);

impl<W: FastMonoid, Op: WriterOp<W>> WriterOp<W> for ListenWriter<Op> {
    type Output = (Op::Output, W);
    #[inline(always)]
    fn run_writer(self) -> ((Op::Output, W), W) {
        let (a, w) = self.0.run_writer();
        ((a, w.clone()), w)
    }
}

/// Censor operation - modify the log.
pub struct CensorWriter<Op, F>(pub Op, pub F);

impl<W: FastMonoid, Op: WriterOp<W>, F: FnOnce(W) -> W> WriterOp<W> for CensorWriter<Op, F> {
    type Output = Op::Output;
    #[inline(always)]
    fn run_writer(self) -> (Op::Output, W) {
        let (a, w) = self.0.run_writer();
        (a, (self.1)(w))
    }
}

/// Extension trait for chaining writer operations.
pub trait WriterOpExt<W: FastMonoid>: WriterOp<W> + Sized {
    /// Map over the result.
    #[inline(always)]
    fn map_writer<B, F: FnOnce(Self::Output) -> B>(self, f: F) -> MapWriter<Self, F> {
        MapWriter(self, f)
    }

    /// Chain with another operation.
    #[inline(always)]
    fn and_then_writer<Op2: WriterOp<W>, F: FnOnce(Self::Output) -> Op2>(
        self,
        f: F,
    ) -> AndThenWriter<Self, F> {
        AndThenWriter(self, f)
    }

    /// Sequence, discarding first result.
    #[inline(always)]
    fn then_writer<Op2: WriterOp<W>>(self, next: Op2) -> ThenWriter<Self, Op2> {
        ThenWriter(self, next)
    }

    /// Listen to the log.
    #[inline(always)]
    fn listen_writer(self) -> ListenWriter<Self> {
        ListenWriter(self)
    }

    /// Modify the log.
    #[inline(always)]
    fn censor_writer<F: FnOnce(W) -> W>(self, f: F) -> CensorWriter<Self, F> {
        CensorWriter(self, f)
    }
}

impl<W: FastMonoid, Op: WriterOp<W>> WriterOpExt<W> for Op {}

// =============================================================================
// Convenience Functions
// =============================================================================

/// Create a pure writer operation.
#[inline(always)]
pub fn pure_writer<W: FastMonoid, A>(a: A) -> PureWriter<W, A> {
    PureWriter(a, PhantomData)
}

/// Create a tell writer operation.
#[inline(always)]
pub fn tell_writer<W: FastMonoid>(w: W) -> TellWriter<W> {
    TellWriter(w)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::{String, ToString};
    use alloc::vec;
    use alloc::vec::Vec;

    #[test]
    fn test_writer_op_pure() {
        let op = pure_writer::<String, _>(42);
        let (result, log) = op.run_writer();
        assert_eq!(result, 42);
        assert_eq!(log, "");
    }

    #[test]
    fn test_writer_op_tell() {
        let op = tell_writer("Hello".to_string());
        let ((), log) = op.run_writer();
        assert_eq!(log, "Hello");
    }

    #[test]
    fn test_writer_op_map() {
        let op = pure_writer::<String, _>(21).map_writer(|x| x * 2);
        let (result, log) = op.run_writer();
        assert_eq!(result, 42);
        assert_eq!(log, "");
    }

    #[test]
    fn test_writer_op_and_then() {
        let op = tell_writer("Hello, ".to_string())
            .and_then_writer(|()| tell_writer("World!".to_string()));
        let ((), log) = op.run_writer();
        assert_eq!(log, "Hello, World!");
    }

    #[test]
    fn test_writer_op_then() {
        let op = tell_writer("Hello, ".to_string()).then_writer(tell_writer("World!".to_string()));
        let ((), log) = op.run_writer();
        assert_eq!(log, "Hello, World!");
    }

    #[test]
    fn test_writer_op_listen() {
        let op = tell_writer("Log".to_string())
            .and_then_writer(|()| pure_writer::<String, _>(42))
            .listen_writer();
        let ((result, inner_log), log) = op.run_writer();
        assert_eq!(result, 42);
        assert_eq!(inner_log, "Log");
        assert_eq!(log, "Log");
    }

    #[test]
    fn test_writer_op_censor() {
        let op = tell_writer("hello".to_string()).censor_writer(|s| s.to_uppercase());
        let ((), log) = op.run_writer();
        assert_eq!(log, "HELLO");
    }

    #[test]
    fn test_writer_op_vec() {
        let op = tell_writer(vec![1, 2]).and_then_writer(|()| tell_writer(vec![3, 4]));
        let ((), log): ((), Vec<i32>) = op.run_writer();
        assert_eq!(log, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_writer_op_long_chain() {
        // Chain of 10 tells - fully inlined
        let op = tell_writer(vec![1i32])
            .then_writer(tell_writer(vec![2]))
            .then_writer(tell_writer(vec![3]))
            .then_writer(tell_writer(vec![4]))
            .then_writer(tell_writer(vec![5]))
            .then_writer(tell_writer(vec![6]))
            .then_writer(tell_writer(vec![7]))
            .then_writer(tell_writer(vec![8]))
            .then_writer(tell_writer(vec![9]))
            .then_writer(tell_writer(vec![10]));
        let ((), log): ((), Vec<i32>) = op.run_writer();
        assert_eq!(log, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    }
}
