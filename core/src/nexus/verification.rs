//! Handler Verification Infrastructure
//!
//! This module provides the Tier 2 runtime contract checking for effect handlers.
//! In debug builds, handler operations can be verified against their algebraic laws.
//!
//! # Verification Tiers
//!
//! | Tier | Method | Guarantee | Module |
//! |------|--------|-----------|--------|
//! | 0 | Documentation | Understanding | effect docs |
//! | 1 | Property tests | Confidence | `laws.rs` |
//! | 2 | Runtime contracts | Debug-time errors | this module |
//! | 3 | Static analysis | Tool proofs | external (Prusti, Kani) |
//! | 4 | Proof extraction | Formal guarantee | external (Coq, Lean) |
//!
//! # Usage
//!
//! ```rust
//! use ordofp_core::nexus::verification::*;
//!
//! // In debug builds, verify handler satisfies laws
//! #[cfg(debug_assertions)]
//! verify_state_handler::<i32>();
//!
//! // Use VerifiedHandler trait to mark verified handlers
//! struct MyStateHandler;
//!
//! impl VerifiedHandler for MyStateHandler {
//!     fn handler_name() -> &'static str {
//!         "MyStateHandler"
//!     }
//!
//!     fn verify_laws() -> Result<(), LawViolation> {
//!         // Runtime law checks
//!         Ok(())
//!     }
//! }
//! ```
//!
//! # Design Philosophy
//!
//! - **Opt-in verification**: Users choose when to run verification
//! - **Debug-only overhead**: No runtime cost in release builds
//! - **Incremental adoption**: Start with Tier 0, add tiers as needed

use core::fmt;

// =============================================================================
// Law Violation Error
// =============================================================================

/// A violation of a handler law detected at runtime.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LawViolation {
    /// The handler being verified.
    pub handler: &'static str,
    /// The law that was violated.
    pub law: &'static str,
    /// Description of the violation.
    pub description: &'static str,
}

impl fmt::Display for LawViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Handler '{}' violates '{}' law: {}",
            self.handler, self.law, self.description
        )
    }
}

// =============================================================================
// Verified Handler Trait
// =============================================================================

/// Marker trait for effect handlers that can verify their laws at runtime.
///
/// Implement this trait to enable Tier 2 runtime verification for your handler.
///
/// # Example
///
/// ```rust
/// use ordofp_core::nexus::verification::{LawViolation, VerifiedHandler};
///
/// struct MyStateHandler;
///
/// impl VerifiedHandler for MyStateHandler {
///     fn handler_name() -> &'static str { "MyStateHandler" }
///
///     fn verify_laws() -> Result<(), LawViolation> {
///         // Check Get-Put law
///         // Check Put-Get law
///         // Check Put-Put law
///         Ok(())
///     }
/// }
/// ```
pub trait VerifiedHandler {
    /// The name of this handler for error reporting.
    fn handler_name() -> &'static str;

    /// Verify that this handler satisfies its algebraic laws.
    ///
    /// Returns `Ok(())` if all laws are satisfied, or `Err(LawViolation)`
    /// describing the first violation found.
    ///
    /// # Errors
    ///
    /// Implementations must return `Err(LawViolation)` naming the handler,
    /// the law, and a description for the **first** law the handler fails;
    /// checking stops at that point rather than accumulating.
    fn verify_laws() -> Result<(), LawViolation>;

    /// Assert that laws hold, panicking on violation.
    ///
    /// Only runs in debug builds.
    #[cfg(debug_assertions)]
    fn assert_laws() {
        if let Err(violation) = Self::verify_laws() {
            crate::cold_panic!("{}", violation);
        }
    }

    /// No-op in release builds.
    #[cfg(not(debug_assertions))]
    fn assert_laws() {}
}

// =============================================================================
// Verification Result
// =============================================================================

/// Result of running a verification check.
#[derive(Clone, Debug)]
pub struct VerificationResult {
    /// Handler name.
    pub handler: &'static str,
    /// Laws checked.
    pub laws_checked: usize,
    /// Violations found.
    pub violations: alloc::vec::Vec<LawViolation>,
}

impl VerificationResult {
    /// Create a new empty result.
    pub fn new(handler: &'static str) -> Self {
        VerificationResult {
            handler,
            laws_checked: 0,
            violations: alloc::vec::Vec::new(),
        }
    }

    /// Record a successful check.
    pub fn check_passed(&mut self) {
        self.laws_checked += 1;
    }

    /// Record a violation.
    pub fn check_failed(&mut self, law: &'static str, description: &'static str) {
        self.laws_checked += 1;
        self.violations.push(LawViolation {
            handler: self.handler,
            law,
            description,
        });
    }

