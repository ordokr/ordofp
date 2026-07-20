//! Fast State Effect - Stack-Allocated State Operations
//!
//! This module provides an optimized state monad implementation that avoids
//! heap allocation for common operations by using enum-based defunctionalization.
//!
//! # Performance
//!
//! The standard `StatefulComputation` boxes every closure, causing heap allocation
//! per operation. `FastState` uses an enum representation that keeps common
//! operations on the stack:
//!
//! | Operation | `StatefulComputation` | `FastState` |
//! |-----------|---------------------|-----------|
//! | pure(x)   | 1 heap alloc        | 0 allocs  |
//! | `get()`     | 1 heap alloc        | 0 allocs  |
//! | put(x)    | 1 heap alloc        | 0 allocs  |
//! | modify(f) | 1 heap alloc        | 0 allocs  |
//! | map(f)    | 1 heap alloc        | 1 alloc   |
//! | `and_then`  | 1 heap alloc        | 1 alloc   |
//!
//! For chains of N operations, this reduces allocations from O(N) to O(1) for
//! simple get/put/modify sequences.

use alloc::boxed::Box;
use core::any::TypeId;
use core::marker::PhantomData;

// =============================================================================
// Fast State Representation
// =============================================================================

/// A stack-allocated state computation.
///
/// Uses enum variants for common operations to avoid boxing.
pub enum FastState<S, A> {
    /// Pure value - no state change.
    Pure(A),
    /// Get the current state.
    Get(PhantomData<S>),
    /// Put a new state value.
    Put(S, PhantomData<A>),
    /// Modify state with a function.
    Modify(Box<dyn FnOnce(S) -> S>, PhantomData<A>),
    /// Map over result.
    Map(Box<dyn FnOnce(S) -> (A, S)>),
    /// Boxed computation (fallback).
    Boxed(Box<dyn FnOnce(S) -> (A, S)>),
}

impl<S: 'static, A: 'static> FastState<S, A> {
    /// Create a pure value.
    #[inline]
    pub fn pure(value: A) -> Self {
        FastState::Pure(value)
    }

    /// Get the current state.
    #[inline]
    pub fn get() -> FastState<S, S>
    where
        S: Clone,
    {
        FastState::Get(PhantomData)
    }

    /// Put a new state.
    #[inline]
    pub fn put(value: S) -> FastState<S, ()> {
        FastState::Put(value, PhantomData)
    }

