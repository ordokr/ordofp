//! Algebraic Effects - Full algebraic effect handler infrastructure
//!
//! > *"Tractator Algebraicus"*
//! > — Algebraic handler. (Neo-Latin)
//!
//! This module provides full algebraic effect handlers inspired by OCaml 5.0
//! and Koka. Algebraic effects allow expressing side effects as operations
//! that can be interpreted by handlers, enabling modular and composable
//! effect handling.
//!
//! # Design
//!
//! Algebraic effects decompose effectful programs into:
//! - **Operations**: Descriptions of effects to perform
//! - **Handlers**: Interpretations of those effects
//! - **Continuations**: The rest of the computation after an effect
//!
//! # Inspired By
//!
//! - OCaml 5.0's effect handlers with one-shot continuations
//! - Koka's row-polymorphic effect system
//! - Eff programming language
//!
//! # Scholastic Naming
//!
//! | English | Latin | Etymology |
//! |---------|-------|-----------|
//! | Algebraic | Algebraicus | From Arabic *al-jabr* via Latin |
//! | Operation | Operatio | *operatio* = work, operation |
//! | Perform | Perficere | *perficere* = to accomplish |
//! | Handle | Tractare | *tractare* = to handle, manage |
//! | Resume | Resumere | *resumere* = to take back |
//!
//! # Example
//!
//! ```rust
//! use ordofp_core::effects::algebraic::{EffectusAlgebraicus, Operatio};
//!
//! // Define an effect
//! enum ConsoleOp {
//!     Print(String),
//!     ReadLine,
//! }
//!
//! impl EffectusAlgebraicus for ConsoleOp {
//!     type Result = String;
//! }
//!
//! // Wrap it in an operation to be performed
//! let op = Operatio::new(ConsoleOp::Print("Hello!".to_string()));
//! match op.effect() {
//!     ConsoleOp::Print(msg) => assert_eq!(msg, "Hello!"),
//!     ConsoleOp::ReadLine => panic!("unexpected"),
//! }
//! ```

use super::continuation_v2::ContinuatioSemel;
use core::marker::PhantomData;

/// Marker trait for algebraic effects.
///
/// An algebraic effect defines operations that can be performed and
/// the type of result they produce when handled.
///
/// # Example
///
/// ```rust
/// use ordofp_core::effects::algebraic::EffectusAlgebraicus;
///
/// enum StateOp<S> {
///     Get,
///     Put(S),
/// }
///
/// impl<S: Clone + Send + Sync + 'static> EffectusAlgebraicus for StateOp<S> {
///     type Result = S;
/// }
/// ```
pub trait EffectusAlgebraicus: Send + Sync + 'static {
    /// The type of value produced when this effect is handled.
    type Result: Send + 'static;
}

/// An effect operation that can be performed.
///
/// `Operatio<E>` wraps an algebraic effect to be performed during
/// computation. The operation is a description - it doesn't execute
/// until handled.
///
/// > *"Operatio est actus"* — An operation is an act.
#[derive(Debug)]
pub struct Operatio<E: EffectusAlgebraicus> {
    effect: E,
}

impl<E: EffectusAlgebraicus> Operatio<E> {
    /// Create a new operation.
    #[inline]
    pub fn new(effect: E) -> Self {
        Operatio { effect }
    }

    /// Get a reference to the effect.
    #[inline]
    pub fn effect(&self) -> &E {
        &self.effect
    }

    /// Consume and return the effect.
    #[inline]
    pub fn into_effect(self) -> E {
        self.effect
    }
}

/// The result of running an effectful computation.
///
/// Either the computation completed with a value, or it suspended
/// to perform an effect.
pub enum ComputatioStatus<E: EffectusAlgebraicus, A> {
    /// Computation completed with a final value.
    Completus(A),

    /// Computation suspended to perform an effect.
    Suspensus {
        /// The effect operation being performed.
        operatio: E,
        /// Continuation to resume after handling the effect.
        continuatio: ContinuatioSemel<E::Result, ComputatioStatus<E, A>>,
    },
}

impl<E: EffectusAlgebraicus, A> ComputatioStatus<E, A> {
    /// Create a completed status.
    #[inline]
    pub fn complete(value: A) -> Self {
        ComputatioStatus::Completus(value)
    }

    /// Check if the computation is complete.
    #[inline]
    pub fn is_complete(&self) -> bool {
        matches!(self, ComputatioStatus::Completus(_))
    }

