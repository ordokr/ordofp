//! Async Core Module - Asynchronous functional programming primitives.
//!
//! > *"Motus est actus entis in potentia."*
//! > — Motion is the actuality of that which exists potentially. (Aristotle, Physics III)
//!
//! This module provides the core async primitives for `OrdoFP` 2.0, enabling
//! functional programming patterns in asynchronous Rust code.
//!
//! # Overview
//!
//! The async module provides:
//!
//! - [`FunctorAsync`] - Async mapping over values in a context
//! - [`ApplicatioAsync`] - Async applicative functor operations
//! - [`MonadAsync`] - Async monadic `bind/flat_map` operations
//! - [`Futurus`] - A monadic wrapper around `Future` with FP operations
//! - [`Flumen`] - A monadic wrapper around `Stream` with FP operations
//! - [`TraversableAsync`] - Async traversal of data structures
//!
//! # Scholastic Naming
//!
//! Following `OrdoFP`'s tradition, async types use Latin names:
//!
//! | English | Latin | Etymology |
//! |---------|-------|-----------|
//! | Future | Futurus | *futurus* = about to be |
//! | Stream | Flumen | *flumen* = flowing stream |
//! | Pending | Pendens | *pendens* = hanging/waiting |
//! | Ready | Paratus | *paratus* = prepared |
//!
//! # Feature Flags
//!
//! This module requires the `async` feature:
//!
//! ```toml
//! [dependencies]
//! ordofp = { version = "2.0", features = ["async"] }
//! ```
//!
//! For runtime integration, use `tokio` or `smol` features:
//!
//! ```toml
//! ordofp = { version = "2.0", features = ["tokio"] }
//! ```
//!
//! # Example
//!
//! ```rust
//! # use core::future::Future;
//! # use core::pin::Pin;
//! # use core::task::{Context, Poll, Waker};
//! #
//! # fn block_on<F: Future>(fut: F) -> F::Output {
//! #     let mut fut = Box::pin(fut);
//! #     let mut cx = Context::from_waker(Waker::noop());
//! #     loop {
//! #         if let Poll::Ready(out) = fut.as_mut().poll(&mut cx) {
//! #             return out;
//! #         }
//! #     }
//! # }
//! #
//! use ordofp_core::async_core::Futurus;
//!
//! let fut = Futurus::purus(42);
//! let doubled = fut.fmap(|x| x * 2);
//! assert_eq!(block_on(doubled), 84);
//! ```

mod applicative_async;
pub mod fibra;
mod flumen;
#[cfg(feature = "fusion")]
mod flumen_fusus;
mod functor_async;
mod futurus;
mod monad_async;
pub mod praefectus;
pub mod res;
pub mod runtime;
mod traversable_async;

// OrdoFP 4.0 Phase 3: Advanced Fiber Runtime
pub mod concurrent;
pub mod scheduler;
pub mod zio;

pub use applicative_async::{ApplicatioAsync, ApplicatioAsyncMut};
pub use fibra::{
    Fibra, FibraAmbitus, FibraError, FibraExitus, FibraId, FibraManubrium, FibraStatus, certamen,
    certamen_multi, deficere, fibra, furca, par, par_omnes, par_sequence, purus, race, sequence,
    sequentia, spawn, transire, transire_par, zip_par,
};
pub use flumen::{BoxStream, Flumen};
#[cfg(feature = "fusion")]
pub use flumen_fusus::{ChunksState, FlumenFusus, Gradus, ZipFusus};
pub use functor_async::{FunctorAsync, FunctorAsyncMut};
pub use futurus::Futurus;
pub use monad_async::{MonadAsync, MonadAsyncMut, flatten_option_async, flatten_result_async};
pub use praefectus::{
    EventusSupervisio, InfansSpecificatio, IntensitasRestart, PolitiaDefectus, Praefectus,
    StatusInfans, StrategiaMora, StrategiaRestart, supervisor_omnes_pro_uno,
    supervisor_reliqui_pro_uno, supervisor_unus_pro_uno,
};
pub use res::{
    Piscina, Res, ResAsync, amplexus, amplexus_async, bracket, bracket_async, finaliter,
    finaliter_async, zip_all_res, zip_res,
};
pub use runtime::{
    CurrentRuntime, FutureRuntimeExt, JoinError, JoinManubrium, NullRuntime, RuntimeGenerare,
};
// (The former custom IntoFuture re-export is gone: the trait duplicated
// core::future::IntoFuture, which the 2024 prelude provides.)
pub use traversable_async::{
    OptionTraverseAsync, ResultTraverseAsync, TraversableAsync, TraversableAsyncParallel,
    map_option_async, map_result_async, traverse_vec_async,
};

// OrdoFP 4.0 Phase 3 exports
pub use scheduler::{
    IndiciumExecutionis, MunusFibrae, OrdinariusConfig, PolitiaCircularis, PolitiaFortuita,
    PolitiaFurti, Prioritas, Statisticae,
};
pub use zio::{
    Ambitus, Causa, Exitus, Io, Task, UTask, Uio, Zio, environment, fail, from_option, from_result,
    succeed,
};

#[cfg(feature = "std")]
pub use concurrent::{CaudaBackpressure, MVarSync, Permissum, Referentia, Semaphorum};

#[cfg(feature = "std")]
pub use concurrent::Dilatum;

#[cfg(feature = "std")]
pub use scheduler::{OrdoGlobalis, OrdoLocalis};

/// Type alias for a pinned, boxed, sendable future.
///
/// This is the standard async type used throughout `OrdoFP`'s async module
/// for trait objects and stored futures.
pub type BoxFuture<'a, T> =
    core::pin::Pin<alloc::boxed::Box<dyn core::future::Future<Output = T> + Send + 'a>>;

/// Type alias for a static boxed future (no lifetime parameter).
pub type BoxFutureStatic<T> =
    core::pin::Pin<alloc::boxed::Box<dyn core::future::Future<Output = T> + Send + 'static>>;
