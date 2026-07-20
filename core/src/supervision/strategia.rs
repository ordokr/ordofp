//! Supervision Strategies
//!
//! > *"Strategia est ars belli"*
//! > — Strategy is the art of war. (Latin)
//!
//! This module defines the restart strategies for supervision trees.

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::time::Duration;

// =============================================================================
// Restart Intensity
// =============================================================================

/// Tracks restart attempts within a time window.
///
/// Used to detect excessive restart rates and trigger escalation.
///
/// # Latin Etymology
/// *Intensitas restitutio* = restart intensity.
#[derive(Debug, Clone)]
pub struct IntensitasRestitutio {
    /// Maximum restarts allowed in the window.
    max_restarts: u32,
    /// Time window in seconds.
    window_secs: u64,
    /// Timestamps of recent restarts (Unix epoch seconds).
    restart_times: Vec<u64>,
}

impl IntensitasRestitutio {
    /// Create a new restart intensity tracker.
    pub fn new(max_restarts: u32, window_secs: u64) -> Self {
        IntensitasRestitutio {
            max_restarts,
            window_secs,
            restart_times: Vec::with_capacity(max_restarts as usize),
        }
    }

    /// Record a restart and check if intensity is exceeded.
    ///
    /// Returns `true` if the restart is allowed, `false` if intensity exceeded.
    #[inline]
    pub fn record_restart(&mut self, current_time: u64) -> bool {
        // Remove old restart times outside the window
        let cutoff = current_time.saturating_sub(self.window_secs);
        self.restart_times.retain(|&t| t >= cutoff);

        // Check if we've exceeded the limit
        if self.restart_times.len() >= self.max_restarts as usize {
            return false;
        }

        // Record this restart
        self.restart_times.push(current_time);
        true
    }

    /// Get the current restart count within the window.
    #[inline]
    pub fn current_count(&self) -> usize {
        self.restart_times.len()
    }

    /// Get the configured intensity window, in seconds.
    #[inline]
    pub fn window_secs(&self) -> u64 {
        self.window_secs
    }

    /// Reset the restart counter.
    #[inline]
    pub fn reset(&mut self) {
        self.restart_times.clear();
    }
}

impl Default for IntensitasRestitutio {
    fn default() -> Self {
        // Default: 3 restarts per 60 seconds
        Self::new(3, 60)
    }
}

// =============================================================================
// Supervision Strategy
// =============================================================================

/// Supervision strategy for handling child failures.
///
/// # Latin Etymology
/// *Strategia supervisionis* = supervision strategy.
#[derive(Debug, Clone)]
pub enum StrategiaSupervisionis {
    /// One-for-one: Only restart the failed child.
    ///
    /// *Unus pro uno* = one for one.
    UnusProUno {
        /// Restart intensity limits.
        intensity: IntensitasRestitutio,
    },

    /// All-for-one: Restart all children when one fails.
    ///
    /// *Omnes pro uno* = all for one.
    OmnesProUno {
        /// Restart intensity limits.
        intensity: IntensitasRestitutio,
    },

    /// Rest-for-one: Restart failed child and all children started after it.
    ///
    /// *Reliqui pro uno* = the rest for one.
    ReliquiProUno {
        /// Restart intensity limits.
        intensity: IntensitasRestitutio,
    },

    /// Simple one-for-one without intensity tracking.
    ///
    /// *Simplex* = simple.
    Simplex,

    /// Named strategy with a fixed restart budget.
    ///
    /// *Proprius* = custom, one's own.
    ///
    /// Note: despite the name, there is no user-defined decision *function* —
    /// the variant holds only a label and a restart counter. `decide` treats
    /// `max_restarts` as a countdown (decrementing it on every failure) and
    /// escalates once it reaches zero. A closure-based custom strategy is
    /// possible future work.
    Proprius {
        /// Name of the custom strategy.
        name: String,
        /// Remaining restarts before escalation (mutated by `decide` as a
        /// countdown).
        max_restarts: u32,
    },
}

impl StrategiaSupervisionis {
    /// Create a one-for-one strategy.
    ///
    /// Only the failed child is restarted.
    pub fn unus_pro_uno(max_restarts: u32, window: Duration) -> Self {
        StrategiaSupervisionis::UnusProUno {
            intensity: IntensitasRestitutio::new(max_restarts, window.as_secs()),
        }
    }

