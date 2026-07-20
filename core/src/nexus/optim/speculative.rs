//! Speculative Execution Combinators (sequential implementation)
//!
//! This module provides combinators shaped like speculative execution —
//! trying multiple computation branches and using the first successful
//! result. **The current implementation runs branches sequentially, in
//! order**; no threads are spawned and nothing executes in parallel. The
//! combinators still short-circuit on the first success, so they behave as
//! ordered fallback chains today, with parallel execution as possible
//! future work.
//!
//! # When Speculative Execution is Safe
//!
//! Speculative execution is safe when effects are **total** (always terminate)
//! and **commutative** with each other. This ensures:
//!
//! 1. All branches will complete (no hanging)
//! 2. Reordering or parallelizing branches doesn't affect semantics
//!
//! (The branches here use `ErrorComputation`, i.e. the Error effect — "total"
//! refers to termination of each branch, not to the absence of failure.)
//!
//! # Use Cases
//!
//! - **Hedged Requests**: Send same request to multiple servers, use first response
//! - **Algorithm Racing**: Try multiple algorithms, use fastest result
//! - **Fallback Chains**: Try primary, fall back to secondary on failure
//!
//! # Example
//!
//! ```rust
//! use ordofp_core::nexus::effects::error::ErrorComputation;
//! use ordofp_core::nexus::optim::race;
//!
//! // Try multiple computation strategies (sequentially, first listed wins)
//! let result = race(vec![
//!     ErrorComputation::<&str, i32>::ok(100), // e.g. slow_but_accurate()
//!     ErrorComputation::ok(7),                // e.g. fast_approximation()
//!     ErrorComputation::ok(1),                // e.g. cached_result()
//! ]);
//! assert_eq!(result, Some(Ok(100)));
//! ```

use alloc::vec::Vec;

use crate::nexus::effects::error::ErrorComputation;

/// Default maximum retry/attempt count for speculative execution.
pub const DEFAULT_MAX_RETRIES: usize = 3;

// =============================================================================
// Speculative Execution Result
// =============================================================================

/// Result of speculative execution.
#[derive(Clone, Debug)]
pub enum SpeculativeResult<A, E> {
    /// A branch succeeded with this value.
    Success(A),
    /// All branches failed with these errors.
    AllFailed(Vec<E>),
    /// No branches were provided.
    NoBranches,
}

impl<A, E> SpeculativeResult<A, E> {
    /// Convert to Option, discarding errors.
    #[inline]
    pub fn ok(self) -> Option<A> {
        match self {
            SpeculativeResult::Success(a) => Some(a),
            _ => None,
        }
    }

    /// Convert to Result.
    ///
    /// # Errors
    ///
    /// Returns `Err` with the accumulated branch errors (in branch order)
    /// when every branch failed (`AllFailed`), and `Err(Vec::new())` when
    /// no branches were provided (`NoBranches`) — so an empty error list
    /// distinguishes "nothing to try" from "everything failed".
    #[inline]
    pub fn to_result(self) -> Result<A, Vec<E>> {
        match self {
            SpeculativeResult::Success(a) => Ok(a),
            SpeculativeResult::AllFailed(errs) => Err(errs),
            SpeculativeResult::NoBranches => Err(Vec::new()),
        }
    }

    /// Check if execution succeeded.
    #[inline]
    pub fn is_success(&self) -> bool {
        matches!(self, SpeculativeResult::Success(_))
    }
}

// =============================================================================
// Speculative Execution Functions
// =============================================================================

/// Execute computations speculatively, returning first success.
///
/// Runs all branches (potentially in parallel) and returns the first
/// successful result. If all branches fail, returns all errors.
///
/// # Example
///
/// ```rust
/// use ordofp_core::nexus::effects::error::ErrorComputation;
/// use ordofp_core::nexus::optim::speculative::{speculative, SpeculativeResult};
///
/// let result = speculative(vec![
///     ErrorComputation::err("first fails"),
///     ErrorComputation::ok(42),
///     ErrorComputation::ok(43),
/// ]);
/// assert!(matches!(result, SpeculativeResult::Success(42)));
/// ```
#[inline]
pub fn speculative<A, E>(branches: Vec<ErrorComputation<E, A>>) -> SpeculativeResult<A, E> {
    if branches.is_empty() {
        return SpeculativeResult::NoBranches;
    }

    // Pre-allocate error buffer with worst-case capacity to avoid reallocation
    // in the all-failure path.
    let mut errors = Vec::with_capacity(branches.len());

    for branch in branches {
        match branch.run() {
            Ok(a) => return SpeculativeResult::Success(a),
            Err(e) => errors.push(e),
        }
    }

    SpeculativeResult::AllFailed(errors)
}

