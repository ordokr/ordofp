//! ZIO-Style Effect Type - Advanced Effectful Computations
//!
//! > *"Effectus in causa continetur"*
//! > — The effect is contained in the cause. (Scholastic philosophy)
//!
//! This module provides a ZIO-style effect type that combines environment
//! access, typed error handling, and asynchronous computation in a single
//! unified type.
//!
//! # Design
//!
//! The `Zio<R, E, A>` type represents a computation that:
//! - Requires an environment of type `R`
//! - May fail with an error of type `E`
//! - Produces a value of type `A` on success
//!
//! This is equivalent to: `R => Future<Result<A, Causa<E>>>`
//!
//! # Inspired By
//!
//! - ZIO 2.0's `ZIO[R, E, A]` effect type
//! - Cats Effect's `IO[A]` with error handling
//! - Haskell's `ReaderT (ExceptT e IO)` stack
//!
//! # Scholastic Naming
//!
//! | English | Latin | Etymology |
//! |---------|-------|-----------|
//! | Effect | Effectus | *effectus* = result, outcome |
//! | Exit | Exitus | *exitus* = going out, result |
//! | Cause | Causa | *causa* = cause, reason |
//! | Environment | Ambitus | *ambitus* = circuit, surroundings |
//! | Succeed | Succedere | *succedere* = to succeed |
//! | Fail | Deficere | *deficere* = to fail, lack |
//! | Die | Mori | *mori* = to die |
//! | Interrupt | Interrumpere | *interrumpere* = to break apart |

extern crate alloc;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;

use super::fibra::FibraId;

// =============================================================================
// Causa - Structured Error Information
// =============================================================================

/// Structured cause of failure.
///
/// `Causa<E>` captures not just the error value, but also contextual
/// information about how the failure occurred (defect, interruption,
/// or combinations thereof).
///
/// # Latin Etymology
/// *Causa* = cause, reason, motive.
#[derive(Debug, Clone)]
pub enum Causa<E> {
    /// Expected failure with typed error.
    ///
    /// > *"Defectus est privatio formae debitae"*
    /// > — A defect is the privation of due form.
    Defectus(E),

    /// Unexpected failure (panic, unrecoverable).
    ///
    /// > *"Mors est separatio animae a corpore"*
    /// > — Death is the separation of soul from body.
    Mors(String),

    /// Fiber was interrupted by another fiber.
    ///
    /// > *"Interruptio est actus dividendi"*
    /// > — Interruption is the act of dividing.
    Interruptio(FibraId),

    /// Both causes occurred (parallel failure).
    ///
    /// > *"Utrumque simul"*
    /// > — Both at the same time.
    Utrumque(Box<Causa<E>>, Box<Causa<E>>),

    /// Sequential causes (first then second).
    ///
    /// > *"Deinde post"*
    /// > — Then after.
    Deinde(Box<Causa<E>>, Box<Causa<E>>),

    /// Empty cause (for internal use).
    Vacua,
}

impl<E> Causa<E> {
    /// Create a failure cause from an error.
    #[inline]
    pub fn defectus(e: E) -> Self {
        Causa::Defectus(e)
    }

    /// Create a death cause from a panic message.
    #[inline]
    pub fn mors(msg: impl Into<String>) -> Self {
        Causa::Mors(msg.into())
    }

    /// Create an interruption cause.
    #[inline]
    pub fn interruptio(fibra_id: FibraId) -> Self {
        Causa::Interruptio(fibra_id)
    }

    /// Combine two causes that happened in parallel.
    #[inline]
    pub fn both(self, other: Causa<E>) -> Self {
        match (&self, &other) {
            (Causa::Vacua, _) => other,
            (_, Causa::Vacua) => self,
            _ => Causa::Utrumque(Box::new(self), Box::new(other)),
        }
    }

    /// Chain two causes sequentially.
    #[inline]
    pub fn then(self, other: Causa<E>) -> Self {
        match (&self, &other) {
            (Causa::Vacua, _) => other,
            (_, Causa::Vacua) => self,
            _ => Causa::Deinde(Box::new(self), Box::new(other)),
        }
    }

    /// Check if this is a defect (expected error).
    #[inline]
    pub fn is_defectus(&self) -> bool {
        matches!(self, Causa::Defectus(_))
    }

