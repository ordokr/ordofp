//! Built-in Algebraic Effects - Common effect patterns
//!
//! > *"Effectus Fundamentales"*
//! > — Fundamental effects. (Neo-Latin)
//!
//! This module provides built-in algebraic effect definitions and handlers
//! for common effect patterns: State, Reader, Writer, and Error.
//!
//! # Available Effects
//!
//! | Effect | Latin Name | Description |
//! |--------|------------|-------------|
//! | State | `StatusOp` | Mutable state access |
//! | Reader | `LectorOp` | Read-only environment |
//! | Writer | `ScriptorOp` | Output accumulation |
//! | Error | `ErrorOp` | Recoverable errors |
//!
//! # Example
//!
//! ```rust
//! use ordofp_core::effects::algebraic::{ComputatioStatus, run_with_handler};
//! use ordofp_core::effects::builtin::{StatusHandler, StatusOp};
//! use ordofp_core::effects::continuation_v2::ContinuatioSemel;
//!
//! let mut handler = StatusHandler::new(0);
//! let result = run_with_handler(&mut handler, || ComputatioStatus::Suspensus {
//!     operatio: StatusOp::Put(42),
//!     continuatio: ContinuatioSemel::new(|x| ComputatioStatus::Completus(x)),
//! });
//! assert_eq!(result, 42);
//! assert_eq!(handler.state, 42);
//! ```

use super::algebraic::{EffectusAlgebraicus, TractatorAlgebraicus};
use super::continuation_v2::ContinuatioSemel;
use alloc::vec::Vec;
use core::marker::PhantomData;

// =============================================================================
// State Effect (StatusOp)
// =============================================================================

/// State effect operations.
///
/// `StatusOp<S>` provides operations for reading and writing mutable state
/// of type `S`.
///
/// > *"Status mutabilis"* — Mutable state.
///
/// # Operations
///
/// - `Get`: Read the current state
/// - `Put(S)`: Replace the state with a new value
/// - `Modify(fn)`: Apply a function to modify the state
///
/// # Example
///
/// ```rust
/// use ordofp_core::effects::builtin::StatusOp;
///
/// // Get the current counter value
/// let get_op = StatusOp::<i32>::Get;
///
/// // Set the counter to 42
/// let put_op = StatusOp::Put(42);
/// ```
#[derive(Debug, Clone)]
pub enum StatusOp<S> {
    /// Get the current state value.
    Get,
    /// Replace the state with a new value.
    Put(S),
}

impl<S: Clone + Send + Sync + 'static> EffectusAlgebraicus for StatusOp<S> {
    type Result = S;
}

/// Handler for state effects.
///
/// `StatusHandler<S>` maintains state of type `S` and handles state operations.
///
/// # Example
///
/// ```rust
/// use ordofp_core::effects::builtin::StatusHandler;
///
/// let handler = StatusHandler::new(0);
/// assert_eq!(handler.state, 0);
/// ```
pub struct StatusHandler<S> {
    /// The current state value.
    pub state: S,
}

impl<S> StatusHandler<S> {
    /// Create a new state handler with an initial value.
    #[inline]
    pub fn new(initial: S) -> Self {
        StatusHandler { state: initial }
    }

    /// Get a reference to the current state.
    #[inline]
    pub fn get_state(&self) -> &S {
        &self.state
    }

    /// Get a mutable reference to the current state.
    #[inline]
    pub fn get_state_mut(&mut self) -> &mut S {
        &mut self.state
    }
}

impl<S: Clone + Send + Sync + 'static> TractatorAlgebraicus<StatusOp<S>> for StatusHandler<S> {
    type Output = S;

    #[inline]
    fn handle_return(&self, value: S) -> S {
        value
    }

    #[inline]
    fn handle_operation(&mut self, op: StatusOp<S>, cont: ContinuatioSemel<S, S>) -> S {
        match op {
            StatusOp::Get => cont.resume(self.state.clone()),
            StatusOp::Put(s) => {
                self.state = s.clone();
                cont.resume(s)
            }
        }
    }
}

// =============================================================================
// Reader Effect (LectorOp)
// =============================================================================

/// Reader effect operations.
///
/// `LectorOp<R>` provides read-only access to an environment of type `R`.
///
/// > *"Lector ambientis"* — Reader of environment.
///
/// # Operations
///
/// - `Ask`: Get the entire environment
/// - `Asks(fn)`: Get a projection of the environment
///
/// # Example
///
/// ```rust
/// use ordofp_core::effects::builtin::LectorOp;
///
/// #[derive(Clone)]
/// struct Config {
///     debug: bool,
/// }
///
/// // Get the entire config
/// let ask_op = LectorOp::<Config>::Ask;
/// ```
#[derive(Debug, Clone)]
pub enum LectorOp<R> {
    /// Get the environment.
    Ask,
    /// Marker for type inference.
    ///
    /// Carries `Infallible` so this variant can never actually be
    /// constructed; it exists solely so `R` is used in the enum body.
    #[doc(hidden)]
    _Phantom(PhantomData<R>, core::convert::Infallible),
}

