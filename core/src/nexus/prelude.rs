//! `OrdoFP` Nexus Prelude
//!
//! Convenient re-exports for common Nexus types and functions.
//!
//! # Usage
//!
//! ```rust
//! use ordofp_core::nexus::prelude::*;
//!
//! // Now you have access to all common Nexus types
//! let comp: Eff<Pure, i32> = pure(42);
//! assert_eq!(comp.run_pure(), 42);
//! ```

// =============================================================================
// Core Types
// =============================================================================

/// The central effectful computation type.
pub use super::effect::Eff;

/// Marker trait for effects.
pub use super::effect::EffectMarker;

// =============================================================================
// Effect Markers
// =============================================================================

pub use super::effect::{Error, ErrorEff, IO, IoEff, Reader, ReaderEff, State, StateEff};

// =============================================================================
// Effect Rows
// =============================================================================

pub use super::row::{EffectRow, Pure, Row};

// Row type aliases
pub use super::row::{
    AsyncRow, ErrorRow, FullSyncRow, IoErrorRow, IoRow, IoStateRow, ReaderRow, StateRow, WriterRow,
};

// Row operations
pub use super::row::{row_eq, row_subset};

// Bit constants (for custom effects)
pub use super::row::{
    ASYNC_BIT, CHOICE_BIT, ERROR_BIT, IO_BIT, NONDET_BIT, READER_BIT, STATE_BIT, USER_EFFECT_START,
    WRITER_BIT,
};

// =============================================================================
// Representations
// =============================================================================

pub use super::repr::{ContRepr, EffRepr, ErrorRepr, PureRepr, ReaderRepr, StateRepr};

// =============================================================================
// Handlers
// =============================================================================

pub use super::handler::{
    ErrorHandler, Handler, InlineHandler, ReaderHandler, StateHandler, handle, run_error,
    run_reader, run_state,
};

// =============================================================================
// Operations
// =============================================================================

pub use super::ops::{
    apply2, discard, if_else, iterate_until, lift, perform, pure, replicate, sequence,
    sequence_vec, traverse, tuple, unless, when,
};

// =============================================================================
// Concrete Effect Implementations
// =============================================================================

// State
pub use super::effects::state::{
    StateEffect, StateOp, StatefulComputation, state_get, state_gets, state_modify, state_put,
};

// Reader
pub use super::effects::reader::{
    ReaderComputation, ReaderEffect, ReaderOp, reader_ask, reader_asks,
};

// Writer
pub use super::effects::writer::{Monoid, WriterComputation, WriterEffect, WriterOp, writer_tell};

// Error
pub use super::effects::error::{
    ErrorComputation, ErrorEffect, ErrorOp, error_ok, error_throw, first_success,
    partition_results, sequence_results,
};

// IO
pub use super::effects::io::{IoComputation, IoEffect, IoOp, io_perform};

#[cfg(feature = "std")]
pub use super::effects::io::{current_time_millis, delay, print_line, read_line};

// =============================================================================
// Row Type Aliases (Convenience)
// =============================================================================

/// Type alias for pure effect row.
pub type PureRow = Pure;

// =============================================================================
// Effect Row Macro
// =============================================================================

/// Create an effect row from effect types.
///
/// # Example
///
/// ```rust
/// use ordofp_core::nexus::prelude::*;
///
/// struct Config;
/// struct AppError;
///
/// type MyEffects = effect_row![State<i32>, Reader<Config>, Error<AppError>];
/// ```
pub use crate::effect_row;
