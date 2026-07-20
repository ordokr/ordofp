//! Effect Handler Laws and Verification
//!
//! This module documents and tests the algebraic laws that effect handlers
//! must satisfy. `OrdoFP` follows a **tiered verification approach** that
//! provides increasing levels of confidence without requiring full formal proofs.
//!
//! # Verification Tiers
//!
//! | Tier | Method | Guarantee | Location |
//! |------|--------|-----------|----------|
//! | 0 | Documentation | Understanding | Effect module docs |
//! | 1 | Property tests | Confidence (tests pass) | This module |
//! | 2 | Runtime contracts | Debug-time errors | `verification.rs` |
//! | 3 | Static analysis | Tool-dependent proofs | External (Prusti, Kani) |
//! | 4 | Proof extraction | Formal guarantee | External (Coq, Lean) |
//!
//! ## What Each Tier Provides
//!
//! - **Tier 0**: Every handler documents its laws in module-level docs.
//!   Users can understand what guarantees to expect.
//!
//! - **Tier 1**: This module provides verification functions that test laws
//!   with concrete values. Tests give confidence but don't prove correctness.
//!
//! - **Tier 2**: The `verification` module provides runtime law checking that
//!   runs in debug builds. Violations cause panics, catching bugs early.
//!
//! - **Tier 3-4**: Deferred to external tools and future research.
//!
//! # Handler Laws
//!
//! All effect handlers must satisfy these fundamental laws:
//!
//! ## 1. Pure Law (Return/Unit)
//!
//! ```text
//! handle(pure(x)) = pure(x)
//! ```
//!
//! Handling a pure value should not change it.
//!
//! ## 2. Bind Law (Naturality)
//!
//! ```text
//! handle(m.and_then(f)) = handle(m).and_then(|x| handle(f(x)))
//! ```
//!
//! Handling distributes over monadic bind.
//!
//! ## 3. Effect-Specific Laws
//!
//! ### State Laws
//!
//! ```text
//! get.and_then(|s| put(s)) = pure(())           // Get-Put
//! put(s).and_then(|_| get) = put(s).map(|_| s)  // Put-Get
//! put(s1).and_then(|_| put(s2)) = put(s2)       // Put-Put
//! ```
//!
//! ### Reader Laws
//!
//! ```text
//! ask.and_then(|e| ask.map(|_| e)) = ask        // Ask-Ask
//! local(f, ask) = ask.map(f)                     // Local-Ask
//! ```
//!
//! ### Error Laws
//!
//! ```text
//! catch(throw(e), h) = h(e)                      // Catch-Throw
//! catch(pure(x), h) = pure(x)                    // Catch-Pure
//! throw(e).and_then(f) = throw(e)               // Throw-Bind
//! ```
//!
//! # Current Status
//!
//! - Tier 0: Complete (this documentation)
//! - Tier 1: Implemented below
//! - Tier 2: Implemented in [`super::verification`] (debug-build law checks)
//! - Tier 3-4: Future research

// Law-check functions take their inputs by value by design: they are
// quickcheck-style value properties whose arguments are consumed test data.
#![allow(clippy::needless_pass_by_value)]

use super::effects::error::ErrorComputation;
use super::effects::reader::ReaderComputation;
use super::effects::state::StatefulComputation;

// =============================================================================
// State Handler Laws
// =============================================================================

/// Verify the Get-Put law: `get.and_then(|s`| put(s)) = pure(())
///
/// Getting state and immediately putting it back should be a no-op.
pub fn verify_state_get_put<S: Clone + PartialEq + 'static>(initial: S) -> bool {
    let get_put =
        StatefulComputation::<S, S>::get().and_then(move |s| StatefulComputation::<S, ()>::put(s));
    let pure_unit = StatefulComputation::<S, ()>::pure(());

    let (_r1, s1) = get_put.run(initial.clone());
    let (_r2, s2) = pure_unit.run(initial);

    // r1 and r2 are both () so always equal; only compare states
    s1 == s2
}

/// Verify the Put-Get law: `put(s).and_then`(|_| get) = put(s).map(|_| s)
///
/// Putting state and getting it should return the put value.
pub fn verify_state_put_get<S: Clone + PartialEq + 'static>(new_state: S, initial: S) -> bool {
    let put_get = StatefulComputation::<S, ()>::put(new_state.clone())
        .and_then(|()| StatefulComputation::<S, S>::get());

    let (result, final_state) = put_get.run(initial);

    result == new_state && final_state == new_state
}

/// Verify the Put-Put law: `put(s1).and_then`(|_| put(s2)) = put(s2)
///
/// Two consecutive puts should be equivalent to just the second put.
pub fn verify_state_put_put<S: Clone + PartialEq + 'static>(s1: S, s2: S, initial: S) -> bool {
    let s2_clone = s2.clone();
    let put_put = StatefulComputation::<S, ()>::put(s1)
        .and_then(move |()| StatefulComputation::<S, ()>::put(s2_clone));
    let just_put = StatefulComputation::<S, ()>::put(s2);

    let (_r1, state1) = put_put.run(initial.clone());
    let (_r2, state2) = just_put.run(initial);

    // r1 and r2 are both () so always equal; only compare states
    state1 == state2
}

