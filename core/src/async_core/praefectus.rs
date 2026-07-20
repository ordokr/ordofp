//! Fiber Supervision - Praefectus
//!
//! > *"Praefectus vigilat super omnes"*
//! > — The supervisor watches over all. (Latin)
//!
//! This module provides supervision strategies for fiber hierarchies,
//! inspired by Erlang's OTP supervision trees and ZIO's fiber supervision.
//!
//! # Overview
//!
//! Supervisors manage collections of fibers, handling their lifecycle,
//! failures, and restarts according to configurable strategies.
//!
//! # Scholastic Naming
//!
//! | English | Latin | Etymology |
//! |---------|-------|-----------|
//! | Supervisor | Praefectus | *praefectus* = overseer, commander |
//! | Strategy | Strategia | *strategia* = generalship |
//! | Restart | Renovare | *renovare* = to renew |
//! | Stop | Sistere | *sistere* = to stop |
//! | Child | Infans | *infans* = child |
//!
//! # Example
//!
//! ```rust
//! use ordofp_core::async_core::praefectus::{Praefectus, StrategiaRestart};
//! use ordofp_core::async_core::runtime::TokioRuntime;
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut supervisor = Praefectus::<TokioRuntime>::new(StrategiaRestart::UnusProUno);
//!     supervisor.add_child_fn("worker1", || async { /* work */ });
//!
//!     // Request an immediate shutdown so this example terminates deterministically.
//!     supervisor.stop_handle().abrogare();
//!     supervisor.start().await;
//! }
//! ```

// =============================================================================
// Default Constants
// =============================================================================

/// Default shutdown timeout for child fibers, in milliseconds.
pub const DEFAULT_SHUTDOWN_TIMEOUT_MS: u64 = 5000;

/// Default maximum number of restarts within the intensity window.
pub const DEFAULT_MAX_RESTARTS: u32 = 3;

/// Default time window (in seconds) for restart intensity tracking.
pub const DEFAULT_RESTART_WITHIN_SECONDS: u32 = 5;

/// Default initial delay (in milliseconds) for exponential backoff.
pub const DEFAULT_EXPONENS_INITIAL_MS: u64 = 100;

/// Default maximum delay (in milliseconds) for exponential backoff.
pub const DEFAULT_EXPONENS_MAX_MS: u64 = 30000;

/// Default multiplication factor for exponential backoff.
pub const DEFAULT_EXPONENS_FACTOR: f64 = 2.0;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use super::fibra::{Fibra, FibraError, FibraManubrium, TesseraAbrogationis};
use super::runtime::RuntimeGenerare;

// =============================================================================
// Supervision Strategies
// =============================================================================

/// Restart strategy for supervised fibers.
///
/// # Latin Etymology
/// *Strategia* = strategy, plan (from Greek *strategia*).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StrategiaRestart {
    /// If one child fails, only restart that child.
    ///
    /// *Unus pro uno* = one for one.
    #[default]
    UnusProUno,

    /// If one child fails, restart all children.
    ///
    /// *Omnes pro uno* = all for one.
    OmnesProUno,

    /// If one child fails, restart it and all children started after it.
    ///
    /// *Reliqui pro uno* = rest for one.
    ReliquiProUno,

    /// Never restart; let failures propagate.
    ///
    /// *Nullus* = none.
    Nullus,
}

/// Restart intensity limits.
///
/// Defines how many restarts are allowed in a given time period.
#[derive(Debug, Clone, Copy)]
pub struct IntensitasRestart {
    /// Maximum number of restarts.
    pub max_restarts: u32,
    /// Time window in seconds.
    pub within_seconds: u32,
}

impl Default for IntensitasRestart {
    fn default() -> Self {
        IntensitasRestart {
            max_restarts: DEFAULT_MAX_RESTARTS,
            within_seconds: DEFAULT_RESTART_WITHIN_SECONDS,
        }
    }
}

// =============================================================================
// Child Specification
// =============================================================================

/// Specification for a supervised child fiber.
///
/// # Latin Etymology
/// *Specificatio* = specification, description.
pub struct InfansSpecificatio {
    /// Unique identifier for the child.
    pub nomen: String,
    /// Factory function to create the child fiber.
    pub fabricare: Arc<dyn Fn() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>,
    /// Whether to restart on failure.
    pub restart: bool,
    /// Shutdown timeout in milliseconds.
    pub shutdown_timeout_ms: u64,
}

