//! Common Effect Types - Standard effect markers
//!
//! > *"Communes effectus"*
//! > — Common effects. (Neo-Latin)
//!
//! This module provides standard effect type markers used throughout
//! the `OrdoFP` ecosystem.

use super::Effectus;
use core::marker::PhantomData;

/// IO Effect marker.
///
/// `IoEffectus` marks computations that perform input/output operations,
/// such as reading files, network requests, or console output.
///
/// > *"Actio ad extra"* — Action towards the outside.
///
/// # Example
///
/// ```rust
/// use ordofp_core::effects::{IoEffectus, EffectusHandlerAsync};
///
/// struct FileHandler;
///
/// impl EffectusHandlerAsync<IoEffectus> for FileHandler {
///     type Output = Result<String, std::io::Error>;
///
///     async fn handle_async(&self, _: IoEffectus) -> Self::Output {
///         Ok("file contents".to_string())
///     }
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct IoEffectus;

impl Effectus for IoEffectus {}

impl IoEffectus {
    /// Create a new IO effect marker.
    pub const fn new() -> Self {
        IoEffectus
    }
}

/// State Effect marker.
///
/// `StatusEffectus<S>` marks computations that read or modify state of type `S`.
///
/// > *"Status mutabilis"* — Changeable state.
///
/// # Example
///
/// ```rust
/// use ordofp_core::effects::{StatusEffectus, EffectusHandler};
///
/// struct CounterHandler {
///     counter: std::cell::RefCell<i32>,
/// }
///
/// impl EffectusHandler<StatusEffectus<i32>> for CounterHandler {
///     type Output = i32;
///
///     fn handle(&self, _: StatusEffectus<i32>) -> Self::Output {
///         *self.counter.borrow()
///     }
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct StatusEffectus<S> {
    _phantom: PhantomData<S>,
}

impl<S: Send + Sync + 'static> Effectus for StatusEffectus<S> {}

impl<S> StatusEffectus<S> {
    /// Create a new state effect marker.
    pub const fn new() -> Self {
        StatusEffectus {
            _phantom: PhantomData,
        }
    }
}

/// Error Effect marker.
///
/// `ErrorEffectus<E>` marks computations that may fail with an error of type `E`.
///
/// > *"Error possibilis"* — Possible error.
///
/// # Example
///
/// ```rust
/// use ordofp_core::effects::{ErrorEffectus, EffectusHandler};
///
/// struct ValidationHandler;
///
/// impl EffectusHandler<ErrorEffectus<String>> for ValidationHandler {
///     type Output = Result<(), String>;
///
///     fn handle(&self, _: ErrorEffectus<String>) -> Self::Output {
///         Ok(())
///     }
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ErrorEffectus<E> {
    _phantom: PhantomData<E>,
}

impl<E: Send + Sync + 'static> Effectus for ErrorEffectus<E> {}

impl<E> ErrorEffectus<E> {
    /// Create a new error effect marker.
    pub const fn new() -> Self {
        ErrorEffectus {
            _phantom: PhantomData,
        }
    }
}

/// Async Effect marker.
///
/// `AsyncEffectus` marks computations that involve asynchronous operations.
///
/// > *"Effectus asynchronus"* — Asynchronous effect.
///
/// # Example
///
/// ```rust
/// use ordofp_core::effects::{AsyncEffectus, EffectusHandlerAsync};
///
/// struct AsyncHandler;
///
/// impl EffectusHandlerAsync<AsyncEffectus> for AsyncHandler {
///     type Output = ();
///
///     async fn handle_async(&self, _: AsyncEffectus) -> Self::Output {
///         // Async operation
///     }
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AsyncEffectus;

impl Effectus for AsyncEffectus {}

impl AsyncEffectus {
    /// Create a new async effect marker.
    pub const fn new() -> Self {
        AsyncEffectus
    }
}

/// Reader Effect marker.
///
/// `ReaderEffectus<R>` marks computations that read from an environment of type `R`.
///
/// > *"Lector ambientis"* — Reader of environment.
///
/// This effect is typically handled by providing the environment value,
/// similar to the Reader monad.
///
/// # Example
///
/// ```rust
/// use ordofp_core::effects::{ReaderEffectus, EffectusHandler};
///
/// #[derive(Clone)]
/// struct Config {
///     api_key: String,
/// }
///
/// struct ConfigHandler {
///     config: Config,
/// }
///
/// impl EffectusHandler<ReaderEffectus<Config>> for ConfigHandler {
///     type Output = Config;
///
///     fn handle(&self, _: ReaderEffectus<Config>) -> Self::Output {
///         self.config.clone()
///     }
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ReaderEffectus<R> {
    _phantom: PhantomData<R>,
}

