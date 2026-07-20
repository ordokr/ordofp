//! State Effect - Mutable State Threading
//!
//! The State effect provides mutable state that is threaded through
//! computations. This is the functional equivalent of mutable variables.
//!
//! # Handler Laws
//!
//! The State handler must satisfy these algebraic laws:
//!
//! ## Get-Put Law
//! ```text
//! get.and_then(|s| put(s)) = pure(())
//! ```
//! Getting the state and immediately putting it back is a no-op.
//!
//! ## Put-Get Law
//! ```text
//! put(s).and_then(|_| get) = put(s).map(|_| s)
//! ```
//! Putting a state and then getting returns the put value.
//!
//! ## Put-Put Law
//! ```text
//! put(s1).and_then(|_| put(s2)) = put(s2)
//! ```
//! Two consecutive puts is equivalent to just the second put.
//!
//! # Verification Tier
//!
//! **Tier 1**: Laws are tested via property-based tests in `nexus::laws`.
//!
//! # Performance
//!
//! When a computation uses only the State effect, it compiles to
//! the same code as manually threading state through functions.

use alloc::boxed::Box;
use core::any::TypeId;
use core::marker::PhantomData;

use crate::nexus::effect::{Eff, EffectMarker};
use crate::nexus::row::{Row, STATE_BIT};

// =============================================================================
// State Effect Type
// =============================================================================

/// The State effect marker type.
///
/// `StateEffect<S>` represents computations that can read and modify
/// state of type `S`.
#[derive(Copy, Clone, Debug)]
pub struct StateEffect<S> {
    _marker: PhantomData<S>,
}

impl<S> EffectMarker for StateEffect<S> {
    const BIT: u128 = STATE_BIT;
    const NAME: &'static str = "State";
}

/// Type alias for a row containing only State.
pub type StateRow = Row<STATE_BIT>;

// =============================================================================
// State Operations
// =============================================================================

/// Operations that can be performed with the State effect.
pub enum StateOp<S> {
    /// Get the current state.
    Get,
    /// Set a new state.
    Put(S),
}

impl<S: Clone> Clone for StateOp<S> {
    fn clone(&self) -> Self {
        match self {
            StateOp::Get => StateOp::Get,
            StateOp::Put(s) => StateOp::Put(s.clone()),
        }
    }
}

// =============================================================================
// State Effect Functions
// =============================================================================

/// Get the current state.
///
/// # Example
///
/// ```rust
/// use ordofp_core::nexus::effects::state::StatefulComputation;
///
/// let comp = StatefulComputation::<i32, i32>::get().map(|x| x + 1);
/// let (result, state) = comp.run(10);
/// assert_eq!(result, 11);
/// assert_eq!(state, 10);
/// ```
pub fn state_get<S: Clone + 'static>() -> Eff<StateRow, S> {
    Eff::lazy(|| {
        // In a full implementation, this would be handled by the State handler
        crate::cold_panic!("state_get requires State handler")
    })
}

/// Set the state to a new value.
///
/// # Example
///
/// ```rust
/// use ordofp_core::nexus::effects::state::StatefulComputation;
///
/// let comp = StatefulComputation::<i32, ()>::put(42);
/// let ((), state) = comp.run(0);
/// assert_eq!(state, 42);
/// ```
pub fn state_put<S: 'static>(_value: S) -> Eff<StateRow, ()> {
    Eff::lazy(move || crate::cold_panic!("state_put requires State handler"))
}

/// Modify the state with a function.
///
/// # Example
///
/// ```rust
/// use ordofp_core::nexus::effects::state::StatefulComputation;
///
/// let comp = StatefulComputation::<i32, ()>::modify(|x: i32| x + 10);
/// let ((), state) = comp.run(32);
/// assert_eq!(state, 42);
/// ```
pub fn state_modify<S: 'static, F: FnOnce(S) -> S + 'static>(_f: F) -> Eff<StateRow, ()> {
    Eff::lazy(|| crate::cold_panic!("state_modify requires State handler"))
}