/// Run the **first listed** computation and return its result.
///
/// Despite the name, no racing happens: the current implementation simply
/// runs the first branch in the vector (the rest are dropped unrun). Unlike
/// `speculative`, this doesn't distinguish between success and failure —
/// it returns whatever the first branch produces.
///
/// # Example
///
/// ```rust
/// use ordofp_core::nexus::effects::error::ErrorComputation;
/// use ordofp_core::nexus::optim::speculative::race;
///
/// let result = race(vec![
///     ErrorComputation::<&str, i32>::ok(1),
///     ErrorComputation::ok(2), // never run
/// ]);
/// assert_eq!(result, Some(Ok(1))); // the first branch's result
/// ```
#[inline]
pub fn race<A, E>(branches: Vec<ErrorComputation<E, A>>) -> Option<Result<A, E>> {
    branches
        .into_iter()
        .next()
        .map(super::super::effects::error::ErrorComputation::run)
}

/// Find first success among computations.
///
/// Similar to `speculative` but returns a simpler Result type.
///
/// # Errors
///
/// Returns `Err` with every branch's error (in branch order) when all
/// branches fail, or `Err(Vec::new())` when `branches` is empty.
#[inline]
pub fn first_success<A, E>(branches: Vec<ErrorComputation<E, A>>) -> Result<A, Vec<E>> {
    speculative(branches).to_result()
}

// =============================================================================
// Hedged Execution
// =============================================================================

/// Configuration for hedged requests.
#[derive(Clone, Copy, Debug)]
pub struct HedgeConfig {
    /// Maximum number of parallel attempts.
    pub max_attempts: usize,
    /// Whether to cancel pending attempts on first success.
    pub cancel_on_success: bool,
}

impl Default for HedgeConfig {
    fn default() -> Self {
        HedgeConfig {
            max_attempts: DEFAULT_MAX_RETRIES,
            cancel_on_success: true,
        }
    }
}

/// Execute with hedging - multiple attempts (currently sequential).
///
/// Conceptually for reducing tail latency by racing multiple requests; the
/// current implementation creates `max_attempts` computations and runs them
/// **in order**, returning the first success.
///
/// # Example
///
/// ```rust
/// use ordofp_core::nexus::effects::error::ErrorComputation;
/// use ordofp_core::nexus::optim::speculative::{hedge, HedgeConfig, SpeculativeResult};
///
/// let result: SpeculativeResult<i32, &str> = hedge(
///     HedgeConfig::default(),
///     || ErrorComputation::ok(42), // e.g. fetch_from_server()
/// );
/// assert!(matches!(result, SpeculativeResult::Success(42)));
/// ```
pub fn hedge<A, E, F>(config: HedgeConfig, make_attempt: F) -> SpeculativeResult<A, E>
where
    F: Fn() -> ErrorComputation<E, A>,
{
    // Pre-allocate exactly as many slots as we'll create.
    let mut branches: Vec<_> = Vec::with_capacity(config.max_attempts);
    branches.extend((0..config.max_attempts).map(|_| make_attempt()));
    speculative(branches)
}

// =============================================================================
// Fallback Chains
// =============================================================================

/// Execute with fallback on failure.
///
/// Try the primary computation; if it fails, try the fallback.
///
/// # Example
///
/// ```rust
/// use ordofp_core::nexus::effects::error::ErrorComputation;
/// use ordofp_core::nexus::optim::speculative::with_fallback;
///
/// let result: Result<i32, &str> = with_fallback(
///     || ErrorComputation::err("cache miss"), // e.g. fetch_from_cache()
///     || ErrorComputation::ok(42),            // e.g. fetch_from_database()
/// );
/// assert_eq!(result, Ok(42));
/// ```
///
/// # Errors
///
/// Returns the fallback's error when both computations fail; the
/// primary's error is discarded once the fallback runs.
#[inline]
pub fn with_fallback<A, E, F1, F2>(primary: F1, fallback: F2) -> Result<A, E>
where
    F1: FnOnce() -> ErrorComputation<E, A>,
    F2: FnOnce() -> ErrorComputation<E, A>,
{
    match primary().run() {
        Ok(a) => Ok(a),
        Err(_) => fallback().run(),
    }
}