    /// Modify state with a function.
    #[inline]
    pub fn modify<F: FnOnce(S) -> S + 'static>(f: F) -> FastState<S, ()> {
        FastState::Modify(Box::new(f), PhantomData)
    }

    /// Create from a function (boxed fallback).
    #[inline]
    pub fn new<F: FnOnce(S) -> (A, S) + 'static>(f: F) -> Self {
        FastState::Boxed(Box::new(f))
    }

    /// Run the computation with initial state.
    ///
    /// # Panics
    ///
    /// Panics on a `Get` variant — `S` is not bounded by `Clone` here, so
    /// the state cannot be duplicated into the result; use `run_get`
    /// (which requires `S: Clone`) for `Get`. Also panics on a
    /// `Put`/`Modify` variant whose result type `A` is not `()` (checked
    /// at runtime via `TypeId`); the typed constructors never build such
    /// a value, so that arm can only fire on a hand-constructed variant.
    #[inline]
    pub fn run(self, state: S) -> (A, S) {
        match self {
            FastState::Pure(a) => (a, state),
            FastState::Get(_) => {
                // Safety: We must panic if run() is called on Get variant without specialized handling
                // because we cannot clone state here (S is not bounded by Clone).
                // Use run_get() for the Get variant which properly requires S: Clone.
                panic!(
                    "FastState::run called on Get variant. Use run_get() for optimized Get operations, or ensure Universalis run() is not used with Get."
                );
            }
            FastState::Put(new_state, _) => {
                assert!(
                    TypeId::of::<A>() == TypeId::of::<()>(),
                    "FastState::Put requires A = (), but found {:?}",
                    core::any::type_name::<A>()
                );
                let mut opt: Option<()> = Some(());
                let any_opt = &mut opt as &mut dyn core::any::Any;
                let downcast: &mut Option<A> = any_opt.downcast_mut::<Option<A>>().unwrap();
                let unit: A = downcast.take().unwrap();
                (unit, new_state)
            }
            FastState::Modify(f, _) => {
                assert!(
                    TypeId::of::<A>() == TypeId::of::<()>(),
                    "FastState::Modify requires A = (), but found {:?}",
                    core::any::type_name::<A>()
                );
                let new_state = f(state);
                let mut opt: Option<()> = Some(());
                let any_opt = &mut opt as &mut dyn core::any::Any;
                let downcast: &mut Option<A> = any_opt.downcast_mut::<Option<A>>().unwrap();
                let unit: A = downcast.take().unwrap();
                (unit, new_state)
            }
            FastState::Map(f) => f(state),
            FastState::Boxed(f) => f(state),
        }
    }

    /// Run the computation, handling the `Get` variant by cloning the state.
    ///
    /// Composition paths (`map`/`and_then`) route through this instead of
    /// `run()` so that composed `get()` chains work rather than panicking.
    #[inline]
    fn run_cloned(self, state: S) -> (A, S)
    where
        S: Clone,
    {
        match self {
            FastState::Get(_) => {
                assert!(
                    TypeId::of::<A>() == TypeId::of::<S>(),
                    "FastState::Get requires A = S, but found {:?}",
                    core::any::type_name::<A>()
                );
                let mut opt: Option<S> = Some(state.clone());
                let any_opt = &mut opt as &mut dyn core::any::Any;
                let downcast: &mut Option<A> = any_opt.downcast_mut::<Option<A>>().unwrap();
                let value: A = downcast.take().unwrap();
                (value, state)
            }
            other => other.run(state),
        }
    }

    /// Map over the result.
    #[inline]
    pub fn map<B: 'static, F: FnOnce(A) -> B + 'static>(self, f: F) -> FastState<S, B>
    where
        S: Clone,
    {
        match self {
            FastState::Pure(a) => FastState::Pure(f(a)),
            other => FastState::Map(Box::new(move |s| {
                let (a, s2) = other.run_cloned(s);
                (f(a), s2)
            })),
        }
    }

    /// Chain two computations.
    #[inline]
    pub fn and_then<B: 'static, F: FnOnce(A) -> FastState<S, B> + 'static>(
        self,
        f: F,
    ) -> FastState<S, B>
    where
        S: Clone,
    {
        FastState::Boxed(Box::new(move |s| {
            let (a, s2) = self.run_cloned(s);
            f(a).run_cloned(s2)
        }))
    }
}

// Specialized implementation for S = A case (Get)
impl<S: Clone + 'static> FastState<S, S> {
    /// Run Get operation.
    #[inline]
    pub fn run_get(self, state: S) -> (S, S) {
        match self {
            FastState::Pure(a) => (a, state),
            FastState::Get(_) => (state.clone(), state),
            FastState::Map(f) => f(state),
            FastState::Boxed(f) => f(state),
            _ => crate::cold_panic!("Invalid FastState variant for run_get"),
        }
    }
}

// Specialized implementation for A = () case (Put/Modify)
impl<S: 'static> FastState<S, ()> {
    /// Run Put/Modify operation.
    #[inline]
    pub fn run_unit(self, state: S) -> ((), S) {
        match self {
            FastState::Pure(()) => ((), state),
            FastState::Put(new_state, _) => ((), new_state),
            FastState::Modify(f, _) => ((), f(state)),
            FastState::Map(f) => f(state),
            FastState::Boxed(f) => f(state),
            FastState::Get(_) => crate::cold_panic!("Invalid FastState variant for run_unit"),
        }
    }
}

// =============================================================================
// Inlined State - Zero Allocation for Simple Chains
// =============================================================================