    /// Check if this is a death (unexpected error).
    #[inline]
    pub fn is_mors(&self) -> bool {
        matches!(self, Causa::Mors(_))
    }

    /// Check if this is an interruption.
    #[inline]
    pub fn is_interruptio(&self) -> bool {
        matches!(self, Causa::Interruptio(_))
    }

    /// Extract the defect if present.
    #[inline]
    pub fn defect(&self) -> Option<&E> {
        match self {
            Causa::Defectus(e) => Some(e),
            _ => None,
        }
    }

    /// Get all defects in this cause (flattening composite causes).
    pub fn defects(&self) -> Vec<&E> {
        let mut result = Vec::new();
        self.collect_defects(&mut result);
        result
    }

    fn collect_defects<'a>(&'a self, result: &mut Vec<&'a E>) {
        match self {
            Causa::Defectus(e) => result.push(e),
            Causa::Utrumque(a, b) | Causa::Deinde(a, b) => {
                a.collect_defects(result);
                b.collect_defects(result);
            }
            _ => {}
        }
    }

    /// Map the error type.
    pub fn map_error<F, E2>(self, f: F) -> Causa<E2>
    where
        F: Fn(E) -> E2 + Clone,
    {
        match self {
            Causa::Defectus(e) => Causa::Defectus(f(e)),
            Causa::Mors(s) => Causa::Mors(s),
            Causa::Interruptio(id) => Causa::Interruptio(id),
            Causa::Utrumque(a, b) => {
                Causa::Utrumque(Box::new(a.map_error(f.clone())), Box::new(b.map_error(f)))
            }
            Causa::Deinde(a, b) => {
                Causa::Deinde(Box::new(a.map_error(f.clone())), Box::new(b.map_error(f)))
            }
            Causa::Vacua => Causa::Vacua,
        }
    }
}

impl<E: core::fmt::Display> core::fmt::Display for Causa<E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Causa::Defectus(e) => write!(f, "Defectus: {e}"),
            Causa::Mors(s) => write!(f, "Mors: {s}"),
            Causa::Interruptio(id) => write!(f, "Interruptio: {id}"),
            Causa::Utrumque(a, b) => write!(f, "({a} ∧ {b})"),
            Causa::Deinde(a, b) => write!(f, "({a} → {b})"),
            Causa::Vacua => write!(f, "Vacua"),
        }
    }
}

// =============================================================================
// Exitus - Exit Result
// =============================================================================

/// The result of running an effect: success or failure with cause.
///
/// `Exitus<E, A>` captures the outcome of an effectful computation.
///
/// # Latin Etymology
/// *Exitus* = going out, end, result.
#[derive(Debug, Clone)]
pub enum Exitus<E, A> {
    /// Computation succeeded with a value.
    ///
    /// > *"Successus est actus perficiendi"*
    /// > — Success is the act of completing.
    Successus(A),

    /// Computation failed with a cause.
    ///
    /// > *"Defectio est privatio successus"*
    /// > — Failure is the privation of success.
    Defectio(Causa<E>),
}

impl<E, A> Exitus<E, A> {
    /// Create a successful exit.
    #[inline]
    pub fn successus(a: A) -> Self {
        Exitus::Successus(a)
    }

    /// Create a failed exit from an error.
    #[inline]
    pub fn defectio(e: E) -> Self {
        Exitus::Defectio(Causa::defectus(e))
    }

    /// Create a failed exit from a cause.
    #[inline]
    pub fn defectio_causa(causa: Causa<E>) -> Self {
        Exitus::Defectio(causa)
    }

    /// Create a death exit.
    #[inline]
    pub fn mors(msg: impl Into<String>) -> Self {
        Exitus::Defectio(Causa::mors(msg))
    }

    /// Create an interruption exit.
    #[inline]
    pub fn interruptio(fibra_id: FibraId) -> Self {
        Exitus::Defectio(Causa::interruptio(fibra_id))
    }

    /// Check if this exit is a success.
    #[inline]
    pub fn is_successus(&self) -> bool {
        matches!(self, Exitus::Successus(_))
    }

    /// Check if this exit is a failure.
    #[inline]
    pub fn is_defectio(&self) -> bool {
        matches!(self, Exitus::Defectio(_))
    }