/// Chain of fallbacks - try each in order until one succeeds.
///
/// # Example
///
/// ```rust
/// use ordofp_core::nexus::effects::error::ErrorComputation;
/// use ordofp_core::nexus::optim::speculative::fallback_chain;
///
/// let result = fallback_chain(vec![
///     || ErrorComputation::<&str, i32>::err("local cache miss"),
///     || ErrorComputation::err("distributed cache miss"),
///     || ErrorComputation::ok(42), // e.g. try_database()
///     || ErrorComputation::ok(99), // never run: try_external_api()
/// ]);
/// assert_eq!(result, Ok(42));
/// ```
///
/// # Errors
///
/// Returns `Err` with every attempt's error (in attempt order) when all
/// attempts fail, or `Err(Vec::new())` when `attempts` is empty.
pub fn fallback_chain<A, E, F>(attempts: Vec<F>) -> Result<A, Vec<E>>
where
    F: FnOnce() -> ErrorComputation<E, A>,
{
    // Pre-allocate error accumulator with worst-case capacity.
    let mut errors = Vec::with_capacity(attempts.len());

    for attempt in attempts {
        match attempt().run() {
            Ok(a) => return Ok(a),
            Err(e) => errors.push(e),
        }
    }

    Err(errors)
}

// =============================================================================
// Timeout Simulation
// =============================================================================

/// A computation with a simulated timeout.
///
/// The timeout is recorded but not enforced — enforcement needs an async
/// runtime.
pub struct TimedComputation<A, E> {
    computation: ErrorComputation<E, A>,
    _timeout_ms: u64,
}

impl<A, E> TimedComputation<A, E> {
    /// Create a timed computation.
    pub fn new(computation: ErrorComputation<E, A>, timeout_ms: u64) -> Self {
        TimedComputation {
            computation,
            _timeout_ms: timeout_ms,
        }
    }

    /// Run the computation (timeout not actually enforced without async).
    ///
    /// # Errors
    ///
    /// Propagates the wrapped computation's error unchanged; the recorded
    /// timeout never produces an error of its own.
    pub fn run(self) -> Result<A, E> {
        self.computation.run()
    }
}

/// Create a timed computation.
pub fn with_timeout<A, E>(
    computation: ErrorComputation<E, A>,
    timeout_ms: u64,
) -> TimedComputation<A, E> {
    TimedComputation::new(computation, timeout_ms)
}

// =============================================================================
// Retry Logic
// =============================================================================

/// Configuration for retry behavior.
#[derive(Clone, Copy, Debug)]
pub struct RetryConfig {
    /// Maximum number of retry attempts.
    pub max_retries: usize,
    /// Whether to use exponential backoff (advisory only).
    ///
    /// Note: [`retry`] currently ignores this flag — there is no clock in
    /// `no_std` scope, so no delay is inserted between attempts.
    pub exponential_backoff: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        RetryConfig {
            max_retries: DEFAULT_MAX_RETRIES,
            exponential_backoff: true,
        }
    }
}

/// Retry a computation on failure.
///
/// # Example
///
/// ```rust
/// use ordofp_core::nexus::effects::error::ErrorComputation;
/// use ordofp_core::nexus::optim::speculative::{retry, RetryConfig};
///
/// let result: Result<i32, &str> = retry(
///     RetryConfig::default(),
///     || ErrorComputation::ok(42), // e.g. possibly_failing_operation()
/// );
/// assert_eq!(result, Ok(42));
/// ```
///
/// # Errors
///
/// Returns the error from the **last** attempt when all
/// `max_retries + 1` runs fail; earlier errors are overwritten, not
/// accumulated. No delay is inserted between attempts (see
/// [`RetryConfig::exponential_backoff`]).
///
/// # Panics
///
/// Panics only if the internal "at least one attempt ran" invariant is
/// violated, which cannot happen (the loop always runs `max_retries + 1`
/// times) and would indicate a bug in this crate.
pub fn retry<A, E, F>(config: RetryConfig, operation: F) -> Result<A, E>
where
    F: Fn() -> ErrorComputation<E, A>,
    E: Clone,
{
    let mut last_error = None;

    for _ in 0..=config.max_retries {
        match operation().run() {
            Ok(a) => return Ok(a),
            Err(e) => last_error = Some(e),
        }
    }

    Err(last_error.expect("Should have at least one error after retries"))
}

// =============================================================================
// Circuit Breaker Pattern
// =============================================================================

/// State of a circuit breaker.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CircuitState {
    /// Circuit is closed, requests flow through.
    Closed,
    /// Circuit is open, requests are rejected.
    Open,
    /// Circuit is half-open, testing if service recovered.
    HalfOpen,
}

/// A simple circuit breaker.
pub struct CircuitBreaker {
    state: CircuitState,
    failure_count: usize,
    failure_threshold: usize,
    success_count: usize,
    success_threshold: usize,
}