// =============================================================================
// Reader Handler Laws
// =============================================================================

/// Verify that ask is idempotent: `ask.and_then(|e`| ask.map(|_| e)) = ask
///
/// Asking twice and discarding the second should equal asking once.
pub fn verify_reader_ask_ask<E: Clone + PartialEq + 'static>(env: &E) -> bool {
    let ask_ask = ReaderComputation::<E, E>::ask()
        .and_then(move |e| ReaderComputation::<E, E>::ask().map(move |_| e));
    let just_ask = ReaderComputation::<E, E>::ask();

    ask_ask.run(env) == just_ask.run(env)
}

// =============================================================================
// Error Handler Laws
// =============================================================================

/// Verify Catch-Pure: catch(pure(x), h) = pure(x)
///
/// Catching on a successful computation should return the success.
pub fn verify_error_catch_pure<E: Clone + PartialEq, A: Clone + PartialEq>(value: A) -> bool {
    let pure_comp = ErrorComputation::<E, A>::ok(value.clone());
    let caught = pure_comp.or_else(|_| ErrorComputation::ok(value.clone()));

    caught.run() == Ok(value)
}

/// Verify Catch-Throw: catch(throw(e), h) = h(e)
///
/// Catching a thrown error should invoke the handler.
pub fn verify_error_catch_throw<E: Clone + PartialEq, A: Clone + PartialEq>(
    error: E,
    recovery: A,
) -> bool {
    let throw_comp = ErrorComputation::<E, A>::err(error);
    let caught = throw_comp.or_else(|_| ErrorComputation::ok(recovery.clone()));

    caught.run() == Ok(recovery)
}

/// Verify Throw-Bind: `throw(e).and_then(f)` = throw(e)
///
/// Binding after an error should propagate the error.
///
/// # Panics
///
/// The continuation passed to `and_then` panics if invoked — that is the
/// law under test: a correct Error effect short-circuits past the bind,
/// so the panic firing means the law (not this function) is broken.
pub fn verify_error_throw_bind<E: Clone + PartialEq, A: Clone + PartialEq, B: Clone + PartialEq>(
    error: E,
) -> bool {
    // The closure panics if invoked; err() must short-circuit so it never is.
    let throw_bind = ErrorComputation::<E, A>::err(error.clone())
        .and_then(|_: A| -> ErrorComputation<E, B> { panic!("Should not be called") });

    throw_bind.run() == Err(error)
}

// =============================================================================
// Writer Handler Laws
// =============================================================================

use super::effects::writer::{Monoid, WriterComputation};

/// Verify Tell-Empty: tell(empty) = pure(())
///
/// Telling the identity element should be a no-op.
pub fn verify_writer_tell_empty<W: Monoid + PartialEq + 'static>() -> bool {
    let tell_empty = WriterComputation::<W, ()>::tell(W::empty());
    let pure_unit = WriterComputation::<W, ()>::pure(());

    let (_r1, w1) = tell_empty.run();
    let (_r2, w2) = pure_unit.run();

    // r1 and r2 are both () so always equal; only compare logs
    w1 == w2
}

/// Verify Listen-Pure: listen(pure(x)) = pure((x, empty))
///
/// Listening to a pure value produces the value with empty log.
pub fn verify_writer_listen_pure<
    W: Monoid + PartialEq + Clone + 'static,
    A: Clone + PartialEq + 'static,
>(
    value: A,
) -> bool {
    let listened = WriterComputation::<W, A>::pure(value.clone()).listen();
    let ((result, inner_log), outer_log) = listened.run();

    result == value && inner_log == W::empty() && outer_log == W::empty()
}

// =============================================================================
// Monad Laws
// =============================================================================

/// Verify left identity for State: `pure(a).and_then(f) = f(a)`
///
/// The left identity (also called *return-bind*) monad law states that wrapping
/// a value with `pure` and then immediately binding over it with `f` is
/// equivalent to just calling `f(a)` directly.  For `StatefulComputation` this
/// means that `StatefulComputation::pure(a).and_then(f)` must produce the same
/// `(result, final_state)` pair as `f(a)` when both are run from the same
/// initial state.
///
/// This is a Tier-1 property test: it provides confidence through a concrete
/// counterexample check but does not constitute a formal proof.
///
/// # Parameters
///
/// * `a` — The `i32` value used to construct the test computation `pure(a)`.
/// * `initial` — The starting state threaded through both sides of the equation.
///
/// # Returns
///
/// `true` if the law holds for the given inputs, `false` if a violation is
/// detected.
///
/// # Example
///
/// ```rust
/// use ordofp_core::nexus::laws::verify_monad_left_identity_state;
///
/// // Law holds for arbitrary value and initial state
/// assert!(verify_monad_left_identity_state(42, 0));
/// ```
pub fn verify_monad_left_identity_state(a: i32, initial: i32) -> bool {
    let f = |x: i32| StatefulComputation::<i32, i32>::new(move |s| (x * 2, s + 1));

    let left = StatefulComputation::<i32, i32>::pure(a).and_then(f);
    let right = f(a);

    left.run(initial) == right.run(initial)
}