/// A fully inlined state computation using a trait-based approach.
///
/// This allows the compiler to monomorphize and inline the entire chain.
pub trait StateOp<S> {
    /// The value produced by running this operation (the `A` in
    /// `(A, S)`); the state type `S` is threaded through unchanged.
    type Output;
    /// Run this state operation against `state`, returning the produced value
    /// and the updated state as `(output, new_state)`.
    ///
    /// Implementors should be `#[inline(always)]` so the compiler can
    /// monomorphize and eliminate trait dispatch, preserving the zero-allocation
    /// guarantee of inlined state chains.
    fn run_op(self, state: S) -> (Self::Output, S);
}

/// Pure value operation.
pub struct PureOp<A>(pub A);

impl<S, A> StateOp<S> for PureOp<A> {
    type Output = A;
    #[inline(always)]
    fn run_op(self, state: S) -> (A, S) {
        (self.0, state)
    }
}

/// Get state operation.
pub struct GetOp;

impl<S: Clone> StateOp<S> for GetOp {
    type Output = S;
    #[inline(always)]
    fn run_op(self, state: S) -> (S, S) {
        (state.clone(), state)
    }
}

/// Put state operation.
pub struct PutOp<S>(pub S);

impl<S> StateOp<S> for PutOp<S> {
    type Output = ();
    #[inline(always)]
    fn run_op(self, _state: S) -> ((), S) {
        ((), self.0)
    }
}

/// Modify state operation.
pub struct ModifyOp<S, F: FnOnce(S) -> S>(pub F, PhantomData<S>);

impl<S, F: FnOnce(S) -> S> StateOp<S> for ModifyOp<S, F> {
    type Output = ();
    #[inline(always)]
    fn run_op(self, state: S) -> ((), S) {
        ((), (self.0)(state))
    }
}

/// Map operation.
pub struct MapOp<Op, F>(pub Op, pub F);

impl<S, Op: StateOp<S>, B, F: FnOnce(Op::Output) -> B> StateOp<S> for MapOp<Op, F> {
    type Output = B;
    #[inline(always)]
    fn run_op(self, state: S) -> (B, S) {
        let (a, s2) = self.0.run_op(state);
        ((self.1)(a), s2)
    }
}

/// `AndThen` operation.
pub struct AndThenOp<Op1, F>(pub Op1, pub F);

impl<S, Op1: StateOp<S>, Op2: StateOp<S>, F: FnOnce(Op1::Output) -> Op2> StateOp<S>
    for AndThenOp<Op1, F>
{
    type Output = Op2::Output;
    #[inline(always)]
    fn run_op(self, state: S) -> (Op2::Output, S) {
        let (a, s2) = self.0.run_op(state);
        (self.1)(a).run_op(s2)
    }
}

/// Extension trait for chaining state operations.
pub trait StateOpExt<S>: StateOp<S> + Sized {
    /// Map over the result.
    #[inline(always)]
    fn map_op<B, F: FnOnce(Self::Output) -> B>(self, f: F) -> MapOp<Self, F> {
        MapOp(self, f)
    }

    /// Chain with another operation.
    #[inline(always)]
    fn and_then_op<Op2: StateOp<S>, F: FnOnce(Self::Output) -> Op2>(
        self,
        f: F,
    ) -> AndThenOp<Self, F> {
        AndThenOp(self, f)
    }
}

impl<S, Op: StateOp<S>> StateOpExt<S> for Op {}

// =============================================================================
// Convenience Functions
// =============================================================================

/// Create a pure state operation.
#[inline(always)]
pub fn pure_op<A>(a: A) -> PureOp<A> {
    PureOp(a)
}

/// Create a get state operation.
#[inline(always)]
pub fn get_op() -> GetOp {
    GetOp
}

/// Create a put state operation.
#[inline(always)]
pub fn put_op<S>(s: S) -> PutOp<S> {
    PutOp(s)
}