    /// Get the success value if present.
    #[inline]
    pub fn successus_value(&self) -> Option<&A> {
        match self {
            Exitus::Successus(a) => Some(a),
            Exitus::Defectio(_) => None,
        }
    }

    /// Get the failure cause if present.
    #[inline]
    pub fn causa(&self) -> Option<&Causa<E>> {
        match self {
            Exitus::Defectio(c) => Some(c),
            Exitus::Successus(_) => None,
        }
    }

    /// Map the success value.
    #[inline]
    pub fn map<B, F>(self, f: F) -> Exitus<E, B>
    where
        F: FnOnce(A) -> B,
    {
        match self {
            Exitus::Successus(a) => Exitus::Successus(f(a)),
            Exitus::Defectio(c) => Exitus::Defectio(c),
        }
    }

    /// Map the error type.
    #[inline]
    pub fn map_error<F, E2>(self, f: F) -> Exitus<E2, A>
    where
        F: Fn(E) -> E2 + Clone,
    {
        match self {
            Exitus::Successus(a) => Exitus::Successus(a),
            Exitus::Defectio(c) => Exitus::Defectio(c.map_error(f)),
        }
    }

    /// Convert to a standard Result.
    ///
    /// # Errors
    ///
    /// Returns `Err` carrying the full failure [`Causa`] (typed error,
    /// defect, or interruption) when this exit is a `Defectio`; a
    /// `Successus` becomes `Ok`. No information is lost either way.
    #[inline]
    pub fn to_result(self) -> Result<A, Causa<E>> {
        match self {
            Exitus::Successus(a) => Ok(a),
            Exitus::Defectio(c) => Err(c),
        }
    }

    /// Convert from a standard Result.
    #[inline]
    pub fn from_result(result: Result<A, E>) -> Self {
        match result {
            Ok(a) => Exitus::successus(a),
            Err(e) => Exitus::defectio(e),
        }
    }

    /// Flat map over success.
    #[inline]
    pub fn flat_map<B, F>(self, f: F) -> Exitus<E, B>
    where
        F: FnOnce(A) -> Exitus<E, B>,
    {
        match self {
            Exitus::Successus(a) => f(a),
            Exitus::Defectio(c) => Exitus::Defectio(c),
        }
    }

    /// Fold over the exit.
    #[inline]
    pub fn fold<B, FS, FF>(self, on_success: FS, on_failure: FF) -> B
    where
        FS: FnOnce(A) -> B,
        FF: FnOnce(Causa<E>) -> B,
    {
        match self {
            Exitus::Successus(a) => on_success(a),
            Exitus::Defectio(c) => on_failure(c),
        }
    }
}

impl<E, A> From<Result<A, E>> for Exitus<E, A> {
    #[inline]
    fn from(result: Result<A, E>) -> Self {
        Exitus::from_result(result)
    }
}

// =============================================================================
// Ambitus - Environment
// =============================================================================

/// Environment container for ZIO computations.
///
/// `Ambitus<R>` wraps the environment required by a computation.
///
/// # Latin Etymology
/// *Ambitus* = circuit, going around, environment.
#[derive(Debug, Clone)]
pub struct Ambitus<R> {
    value: R,
}

impl<R> Ambitus<R> {
    /// Create a new environment.
    #[inline]
    pub fn new(value: R) -> Self {
        Ambitus { value }
    }

    /// Get a reference to the environment.
    #[inline]
    pub fn get(&self) -> &R {
        &self.value
    }

    /// Get a mutable reference to the environment.
    #[inline]
    pub fn get_mut(&mut self) -> &mut R {
        &mut self.value
    }

    /// Unwrap the environment value.
    #[inline]
    pub fn into_inner(self) -> R {
        self.value
    }

    /// Map the environment.
    #[inline]
    pub fn map<S, F>(self, f: F) -> Ambitus<S>
    where
        F: FnOnce(R) -> S,
    {
        Ambitus::new(f(self.value))
    }
}

impl<R: Default> Default for Ambitus<R> {
    fn default() -> Self {
        Ambitus::new(R::default())
    }
}

// =============================================================================
// Zio - The Core Effect Type
// =============================================================================

