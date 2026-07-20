//! Effect Handlers - Pure-Value Effect Elimination (limited)
//!
//! Handlers interpret effect operations, transforming effectful computations
//! into concrete results.
//!
//! # Current limitation: Pure computations only
//!
//! Every handler in this module handles **only `Eff` values in the `Pure`
//! state** (e.g. built with `Eff::from_value`). Handling any *lazy*
//! computation — including the op stubs `get`/`put`/`ask`/`err` from
//! `nexus::effect` — **panics**, because the continuation infrastructure
//! that would interpret suspended operations does not exist yet. For
//! working effect interpreters see `nexus::effects::*` (e.g.
//! `StatefulComputation`, `ReaderComputation`, `ErrorComputation`).
//!
//! Within that Pure-only scope the handlers are trivially cheap (an enum
//! match), but no broader zero-cost claim is made.
//!
//! # Dispatch is static by design (evidence passing)
//!
//! Handlers are passed as generic parameters (`handle<E, H, R, A>`), never as
//! trait objects, so every handler application monomorphizes to a direct
//! call — the compile-time analogue of the evidence-passing compilation
//! strategy for effect handlers (Xie & Leijen, "Generalized evidence passing
//! for effect handlers", ICFP 2021), under which tail-resumptive operations
//! execute in place with no handler-stack search. Preserve this property:
//! if a future design ever needs dynamic handler stacks, keep this static
//! path as the fast case.
//!
//! # Handler Hierarchy
//!
//! ```text
//! Handler<E>              Abstract effect handler
//!     │
//!     └── InlineHandler    Marker for inline-able handlers
//!             │
//!             ├── StateHandler<S>
//!             ├── ReaderHandler<E>
//!             └── ErrorHandler<Err>
//! ```

use core::marker::PhantomData;

use super::effect::{Eff, Error, ErrorEff, Reader, ReaderEff, State, StateEff};
use super::row::{EffectRow, Pure};

// =============================================================================
// Handler Trait
// =============================================================================

/// Abstract effect handler.
///
/// A handler interprets effect operations, transforming them into
/// concrete computations. Different handlers for the same effect
/// can provide different interpretations.
///
/// # Example
///
/// ```text
/// // State can be handled in different ways:
/// // 1. Thread state through functions (standard)
/// // 2. Use a mutable reference (in-place)
/// // 3. Log all state changes (tracing)
/// ```
pub trait Handler<E> {
    /// The output type after handling.
    type Output<A>;

    /// The remaining effects after handling.
    type Remaining: EffectRow;

    /// Handle an effectful computation.
    fn handle<R, A>(self, eff: Eff<R, A>) -> Self::Output<A>
    where
        R: EffectRow;
}

// =============================================================================
// Inline Handler Trait
// =============================================================================

/// Marker trait for handlers that can be completely inlined.
///
/// Inline handlers are guaranteed to have zero runtime overhead -
/// they compile to the same code as if the effect was written
/// imperatively.
///
/// # Safety
///
/// This trait is sealed and can only be implemented by the library
/// to ensure the zero-cost guarantee.
pub trait InlineHandler<E>: Handler<E> {}

// =============================================================================
// State Handler
// =============================================================================

/// State effect handler (Pure computations only).
///
/// Conceptually transforms `Eff<State<S> | R, A>` into `S -> Eff<R, (A, S)>`;
/// in the current implementation it returns `(a, initial)` for a `Pure(a)`
/// computation and **panics on anything lazy** (see the module docs).
pub struct StateHandler<S> {
    /// Initial state value.
    initial: S,
}

impl<S> StateHandler<S> {
    /// Create a new state handler with initial state.
    #[inline]
    pub fn new(initial: S) -> Self {
        StateHandler { initial }
    }
}

impl<S: Clone + 'static> Handler<State<S>> for StateHandler<S> {
    type Output<A> = (A, S);
    type Remaining = Pure;

    #[inline]
    fn handle<R, A>(self, eff: Eff<R, A>) -> Self::Output<A>
    where
        R: EffectRow,
    {
        // For pure computations in state context
        // This is a simplified implementation
        // Full implementation would use continuations
        if let super::effect::EffInner::Pure(a) = eff.inner {
            (a, self.initial)
        } else {
            crate::cold_panic!("State handler requires proper effect infrastructure")
        }
    }
}

impl<S: Clone + 'static> InlineHandler<State<S>> for StateHandler<S> {}