impl InfansSpecificatio {
    /// Create a new child specification.
    pub fn new<F, Fut>(nomen: impl Into<String>, fabricare: F) -> Self
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        InfansSpecificatio {
            nomen: nomen.into(),
            fabricare: Arc::new(move || Box::pin(fabricare())),
            restart: true,
            shutdown_timeout_ms: DEFAULT_SHUTDOWN_TIMEOUT_MS,
        }
    }

    /// Set whether to restart on failure.
    #[inline]
    pub fn with_restart(mut self, restart: bool) -> Self {
        self.restart = restart;
        self
    }

    /// Set shutdown timeout.
    #[inline]
    pub fn with_shutdown_timeout(mut self, timeout_ms: u64) -> Self {
        self.shutdown_timeout_ms = timeout_ms;
        self
    }
}

// =============================================================================
// Supervised Child State
// =============================================================================

/// Current state of a supervised child.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusInfans {
    /// Child is starting.
    Incipit,
    /// Child is running.
    Currens,
    /// Child has stopped normally.
    Perfectus,
    /// Child has failed.
    Defectus,
    /// Child is being restarted.
    Renovatur,
    /// Child was cancelled during supervisor shutdown or escalation.
    ///
    /// *Terminatus* = terminated, ended.
    Terminatus,
}

// =============================================================================
// Praefectus - Supervisor
// =============================================================================

/// A supervisor that manages a collection of child fibers.
///
/// # Latin Etymology
/// *Praefectus* = one placed in charge, overseer, commander.
///
/// # Type Parameters
///
/// - `R`: The runtime to use for spawning fibers
///
/// # Restart policy
///
/// Restart-on-failure only: a child that completes normally (`Ok`) is left in
/// the [`StatusInfans::Perfectus`] state and is **not** restarted. Only a child
/// that fails (panics, is cancelled, or otherwise returns `Err`) is a candidate
/// for restart, subject to the per-child [`InfansSpecificatio::restart`] flag,
/// the [`IntensitasRestart`] gate, and the [`StrategiaRestart`] fan-out.
///
/// # Example
///
/// ```rust
/// use ordofp_core::async_core::praefectus::{Praefectus, StrategiaRestart};
/// use ordofp_core::async_core::runtime::TokioRuntime;
///
/// let mut supervisor = Praefectus::<TokioRuntime>::new(StrategiaRestart::UnusProUno);
/// supervisor.add_child_fn("worker", || async { /* work */ });
/// assert_eq!(supervisor.child_count(), 1);
/// ```
pub struct Praefectus<R: RuntimeGenerare> {
    /// Restart strategy.
    strategia: StrategiaRestart,
    /// Restart intensity limits.
    intensitas: IntensitasRestart,
    /// Child specifications (not yet started).
    specs: Vec<InfansSpecificatio>,
    /// Whether the supervisor is running.
    running: Arc<AtomicBool>,
    /// Total restart count.
    restart_count: Arc<AtomicU32>,
    /// Stop token: waking the supervision loop and requesting shutdown.
    ///
    /// `stop()` both clears `running` and wakes the loop through this token, so
    /// a supervisor parked on the event `poll_fn` re-polls immediately instead
    /// of waiting for the next child exit.
    sistendum: Arc<TesseraAbrogationis>,
    _runtime: PhantomData<R>,
}

impl<R: RuntimeGenerare> Praefectus<R> {
    /// Create a new supervisor with the given restart strategy.
    #[inline]
    pub fn new(strategia: StrategiaRestart) -> Self {
        Praefectus {
            strategia,
            intensitas: IntensitasRestart::default(),
            // Supervisors typically hold a small, known set of children.
            specs: Vec::with_capacity(8),
            running: Arc::new(AtomicBool::new(false)),
            restart_count: Arc::new(AtomicU32::new(0)),
            sistendum: Arc::new(TesseraAbrogationis::default()),
            _runtime: PhantomData,
        }
    }

    /// Set the restart intensity limits.
    #[inline]
    pub fn with_intensitas(mut self, intensitas: IntensitasRestart) -> Self {
        self.intensitas = intensitas;
        self
    }

    /// Add a child specification.
    #[inline]
    pub fn add_child(&mut self, spec: InfansSpecificatio) {
        self.specs.push(spec);
    }