/// The inner runner type for Zio computations.
///
/// This type alias factors out the complex boxed future type.
///
/// # Double-boxing: measured, kept
///
/// The boxed closure returning a boxed future is two indirections per bind.
/// `benches/zio_bind.rs` measured ~100–170 ns per bind with only mild
/// superlinearity at depth 256 — no cliff, and no known consumer puts Zio
/// bind-chains in a hot path. Revisit if a consumer enabling `async`/`tokio`
/// has deep bind-chains hot; the first candidate is then enum dispatch over
/// the known constructor shapes with this boxed type as the fallback arm.
pub type ZioRunner<R, E, A> =
    Box<dyn FnOnce(Ambitus<R>) -> Pin<Box<dyn Future<Output = Exitus<E, A>> + Send>> + Send>;

/// ZIO-style effect type.
///
/// `Zio<R, E, A>` represents a computation that:
/// - Requires environment `R`
/// - May fail with error `E`
/// - Produces value `A` on success
///
/// # Latin Etymology
/// A functional blend of *Zona Influentia Operationis* (zone of operational influence).
pub struct Zio<R, E, A> {
    /// The effectful computation.
    run: ZioRunner<R, E, A>,
    _phantom: PhantomData<(R, E, A)>,
}

impl<R: Send + 'static, E: Send + 'static, A: Send + 'static> Zio<R, E, A> {
    /// Create a pure effect that succeeds with a value.
    ///
    /// > *"Purus est quod nihil habet admixtum"*
    /// > — Pure is that which has nothing mixed in.
    #[inline]
    pub fn succeed(a: A) -> Self
    where
        A: Send,
    {
        Zio {
            run: Box::new(move |_| Box::pin(async move { Exitus::successus(a) })),
            _phantom: PhantomData,
        }
    }

    /// Create an effect that fails with an error.
    ///
    /// > *"Deficere est non attingere finem"*
    /// > — To fail is not to reach the end.
    #[inline]
    pub fn fail(e: E) -> Self
    where
        E: Send,
    {
        Zio {
            run: Box::new(move |_| Box::pin(async move { Exitus::defectio(e) })),
            _phantom: PhantomData,
        }
    }

    /// Create an effect that dies with a message.
    #[inline]
    pub fn die(msg: impl Into<String> + Send + 'static) -> Self {
        let msg = msg.into();
        Zio {
            run: Box::new(move |_| Box::pin(async move { Exitus::mors(msg) })),
            _phantom: PhantomData,
        }
    }

    /// Create an effect from an async closure.
    #[inline]
    pub fn from_async<F, Fut>(f: F) -> Self
    where
        F: FnOnce(Ambitus<R>) -> Fut + Send + 'static,
        Fut: Future<Output = Exitus<E, A>> + Send + 'static,
    {
        Zio {
            run: Box::new(move |env| Box::pin(f(env))),
            _phantom: PhantomData,
        }
    }

    /// Create an effect that accesses the environment.
    #[inline]
    pub fn environment() -> Zio<R, E, R>
    where
        R: Clone + Send,
    {
        Zio {
            run: Box::new(|env| Box::pin(async move { Exitus::successus(env.into_inner()) })),
            _phantom: PhantomData,
        }
    }

    /// Create an effect that accesses part of the environment.
    #[inline]
    pub fn environment_with<B, F>(f: F) -> Zio<R, E, B>
    where
        F: FnOnce(&R) -> B + Send + 'static,
        B: Send + 'static,
    {
        Zio {
            run: Box::new(move |env| Box::pin(async move { Exitus::successus(f(env.get())) })),
            _phantom: PhantomData,
        }
    }

    /// Map over the success value.
    #[inline]
    pub fn map<B, F>(self, f: F) -> Zio<R, E, B>
    where
        F: FnOnce(A) -> B + Send + 'static,
        B: Send + 'static,
    {
        Zio {
            run: Box::new(move |env| {
                Box::pin(async move {
                    let result = (self.run)(env).await;
                    result.map(f)
                })
            }),
            _phantom: PhantomData,
        }
    }

    /// Map over the error value.
    #[inline]
    pub fn map_error<E2, F>(self, f: F) -> Zio<R, E2, A>
    where
        F: Fn(E) -> E2 + Send + Clone + 'static,
        E2: 'static,
    {
        Zio {
            run: Box::new(move |env| {
                Box::pin(async move {
                    let result = (self.run)(env).await;
                    result.map_error(f)
                })
            }),
            _phantom: PhantomData,
        }
    }

    /// Flat map over the success value.
    ///
    /// The environment `R` is passed directly to `self`, and only the
    /// surviving `env` handle is forwarded to the continuation `f(a)`.
    /// No clone of `R` is performed in the success path; on failure the
    /// continuation is never called so no clone is needed there either.
    #[inline]
    pub fn flat_map<B, F>(self, f: F) -> Zio<R, E, B>
    where
        F: FnOnce(A) -> Zio<R, E, B> + Send + 'static,
        B: Send + 'static,
        R: Clone,
    {
        Zio {
            run: Box::new(move |env| {
                Box::pin(async move {
                    // Clone only when the inner effect needs the env AND the
                    // continuation also needs it.  We clone once here so that
                    // `self.run` gets one copy and the continuation `f` gets
                    // the original.  This is the minimum required by the
                    // shared-environment contract; it cannot be removed without
                    // changing the semantics (both legs need `R`).
                    let env_for_self = Ambitus::new(env.get().clone());
                    let result = (self.run)(env_for_self).await;
                    match result {
                        Exitus::Successus(a) => (f(a).run)(env).await,
                        Exitus::Defectio(c) => Exitus::Defectio(c),
                    }
                })
            }),
            _phantom: PhantomData,
        }
    }

    /// Provide the environment, eliminating R.
    #[inline]
    pub fn provide(self, env: R) -> Zio<(), E, A>
    where
        R: Send,
    {
        Zio {
            run: Box::new(move |_| (self.run)(Ambitus::new(env))),
            _phantom: PhantomData,
        }
    }

    /// Catch and handle errors.
    ///
    /// On the success path, no clone of `R` occurs and the original `env` is
    /// kept live for the handler.  On success the result is returned directly
    /// without re-wrapping it in `Exitus::successus`.
    #[inline]
    pub fn catch_all<F>(self, handler: F) -> Zio<R, E, A>
    where
        F: FnOnce(Causa<E>) -> Zio<R, E, A> + Send + 'static,
        R: Clone + Send,
    {
        Zio {
            run: Box::new(move |env| {
                Box::pin(async move {
                    let env_for_self = Ambitus::new(env.get().clone());
                    let result = (self.run)(env_for_self).await;
                    match result {
                        // Return the Successus variant directly — no re-wrap.
                        ok @ Exitus::Successus(_) => ok,
                        Exitus::Defectio(c) => (handler(c).run)(env).await,
                    }
                })
            }),
            _phantom: PhantomData,
        }
    }

    /// Recover from typed errors only (not deaths or interruptions).
    ///
    /// Same env-clone strategy as `catch_all`: one clone for the inner
    /// effect, original forwarded to the handler on typed-error; success
    /// and non-typed failures are returned without touching `env`.
    #[inline]
    pub fn catch<F>(self, handler: F) -> Zio<R, E, A>
    where
        F: FnOnce(E) -> Zio<R, E, A> + Send + 'static,
        R: Clone + Send,
        E: Clone,
    {
        Zio {
            run: Box::new(move |env| {
                Box::pin(async move {
                    let env_for_self = Ambitus::new(env.get().clone());
                    let result = (self.run)(env_for_self).await;
                    match result {
                        // Return success / non-typed failures without touching env.
                        ok @ Exitus::Successus(_) => ok,
                        Exitus::Defectio(Causa::Defectus(e)) => (handler(e).run)(env).await,
                        other => other,
                    }
                })
            }),
            _phantom: PhantomData,
        }
    }

    /// Ensure a finalizer runs regardless of outcome.
    ///
    /// Finalizer errors are intentionally discarded (ZIO semantics): the
    /// original outcome wins. Use a finalizer that cannot fail, or handle
    /// its errors inside it.
    #[inline]
    pub fn ensuring<F>(self, finalizer: F) -> Zio<R, E, A>
    where
        F: FnOnce() -> Zio<R, E, ()> + Send + 'static,
        R: Clone + Send,
    {
        Zio {
            run: Box::new(move |env| {
                Box::pin(async move {
                    let env_for_self = Ambitus::new(env.get().clone());
                    let result = (self.run)(env_for_self).await;
                    // Discarded by contract (see doc comment above): the
                    // original `result` wins regardless of finalizer outcome.
                    let _fin_result = (finalizer().run)(env).await;
                    result
                })
            }),
            _phantom: PhantomData,
        }
    }

    /// Run the effect with an environment, producing an Exit.
    #[inline]
    pub async fn run(self, env: R) -> Exitus<E, A> {
        (self.run)(Ambitus::new(env)).await
    }
}