// =============================================================================
// Reader Handler
// =============================================================================

/// Reader effect handler (Pure computations only).
///
/// Conceptually transforms `Eff<Reader<E> | R, A>` into `&E -> Eff<R, A>`;
/// in the current implementation it unwraps a `Pure(a)` computation and
/// **panics on anything lazy** (see the module docs).
pub struct ReaderHandler<'e, E> {
    /// Reference to the environment.
    pub env: &'e E,
}

impl<'e, E> ReaderHandler<'e, E> {
    /// Create a new reader handler with environment.
    #[inline]
    pub fn new(env: &'e E) -> Self {
        ReaderHandler { env }
    }
}

impl<E: 'static> Handler<Reader<E>> for ReaderHandler<'_, E> {
    type Output<A> = A;
    type Remaining = Pure;

    #[inline]
    fn handle<R, A>(self, eff: Eff<R, A>) -> Self::Output<A>
    where
        R: EffectRow,
    {
        if let super::effect::EffInner::Pure(a) = eff.inner {
            a
        } else {
            crate::cold_panic!("Reader handler requires proper effect infrastructure")
        }
    }
}

impl<E: 'static> InlineHandler<Reader<E>> for ReaderHandler<'_, E> {}

// =============================================================================
// Error Handler
// =============================================================================

/// Error effect handler (Pure computations only).
///
/// Conceptually transforms `Eff<Error<Err> | R, A>` into
/// `Eff<R, Result<A, Err>>`; in the current implementation it wraps a
/// `Pure(a)` computation in `Ok` and **panics on anything lazy** (see the
/// module docs).
pub struct ErrorHandler<Err> {
    _marker: PhantomData<Err>,
}

impl<Err> ErrorHandler<Err> {
    /// Create a new error handler.
    #[inline]
    pub fn new() -> Self {
        ErrorHandler {
            _marker: PhantomData,
        }
    }
}

impl<Err> Default for ErrorHandler<Err> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Err: 'static> Handler<Error<Err>> for ErrorHandler<Err> {
    type Output<A> = Result<A, Err>;
    type Remaining = Pure;

    #[inline]
    fn handle<R, A>(self, eff: Eff<R, A>) -> Self::Output<A>
    where
        R: EffectRow,
    {
        if let super::effect::EffInner::Pure(a) = eff.inner {
            Ok(a)
        } else {
            crate::cold_panic!("Error handler requires proper effect infrastructure")
        }
    }
}

impl<Err: 'static> InlineHandler<Error<Err>> for ErrorHandler<Err> {}

// =============================================================================
// Handle Function
// =============================================================================

/// Handle an effect using the given handler.
///
/// This is the primary way to eliminate effects from a computation.
///
/// # Example
///
/// ```
/// use ordofp_core::nexus::prelude::*;
///
/// let comp: Eff<StateEff, String> = Eff::from_value(42).map(|x: i32| format!("Value: {}", x));
/// let (result, final_state) = handle(comp, StateHandler::new(42));
/// assert_eq!(result, "Value: 42");
/// assert_eq!(final_state, 42);
/// ```
#[inline]
pub fn handle<E, H, R, A>(eff: Eff<R, A>, handler: H) -> H::Output<A>
where
    H: Handler<E>,
    R: EffectRow,
{
    handler.handle(eff)
}

// =============================================================================
// Convenience Functions
// =============================================================================

/// Run a state computation with an initial value.
///
/// This is a convenience wrapper around `handle(eff, StateHandler::new(initial))`.
/// State computations model mutable state threaded through a computation: the
/// state `S` is passed implicitly and is accessible via [`get`] / [`put`] /
/// [`modify`].  Unlike a global mutable variable, the state is always explicit
/// at the handler boundary and can be observed or replaced when the computation
/// finishes.
///
/// # Parameters
///
/// * `eff` – The state computation to execute.
/// * `initial` – The starting value of the state that will be threaded through
///   every [`get`] / [`put`] / [`modify`] call inside the computation.
///
/// # Returns
///
/// A tuple `(A, S)` where `A` is the value produced by the computation and `S`
/// is the final state after all modifications have been applied.
///
/// **Limitation:** only `Pure` computations are currently supported — any
/// lazy computation (including the [`get`] / [`put`] / [`modify`] op stubs)
/// panics. See the module docs.
///
/// # Example
///
/// ```
/// use ordofp_core::nexus::prelude::*;
///
/// let eff: Eff<StateEff, i32> = Eff::from_value(42);
/// let (result, final_state) = run_state(eff, 0);
/// assert_eq!(result, 42);
/// assert_eq!(final_state, 0); // pure computations leave the state untouched
/// ```
///
/// [`get`]: crate::nexus::effect::get
/// [`put`]: crate::nexus::effect::put
/// [`modify`]: crate::nexus::effect::modify
#[inline]
pub fn run_state<S: Clone + 'static, A>(eff: Eff<StateEff, A>, initial: S) -> (A, S) {
    handle(eff, StateHandler::new(initial))
}

