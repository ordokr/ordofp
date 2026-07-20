//! Concrete Effect Types
//!
//! This module provides the built-in effect types that come with Nexus:
//!
//! - **State** - Mutable state threading
//! - **Reader** - Read-only environment
//! - **Writer** - Append-only logging
//! - **Error** - Short-circuit error handling
//! - **IO** - Input/output operations
//! - **Incremental** - Dependency-tracking for self-adjusting computation
//! - **Session** - Typed protocol state machines
//! - **Probabilistic** - Sampling and conditioning for probabilistic programming
//! - **Checkpoint** - Serializable computation state for suspend/resume
//! - **Region** - Scoped memory management with bump allocation
//!
//! # Extensibility
//!
//! Users can define custom effects by implementing `EffectMarker`:
//!
//! ```rust
//! use ordofp_core::nexus::EffectMarker;
//! use ordofp_core::nexus::prelude::USER_EFFECT_START;
//!
//! struct MyEffect;
//!
//! impl EffectMarker for MyEffect {
//!     const BIT: u128 = USER_EFFECT_START << 0;
//!     const NAME: &'static str = "MyEffect";
//! }
//!
//! assert_eq!(MyEffect::BIT, USER_EFFECT_START);
//! assert_eq!(MyEffect::NAME, "MyEffect");
//! ```

pub mod checkpoint;
pub mod error;
pub mod incremental;
pub mod io;
pub mod reader;
pub mod region;
pub mod session;
pub mod state;
pub mod writer;

// Probabilistic effects require std for floating-point math (ln, sqrt, exp, etc.)
#[cfg(feature = "std")]
pub mod probabilistic;

pub use checkpoint::{
    Checkpoint, CheckpointComputation, CheckpointContext, CheckpointEffect, CheckpointId,
    CheckpointStore, Checkpointable, ResumableComputation, StepResult,
};
pub use error::{ErrorEffect, ErrorOp};
pub use incremental::{IncrementalContext, IncrementalEffect, InputId, MemoKey};
pub use io::{IoEffect, IoOp};
pub use reader::{ReaderEffect, ReaderOp};
pub use region::{
    Region, RegionBox, RegionComputation, RegionEffect, RegionStats, RegionVec, ScopedAllocator,
    with_region, with_region_capacity,
};
pub use session::{Either, End, Offer, Protocol, Receive, Select, Send, Session, SessionEffect};
pub use state::{StateEffect, StateOp};
pub use writer::{WriterEffect, WriterOp};

#[cfg(feature = "std")]
pub use probabilistic::{
    Bernoulli, Categorical, Distribution, Exponential, InferenceResult, Normal, ProbComputation,
    ProbContext, ProbabilisticEffect, Uniform, importance_sample, likelihood_weighting,
};