    /// Check if the computation is suspended.
    #[inline]
    pub fn is_suspended(&self) -> bool {
        matches!(self, ComputatioStatus::Suspensus { .. })
    }

    /// Extract the completed value, panicking if suspended.
    ///
    /// # Panics
    ///
    /// Panics if the computation is [`ComputatioStatus::Suspensus`], i.e.
    /// still waiting on an unhandled effect.
    #[inline]
    pub fn unwrap(self) -> A {
        match self {
            ComputatioStatus::Completus(a) => a,
            ComputatioStatus::Suspensus { .. } => panic!("Computation is suspended"),
        }
    }

    /// Extract the completed value, panicking with the given message if suspended.
    ///
    /// # Panics
    ///
    /// Panics with the supplied `msg` if the computation is
    /// [`ComputatioStatus::Suspensus`].
    #[inline]
    pub fn expect(self, msg: &str) -> A {
        match self {
            ComputatioStatus::Completus(a) => a,
            ComputatioStatus::Suspensus { .. } => panic!("{}", msg),
        }
    }
}

// Deliberately no wrapper-handler combinators: only types with a
// `TractatorAlgebraicus` impl can actually handle effects.

/// An algebraic effect handler.
///
/// `TractatorAlgebraicus<E>` interprets effects of type `E`, providing
/// implementations for each operation.
///
/// # Handler Semantics
///
/// A handler can:
/// - **Resume**: Continue the computation with a result value
/// - **Abort**: Stop the computation and return a different value
/// - **Transform**: Modify the continuation's result
///
/// # Example
///
/// ```rust
/// use ordofp_core::effects::algebraic::{EffectusAlgebraicus, TractatorAlgebraicus};
/// use ordofp_core::effects::continuation_v2::ContinuatioSemel;
///
/// enum StateOp<S> {
///     Get,
///     Put(S),
/// }
///
/// impl<S: Clone + Send + Sync + 'static> EffectusAlgebraicus for StateOp<S> {
///     type Result = S;
/// }
///
/// struct StateHandler<S> {
///     state: S,
/// }
///
/// impl<S: Clone + Send + Sync + 'static> TractatorAlgebraicus<StateOp<S>> for StateHandler<S> {
///     type Output = S;
///
///     fn handle_return(&self, value: Self::Output) -> Self::Output {
///         value
///     }
///
///     fn handle_operation(
///         &mut self,
///         op: StateOp<S>,
///         cont: ContinuatioSemel<S, Self::Output>,
///     ) -> Self::Output {
///         match op {
///             StateOp::Get => cont.resume(self.state.clone()),
///             StateOp::Put(s) => {
///                 self.state = s.clone();
///                 cont.resume(s)
///             }
///         }
///     }
/// }
///
/// let mut handler = StateHandler { state: 10 };
/// let result = handler.handle_operation(StateOp::Get, ContinuatioSemel::new(|x| x * 2));
/// assert_eq!(result, 20);
/// ```
pub trait TractatorAlgebraicus<E: EffectusAlgebraicus>: Sized {
    /// The final output type of handled computations.
    type Output;

    /// Handle a pure return value.
    ///
    /// This is called when the computation completes without performing effects.
    fn handle_return(&self, value: Self::Output) -> Self::Output;

    /// Handle an effect operation with access to a continuation.
    ///
    /// This is called when the computation performs an effect.
    /// The handler must decide whether to resume, abort, or transform.
    fn handle_operation(
        &mut self,
        operation: E,
        continuation: ContinuatioSemel<E::Result, Self::Output>,
    ) -> Self::Output;
}