impl CircuitBreaker {
    /// Create a new circuit breaker.
    pub fn new(failure_threshold: usize, success_threshold: usize) -> Self {
        CircuitBreaker {
            state: CircuitState::Closed,
            failure_count: 0,
            failure_threshold,
            success_count: 0,
            success_threshold,
        }
    }

    /// Check if requests are allowed.
    #[inline]
    pub fn is_allowed(&self) -> bool {
        !matches!(self.state, CircuitState::Open)
    }

    /// Record a success.
    #[inline]
    pub fn record_success(&mut self) {
        self.failure_count = 0;
        if self.state == CircuitState::HalfOpen {
            self.success_count += 1;
            if self.success_count >= self.success_threshold {
                self.state = CircuitState::Closed;
                self.success_count = 0;
            }
        }
    }

    /// Record a failure.
    #[inline]
    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.success_count = 0;
        if self.failure_count >= self.failure_threshold {
            self.state = CircuitState::Open;
        }
    }

    /// Attempt to half-open the circuit.
    #[inline]
    pub fn attempt_reset(&mut self) {
        if self.state == CircuitState::Open {
            self.state = CircuitState::HalfOpen;
        }
    }

    /// Get the current state.
    #[inline]
    pub fn state(&self) -> CircuitState {
        self.state
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_speculative_first_success() {
        let branches = alloc::vec![
            ErrorComputation::err("fail1"),
            ErrorComputation::ok(42),
            ErrorComputation::ok(43),
        ];
        let result = speculative(branches);
        assert!(matches!(result, SpeculativeResult::Success(42)));
    }

    #[test]
    fn test_speculative_all_fail() {
        let branches: Vec<ErrorComputation<&str, i32>> = alloc::vec![
            ErrorComputation::err("fail1"),
            ErrorComputation::err("fail2"),
        ];
        let result = speculative(branches);
        assert!(matches!(result, SpeculativeResult::AllFailed(_)));
    }

    #[test]
    fn test_speculative_no_branches() {
        let branches: Vec<ErrorComputation<&str, i32>> = alloc::vec![];
        let result = speculative(branches);
        assert!(matches!(result, SpeculativeResult::NoBranches));
    }

    #[test]
    fn test_race() {
        let branches: Vec<ErrorComputation<&str, i32>> =
            alloc::vec![ErrorComputation::ok(1), ErrorComputation::ok(2),];
        let result = race(branches);
        assert_eq!(result, Some(Ok(1)));
    }

    #[test]
    fn test_first_success() {
        let branches: Vec<ErrorComputation<&str, i32>> =
            alloc::vec![ErrorComputation::err("e1"), ErrorComputation::ok(42),];
        let result = first_success(branches);
        assert_eq!(result, Ok(42));
    }

    #[test]
    fn test_with_fallback_primary_succeeds() {
        let result: Result<i32, &str> =
            with_fallback(|| ErrorComputation::ok(1), || ErrorComputation::ok(2));
        assert_eq!(result, Ok(1));
    }

    #[test]
    fn test_with_fallback_uses_fallback() {
        let result: Result<i32, &str> = with_fallback(
            || ErrorComputation::err("primary failed"),
            || ErrorComputation::ok(2),
        );
        assert_eq!(result, Ok(2));
    }

    #[test]
    fn test_fallback_chain() {
        let result = fallback_chain(alloc::vec![
            || ErrorComputation::<&str, i32>::err("e1"),
            || ErrorComputation::<&str, i32>::err("e2"),
            || ErrorComputation::ok(42),
        ]);
        assert_eq!(result, Ok(42));
    }

    #[test]
    fn test_retry_succeeds_first_try() {
        let result: Result<i32, &str> = retry(
            RetryConfig {
                max_retries: 3,
                exponential_backoff: false,
            },
            || ErrorComputation::<&str, i32>::ok(42),
        );
        assert_eq!(result, Ok(42));
    }

    #[test]
    fn test_circuit_breaker() {
        let mut cb = CircuitBreaker::new(2, 1);

        assert!(cb.is_allowed());
        assert_eq!(cb.state(), CircuitState::Closed);

        cb.record_failure();
        assert!(cb.is_allowed());

        cb.record_failure();
        assert!(!cb.is_allowed());
        assert_eq!(cb.state(), CircuitState::Open);

        cb.attempt_reset();
        assert_eq!(cb.state(), CircuitState::HalfOpen);
        assert!(cb.is_allowed());

        cb.record_success();
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn test_hedge() {
        let result: SpeculativeResult<i32, &str> = hedge(
            HedgeConfig {
                max_attempts: 3,
                cancel_on_success: true,
            },
            || ErrorComputation::<&str, i32>::ok(42),
        );
        assert!(matches!(result, SpeculativeResult::Success(42)));
    }
}