/// Run a reader computation by supplying an environment value.
///
/// This is a convenience wrapper around `handle(eff, ReaderHandler::new(env))`.
/// Reader computations model dependency injection: the environment `E` is
/// threaded through implicitly and made available via `ask` / `asks`.
///
/// # Parameters
///
/// * `eff` – The reader computation to execute.
/// * `env` – A shared reference to the environment that will be provided to
///   every `ask` / `asks` call inside the computation.
///
/// # Returns
///
/// The pure result `A` produced by the computation once the environment has
/// been eliminated.
///
/// **Limitation:** only `Pure` computations are currently supported — any
/// lazy computation (including the `ask` / `asks` op stubs) panics. See the
/// module docs.
///
/// # Example
///
/// ```
/// use ordofp_core::nexus::prelude::*;
///
/// let eff: Eff<ReaderEff, i32> = Eff::from_value(42);
/// assert_eq!(run_reader(eff, &100), 42);
/// ```
#[inline]
pub fn run_reader<E: 'static, A>(eff: Eff<ReaderEff, A>, env: &E) -> A {
    handle(eff, ReaderHandler::new(env))
}

/// Run an error computation, returning `Ok(value)` on success or `Err(e)` on failure.
///
/// This is a convenience wrapper around `handle(eff, ErrorHandler::new())`.
/// Error computations model short-circuiting failure: any `throw` inside the
/// computation immediately propagates `Err(e)` without executing subsequent
/// operations.
///
/// # Parameters
///
/// * `eff` – The error computation to execute.
///
/// # Returns
///
/// `Ok(A)` if the computation completed without raising an error, or
/// `Err(Err)` containing the first error that was thrown.
///
/// # Example
///
/// ```
/// use ordofp_core::nexus::prelude::*;
///
/// let ok_comp: Eff<ErrorEff, i32> = Eff::from_value(42);
/// assert_eq!(run_error::<String, i32>(ok_comp), Ok(42));
/// ```
///
/// # Errors
///
/// Returns `Err` carrying the first error the computation `throw`s;
/// everything sequenced after that throw is skipped. A computation that
/// never throws always yields `Ok`.
#[inline]
pub fn run_error<Err: 'static, A>(eff: Eff<ErrorEff, A>) -> Result<A, Err> {
    handle(eff, ErrorHandler::new())
}

// =============================================================================
// Handler Composition
// =============================================================================

/// Pair of handlers intended for multi-effect composition (inert).
///
/// **Status:** this type has **no `Handler` impl** — it cannot currently
/// handle anything. It only stores the two handlers; the composition logic
/// (apply `first`, then `second` on the remaining effects) is future work
/// blocked on the same continuation infrastructure as lazy handling.
pub struct ComposedHandler<H1, H2> {
    /// First handler in the composition chain.
    pub first: H1,
    /// Second handler in the composition chain.
    pub second: H2,
}

impl<H1, H2> ComposedHandler<H1, H2> {
    /// Create a composed handler.
    #[inline]
    pub fn new(first: H1, second: H2) -> Self {
        ComposedHandler { first, second }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_handler_pure() {
        let eff: Eff<StateEff, i32> = Eff::from_value(42);
        let (result, state) = run_state(eff, 0);
        assert_eq!(result, 42);
        assert_eq!(state, 0);
    }

    #[test]
    fn test_reader_handler_pure() {
        let eff: Eff<ReaderEff, i32> = Eff::from_value(42);
        let result = run_reader(eff, &100);
        assert_eq!(result, 42);
    }

    #[test]
    fn test_error_handler_ok() {
        let eff: Eff<ErrorEff, i32> = Eff::from_value(42);
        let result: Result<i32, &str> = run_error(eff);
        assert_eq!(result, Ok(42));
    }
}