/// Get a value derived from the state.
///
/// # Example
///
/// ```rust
/// use ordofp_core::nexus::effects::state::StatefulComputation;
///
/// #[derive(Clone)]
/// struct Config {
///     value: i32,
/// }
///
/// let comp = StatefulComputation::<Config, Config>::get().map(|c: Config| c.value);
/// let (value, _state) = comp.run(Config { value: 42 });
/// assert_eq!(value, 42);
/// ```
pub fn state_gets<S: 'static, A: 'static, F: Fn(&S) -> A + 'static>(_f: F) -> Eff<StateRow, A> {
    Eff::lazy(|| crate::cold_panic!("state_gets requires State handler"))
}

// =============================================================================
// Concrete State Implementation
// =============================================================================

/// A concrete stateful computation that can be run.
///
/// This uses an enum representation to avoid heap allocation for common
/// operations like pure, get, put, and modify.
///
/// # Variant type invariants
///
/// The `Get`, `Put`, and `Modify` variants carry implicit relationships
/// between the enum's type parameters and the variant payload:
///
/// * `Get` implies `A == S` (the returned value is a clone of the state).
/// * `Put(_)` implies `A == ()`.
/// * `Modify(_)` implies `A == ()`.
///
/// The safe, specialized constructors (`StatefulComputation::<S, S>::get()`,
/// `StatefulComputation::<S, ()>::put()`, and `StatefulComputation::<S, ()>::modify()`)
/// only build these variants with the correct type relationship.
///
/// However, because the variants are `pub`, external callers can violate the
/// invariant by hand (e.g. `StatefulComputation::<i32, ()>::Get`). Runtime
/// code paths that interpret these variants therefore guard every access with
/// a `TypeId` assertion (panicking on mismatch), and then convert the payload
/// to `A` through a safe `Option<…> as &mut dyn Any` downcast — no `unsafe`
/// transmute is involved; the conversion is heap-allocation-free but does go
/// through dynamic-typing machinery.
#[must_use = "computations do nothing unless run"]
#[repr(u8)]
pub enum StatefulComputation<S, A> {
    /// Pure value - no state change, no allocation.
    Pure(A),
    /// Get the current state - no allocation (only valid when A = S).
    Get,
    /// Put a new state value - no allocation (only valid when A = ()).
    Put(S),
    /// Modify state with a boxed function (only valid when A = ()).
    Modify(Box<dyn FnOnce(S) -> S>),
    /// Boxed computation (for complex chains).
    Boxed(Box<dyn FnOnce(S) -> (A, S)>),
}

impl<S: Clone + 'static, A: 'static> StatefulComputation<S, A> {
    /// Create a new stateful computation from a function.
    #[inline(always)]
    pub fn new<F: FnOnce(S) -> (A, S) + 'static>(f: F) -> Self {
        StatefulComputation::Boxed(Box::new(f))
    }

    /// Pure value in state context - NO HEAP ALLOCATION.
    #[inline(always)]
    pub fn pure(value: A) -> Self {
        StatefulComputation::Pure(value)
    }

    /// Map over the result.
    #[inline(always)]
    pub fn map<B: 'static, F: FnOnce(A) -> B + 'static>(self, f: F) -> StatefulComputation<S, B> {
        match self {
            StatefulComputation::Pure(a) => StatefulComputation::Pure(f(a)),
            _ => StatefulComputation::Boxed(Box::new(move |s| {
                let (a, s2) = self.run_internal(s);
                (f(a), s2)
            })),
        }
    }

    /// Chain two stateful computations.
    #[inline(always)]
    pub fn and_then<B: 'static, F: FnOnce(A) -> StatefulComputation<S, B> + 'static>(
        self,
        f: F,
    ) -> StatefulComputation<S, B> {
        StatefulComputation::Boxed(Box::new(move |s| {
            let (a, s2) = self.run_internal(s);
            f(a).run_internal(s2)
        }))
    }

    /// Internal run that handles all variants (used by `map/and_then`).
    #[inline]
    fn run_internal(self, state: S) -> (A, S) {
        match self {
            StatefulComputation::Pure(a) => (a, state),
            StatefulComputation::Boxed(f) => f(state),
            // For Get/Put/Modify, the variant invariant (see enum doc) fixes the
            // relationship between `A` and the variant payload type. We verify
            // it with a `TypeId` assertion, then move the value into `A` via a
            // safe `Option<…> as &mut dyn Any` downcast + take — entirely safe
            // code, no transmute; stack-only, no heap allocation.
            StatefulComputation::Get => {
                assert!(
                    TypeId::of::<A>() == TypeId::of::<S>(),
                    "StatefulComputation::Get type mismatch: A must be S"
                );
                let mut opt: Option<S> = Some(state.clone());
                let any_opt = &mut opt as &mut dyn core::any::Any;
                let downcast: &mut Option<A> = any_opt.downcast_mut::<Option<A>>().unwrap();
                let a: A = downcast.take().unwrap();
                (a, state)
            }
            StatefulComputation::Put(new_state) => {
                assert!(
                    TypeId::of::<A>() == TypeId::of::<()>(),
                    "StatefulComputation::Put type mismatch: A must be ()"
                );
                let mut opt: Option<()> = Some(());
                let any_opt = &mut opt as &mut dyn core::any::Any;
                let downcast: &mut Option<A> = any_opt.downcast_mut::<Option<A>>().unwrap();
                let unit: A = downcast.take().unwrap();
                (unit, new_state)
            }
            StatefulComputation::Modify(f) => {
                assert!(
                    TypeId::of::<A>() == TypeId::of::<()>(),
                    "StatefulComputation::Modify type mismatch: A must be ()"
                );
                let mut opt: Option<()> = Some(());
                let any_opt = &mut opt as &mut dyn core::any::Any;
                let downcast: &mut Option<A> = any_opt.downcast_mut::<Option<A>>().unwrap();
                let unit: A = downcast.take().unwrap();
                (unit, f(state))
            }
        }
    }
}