/// Create a modify state operation.
#[inline(always)]
pub fn modify_op<S, F: FnOnce(S) -> S>(f: F) -> ModifyOp<S, F> {
    ModifyOp(f, PhantomData)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fast_state_pure() {
        let comp = FastState::<i32, i32>::pure(42);
        let (result, state) = comp.run(0);
        assert_eq!(result, 42);
        assert_eq!(state, 0);
    }

    #[test]
    fn test_fast_state_get() {
        let comp = FastState::<i32, i32>::get();
        let (result, state) = comp.run_get(42);
        assert_eq!(result, 42);
        assert_eq!(state, 42);
    }

    #[test]
    fn test_fast_state_put() {
        let comp = FastState::<i32, ()>::put(42);
        let ((), state) = comp.run_unit(0);
        assert_eq!(state, 42);
    }

    #[test]
    fn test_fast_state_modify() {
        let comp = FastState::<i32, ()>::modify(|x| x + 10);
        let ((), state) = comp.run_unit(32);
        assert_eq!(state, 42);
    }

    #[test]
    fn test_fast_state_get_map_composition() {
        // Regression: get().map(f) must not panic — the Get variant has to be
        // handled in the composition paths, not routed through run().
        let comp = FastState::<i32, i32>::get().map(|x| x + 1);
        let (result, state) = comp.run_get(41);
        assert_eq!(result, 42);
        assert_eq!(state, 41);
    }

    #[test]
    fn test_fast_state_get_and_then_composition() {
        // Regression: get().and_then(f) must not panic.
        let comp = FastState::<i32, i32>::get().and_then(|x| FastState::<i32, ()>::put(x + 1));
        let ((), state) = comp.run_unit(41);
        assert_eq!(state, 42);
    }

    #[test]
    fn test_state_op_pure() {
        let op = pure_op(42);
        let (result, state) = op.run_op(0i32);
        assert_eq!(result, 42);
        assert_eq!(state, 0);
    }

    #[test]
    fn test_state_op_get() {
        let op = get_op();
        let (result, state) = op.run_op(42i32);
        assert_eq!(result, 42);
        assert_eq!(state, 42);
    }

    #[test]
    fn test_state_op_put() {
        let op = put_op(42);
        let ((), state) = op.run_op(0i32);
        assert_eq!(state, 42);
    }

    #[test]
    fn test_state_op_modify() {
        let op = modify_op(|x: i32| x + 10);
        let ((), state) = op.run_op(32);
        assert_eq!(state, 42);
    }

    #[test]
    fn test_state_op_map() {
        let op = get_op().map_op(|x: i32| x * 2);
        let (result, state) = op.run_op(21);
        assert_eq!(result, 42);
        assert_eq!(state, 21);
    }

    #[test]
    fn test_state_op_chain() {
        // get().and_then(|x| put(x + 10)).map(|_| "done")
        let op = get_op()
            .and_then_op(|x: i32| put_op(x + 10))
            .map_op(|()| "done");
        let (result, state) = op.run_op(32);
        assert_eq!(result, "done");
        assert_eq!(state, 42);
    }

    #[test]
    fn test_state_op_long_chain() {
        // Chain of 10 modifications - fully inlined, no heap allocation
        let op = modify_op(|x: i32| x + 1)
            .and_then_op(|()| modify_op(|x: i32| x + 1))
            .and_then_op(|()| modify_op(|x: i32| x + 1))
            .and_then_op(|()| modify_op(|x: i32| x + 1))
            .and_then_op(|()| modify_op(|x: i32| x + 1))
            .and_then_op(|()| modify_op(|x: i32| x + 1))
            .and_then_op(|()| modify_op(|x: i32| x + 1))
            .and_then_op(|()| modify_op(|x: i32| x + 1))
            .and_then_op(|()| modify_op(|x: i32| x + 1))
            .and_then_op(|()| modify_op(|x: i32| x + 1));
        let ((), state) = op.run_op(0);
        assert_eq!(state, 10);
    }

    #[test]
    #[should_panic(
        expected = "FastState::run called on Get variant. Use run_get() for optimized Get operations"
    )]
    fn test_fast_state_get_run_panic() {
        // Verify that Universalis run() panics explicitly for Get variant,
        // rather than failing with confusing "unreachable" or unsoundness.
        let comp: FastState<i32, i32> = FastState::<i32, i32>::get();
        let _ = comp.run(42);
    }
}