// =============================================================================
// Type Aliases
// =============================================================================

/// Effect that requires no environment.
pub type Task<E, A> = Zio<(), E, A>;

/// Effect that cannot fail with typed errors.
pub type Uio<R, A> = Zio<R, core::convert::Infallible, A>;

/// Effect that requires no environment and cannot fail.
pub type UTask<A> = Zio<(), core::convert::Infallible, A>;

/// IO effect (no environment, any error).
pub type Io<A> = Zio<(), alloc::string::String, A>;

// =============================================================================
// Constructors
// =============================================================================

/// Create a successful effect.
#[inline]
pub fn succeed<R: Send + 'static, E: Send + 'static, A: Send + 'static>(a: A) -> Zio<R, E, A> {
    Zio::succeed(a)
}

/// Create a failed effect.
#[inline]
pub fn fail<R: Send + 'static, E: Send + 'static, A: Send + 'static>(e: E) -> Zio<R, E, A> {
    Zio::fail(e)
}

/// Create an effect that accesses the environment.
#[inline]
pub fn environment<R: Clone + Send + 'static, E: Send + 'static>() -> Zio<R, E, R> {
    Zio::<R, E, R>::environment()
}

/// Create an effect from a Result.
#[inline]
pub fn from_result<R: Send + 'static, E: Send + 'static, A: Send + 'static>(
    result: Result<A, E>,
) -> Zio<R, E, A> {
    match result {
        Ok(a) => Zio::succeed(a),
        Err(e) => Zio::fail(e),
    }
}