    /// Add a child from a factory function.
    #[inline]
    pub fn add_child_fn<F, Fut>(&mut self, nomen: impl Into<String>, f: F)
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.add_child(InfansSpecificatio::new(nomen, f));
    }

    /// Get the restart strategy.
    #[inline]
    pub fn strategia(&self) -> StrategiaRestart {
        self.strategia
    }

    /// Get the number of children.
    #[inline]
    pub fn child_count(&self) -> usize {
        self.specs.len()
    }

    /// Check if the supervisor is running.
    #[inline]
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Stop the supervisor.
    ///
    /// Clears the running flag **and** wakes the supervision loop through the
    /// stop token, so a loop parked on the event `poll_fn` re-polls at once and
    /// begins a joined shutdown. (The old flag-only stop never woke the loop.)
    #[inline]
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
        self.sistendum.abrogare();
    }

    /// Get a handle to the stop token.
    ///
    /// Calling [`abrogare`](TesseraAbrogationis::abrogare) on the returned token
    /// requests a joined shutdown of the supervision loop from anywhere,
    /// including after ownership of the `Praefectus` has been moved into
    /// [`start`](Self::start).
    #[inline]
    pub fn stop_handle(&self) -> Arc<TesseraAbrogationis> {
        self.sistendum.clone()
    }

    /// Start all children and begin supervision.
    ///
    /// Returns a fiber that runs the supervision loop.
    // One linear supervision loop; splitting it would scatter the restart
    // policy across helpers without making it clearer.
    #[allow(clippy::too_many_lines)]
    pub fn start(self) -> Fibra<()> {
        let running = self.running.clone();
        let restart_count = self.restart_count.clone();
        let strategia = self.strategia;
        let intensitas = self.intensitas;
        let sistendum = self.sistendum.clone();
        let specs = self.specs;

        running.store(true, Ordering::SeqCst);

        Fibra::new(async move {
            // Start all children - create specs with initial state
            let mut children: Vec<(
                InfansSpecificatio,
                Option<FibraManubrium<()>>,
                StatusInfans,
                u32,
            )> = specs
                .into_iter()
                .map(|spec| (spec, None, StatusInfans::Incipit, 0u32))
                .collect();

            // Spawn initial children
            for (spec, handle, status, _restart_count) in &mut children {
                let fut = (spec.fabricare)();
                let fibra = Fibra::new(fut);
                let id = fibra.id();
                let cancelled = fibra.cancellation_token();
                let join_handle = R::spawn(fibra);
                *handle = Some(FibraManubrium::new(id, join_handle, cancelled));
                *status = StatusInfans::Currens;
            }

            // Restart-intensity tracking, mirroring
            // `supervision::IntensitasRestitutio::record_restart`: keep the
            // timestamps of recent restarts and escalate once the count within
            // the window reaches `max_restarts`. (The public `IntensitasRestart`
            // is a plain `Copy` config, so the sliding window lives here.)
            let max_restarts = intensitas.max_restarts as usize;
            let window_secs = u64::from(intensitas.within_seconds);
            let mut restart_times: Vec<u64> = Vec::with_capacity(max_restarts);
            let origo = std::time::Instant::now();

            loop {
                // Wait for: any running child to exit, or a stop request. Register
                // the stop waker FIRST (same race-free ordering as `Fibra::poll`),
                // then check stop, then poll children via `poll_conjunctio` — which
                // registers each child's exit waker. Returns `Pending` when nothing
                // has happened, so the loop parks instead of busy-spinning.
                let event = core::future::poll_fn(|cx| {
                    use core::task::Poll;
                    sistendum.registrare(cx);
                    if sistendum.est_abrogata() || !running.load(Ordering::SeqCst) {
                        return Poll::Ready(None);
                    }
                    for (i, (_spec, handle, status, _rc)) in children.iter_mut().enumerate() {
                        if *status == StatusInfans::Currens
                            && let Some(h) = handle.as_mut()
                            && let Poll::Ready(exitus) = h.poll_conjunctio(cx)
                        {
                            return Poll::Ready(Some((i, exitus)));
                        }
                    }
                    Poll::Pending
                })
                .await;

                let Some((idx, exitus)) = event else { break }; // stop requested

                // The child has exited; its handle is spent.
                children[idx].1 = None;
                if exitus.is_ok() {
                    // Normal completion: a finished worker is done, not failed.
                    children[idx].2 = StatusInfans::Perfectus;
                    continue;
                }

                // Failure. Honor the per-child restart flag (the ModusRestitutio-
                // like field): a child marked non-restartable stays failed.
                if !children[idx].0.restart {
                    children[idx].2 = StatusInfans::Defectus;
                    continue;
                }

                // Determine the restart set per strategy, mirroring
                // `supervision::StrategiaSupervisionis::decide`'s index mapping
                // (strategia.rs:195-238).
                let len = children.len();
                let indices: Vec<usize> = match strategia {
                    StrategiaRestart::UnusProUno => alloc::vec![idx],
                    StrategiaRestart::OmnesProUno => (0..len).collect(),
                    StrategiaRestart::ReliquiProUno => (idx..len).collect(),
                    // Never restart; let the failure stand.
                    StrategiaRestart::Nullus => Vec::new(),
                };
                if indices.is_empty() {
                    children[idx].2 = StatusInfans::Defectus;
                    continue;
                }

                // Gate on restart intensity (sliding window).
                let now_secs = origo.elapsed().as_secs();
                let cutoff = now_secs.saturating_sub(window_secs);
                restart_times.retain(|&t| t >= cutoff);
                if restart_times.len() >= max_restarts {
                    // Escalate: intensity exceeded — cancel AND JOIN all children,
                    // then end the supervisor.
                    for (_s, h, st, _rc) in &mut children {
                        if let Some(handle) = h.take() {
                            handle.cancel();
                            let _ = handle.conjungere().await; // cancel now wakes (Task 13)
                        }
                        *st = StatusInfans::Terminatus;
                    }
                    break;
                }
                restart_times.push(now_secs);

                // Apply the strategy: cancel-and-join any still-running member of
                // the restart set, then respawn it into the same slot (indexes are
                // stable across respawns — slots are never added or removed).
                for i in indices {
                    if let Some(old) = children[i].1.take() {
                        old.cancel();
                        let _ = old.conjungere().await;
                    }
                    let fut = (children[i].0.fabricare)();
                    let fibra = Fibra::new(fut);
                    let id = fibra.id();
                    let token = fibra.cancellation_token();
                    let join_handle = R::spawn(fibra);
                    children[i].1 = Some(FibraManubrium::new(id, join_handle, token));
                    children[i].2 = StatusInfans::Currens;
                    children[i].3 += 1;
                    restart_count.fetch_add(1, Ordering::SeqCst);
                }
            }

            // Shutdown: cancel AND join every child (the old code cancelled
            // without joining, leaking running children).
            for (_spec, handle, status, _rc) in &mut children {
                if let Some(h) = handle.take() {
                    h.cancel();
                    let _ = h.conjungere().await;
                }
                *status = StatusInfans::Terminatus;
            }

            // The supervision loop has now actually terminated (both the
            // escalation `break` path and the normal stop-request path land
            // here), so clear `running` — otherwise `is_running()` kept
            // reporting `true` after the supervisor was long gone.
            running.store(false, Ordering::SeqCst);
        })
    }
}