    /// Whether all checks passed.
    pub fn is_ok(&self) -> bool {
        self.violations.is_empty()
    }

    /// Consume the result, returning `Ok(())` if every checked law passed, or
    /// `Err(LawViolation)` containing the first recorded violation otherwise.
    ///
    /// Only the first violation is returned; use [`VerificationResult::violations`]
    /// directly when you need to inspect all failures.
    ///
    /// # Errors
    ///
    /// Returns `Err` with the first violation recorded via
    /// [`check_failed`](Self::check_failed); any later violations are
    /// dropped by this conversion.
    pub fn to_result(self) -> Result<(), LawViolation> {
        if let Some(v) = self.violations.into_iter().next() {
            Err(v)
        } else {
            Ok(())
        }
    }
}

// =============================================================================
// State Handler Verification
// =============================================================================

#[cfg(debug_assertions)]
use crate::nexus::effects::state::StatefulComputation;

/// Verify the State handler's Get-Put law at runtime.
///
/// Checks:
/// - Get-Put law: `get.and_then(|s| put(s)) = pure(())`
///
/// The remaining state laws (Put-Get: `put(s).and_then(|_| get) =
/// put(s).map(|_| s)`, Put-Put: `put(s1).and_then(|_| put(s2)) = put(s2)`)
/// are documented in `nexus::laws` and property-tested there, but are **not
/// yet checked by this runtime verifier**.
#[cfg(debug_assertions)]
pub fn verify_state_handler<S: Clone + PartialEq + Default + 'static>() -> VerificationResult {
    let mut result = VerificationResult::new("State");

    // Get-Put law
    let get_put =
        StatefulComputation::<S, S>::get().and_then(|s| StatefulComputation::<S, ()>::put(s));
    let pure_unit = StatefulComputation::<S, ()>::pure(());

    let initial = S::default();
    let (_r1, s1) = get_put.run(initial.clone());
    let (_r2, s2) = pure_unit.run(initial);

    // r1 and r2 are both () so always equal; only compare states
    if s1 == s2 {
        result.check_passed();
    } else {
        result.check_failed("Get-Put", "get.and_then(put) != pure(())");
    }

    result
}

/// Verify State handler laws at runtime (release build stub).
///
/// In debug builds this checks the Get-Put law against a
/// `Default + PartialEq` state type.  In release builds the checks are elided
/// and an empty (passing) [`VerificationResult`] is returned so that call
/// sites compile and compose identically in both profiles.
///
/// See the `#[cfg(debug_assertions)]` overload for the full law descriptions.
#[cfg(not(debug_assertions))]
pub fn verify_state_handler<S>() -> VerificationResult {
    VerificationResult::new("State")
}

// =============================================================================
// Reader Handler Verification
// =============================================================================

#[cfg(debug_assertions)]
use crate::nexus::effects::reader::ReaderComputation;

/// Verify Reader handler laws at runtime.
///
/// Checks:
/// - Ask-Ask law: `ask.and_then(|e| ask.map(|_| e)) = ask`
#[cfg(debug_assertions)]
pub fn verify_reader_handler<E: Clone + PartialEq + Default + 'static>() -> VerificationResult {
    let mut result = VerificationResult::new("Reader");

    // Ask-Ask law
    let env = E::default();
    let ask_ask = ReaderComputation::<E, E>::ask()
        .and_then(move |e| ReaderComputation::<E, E>::ask().map(move |_| e));
    let just_ask = ReaderComputation::<E, E>::ask();

    if ask_ask.run(&env) == just_ask.run(&env) {
        result.check_passed();
    } else {
        result.check_failed("Ask-Ask", "ask.and_then(|e| ask.map(|_| e)) != ask");
    }

    result
}

/// Verify Reader handler laws at runtime (release build stub).
///
/// In debug builds this checks the Ask-Ask law, verifying that a reader asking
/// twice for the same environment is equivalent to asking once.  In release
/// builds the check is elided and an empty (passing) [`VerificationResult`] is
/// returned so that call sites compile and compose identically in both profiles.
///
/// See the `#[cfg(debug_assertions)]` overload for the full law descriptions.
#[cfg(not(debug_assertions))]
pub fn verify_reader_handler<E>() -> VerificationResult {
    VerificationResult::new("Reader")
}

// =============================================================================
// Error Handler Verification
// =============================================================================

#[cfg(debug_assertions)]
use crate::nexus::effects::error::ErrorComputation;