/// Specialized implementation for Get operation (A = S).
impl<S: Clone + 'static> StatefulComputation<S, S> {
    /// Get the current state - NO HEAP ALLOCATION.
    #[inline(always)]
    pub fn get() -> Self {
        StatefulComputation::Get
    }

    /// Run Get computation (specialized for A = S).
    ///
    /// # Panics
    ///
    /// Panics on a `Put` or `Modify` variant: those carry a result type of
    /// `()`, which cannot be an `S` here. The typed constructors never
    /// produce such a value at `A = S`, so this can only fire on a
    /// hand-constructed variant.
    #[inline(always)]
    pub fn run_get(self, initial: S) -> (S, S) {
        match self {
            StatefulComputation::Pure(a) => (a, initial),
            StatefulComputation::Get => (initial.clone(), initial),
            StatefulComputation::Boxed(f) => f(initial),
            StatefulComputation::Put(_) => {
                panic!("StatefulComputation::Put encountered in run_get where A must be S")
            }
            StatefulComputation::Modify(_) => {
                panic!("StatefulComputation::Modify encountered in run_get where A must be S")
            }
        }
    }
}

/// Specialized implementation for Put/Modify operations (A = ()).
impl<S: 'static> StatefulComputation<S, ()> {
    /// Set the state - NO HEAP ALLOCATION.
    #[inline(always)]
    pub fn put(value: S) -> Self {
        StatefulComputation::Put(value)
    }

    /// Modify the state with a function.
    #[inline(always)]
    pub fn modify<F: FnOnce(S) -> S + 'static>(f: F) -> Self {
        StatefulComputation::Modify(Box::new(f))
    }

    /// Run Put/Modify computation (specialized for A = ()).
    ///
    /// # Panics
    ///
    /// Panics on a `Get` variant: its result type is `S`, not the `()`
    /// this specialization returns. The typed constructors never produce
    /// a `Get` at `A = ()`, so this can only fire on a hand-constructed
    /// variant.
    #[inline(always)]
    pub fn run_unit(self, initial: S) -> ((), S) {
        match self {
            StatefulComputation::Pure(()) => ((), initial),
            StatefulComputation::Put(new_state) => ((), new_state),
            StatefulComputation::Modify(f) => ((), f(initial)),
            StatefulComputation::Boxed(f) => f(initial),
            StatefulComputation::Get => {
                panic!("StatefulComputation::Get encountered in run_unit where A must be ()")
            }
        }
    }
}