    /// Create an all-for-one strategy.
    ///
    /// All children are restarted when one fails.
    pub fn omnes_pro_uno(max_restarts: u32, window: Duration) -> Self {
        StrategiaSupervisionis::OmnesProUno {
            intensity: IntensitasRestitutio::new(max_restarts, window.as_secs()),
        }
    }

    /// Create a rest-for-one strategy.
    ///
    /// Failed child and all children started after it are restarted.
    pub fn reliqui_pro_uno(max_restarts: u32, window: Duration) -> Self {
        StrategiaSupervisionis::ReliquiProUno {
            intensity: IntensitasRestitutio::new(max_restarts, window.as_secs()),
        }
    }

    /// Create a simple strategy (always restart, no limits).
    pub fn simplex() -> Self {
        StrategiaSupervisionis::Simplex
    }

    /// Create a custom strategy.
    pub fn proprius(name: impl Into<String>, max_restarts: u32) -> Self {
        StrategiaSupervisionis::Proprius {
            name: name.into(),
            max_restarts,
        }
    }

    /// Get the configured restart-intensity window, in seconds, if this
    /// strategy tracks one.
    ///
    /// `Simplex` and `Proprius` escalate without a time-windowed intensity
    /// tracker, so they have no window to report.
    #[inline]
    pub fn window_secs(&self) -> Option<u64> {
        match self {
            StrategiaSupervisionis::UnusProUno { intensity }
            | StrategiaSupervisionis::OmnesProUno { intensity }
            | StrategiaSupervisionis::ReliquiProUno { intensity } => Some(intensity.window_secs()),
            StrategiaSupervisionis::Simplex | StrategiaSupervisionis::Proprius { .. } => None,
        }
    }

    /// Get the strategy name.
    #[inline]
    pub fn name(&self) -> &str {
        match self {
            StrategiaSupervisionis::UnusProUno { .. } => "UnusProUno",
            StrategiaSupervisionis::OmnesProUno { .. } => "OmnesProUno",
            StrategiaSupervisionis::ReliquiProUno { .. } => "ReliquiProUno",
            StrategiaSupervisionis::Simplex => "Simplex",
            StrategiaSupervisionis::Proprius { name, .. } => name,
        }
    }

    /// Determine the action to take for a child failure.
    ///
    /// Returns which children to restart (by index) or an escalation action.
    pub fn decide(
        &mut self,
        failed_child_index: usize,
        total_children: usize,
        current_time: u64,
    ) -> RestartDecision {
        match self {
            StrategiaSupervisionis::UnusProUno { intensity } => {
                if intensity.record_restart(current_time) {
                    RestartDecision::Restart(vec![failed_child_index])
                } else {
                    RestartDecision::Escalate
                }
            }

            StrategiaSupervisionis::OmnesProUno { intensity } => {
                if intensity.record_restart(current_time) {
                    RestartDecision::Restart((0..total_children).collect())
                } else {
                    RestartDecision::Escalate
                }
            }

            StrategiaSupervisionis::ReliquiProUno { intensity } => {
                if intensity.record_restart(current_time) {
                    RestartDecision::Restart((failed_child_index..total_children).collect())
                } else {
                    RestartDecision::Escalate
                }
            }

            StrategiaSupervisionis::Simplex => RestartDecision::Restart(vec![failed_child_index]),

            StrategiaSupervisionis::Proprius { max_restarts, .. } => {
                // Simple counter-based approach for custom strategies
                if *max_restarts > 0 {
                    *max_restarts -= 1;
                    RestartDecision::Restart(vec![failed_child_index])
                } else {
                    RestartDecision::Escalate
                }
            }
        }
    }
}

impl Default for StrategiaSupervisionis {
    fn default() -> Self {
        // Default: one-for-one with 3 restarts per minute
        Self::unus_pro_uno(3, Duration::from_mins(1))
    }
}

// =============================================================================
// Restart Decision
// =============================================================================

/// The decision made by a supervision strategy.
///
/// # Latin Etymology
/// *Decisio restitutio* = restart decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RestartDecision {
    /// Restart the specified children (by index).
    Restart(Vec<usize>),

    /// Escalate to the parent supervisor.
    Escalate,

    /// Terminate the supervisor.
    Terminate,

    /// Ignore the failure.
    Ignore,
}