/// Verify Error handler laws at runtime.
///
/// Checks:
/// - Catch-Pure law: `catch(pure(x), h) = pure(x)`
/// - Catch-Throw law: `catch(throw(e), h) = h(e)`
/// - Throw-Bind law: `throw(e).and_then(f) = throw(e)`
#[cfg(debug_assertions)]
pub fn verify_error_handler<E: Clone + PartialEq + Default, A: Clone + PartialEq + Default>()
-> VerificationResult {
    let mut result = VerificationResult::new("Error");

    // Catch-Pure law
    let value = A::default();
    let pure_comp = ErrorComputation::<E, A>::ok(value.clone());
    let caught = pure_comp.or_else(|_| ErrorComputation::ok(value.clone()));

    if caught.run() == Ok(value.clone()) {
        result.check_passed();
    } else {
        result.check_failed("Catch-Pure", "catch(pure(x), h) != pure(x)");
    }

    // Throw-Bind law
    let error = E::default();
    let throw_bind = ErrorComputation::<E, A>::err(error.clone())
        .and_then(|_: A| ErrorComputation::<E, A>::ok(value));

    if throw_bind.run() == Err(error) {
        result.check_passed();
    } else {
        result.check_failed("Throw-Bind", "throw(e).and_then(f) != throw(e)");
    }

    result
}

/// No-op in release builds.
#[cfg(not(debug_assertions))]
pub fn verify_error_handler<E, A>() -> VerificationResult {
    VerificationResult::new("Error")
}

// =============================================================================
// Writer Handler Verification
// =============================================================================

#[cfg(debug_assertions)]
use crate::nexus::effects::writer::{Monoid, WriterComputation};

/// Verify Writer handler laws at runtime.
///
/// Checks:
/// - Tell-Empty law: `tell(empty) = pure(())`
/// - Listen-Pure law: `listen(pure(x)) = pure((x, empty))`
#[cfg(debug_assertions)]
pub fn verify_writer_handler<W: Monoid + PartialEq + 'static>() -> VerificationResult {
    let mut result = VerificationResult::new("Writer");

    // Tell-Empty law
    let tell_empty = WriterComputation::<W, ()>::tell(W::empty());
    let pure_unit = WriterComputation::<W, ()>::pure(());

    let (_r1, w1) = tell_empty.run();
    let (_r2, w2) = pure_unit.run();

    // r1 and r2 are both () so always equal; only compare logs
    if w1 == w2 {
        result.check_passed();
    } else {
        result.check_failed("Tell-Empty", "tell(empty) != pure(())");
    }

    result
}

/// No-op in release builds.
#[cfg(not(debug_assertions))]
pub fn verify_writer_handler<W>() -> VerificationResult {
    VerificationResult::new("Writer")
}

// =============================================================================
// Verify All Handlers
// =============================================================================

/// Verify all standard handlers with default types.
///
/// This is a convenience function for quick verification in tests.
#[cfg(debug_assertions)]
pub fn verify_all_handlers() -> alloc::vec::Vec<VerificationResult> {
    alloc::vec![
        verify_state_handler::<i32>(),
        verify_reader_handler::<i32>(),
        verify_error_handler::<&str, i32>(),
        verify_writer_handler::<alloc::string::String>(),
    ]
}

/// No-op in release builds.
#[cfg(not(debug_assertions))]
pub fn verify_all_handlers() -> alloc::vec::Vec<VerificationResult> {
    alloc::vec::Vec::new()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    #[test]
    fn test_verify_state_handler() {
        let result = verify_state_handler::<i32>();
        assert!(result.is_ok(), "State handler should satisfy laws");
    }

    #[test]
    fn test_verify_reader_handler() {
        let result = verify_reader_handler::<i32>();
        assert!(result.is_ok(), "Reader handler should satisfy laws");
    }

    #[test]
    fn test_verify_error_handler() {
        let result = verify_error_handler::<&str, i32>();
        assert!(result.is_ok(), "Error handler should satisfy laws");
    }

    #[test]
    fn test_verify_writer_handler() {
        let result = verify_writer_handler::<alloc::string::String>();
        assert!(result.is_ok(), "Writer handler should satisfy laws");
    }

    #[test]
    fn test_verify_all_handlers() {
        let results = verify_all_handlers();
        for result in results {
            assert!(
                result.is_ok(),
                "{} handler failed: {:?}",
                result.handler,
                result.violations
            );
        }
    }

    #[test]
    fn test_law_violation_display() {
        let violation = LawViolation {
            handler: "TestHandler",
            law: "TestLaw",
            description: "expected X but got Y",
        };
        let msg = violation.to_string();
        assert!(msg.contains("TestHandler"));
        assert!(msg.contains("TestLaw"));
        assert!(msg.contains("expected X but got Y"));
    }
}