impl<R: Clone + Send + Sync + 'static> EffectusAlgebraicus for LectorOp<R> {
    type Result = R;
}

/// Handler for reader effects.
///
/// `LectorHandler<R>` provides read-only access to an environment.
///
/// # Example
///
/// ```rust
/// use ordofp_core::effects::builtin::LectorHandler;
///
/// #[derive(Clone)]
/// struct Config {
///     debug: bool,
/// }
///
/// let config = Config { debug: true };
/// let handler = LectorHandler::new(config);
/// assert!(handler.env().debug);
/// ```
pub struct LectorHandler<R> {
    env: R,
}

impl<R> LectorHandler<R> {
    /// Create a new reader handler with the given environment.
    #[inline]
    pub fn new(env: R) -> Self {
        LectorHandler { env }
    }

    /// Get a reference to the environment.
    #[inline]
    pub fn env(&self) -> &R {
        &self.env
    }
}

impl<R: Clone + Send + Sync + 'static> TractatorAlgebraicus<LectorOp<R>> for LectorHandler<R> {
    type Output = R;

    #[inline]
    fn handle_return(&self, value: R) -> R {
        value
    }

    #[inline]
    fn handle_operation(&mut self, op: LectorOp<R>, cont: ContinuatioSemel<R, R>) -> R {
        match op {
            LectorOp::Ask => cont.resume(self.env.clone()),
            LectorOp::_Phantom(_, never) => match never {},
        }
    }
}

// =============================================================================
// Writer Effect (ScriptorOp)
// =============================================================================

/// Writer effect operations.
///
/// `ScriptorOp<W>` provides output accumulation capabilities.
///
/// > *"Scriptor notitiarum"* — Writer of information.
///
/// # Operations
///
/// - `Tell(W)`: Append output to the log
///
/// # Example
///
/// ```rust
/// use ordofp_core::effects::builtin::ScriptorOp;
///
/// // Log a message
/// let tell_op = ScriptorOp::Tell("Processing...".to_string());
/// ```
#[derive(Debug, Clone)]
pub enum ScriptorOp<W> {
    /// Append output to the accumulated log.
    Tell(W),
}

impl<W: Clone + Send + Sync + 'static> EffectusAlgebraicus for ScriptorOp<W> {
    type Result = ();
}

/// Handler for writer effects.
///
/// `ScriptorHandler<W>` accumulates output of type `W`.
///
/// # Example
///
/// ```rust
/// use ordofp_core::effects::builtin::ScriptorHandler;
///
/// let handler = ScriptorHandler::<String>::new();
/// // Use handler, then get output
/// let logs = handler.output();
/// assert!(logs.is_empty());
/// ```
pub struct ScriptorHandler<W> {
    output: Vec<W>,
}

impl<W> ScriptorHandler<W> {
    /// Create a new writer handler.
    #[inline]
    pub fn new() -> Self {
        ScriptorHandler { output: Vec::new() }
    }

    /// Get a reference to the accumulated output.
    #[inline]
    pub fn output(&self) -> &[W] {
        &self.output
    }

    /// Take the accumulated output.
    #[inline]
    pub fn into_output(self) -> Vec<W> {
        self.output
    }

    /// Clear the accumulated output.
    #[inline]
    pub fn clear(&mut self) {
        self.output.clear();
    }
}

impl<W> Default for ScriptorHandler<W> {
    fn default() -> Self {
        Self::new()
    }
}

impl<W: Clone + Send + Sync + 'static> TractatorAlgebraicus<ScriptorOp<W>> for ScriptorHandler<W> {
    type Output = ();

    #[inline]
    fn handle_return(&self, _value: ()) {}

    #[inline]
    fn handle_operation(&mut self, op: ScriptorOp<W>, cont: ContinuatioSemel<(), ()>) {
        match op {
            ScriptorOp::Tell(w) => {
                self.output.push(w);
                cont.resume(());
            }
        }
    }
}

// =============================================================================
// Error Effect (ErrorOp)
// =============================================================================

/// Error effect operations.
///
/// `ErrorOp<E>` provides recoverable error handling.
///
/// > *"Error recuperabilis"* — Recoverable error.
///
/// # Operations
///
/// - `Raise(E)`: Raise an error
///
/// # Example
///
/// ```rust
/// use ordofp_core::effects::builtin::ErrorOp;
///
/// // Raise a validation error
/// let error_op = ErrorOp::Raise("Invalid input".to_string());
/// ```
#[derive(Debug, Clone)]
pub enum ErrorOp<E> {
    /// Raise an error.
    Raise(E),
}