/// Run a computation with an algebraic effect handler.
///
/// This function interprets all effects in a computation using the
/// provided handler.
///
/// # Panics
///
/// Panics with "Nested effects require deep handlers" when the continuation
/// of a handled operation suspends again — this is a shallow (one-shot)
/// runner.
///
/// # Example
///
/// ```rust
/// use ordofp_core::effects::algebraic::{
///     EffectusAlgebraicus, TractatorAlgebraicus, pure_effect, run_with_handler,
/// };
/// use ordofp_core::effects::continuation_v2::ContinuatioSemel;
///
/// enum CounterOp {
///     Get,
/// }
///
/// impl EffectusAlgebraicus for CounterOp {
///     type Result = i32;
/// }
///
/// struct CounterHandler {
///     value: i32,
/// }
///
/// impl TractatorAlgebraicus<CounterOp> for CounterHandler {
///     type Output = i32;
///
///     fn handle_return(&self, value: i32) -> i32 {
///         value
///     }
///
///     fn handle_operation(
///         &mut self,
///         _op: CounterOp,
///         cont: ContinuatioSemel<i32, i32>,
///     ) -> i32 {
///         cont.resume(self.value)
///     }
/// }
///
/// let mut handler = CounterHandler { value: 42 };
/// let result = run_with_handler(&mut handler, || pure_effect(42));
/// assert_eq!(result, 42);
/// ```
pub fn run_with_handler<E, H, A, F>(handler: &mut H, computation: F) -> A
where
    E: EffectusAlgebraicus,
    H: TractatorAlgebraicus<E, Output = A>,
    F: FnOnce() -> ComputatioStatus<E, A>,
    A: 'static,
{
    match computation() {
        ComputatioStatus::Completus(a) => handler.handle_return(a),
        ComputatioStatus::Suspensus {
            operatio,
            continuatio,
        } => {
            // The continuation returns ComputatioStatus<E, A>, but we need to
            // recursively handle it. For now, we pass a wrapped continuation
            // that unwraps the status.
            let wrapped_cont = continuatio.map(|status: ComputatioStatus<E, A>| {
                match status {
                    ComputatioStatus::Completus(a) => a,
                    ComputatioStatus::Suspensus { .. } => {
                        // In a full implementation, we would recursively handle.
                        // For now, panic if there are nested effects.
                        panic!("Nested effects require deep handlers")
                    }
                }
            });
            handler.handle_operation(operatio, wrapped_cont)
        }
    }
}

/// Create a pure (effect-free) computation.
///
/// # Example
///
/// ```rust
/// use ordofp_core::effects::algebraic::{ComputatioStatus, EffectusAlgebraicus, pure_effect};
///
/// enum MyEffect {}
///
/// impl EffectusAlgebraicus for MyEffect {
///     type Result = ();
/// }
///
/// let comp: ComputatioStatus<MyEffect, i32> = pure_effect(42);
/// assert!(comp.is_complete());
/// ```
pub fn pure_effect<E: EffectusAlgebraicus, A>(value: A) -> ComputatioStatus<E, A> {
    ComputatioStatus::Completus(value)
}

/// A handler that simply returns values unchanged.
///
/// This is useful as a base case or for testing.
#[derive(Debug, Clone, Copy, Default)]
pub struct IdentityHandler;

impl<E: EffectusAlgebraicus<Result = A>, A: Clone + 'static> TractatorAlgebraicus<E>
    for IdentityHandler
{
    type Output = A;

    #[inline]
    fn handle_return(&self, value: A) -> A {
        value
    }

    fn handle_operation(&mut self, _operation: E, _continuation: ContinuatioSemel<A, A>) -> A {
        panic!("IdentityHandler cannot handle effects")
    }
}

/// Marker trait for effects that can be handled by a specific handler.
///
/// This trait establishes the relationship between effects and their handlers.
pub trait HandledBy<H>: EffectusAlgebraicus + Sized
where
    H: TractatorAlgebraicus<Self>,
{
}

impl<E, H> HandledBy<H> for E
where
    E: EffectusAlgebraicus + Sized,
    H: TractatorAlgebraicus<E>,
{
}

/// A handler built from closures.
pub struct ClosureHandler<E, A, RetFn, OpFn>
where
    E: EffectusAlgebraicus,
    RetFn: Fn(A) -> A,
    OpFn: FnMut(E, ContinuatioSemel<E::Result, A>) -> A,
{
    return_fn: RetFn,
    operation_fn: OpFn,
    _phantom: PhantomData<(E, A)>,
}

impl<E, A, RetFn, OpFn> ClosureHandler<E, A, RetFn, OpFn>
where
    E: EffectusAlgebraicus,
    RetFn: Fn(A) -> A,
    OpFn: FnMut(E, ContinuatioSemel<E::Result, A>) -> A,
{
    /// Create a new closure-based handler.
    pub fn new(return_fn: RetFn, operation_fn: OpFn) -> Self {
        ClosureHandler {
            return_fn,
            operation_fn,
            _phantom: PhantomData,
        }
    }
}

impl<E, A, RetFn, OpFn> TractatorAlgebraicus<E> for ClosureHandler<E, A, RetFn, OpFn>
where
    E: EffectusAlgebraicus,
    A: 'static,
    RetFn: Fn(A) -> A,
    OpFn: FnMut(E, ContinuatioSemel<E::Result, A>) -> A,
{
    type Output = A;

    #[inline]
    fn handle_return(&self, value: A) -> A {
        (self.return_fn)(value)
    }

    #[inline]
    fn handle_operation(
        &mut self,
        operation: E,
        continuation: ContinuatioSemel<E::Result, A>,
    ) -> A {
        (self.operation_fn)(operation, continuation)
    }
}

