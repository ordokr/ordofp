//! Supervision Trees - Structured Fault Tolerance
//!
//! > *"Arbor supervisio est custos infantum"*
//! > — The supervision tree is the guardian of children. (Neo-Latin)
//!
//! This module provides Erlang-style supervision trees for fault-tolerant systems,
//! integrating with the effect system for structured error handling.
//!
//! # Overview
//!
//! Supervision trees provide a hierarchical structure where parent supervisors
//! manage child workers and sub-supervisors. When a child fails, the supervisor
//! applies a restart strategy to recover.
//!
//! # Scholastic Naming
//!
//! | English | Latin | Etymology |
//! |---------|-------|-----------|
//! | Supervision Tree | Arbor | *arbor* = tree |
//! | Child | Infans | *infans* = child, offspring |
//! | Strategy | Strategia | *strategia* = generalship, strategy |
//! | Restart | Restitutio | *restitutio* = restoration |
//! | Failure | Defectio | *defectio* = failure, defection |
//! | Escalate | Escalare | *escalare* = to climb, escalate |
//!
//! # Supervision Strategies
//!
//! - **`UnusProUno`** (One-for-One): Only the failed child is restarted
//! - **`OmnesProUno`** (All-for-One): All children are restarted when one fails
//! - **`ReliquiProUno`** (Rest-for-One): Failed child and all children started after it are restarted
//!
//! # Example
//!
//! Note: this module models supervision *decisions* — there is no executor
//! that spawns or restarts tasks. You describe the tree, then drive it by
//! reporting failures and applying the returned restart actions yourself.
//!
//! ```rust
//! use ordofp_core::supervision::{Arbor, StrategiaSupervisionis, InfansSpec, ActioRestitutio, SupervisioError};
//! use core::time::Duration;
//!
//! let mut supervisor = Arbor::new("root")
//!     .with_strategy(StrategiaSupervisionis::unus_pro_uno(3, Duration::from_secs(60)))
//!     .add_child(InfansSpec::worker("worker1"))
//!     .add_child(InfansSpec::worker("worker2"));
//!
//! // The tree must be initialized before failures can be reported.
//! supervisor.initialize()?;
//!
//! // When child 0 fails at t=now, ask the tree which restart action to take
//! // (the third argument is `normal_exit`, distinguishing a clean exit from
//! // an abnormal one for `ModusRestitutio::Transiens` children):
//! let now_secs = 1_700_000_000u64;
//! let action = supervisor.handle_failure(0, now_secs, false)?;
//! assert_eq!(action, ActioRestitutio::Restituere);
//! # Ok::<(), SupervisioError>(())
//! ```

mod arbor;
mod infans;
mod strategia;

pub use arbor::*;
pub use infans::*;
pub use strategia::*;

use alloc::string::String;

// =============================================================================
// Supervision Error
// =============================================================================

/// Error type for supervision operations.
///
/// # Latin Etymology
/// *Defectio* = failure, defection.
#[derive(Debug, Clone)]
pub enum SupervisioError {
    /// Maximum restart intensity exceeded.
    ///
    /// *Intensitas excedit* = intensity exceeds.
    IntensitasExcedit {
        /// ID of the child whose restart tripped the limit.
        child_id: String,
        /// Number of restarts recorded for that child so far.
        restarts: u32,
        /// Length of the sliding intensity window, in seconds.
        window_secs: u64,
    },

    /// Child failed to start.
    ///
    /// *Initium defectum* = start failed.
    InitiumDefectum {
        /// ID of the child that failed to start.
        child_id: String,
        /// Human-readable description of why startup failed.
        reason: String,
    },

    /// Child terminated abnormally.
    ///
    /// *Terminatio abnormis* = abnormal termination.
    TerminatioAbnormis {
        /// ID of the child that terminated.
        child_id: String,
        /// Human-readable description of the abnormal termination.
        reason: String,
    },

    /// Supervisor was stopped.
    ///
    /// *Supervisio terminata* = supervision terminated.
    SupervisioTerminata,

    /// Universalis supervision error.
    Alius(String),
}

impl core::fmt::Display for SupervisioError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SupervisioError::IntensitasExcedit {
                child_id,
                restarts,
                window_secs,
            } => {
                write!(
                    f,
                    "restart intensity exceeded for '{child_id}': {restarts} restarts in {window_secs}s"
                )
            }
            SupervisioError::InitiumDefectum { child_id, reason } => {
                write!(f, "child '{child_id}' failed to start: {reason}")
            }
            SupervisioError::TerminatioAbnormis { child_id, reason } => {
                write!(f, "child '{child_id}' terminated abnormally: {reason}")
            }
            SupervisioError::SupervisioTerminata => {
                write!(f, "supervisor was terminated")
            }
            SupervisioError::Alius(msg) => {
                write!(f, "supervision error: {msg}")
            }
        }
    }
}

/// Result type for supervision operations.
pub type SupervisioResult<T> = Result<T, SupervisioError>;

// =============================================================================
// Restart Action
// =============================================================================

/// Action to take after a child failure.
///
/// # Latin Etymology
/// *Actio restitutio* = restart action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActioRestitutio {
    /// Restart the failed child(ren).
    ///
    /// *Restituere* = to restore.
    Restituere,

    /// Stop the supervisor (escalate to parent).
    ///
    /// *Escalare* = to escalate.
    Escalare,

    /// Stop the supervisor without escalation.
    ///
    /// *Terminare* = to terminate.
    Terminare,

    /// Ignore the failure and continue.
    ///
    /// *Ignorare* = to ignore.
    Ignorare,
}

// =============================================================================
// Child Status
// =============================================================================

/// Status of a supervised child.
///
/// # Latin Etymology
/// *Status infantis* = child status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusInfantis {
    /// Child is starting.
    ///
    /// *Incipiens* = beginning.
    Incipiens,

    /// Child is running.
    ///
    /// *Currens* = running.
    Currens,

    /// Child is restarting.
    ///
    /// *Restituens* = restoring.
    Restituens,

    /// Child has stopped normally.
    ///
    /// *Terminatus* = terminated.
    Terminatus,

    /// Child has failed.
    ///
    /// *Defectus* = failed.
    Defectus,
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supervisio_error_display() {
        let err = SupervisioError::IntensitasExcedit {
            child_id: "worker1".into(),
            restarts: 5,
            window_secs: 60,
        };
        let display = alloc::format!("{err}");
        assert!(display.contains("worker1"));
        assert!(display.contains('5'));
    }

    #[test]
    fn test_actio_restitutio() {
        assert_eq!(ActioRestitutio::Restituere, ActioRestitutio::Restituere);
        assert_ne!(ActioRestitutio::Restituere, ActioRestitutio::Escalare);
    }

    #[test]
    fn test_status_infantis() {
        assert_eq!(StatusInfantis::Currens, StatusInfantis::Currens);
        assert_ne!(StatusInfantis::Currens, StatusInfantis::Defectus);
    }
}