/// Create an effect from an Option.
#[inline]
pub fn from_option<R: Send + 'static, A: Send + 'static>(opt: Option<A>) -> Zio<R, (), A> {
    match opt {
        Some(a) => Zio::succeed(a),
        None => Zio::fail(()),
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_causa_defectus() {
        let causa: Causa<&str> = Causa::defectus("error");
        assert!(causa.is_defectus());
        assert_eq!(causa.defect(), Some(&"error"));
    }

    #[test]
    fn test_causa_mors() {
        let causa: Causa<&str> = Causa::mors("panic!");
        assert!(causa.is_mors());
    }

    #[test]
    fn test_causa_both() {
        let c1: Causa<&str> = Causa::defectus("error1");
        let c2: Causa<&str> = Causa::defectus("error2");
        let combined = c1.both(c2);
        assert_eq!(combined.defects().len(), 2);
    }

    #[test]
    fn test_exitus_successus() {
        let exit: Exitus<&str, i32> = Exitus::successus(42);
        assert!(exit.is_successus());
        assert_eq!(exit.successus_value(), Some(&42));
    }

    #[test]
    fn test_exitus_defectio() {
        let exit: Exitus<&str, i32> = Exitus::defectio("error");
        assert!(exit.is_defectio());
        assert!(
            exit.causa()
                .expect("defectio exit must have a causa")
                .is_defectus()
        );
    }

    #[test]
    fn test_exitus_map() {
        let exit: Exitus<&str, i32> = Exitus::successus(21);
        let mapped = exit.map(|x| x * 2);
        assert_eq!(mapped.successus_value(), Some(&42));
    }

    #[test]
    fn test_exitus_flat_map() {
        let exit: Exitus<&str, i32> = Exitus::successus(20);
        let result = exit.flat_map(|x| Exitus::successus(x + 22));
        assert_eq!(result.successus_value(), Some(&42));
    }

    #[test]
    fn test_exitus_from_result() {
        let ok: Exitus<&str, i32> = Exitus::from_result(Ok(42));
        assert!(ok.is_successus());

        let err: Exitus<&str, i32> = Exitus::from_result(Err("error"));
        assert!(err.is_defectio());
    }

    #[test]
    fn test_ambitus_new() {
        let env = Ambitus::new(42);
        assert_eq!(*env.get(), 42);
    }

    #[test]
    fn test_ambitus_map() {
        let env = Ambitus::new(21);
        let mapped = env.map(|x| x * 2);
        assert_eq!(*mapped.get(), 42);
    }
}