/// Verify left identity for Error: `pure(a).and_then(f) = f(a)`
///
/// The left identity (also called *return-bind*) monad law states that
/// wrapping a value with `pure` and then immediately binding over it with `f`
/// is equivalent to just calling `f(a)` directly.  For the `ErrorComputation`
/// monad this means that `ErrorComputation::ok(a).and_then(f)` must produce
/// the same result as `f(a)`.
///
/// This is a Tier-1 property test: it provides confidence through a concrete
/// counterexample check but does not constitute a formal proof.
///
/// # Parameters
///
/// * `a` — The `i32` value used to construct the test computation `ok(a)`.
///
/// # Returns
///
/// `true` if the law holds for the given input, `false` if a violation is
/// detected.
///
/// # Example
///
/// ```rust
/// use ordofp_core::nexus::laws::verify_monad_left_identity_error;
///
/// // Law holds for a successful value
/// assert!(verify_monad_left_identity_error(42));
/// ```
pub fn verify_monad_left_identity_error(a: i32) -> bool {
    let f = |x: i32| ErrorComputation::<&str, i32>::ok(x * 2);

    let left = ErrorComputation::<&str, i32>::ok(a).and_then(f);
    let right = f(a);

    left.run() == right.run()
}

/// Verify right identity for Error: `m.and_then(pure) = m`
///
/// The right identity (also called *return-bind*) monad law states that
/// wrapping a value with `pure` and then immediately binding over it is the
/// same as not binding at all.  For the `ErrorComputation` monad this means
/// that calling `.and_then(|x| ErrorComputation::ok(x))` on any computation
/// is semantically a no-op.
///
/// This is a Tier-1 property test: it provides confidence through a concrete
/// counterexample check but does not constitute a formal proof.
///
/// # Parameters
///
/// * `a` — The `i32` value used to construct the test computation `ok(a)`.
///
/// # Returns
///
/// `true` if the law holds for the given input, `false` if a violation is
/// detected.
///
/// # Example
///
/// ```rust
/// use ordofp_core::nexus::laws::verify_monad_right_identity_error;
///
/// // Law holds for a successful value
/// assert!(verify_monad_right_identity_error(42));
/// ```
pub fn verify_monad_right_identity_error(a: i32) -> bool {
    let m = ErrorComputation::<&str, i32>::ok(a);

    let chained = ErrorComputation::<&str, i32>::ok(a).and_then(ErrorComputation::ok);

    chained.run() == m.run()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // State Laws
    #[test]
    fn test_state_get_put_law() {
        assert!(verify_state_get_put(42i32));
        assert!(verify_state_get_put(0i32));
        assert!(verify_state_get_put(-100i32));
    }

    #[test]
    fn test_state_put_get_law() {
        assert!(verify_state_put_get(100i32, 0i32));
        assert!(verify_state_put_get(42i32, 99i32));
    }

    #[test]
    fn test_state_put_put_law() {
        assert!(verify_state_put_put(1i32, 2i32, 0i32));
        assert!(verify_state_put_put(100i32, 200i32, 50i32));
    }

    // Reader Laws
    #[test]
    fn test_reader_ask_ask_law() {
        assert!(verify_reader_ask_ask(&42i32));
        assert!(verify_reader_ask_ask(&0i32));
    }

    // Error Laws
    #[test]
    fn test_error_catch_pure_law() {
        assert!(verify_error_catch_pure::<&str, i32>(42));
        assert!(verify_error_catch_pure::<&str, i32>(0));
    }

    #[test]
    fn test_error_catch_throw_law() {
        assert!(verify_error_catch_throw("error", 42i32));
    }

    #[test]
    fn test_error_throw_bind_law() {
        assert!(verify_error_throw_bind::<&str, i32, i32>("error"));
    }

    // Writer Laws
    #[test]
    fn test_writer_tell_empty_law() {
        assert!(verify_writer_tell_empty::<alloc::string::String>());
        assert!(verify_writer_tell_empty::<alloc::vec::Vec<i32>>());
    }

    #[test]
    fn test_writer_listen_pure_law() {
        assert!(verify_writer_listen_pure::<alloc::string::String, i32>(42));
        assert!(verify_writer_listen_pure::<alloc::vec::Vec<i32>, &str>(
            "hello"
        ));
    }

    // Monad Laws
    #[test]
    fn test_monad_left_identity_state() {
        assert!(verify_monad_left_identity_state(5, 0));
        assert!(verify_monad_left_identity_state(10, 100));
    }

    #[test]
    fn test_monad_left_identity_error() {
        assert!(verify_monad_left_identity_error(42));
        assert!(verify_monad_left_identity_error(0));
    }

    #[test]
    fn test_monad_right_identity_error() {
        assert!(verify_monad_right_identity_error(42));
        assert!(verify_monad_right_identity_error(0));
    }
}