/// Create a handler from closures.
///
/// # Example
///
/// ```rust
/// use ordofp_core::effects::algebraic::{
///     ComputatioStatus, EffectusAlgebraicus, make_handler, run_with_handler,
/// };
/// use ordofp_core::effects::continuation_v2::ContinuatioSemel;
///
/// enum PingOp {
///     Ping,
/// }
///
/// impl EffectusAlgebraicus for PingOp {
///     type Result = i32;
/// }
///
/// let mut handler = make_handler(
///     |x: i32| x, // return handler
///     |_op: PingOp, cont: ContinuatioSemel<i32, i32>| {
///         // operation handler: always resume with a fixed value
///         cont.resume(7)
///     },
/// );
///
/// let result = run_with_handler(&mut handler, || ComputatioStatus::Suspensus {
///     operatio: PingOp::Ping,
///     continuatio: ContinuatioSemel::new(|x| ComputatioStatus::Completus(x)),
/// });
/// assert_eq!(result, 7);
/// ```
pub fn make_handler<E, A, RetFn, OpFn>(
    return_fn: RetFn,
    operation_fn: OpFn,
) -> ClosureHandler<E, A, RetFn, OpFn>
where
    E: EffectusAlgebraicus,
    RetFn: Fn(A) -> A,
    OpFn: FnMut(E, ContinuatioSemel<E::Result, A>) -> A,
{
    ClosureHandler::new(return_fn, operation_fn)
}

// =============================================================================
// Handler Composition Utilities
// =============================================================================

/// A handler that always resumes with the default result value.
///
/// `DefaultHandler<E, A>` wraps a closure handler that resumes continuations
/// with the default value for the effect's result type.
pub struct DefaultHandler<E: EffectusAlgebraicus, A> {
    _effect: PhantomData<E>,
    _output: PhantomData<A>,
}

impl<E: EffectusAlgebraicus, A> DefaultHandler<E, A>
where
    E::Result: Default,
{
    /// Create a new default handler.
    pub fn new() -> Self {
        DefaultHandler {
            _effect: PhantomData,
            _output: PhantomData,
        }
    }
}

impl<E: EffectusAlgebraicus, A> Default for DefaultHandler<E, A>
where
    E::Result: Default,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<E: EffectusAlgebraicus, A: 'static> TractatorAlgebraicus<E> for DefaultHandler<E, A>
where
    E::Result: Default,
{
    type Output = A;

    #[inline]
    fn handle_return(&self, value: A) -> A {
        value
    }

    #[inline]
    fn handle_operation(
        &mut self,
        _operation: E,
        continuation: ContinuatioSemel<E::Result, A>,
    ) -> A {
        continuation.resume(E::Result::default())
    }
}

/// Create a handler that always resumes with a default value.
///
/// # Example
///
/// ```rust
/// use ordofp_core::effects::algebraic::{EffectusAlgebraicus, default_handler};
///
/// enum MyEffect {}
///
/// impl EffectusAlgebraicus for MyEffect {
///     type Result = i32;
/// }
///
/// let _handler = default_handler::<MyEffect, i32>();
/// ```
pub fn default_handler<E, A>() -> DefaultHandler<E, A>
where
    E: EffectusAlgebraicus,
    A: 'static,
    E::Result: Default,
{
    DefaultHandler::new()
}

/// A builder for constructing complex handler configurations.
///
/// `HandlerConfig` provides a fluent API for building handlers
/// with various options.
pub struct HandlerConfig<E: EffectusAlgebraicus> {
    _effect: PhantomData<E>,
}

impl<E: EffectusAlgebraicus> Default for HandlerConfig<E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<E: EffectusAlgebraicus> HandlerConfig<E> {
    /// Create a new handler configuration.
    pub fn new() -> Self {
        HandlerConfig {
            _effect: PhantomData,
        }
    }

    /// Build a handler from closures.
    pub fn with_closures<A, RetFn, OpFn>(
        self,
        return_fn: RetFn,
        operation_fn: OpFn,
    ) -> ClosureHandler<E, A, RetFn, OpFn>
    where
        RetFn: Fn(A) -> A,
        OpFn: FnMut(E, ContinuatioSemel<E::Result, A>) -> A,
    {
        ClosureHandler::new(return_fn, operation_fn)
    }