impl<R: Send + Sync + 'static> Effectus for ReaderEffectus<R> {}

impl<R> ReaderEffectus<R> {
    /// Create a new reader effect marker.
    pub const fn new() -> Self {
        ReaderEffectus {
            _phantom: PhantomData,
        }
    }
}

/// Writer Effect marker.
///
/// `ScriptorEffectus<W>` marks computations that produce output of type `W`.
///
/// > *"Scriptor notitiarum"* — Writer of information.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ScriptorEffectus<W> {
    _phantom: PhantomData<W>,
}

impl<W: Send + Sync + 'static> Effectus for ScriptorEffectus<W> {}

impl<W> ScriptorEffectus<W> {
    /// Create a new writer effect marker.
    pub const fn new() -> Self {
        ScriptorEffectus {
            _phantom: PhantomData,
        }
    }
}

/// Random Effect marker.
///
/// `RandomEffectus` marks computations that use randomness.
///
/// > *"Sors fortuita"* — Random chance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RandomEffectus;

impl Effectus for RandomEffectus {}

impl RandomEffectus {
    /// Create a new random effect marker.
    pub const fn new() -> Self {
        RandomEffectus
    }
}

/// Time Effect marker.
///
/// `TempusEffectus` marks computations that depend on or manipulate time.
///
/// > *"Tempus fugit"* — Time flies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TempusEffectus;

impl Effectus for TempusEffectus {}

impl TempusEffectus {
    /// Create a new time effect marker.
    pub const fn new() -> Self {
        TempusEffectus
    }
}

/// Resource Effect marker.
///
/// `ResourceEffectus<R>` marks computations that acquire or release resources.
///
/// > *"Res acquisita"* — Acquired resource.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ResourceEffectus<R> {
    _phantom: PhantomData<R>,
}

impl<R: Send + Sync + 'static> Effectus for ResourceEffectus<R> {}

impl<R> ResourceEffectus<R> {
    /// Create a new resource effect marker.
    pub const fn new() -> Self {
        ResourceEffectus {
            _phantom: PhantomData,
        }
    }
}

/// Pure Effect marker (no effects).
///
/// `PurusEffectus` marks computations that have no effects - they are pure.
///
/// > *"Purus effectus"* — Pure (no effect).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PurusEffectus;

impl Effectus for PurusEffectus {}

impl PurusEffectus {
    /// Create a new pure effect marker.
    pub const fn new() -> Self {
        PurusEffectus
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::String;
    use alloc::vec::Vec;

    fn requires_effectus<E: Effectus>() {}

    #[test]
    fn test_io_effectus() {
        requires_effectus::<IoEffectus>();
        let _effect = IoEffectus::new();
    }

    #[test]
    fn test_status_effectus() {
        requires_effectus::<StatusEffectus<i32>>();
        let _effect: StatusEffectus<String> = StatusEffectus::new();
    }

    #[test]
    fn test_error_effectus() {
        requires_effectus::<ErrorEffectus<String>>();
        let _effect: ErrorEffectus<std::io::Error> = ErrorEffectus::new();
    }

    #[test]
    fn test_async_effectus() {
        requires_effectus::<AsyncEffectus>();
        let _effect = AsyncEffectus::new();
    }

    #[test]
    fn test_reader_effectus() {
        requires_effectus::<ReaderEffectus<String>>();
        let _effect: ReaderEffectus<i32> = ReaderEffectus::new();
    }

    #[test]
    fn test_scriptor_effectus() {
        requires_effectus::<ScriptorEffectus<Vec<String>>>();
        let _effect: ScriptorEffectus<String> = ScriptorEffectus::new();
    }

    #[test]
    fn test_random_effectus() {
        requires_effectus::<RandomEffectus>();
        let _effect = RandomEffectus::new();
    }

    #[test]
    fn test_tempus_effectus() {
        requires_effectus::<TempusEffectus>();
        let _effect = TempusEffectus::new();
    }

    #[test]
    fn test_resource_effectus() {
        requires_effectus::<ResourceEffectus<String>>();
        let _effect: ResourceEffectus<()> = ResourceEffectus::new();
    }

    #[test]
    fn test_purus_effectus() {
        requires_effectus::<PurusEffectus>();
        let _effect = PurusEffectus::new();
    }
}
