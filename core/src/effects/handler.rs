//! Effect Handlers - Infrastructure for interpreting effects
//!
//! > *"Tractator effectuum"*
//! > — Handler of effects. (Neo-Latin)

use super::Effectus;
use core::future::Future;

/// Synchronous effect handler capability.
///
/// An `EffectusHandler` interprets an effect and produces a result.
/// This is the synchronous variant for effects that don't require async.
///
/// # Example
///
/// ```rust
/// use ordofp_core::effects::{Effectus, EffectusHandler};
///
/// struct LogEffect { message: String }
/// impl Effectus for LogEffect {}
///
/// struct PrintHandler;
///
/// impl EffectusHandler<LogEffect> for PrintHandler {
///     type Output = ();
///
///     fn handle(&self, effect: LogEffect) -> Self::Output {
///         println!("{}", effect.message);
///     }
/// }
///
/// let handler = PrintHandler;
/// handler.handle(LogEffect { message: "hello".to_string() });
/// ```
pub trait EffectusHandler<E: Effectus> {
    /// The type produced by handling this effect.
    type Output;

    /// Handle the effect, producing an output.
    ///
    /// This method interprets the effect description and performs
    /// the actual side effect (if any).
    fn handle(&self, effect: E) -> Self::Output;
}

/// Async effect handler capability.
///
/// An `EffectusHandlerAsync` interprets an effect asynchronously,
/// returning a `Future` that produces the result.
///
/// # Example
///
/// ```rust
/// use ordofp_core::effects::{EffectusHandlerAsync, IoEffectus};
///
/// struct AsyncFileHandler;
///
/// impl EffectusHandlerAsync<IoEffectus> for AsyncFileHandler {
///     type Output = Result<String, std::io::Error>;
///
///     async fn handle_async(&self, _effect: IoEffectus) -> Self::Output {
///         // Async file operations
///         Ok("file contents".to_string())
///     }
/// }
/// ```
pub trait EffectusHandlerAsync<E: Effectus> {
    /// The type produced by handling this effect.
    type Output;

    /// Handle the effect asynchronously.
    ///
    /// Returns a future that, when polled, interprets the effect
    /// and produces the output.
    fn handle_async(&self, effect: E) -> impl Future<Output = Self::Output> + Send;
}

/// Run an effectful computation with a synchronous handler.
///
/// This function runs a computation that may use effects, using the
/// provided handler to interpret those effects.
///
/// Note that the `effect` argument is a **phantom witness**: it is consumed
/// only to select the `EffectusHandler<E>` impl and is never inspected — the
/// computation receives the *handler* and decides itself which handler
/// methods to invoke.
///
/// # Example
///
/// ```rust
/// use ordofp_core::effects::{Effectus, EffectusHandler, run_effectus};
///
/// struct MyEffect;
/// impl Effectus for MyEffect {}
///
/// struct MyHandler;
/// impl EffectusHandler<MyEffect> for MyHandler {
///     type Output = i32;
///     fn handle(&self, _: MyEffect) -> i32 { 42 }
/// }
///
/// let result = run_effectus(&MyHandler, MyEffect, |_| 42);
/// assert_eq!(result, 42);
/// ```
#[inline]
pub fn run_effectus<E, H, A, F>(handler: &H, _effect: E, computation: F) -> A
where
    E: Effectus,
    H: EffectusHandler<E>,
    F: FnOnce(&H) -> A,
{
    computation(handler)
}

/// Run an effectful async computation with an async handler.
///
/// This function runs an async computation that may use effects,
/// using the provided handler to interpret those effects.
///
/// As with [`run_effectus`], the `effect` argument is a phantom witness used
/// only for impl selection — it is never inspected.
///
/// # Example
///
/// ```rust
/// use ordofp_core::effects::{EffectusHandlerAsync, IoEffectus, run_effectus_async};
///
/// struct MyHandler;
/// impl EffectusHandlerAsync<IoEffectus> for MyHandler {
///     type Output = ();
///     async fn handle_async(&self, _: IoEffectus) -> () { () }
/// }
///
/// async fn example() {
///     let result = run_effectus_async(&MyHandler, IoEffectus, |_| async { 42 }).await;
///     assert_eq!(result, 42);
/// }
/// ```
pub async fn run_effectus_async<E, H, A, F, Fut>(handler: &H, _effect: E, computation: F) -> A
where
    E: Effectus,
    H: EffectusHandlerAsync<E>,
    F: FnOnce(&H) -> Fut,
    Fut: Future<Output = A>,
{
    computation(handler).await
}

/// A handler that does nothing (unit handler).
///
/// Useful for effects that don't need any actual handling,
/// or as a placeholder during testing.
#[derive(Debug, Clone, Copy, Default)]
pub struct UnitHandler;

impl<E: Effectus> EffectusHandler<E> for UnitHandler {
    type Output = ();

    #[inline]
    fn handle(&self, _effect: E) -> Self::Output {}
}

impl<E: Effectus> EffectusHandlerAsync<E> for UnitHandler {
    type Output = ();

    async fn handle_async(&self, _effect: E) {}
}

/// A handler that panics when invoked.
///
/// Useful for testing that certain effects are not used,
/// or as a placeholder for unimplemented handlers.
#[derive(Debug, Clone, Copy, Default)]
pub struct PanicHandler;

impl<E: Effectus> EffectusHandler<E> for PanicHandler {
    type Output = core::convert::Infallible;

    fn handle(&self, _effect: E) -> Self::Output {
        panic!("Effect handler not implemented")
    }
}

/// A composed handler that can handle multiple effect types.
///
/// This allows combining handlers for different effects into a single handler.
#[derive(Debug, Clone, Copy)]
pub struct ComposedHandler<H1, H2> {
    handler1: H1,
    handler2: H2,
}

impl<H1, H2> ComposedHandler<H1, H2> {
    /// Create a new composed handler from two handlers.
    #[inline]
    pub const fn new(handler1: H1, handler2: H2) -> Self {
        ComposedHandler { handler1, handler2 }
    }

    /// Get a reference to the first handler.
    #[inline]
    pub fn first(&self) -> &H1 {
        &self.handler1
    }

    /// Get a reference to the second handler.
    #[inline]
    pub fn second(&self) -> &H2 {
        &self.handler2
    }
}

/// Extension trait for composing handlers.
pub trait EffectusHandlerExt: Sized {
    /// Compose this handler with another handler.
    #[inline]
    fn compose<H2>(self, other: H2) -> ComposedHandler<Self, H2> {
        ComposedHandler::new(self, other)
    }
}

impl<H> EffectusHandlerExt for H {}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestEffect;
    impl Effectus for TestEffect {}

    struct TestHandler;
    impl EffectusHandler<TestEffect> for TestHandler {
        type Output = i32;
        fn handle(&self, _: TestEffect) -> i32 {
            42
        }
    }

    #[test]
    fn test_effectus_handler() {
        let handler = TestHandler;
        let result = handler.handle(TestEffect);
        assert_eq!(result, 42);
    }

    #[test]
    fn test_run_effectus() {
        let handler = TestHandler;
        let result = run_effectus(&handler, TestEffect, |h| h.handle(TestEffect));
        assert_eq!(result, 42);
    }

    #[test]
    fn test_unit_handler() {
        let handler = UnitHandler;
        handler.handle(TestEffect);
        assert_eq!((), ());
    }

    #[test]
    fn test_composed_handler() {
        let h1 = TestHandler;
        let h2 = UnitHandler;
        let composed = h1.compose(h2);

        assert_eq!(composed.first().handle(TestEffect), 42);
        assert_eq!(composed.second().handle(TestEffect), ());
    }
}