    /// Build an identity handler.
    pub fn identity(self) -> IdentityHandler {
        IdentityHandler
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Simple test effect
    #[derive(Debug, Clone)]
    enum TestOp {
        GetValue,
        SetValue(i32),
    }

    impl EffectusAlgebraicus for TestOp {
        type Result = i32;
    }

    // State handler
    struct TestHandler {
        state: i32,
    }

    impl TestHandler {
        fn new(initial: i32) -> Self {
            TestHandler { state: initial }
        }
    }

    impl TractatorAlgebraicus<TestOp> for TestHandler {
        type Output = i32;

        fn handle_return(&self, value: i32) -> i32 {
            value
        }

        fn handle_operation(&mut self, op: TestOp, cont: ContinuatioSemel<i32, i32>) -> i32 {
            match op {
                TestOp::GetValue => cont.resume(self.state),
                TestOp::SetValue(v) => {
                    self.state = v;
                    cont.resume(v)
                }
            }
        }
    }

    #[test]
    fn test_pure_effect() {
        let status: ComputatioStatus<TestOp, i32> = pure_effect(42);
        assert!(status.is_complete());
        assert_eq!(
            status.expect(
                "pure_effect should produce a complete ComputatioStatus with the wrapped value"
            ),
            42
        );
    }

    #[test]
    fn test_computation_status() {
        let complete: ComputatioStatus<TestOp, i32> = ComputatioStatus::complete(42);
        assert!(complete.is_complete());
        assert!(!complete.is_suspended());
    }

    #[test]
    fn test_handler_return() {
        let mut handler = TestHandler::new(0);
        let result = run_with_handler(&mut handler, || pure_effect(42));
        assert_eq!(result, 42);
    }

    #[test]
    fn test_handler_operation() {
        let mut handler = TestHandler::new(10);

        // Create a suspended computation that gets the value
        let result = run_with_handler(&mut handler, || ComputatioStatus::Suspensus {
            operatio: TestOp::GetValue,
            continuatio: ContinuatioSemel::new(|x| ComputatioStatus::Completus(x * 2)),
        });

        assert_eq!(result, 20); // 10 * 2
    }

    #[test]
    fn test_handler_set_operation() {
        let mut handler = TestHandler::new(0);

        let result = run_with_handler(&mut handler, || ComputatioStatus::Suspensus {
            operatio: TestOp::SetValue(42),
            continuatio: ContinuatioSemel::new(ComputatioStatus::Completus),
        });

        assert_eq!(result, 42);
        assert_eq!(handler.state, 42);
    }

    #[test]
    fn test_operatio() {
        let op = Operatio::new(TestOp::GetValue);
        match op.effect() {
            TestOp::GetValue => {}
            _ => panic!("Wrong operation"),
        }
    }

    #[test]
    fn test_closure_handler() {
        let mut handler = make_handler(
            |x: i32| x,
            |op: TestOp, cont: ContinuatioSemel<i32, i32>| match op {
                TestOp::GetValue => cont.resume(100),
                TestOp::SetValue(v) => cont.resume(v),
            },
        );

        let result = run_with_handler(&mut handler, || ComputatioStatus::Suspensus {
            operatio: TestOp::GetValue,
            continuatio: ContinuatioSemel::new(ComputatioStatus::Completus),
        });

        assert_eq!(result, 100);
    }

    #[test]
    fn test_identity_handler() {
        let mut handler = IdentityHandler;
        let result: i32 =
            run_with_handler::<TestOp, _, _, _>(&mut handler, || pure_effect::<TestOp, _>(42));
        assert_eq!(result, 42);
    }

    #[test]
    fn test_handler_config() {
        let config: HandlerConfig<TestOp> = HandlerConfig::new();
        let _identity = config.identity();

        let _config2: HandlerConfig<TestOp> = HandlerConfig::default();
    }

    #[test]
    fn test_handler_config_with_closures() {
        let config: HandlerConfig<TestOp> = HandlerConfig::new();
        let mut handler = config.with_closures(
            |x: i32| x,
            |op: TestOp, cont: ContinuatioSemel<i32, i32>| match op {
                TestOp::GetValue => cont.resume(50),
                TestOp::SetValue(v) => cont.resume(v),
            },
        );

        let result = run_with_handler(&mut handler, || ComputatioStatus::Suspensus {
            operatio: TestOp::GetValue,
            continuatio: ContinuatioSemel::new(ComputatioStatus::Completus),
        });

        assert_eq!(result, 50);
    }
}