impl<E: Clone + Send + Sync + 'static> EffectusAlgebraicus for ErrorOp<E> {
    type Result = core::convert::Infallible;
}

/// Handler for error effects.
///
/// `ErrorHandler<E, A>` handles errors by returning a `Result<A, E>`.
///
/// # Example
///
/// ```rust
/// use ordofp_core::effects::builtin::ErrorHandler;
///
/// let _handler = ErrorHandler::<String, i32>::new();
/// ```
pub struct ErrorHandler<E, A> {
    _error: PhantomData<E>,
    _output: PhantomData<A>,
}

impl<E, A> ErrorHandler<E, A> {
    /// Create a new error handler.
    #[inline]
    pub fn new() -> Self {
        ErrorHandler {
            _error: PhantomData,
            _output: PhantomData,
        }
    }
}

impl<E, A> Default for ErrorHandler<E, A> {
    fn default() -> Self {
        Self::new()
    }
}

impl<E: Clone + Send + Sync + 'static, A: 'static> TractatorAlgebraicus<ErrorOp<E>>
    for ErrorHandler<E, A>
{
    type Output = Result<A, E>;

    #[inline]
    fn handle_return(&self, value: Result<A, E>) -> Result<A, E> {
        value
    }

    #[inline]
    fn handle_operation(
        &mut self,
        op: ErrorOp<E>,
        _cont: ContinuatioSemel<core::convert::Infallible, Result<A, E>>,
    ) -> Result<A, E> {
        match op {
            ErrorOp::Raise(e) => Err(e),
        }
    }
}

// =============================================================================
// Choice Effect (ElectioOp)
// =============================================================================

/// Choice effect operations for non-determinism.
///
/// `ElectioOp<A>` provides non-deterministic choice.
///
/// > *"Electio possibilitatum"* — Choice of possibilities.
///
/// # Operations
///
/// - `Choose(Vec<A>)`: Choose from multiple alternatives
/// - `Fail`: Fail with no alternatives
#[derive(Debug, Clone)]
pub enum ElectioOp<A> {
    /// Choose from multiple alternatives.
    Choose(Vec<A>),
    /// Fail with no alternatives.
    Fail,
}

impl<A: Clone + Send + Sync + 'static> EffectusAlgebraicus for ElectioOp<A> {
    type Result = A;
}

/// Result collector for choice effects (stub).
///
/// **Status:** unlike the other built-in handlers in this module,
/// `ElectioHandler` has **no `TractatorAlgebraicus` impl** — it cannot
/// actually interpret `ElectioOp` effects. It is only a `Vec`-backed
/// results container reserved for a future non-determinism handler that
/// would explore all branches. For working multi-shot choice handling,
/// see `effects::handler_multi`.
pub struct ElectioHandler<A> {
    results: Vec<A>,
}

impl<A> ElectioHandler<A> {
    /// Create a new choice handler.
    #[inline]
    pub fn new() -> Self {
        ElectioHandler {
            results: Vec::new(),
        }
    }

    /// Get the collected results.
    #[inline]
    pub fn results(&self) -> &[A] {
        &self.results
    }

    /// Take the collected results.
    #[inline]
    pub fn into_results(self) -> Vec<A> {
        self.results
    }
}

impl<A> Default for ElectioHandler<A> {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Async Effect (AsyncOp)
// =============================================================================

/// Async effect marker operations.
///
/// `AsyncOp` marks computations that may involve async operations.
///
/// > *"Effectus asynchronus"* — Asynchronous effect.
#[derive(Debug, Clone, Copy)]
pub enum AsyncOp {
    /// Yield control back to the runtime.
    Yield,
}

impl EffectusAlgebraicus for AsyncOp {
    type Result = ();
}

// =============================================================================
// Console Effect (ConsolaOp)
// =============================================================================

/// Console effect operations.
///
/// `ConsolaOp` provides console I/O operations.
///
/// > *"Consola terminalis"* — Terminal console.
#[derive(Debug, Clone)]
pub enum ConsolaOp {
    /// Print a line to the console.
    PrintLine(alloc::string::String),
    /// Read a line from the console.
    ReadLine,
}

impl EffectusAlgebraicus for ConsolaOp {
    type Result = alloc::string::String;
}

/// A mock console handler for testing.
///
/// `MockConsolaHandler` simulates console I/O using provided inputs
/// and capturing outputs.
pub struct MockConsolaHandler {
    inputs: Vec<alloc::string::String>,
    input_index: usize,
    outputs: Vec<alloc::string::String>,
}

impl MockConsolaHandler {
    /// Create a new mock console with predefined inputs.
    #[inline]
    pub fn new(inputs: Vec<alloc::string::String>) -> Self {
        MockConsolaHandler {
            inputs,
            input_index: 0,
            outputs: Vec::new(),
        }
    }