impl RestartDecision {
    /// Check if this decision involves restarting.
    #[inline]
    pub fn is_restart(&self) -> bool {
        matches!(self, RestartDecision::Restart(_))
    }

    /// Check if this decision is an escalation.
    #[inline]
    pub fn is_escalate(&self) -> bool {
        matches!(self, RestartDecision::Escalate)
    }

    /// Get the children to restart, if any.
    #[inline]
    pub fn children_to_restart(&self) -> Option<&[usize]> {
        match self {
            RestartDecision::Restart(children) => Some(children),
            _ => None,
        }
    }
}

// =============================================================================
// Child Restart Type
// =============================================================================

/// How a child should be restarted.
///
/// # Latin Etymology
/// *Modus restitutio* = restart mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ModusRestitutio {
    /// Always restart this child when it terminates.
    ///
    /// *Permanens* = permanent.
    #[default]
    Permanens,

    /// Only restart if the child terminates abnormally.
    ///
    /// *Transiens* = transient.
    Transiens,

    /// Never restart this child.
    ///
    /// *Temporarius* = temporary.
    Temporarius,
}

impl ModusRestitutio {
    /// Check if restart is needed based on termination status.
    #[inline]
    pub fn should_restart(&self, normal_exit: bool) -> bool {
        match self {
            ModusRestitutio::Permanens => true,
            ModusRestitutio::Transiens => !normal_exit,
            ModusRestitutio::Temporarius => false,
        }
    }
}

// =============================================================================
// Shutdown Order
// =============================================================================

/// Order in which children are shut down.
///
/// # Latin Etymology
/// *Ordo terminatio* = shutdown order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OrdoTerminatio {
    /// Shut down in the order they were started (first in, first out).
    ///
    /// *Primus* = first.
    #[default]
    Primus,

    /// Shut down in reverse order (last in, first out).
    ///
    /// *Ultimus* = last.
    Ultimus,

    /// Shut down all at once (parallel).
    ///
    /// *Simul* = simultaneously.
    Simul,
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intensity_within_limit() {
        let mut intensity = IntensitasRestitutio::new(3, 60);

        assert!(intensity.record_restart(100));
        assert!(intensity.record_restart(110));
        assert!(intensity.record_restart(120));
        assert!(!intensity.record_restart(130)); // Exceeds limit
    }

    #[test]
    fn test_intensity_window_expiry() {
        let mut intensity = IntensitasRestitutio::new(3, 60);

        assert!(intensity.record_restart(100));
        assert!(intensity.record_restart(110));
        assert!(intensity.record_restart(120));
        assert!(!intensity.record_restart(130)); // Exceeds limit

        // After window expires
        assert!(intensity.record_restart(200)); // First restart in new window
    }

    #[test]
    fn test_strategy_unus_pro_uno() {
        let mut strategy = StrategiaSupervisionis::unus_pro_uno(2, Duration::from_secs(60));

        let decision = strategy.decide(1, 3, 100);
        assert_eq!(decision, RestartDecision::Restart(vec![1]));

        let decision = strategy.decide(1, 3, 110);
        assert_eq!(decision, RestartDecision::Restart(vec![1]));

        let decision = strategy.decide(1, 3, 120);
        assert_eq!(decision, RestartDecision::Escalate);
    }

    #[test]
    fn test_strategy_omnes_pro_uno() {
        let mut strategy = StrategiaSupervisionis::omnes_pro_uno(1, Duration::from_secs(60));

        let decision = strategy.decide(1, 3, 100);
        assert_eq!(decision, RestartDecision::Restart(vec![0, 1, 2]));
    }

    #[test]
    fn test_strategy_reliqui_pro_uno() {
        let mut strategy = StrategiaSupervisionis::reliqui_pro_uno(1, Duration::from_secs(60));

        let decision = strategy.decide(1, 4, 100);
        assert_eq!(decision, RestartDecision::Restart(vec![1, 2, 3]));
    }

    #[test]
    fn test_modus_restitutio() {
        assert!(ModusRestitutio::Permanens.should_restart(true));
        assert!(ModusRestitutio::Permanens.should_restart(false));

        assert!(!ModusRestitutio::Transiens.should_restart(true));
        assert!(ModusRestitutio::Transiens.should_restart(false));

        assert!(!ModusRestitutio::Temporarius.should_restart(true));
        assert!(!ModusRestitutio::Temporarius.should_restart(false));
    }
}