impl<R: RuntimeGenerare> Default for Praefectus<R> {
    fn default() -> Self {
        Self::new(StrategiaRestart::UnusProUno)
    }
}

// =============================================================================
// Supervision Events
// =============================================================================

/// Events that can occur during supervision.
///
/// # Latin Etymology
/// *Eventus* = outcome, event.
#[derive(Debug, Clone)]
pub enum EventusSupervisio {
    /// A child was started.
    InfansIncepit {
        /// Name of the child, as given in its spec.
        nomen: String,
    },
    /// A child completed normally.
    InfansPerfectus {
        /// Name of the child, as given in its spec.
        nomen: String,
    },
    /// A child failed.
    InfansDefectus {
        /// Name of the child, as given in its spec.
        nomen: String,
        /// Human-readable rendering of the failure that took the child
        /// down; diagnostic only, not intended for programmatic matching.
        error: String,
    },
    /// A child is being restarted.
    InfansRenovatur {
        /// Name of the child, as given in its spec.
        nomen: String,
        /// Restart attempt number for this child (increments with each
        /// restart; feeds [`StrategiaMora::delay_for_attempt`]).
        attempt: u32,
    },
    /// Restart limit exceeded.
    LimesExcessus {
        /// Name of the child whose failure tipped the restart intensity
        /// over the configured limit.
        nomen: String,
    },
    /// Supervisor is shutting down.
    SupervisorSistit,
}