    /// Get the captured outputs.
    #[inline]
    pub fn outputs(&self) -> &[alloc::string::String] {
        &self.outputs
    }

    /// Take the captured outputs.
    #[inline]
    pub fn into_outputs(self) -> Vec<alloc::string::String> {
        self.outputs
    }
}

impl TractatorAlgebraicus<ConsolaOp> for MockConsolaHandler {
    type Output = alloc::string::String;

    fn handle_return(&self, value: alloc::string::String) -> alloc::string::String {
        value
    }

    fn handle_operation(
        &mut self,
        op: ConsolaOp,
        cont: ContinuatioSemel<alloc::string::String, alloc::string::String>,
    ) -> alloc::string::String {
        match op {
            ConsolaOp::PrintLine(s) => {
                self.outputs.push(s);
                cont.resume(alloc::string::String::new())
            }
            ConsolaOp::ReadLine => {
                let input = if self.input_index < self.inputs.len() {
                    let s = self.inputs[self.input_index].clone();
                    self.input_index += 1;
                    s
                } else {
                    alloc::string::String::new()
                };
                cont.resume(input)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::algebraic::run_with_handler;
    use super::*;
    use crate::effects::ComputatioStatus;
    use alloc::string::ToString;
    use alloc::vec;

    #[test]
    fn test_status_handler_get() {
        let mut handler = StatusHandler::new(42);

        let result = run_with_handler(&mut handler, || ComputatioStatus::Suspensus {
            operatio: StatusOp::Get,
            continuatio: ContinuatioSemel::new(ComputatioStatus::Completus),
        });

        assert_eq!(result, 42);
    }

    #[test]
    fn test_status_handler_put() {
        let mut handler = StatusHandler::new(0);

        let result = run_with_handler(&mut handler, || ComputatioStatus::Suspensus {
            operatio: StatusOp::Put(42),
            continuatio: ContinuatioSemel::new(ComputatioStatus::Completus),
        });

        assert_eq!(result, 42);
        assert_eq!(handler.state, 42);
    }

    #[test]
    fn test_lector_handler() {
        let mut handler = LectorHandler::new("test_env".to_string());

        let result = run_with_handler(&mut handler, || ComputatioStatus::Suspensus {
            operatio: LectorOp::Ask,
            continuatio: ContinuatioSemel::new(ComputatioStatus::Completus),
        });

        assert_eq!(result, "test_env");
    }

    #[test]
    fn test_scriptor_handler() {
        let mut handler: ScriptorHandler<&str> = ScriptorHandler::new();

        let _: () = run_with_handler(&mut handler, || ComputatioStatus::Suspensus {
            operatio: ScriptorOp::Tell("log1"),
            continuatio: ContinuatioSemel::new(|()| ComputatioStatus::Completus(())),
        });

        assert_eq!(handler.output(), &["log1"]);
    }

    #[test]
    fn test_error_handler() {
        let mut handler: ErrorHandler<&str, i32> = ErrorHandler::new();

        let result: Result<i32, &str> =
            run_with_handler(&mut handler, || ComputatioStatus::Suspensus {
                operatio: ErrorOp::Raise("error!"),
                continuatio: ContinuatioSemel::new(|_: core::convert::Infallible| unreachable!()),
            });

        assert_eq!(result, Err("error!"));
    }

    #[test]
    fn test_mock_console_print() {
        let mut handler = MockConsolaHandler::new(vec![]);

        let _ = run_with_handler(&mut handler, || ComputatioStatus::Suspensus {
            operatio: ConsolaOp::PrintLine("Hello!".to_string()),
            continuatio: ContinuatioSemel::new(|_| ComputatioStatus::Completus("done".to_string())),
        });

        assert_eq!(handler.outputs(), &["Hello!"]);
    }

    #[test]
    fn test_mock_console_read() {
        let mut handler = MockConsolaHandler::new(vec!["user_input".to_string()]);

        let result = run_with_handler(&mut handler, || ComputatioStatus::Suspensus {
            operatio: ConsolaOp::ReadLine,
            continuatio: ContinuatioSemel::new(ComputatioStatus::Completus),
        });

        assert_eq!(result, "user_input");
    }

    #[test]
    fn test_choice_effect() {
        let choose_op = ElectioOp::Choose(vec![1, 2, 3]);
        match choose_op {
            ElectioOp::Choose(v) => assert_eq!(v, vec![1, 2, 3]),
            ElectioOp::Fail => panic!("Unexpected Fail"),
        }
    }

    #[test]
    fn test_async_op() {
        let _ = AsyncOp::Yield;
    }
}