/// Universalis run implementation.
impl<S: Clone + 'static, A: 'static> StatefulComputation<S, A> {
    /// Run the computation with initial state.
    ///
    /// Note: For Get operations (A = S), use `run_get()` for zero-allocation.
    /// For Put/Modify operations (A = ()), use `run_unit()` for zero-allocation.
    ///
    /// # Panics
    ///
    /// Panics if the variant's result type disagrees with `A`: `Get`
    /// requires `A = S`, while `Put`/`Modify` require `A = ()` (checked at
    /// runtime via `TypeId`). Computations built through the typed
    /// constructors always satisfy this, so it can only fire on a
    /// hand-constructed variant at a mismatched `A`.
    #[inline]
    pub fn run(self, initial: S) -> (A, S) {
        match self {
            StatefulComputation::Pure(a) => (a, initial),
            StatefulComputation::Boxed(f) => f(initial),
            // Handle Get/Put/Modify by boxing (fallback for Universalis case)
            StatefulComputation::Get => {
                assert!(
                    TypeId::of::<A>() == TypeId::of::<S>(),
                    "StatefulComputation::Get type mismatch: A must be S"
                );
                let result = initial.clone();
                let mut opt: Option<S> = Some(result);
                let any_opt = &mut opt as &mut dyn core::any::Any;
                let downcast: &mut Option<A> = any_opt.downcast_mut::<Option<A>>().unwrap();
                let a: A = downcast.take().unwrap();
                (a, initial)
            }
            StatefulComputation::Put(new_state) => {
                assert!(
                    TypeId::of::<A>() == TypeId::of::<()>(),
                    "StatefulComputation::Put type mismatch: A must be ()"
                );
                let mut opt: Option<()> = Some(());
                let any_opt = &mut opt as &mut dyn core::any::Any;
                let downcast: &mut Option<A> = any_opt.downcast_mut::<Option<A>>().unwrap();
                let unit: A = downcast.take().unwrap();
                (unit, new_state)
            }
            StatefulComputation::Modify(f) => {
                assert!(
                    TypeId::of::<A>() == TypeId::of::<()>(),
                    "StatefulComputation::Modify type mismatch: A must be ()"
                );
                let mut opt: Option<()> = Some(());
                let any_opt = &mut opt as &mut dyn core::any::Any;
                let downcast: &mut Option<A> = any_opt.downcast_mut::<Option<A>>().unwrap();
                let unit: A = downcast.take().unwrap();
                (unit, f(initial))
            }
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stateful_pure() {
        let comp = StatefulComputation::<i32, i32>::pure(42);
        let (result, state) = comp.run(0);
        assert_eq!(result, 42);
        assert_eq!(state, 0);
    }

    #[test]
    fn test_stateful_get() {
        let comp = StatefulComputation::<i32, i32>::get();
        let (result, state) = comp.run(42);
        assert_eq!(result, 42);
        assert_eq!(state, 42);
    }

    #[test]
    fn test_stateful_put() {
        let comp = StatefulComputation::<i32, ()>::put(42);
        let ((), state) = comp.run(0);
        assert_eq!(state, 42);
    }

    #[test]
    fn test_stateful_modify() {
        let comp = StatefulComputation::<i32, ()>::modify(|x| x + 10);
        let ((), state) = comp.run(32);
        assert_eq!(state, 42);
    }

    #[test]
    fn test_stateful_map() {
        let comp = StatefulComputation::<i32, i32>::get().map(|x| x * 2);
        let (result, state) = comp.run(21);
        assert_eq!(result, 42);
        assert_eq!(state, 21);
    }

    #[test]
    fn test_stateful_and_then() {
        let comp = StatefulComputation::<i32, i32>::get()
            .and_then(|x| StatefulComputation::<i32, ()>::put(x + 10).map(move |()| x));
        let (result, state) = comp.run(32);
        assert_eq!(result, 32);
        assert_eq!(state, 42);
    }

    #[test]
    fn test_stateful_chain() {
        let comp = StatefulComputation::<i32, ()>::modify(|x| x + 10)
            .and_then(|()| StatefulComputation::<i32, i32>::get())
            .map(|x| x * 2);
        let (result, state) = comp.run(11);
        assert_eq!(result, 42);
        assert_eq!(state, 21);
    }
}