// =============================================================================
// SupervisionPolicy - Error Handling Policy
// =============================================================================

/// Policy for handling child failures.
///
/// # Latin Etymology
/// *Politia* = policy, government.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PolitiaDefectus {
    /// Restart the child.
    #[default]
    Renovare,
    /// Stop the child permanently.
    Sistere,
    /// Escalate to parent supervisor.
    Escalare,
    /// Resume (ignore the error).
    Resumere,
}

/// Decider function type for determining policy.
pub type DeciderDefectus = Box<dyn Fn(&FibraError) -> PolitiaDefectus + Send + Sync>;

/// Create a default decider that always restarts.
#[inline]
pub fn default_decider() -> DeciderDefectus {
    Box::new(|_| PolitiaDefectus::Renovare)
}

// =============================================================================
// Backoff Strategies
// =============================================================================

/// Backoff strategy for restarts.
///
/// # Latin Etymology
/// *Mora* = delay.
#[derive(Debug, Clone, Copy)]
pub enum StrategiaMora {
    /// No delay between restarts.
    Nulla,
    /// Constant delay.
    Constans {
        /// Fixed delay before every restart, in milliseconds.
        delay_ms: u64,
    },
    /// Exponential backoff: `initial_ms * factor^attempt`, capped at
    /// `max_ms`.
    Exponens {
        /// Delay before the first restart (attempt 0), in milliseconds.
        initial_ms: u64,
        /// Upper bound on the computed delay, in milliseconds; growth
        /// saturates here.
        max_ms: u64,
        /// Multiplier applied per attempt (e.g. `2.0` doubles the delay
        /// each restart). Values below `1.0` shrink the delay instead.
        factor: f64,
    },
    /// Linear backoff: `initial_ms + increment_ms * attempt`, capped at
    /// `max_ms`.
    Linearis {
        /// Delay before the first restart (attempt 0), in milliseconds.
        initial_ms: u64,
        /// Amount added to the delay on each subsequent attempt, in
        /// milliseconds.
        increment_ms: u64,
        /// Upper bound on the computed delay, in milliseconds; growth
        /// saturates here.
        max_ms: u64,
    },
}

impl Default for StrategiaMora {
    fn default() -> Self {
        StrategiaMora::Exponens {
            initial_ms: DEFAULT_EXPONENS_INITIAL_MS,
            max_ms: DEFAULT_EXPONENS_MAX_MS,
            factor: DEFAULT_EXPONENS_FACTOR,
        }
    }
}

impl StrategiaMora {
    /// Calculate the delay for a given attempt number.
    #[inline]
    pub fn delay_for_attempt(&self, attempt: u32) -> u64 {
        match self {
            StrategiaMora::Nulla => 0,
            StrategiaMora::Constans { delay_ms } => *delay_ms,
            StrategiaMora::Exponens {
                initial_ms,
                max_ms,
                factor,
            } => {
                let delay = (*initial_ms as f64) * factor.powi(attempt as i32);
                (delay as u64).min(*max_ms)
            }
            StrategiaMora::Linearis {
                initial_ms,
                increment_ms,
                max_ms,
            } => {
                let delay = initial_ms + (increment_ms * u64::from(attempt));
                delay.min(*max_ms)
            }
        }
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Create a supervisor with one-for-one strategy.
#[inline]
pub fn supervisor_unus_pro_uno<R: RuntimeGenerare>() -> Praefectus<R> {
    Praefectus::new(StrategiaRestart::UnusProUno)
}

/// Create a supervisor with all-for-one strategy.
#[inline]
pub fn supervisor_omnes_pro_uno<R: RuntimeGenerare>() -> Praefectus<R> {
    Praefectus::new(StrategiaRestart::OmnesProUno)
}

/// Create a supervisor with rest-for-one strategy.
#[inline]
pub fn supervisor_reliqui_pro_uno<R: RuntimeGenerare>() -> Praefectus<R> {
    Praefectus::new(StrategiaRestart::ReliquiProUno)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::async_core::runtime::NullRuntime;

    #[test]
    fn test_strategia_restart_default() {
        assert_eq!(StrategiaRestart::default(), StrategiaRestart::UnusProUno);
    }

    #[test]
    fn test_intensitas_default() {
        let i = IntensitasRestart::default();
        assert_eq!(i.max_restarts, 3);
        assert_eq!(i.within_seconds, 5);
    }

    #[test]
    fn test_praefectus_new() {
        let sup: Praefectus<NullRuntime> = Praefectus::new(StrategiaRestart::OmnesProUno);
        assert_eq!(sup.strategia(), StrategiaRestart::OmnesProUno);
        assert_eq!(sup.child_count(), 0);
        assert!(!sup.is_running());
    }

    #[test]
    fn test_praefectus_add_child() {
        let mut sup: Praefectus<NullRuntime> = Praefectus::new(StrategiaRestart::UnusProUno);
        sup.add_child_fn("test", || async {});
        assert_eq!(sup.child_count(), 1);
    }

    #[test]
    fn test_strategia_mora_nulla() {
        let s = StrategiaMora::Nulla;
        assert_eq!(s.delay_for_attempt(0), 0);
        assert_eq!(s.delay_for_attempt(10), 0);
    }

    #[test]
    fn test_strategia_mora_constans() {
        let s = StrategiaMora::Constans { delay_ms: 1000 };
        assert_eq!(s.delay_for_attempt(0), 1000);
        assert_eq!(s.delay_for_attempt(10), 1000);
    }

    #[test]
    fn test_strategia_mora_exponens() {
        let s = StrategiaMora::Exponens {
            initial_ms: 100,
            max_ms: 10000,
            factor: 2.0,
        };
        assert_eq!(s.delay_for_attempt(0), 100);
        assert_eq!(s.delay_for_attempt(1), 200);
        assert_eq!(s.delay_for_attempt(2), 400);
        assert_eq!(s.delay_for_attempt(10), 10000); // capped at max
    }

    #[test]
    fn test_strategia_mora_linearis() {
        let s = StrategiaMora::Linearis {
            initial_ms: 100,
            increment_ms: 100,
            max_ms: 500,
        };
        assert_eq!(s.delay_for_attempt(0), 100);
        assert_eq!(s.delay_for_attempt(1), 200);
        assert_eq!(s.delay_for_attempt(2), 300);
        assert_eq!(s.delay_for_attempt(10), 500); // capped at max
    }

    #[test]
    fn test_politia_defectus_default() {
        assert_eq!(PolitiaDefectus::default(), PolitiaDefectus::Renovare);
    }

    #[test]
    fn test_status_infans() {
        assert_eq!(StatusInfans::Currens, StatusInfans::Currens);
        assert_ne!(StatusInfans::Currens, StatusInfans::Defectus);
    }

    /// Regression: `start()`'s supervision loop used to fall off the end of
    /// its shutdown tail without clearing `running`, so `is_running()` kept
    /// reporting `true` after the supervisor had actually terminated.
    ///
    /// `start(self)` consumes the supervisor, so `is_running()` can't be
    /// polled through the public API afterward; `running` is private but
    /// this `tests` module is a descendant of the defining module, so it can
    /// read the same `Arc<AtomicBool>` the moved-away `Praefectus` shares
    /// with its supervision fiber.
    #[test]
    fn is_running_false_after_supervision_loop_terminates() {
        use core::future::Future;
        use core::pin::Pin;
        use core::task::{Context, Poll, Waker};

        // Zero children: `start()`'s only `R::spawn` call sites (initial
        // spawn and restart loops) are both skipped, so `NullRuntime`'s
        // panicking `spawn` is never reached.
        let sup: Praefectus<NullRuntime> = Praefectus::new(StrategiaRestart::UnusProUno);
        let running = sup.running.clone();
        sup.stop_handle().abrogare(); // request shutdown before the first poll
        assert!(!running.load(Ordering::SeqCst));

        let mut fiber = sup.start();
        assert!(running.load(Ordering::SeqCst)); // start() flips it true

        let waker = Waker::noop();
        let mut cx = Context::from_waker(waker);
        let poll = Pin::new(&mut fiber).poll(&mut cx);
        assert!(
            matches!(poll, Poll::Ready(_)),
            "expected the zero-child, pre-stopped supervision loop to resolve on the first poll"
        );

        assert!(
            !running.load(Ordering::SeqCst),
            "is_running() must report false once the supervision loop has terminated"
        );
    }
}
