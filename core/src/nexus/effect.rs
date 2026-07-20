//! Core Effect Types
//!
//! This module defines the central `Eff` type and the `EffectMarker` trait
//! that effects must implement to participate in the type-level effect system.
//!
//! # Performance Notes
//!
//! - `Eff<Pure, A>` is optimized to store just `A`
//! - Single-effect rows use specialized representations
//! - Complex effect combinations may use boxed continuations

use alloc::boxed::Box;
use core::marker::PhantomData;

use super::row::{EffectRow, Pure, Row};

// =============================================================================
// Effect Marker Trait
// =============================================================================

/// Marker trait for effect types.
///
/// Each effect type must implement this trait to specify its bit position
/// in the effect row bitmask. This enables O(1) effect membership checking.
///
/// # Implementing Custom Effects
///
/// ```rust
/// use ordofp_core::nexus::prelude::*;
///
/// // Define a custom logging effect
/// struct LogEffect;
///
/// impl EffectMarker for LogEffect {
///     const BIT: u128 = USER_EFFECT_START << 0; // First user effect
/// }
/// ```
pub trait EffectMarker {
    /// The bit position of this effect in the effect row.
    const BIT: u128;

    /// Human-readable name for debugging.
    const NAME: &'static str = "Effect";
}

// =============================================================================
// Core Eff Type
// =============================================================================

/// The central effectful computation type.
///
/// `Eff<R, A>` represents a computation that:
/// - May perform effects in the effect row `R`
/// - Produces a value of type `A`
///
/// # Performance Characteristics
///
/// | Effect Row | Representation | Notes |
/// |------------|----------------|-------|
/// | `Pure` | Direct value | No overhead |
/// | `Row<STATE_BIT>` | Lazy thunk | Use `StatefulComputation` for efficiency |
/// | `Row<READER_BIT>` | Lazy thunk | Use `ReaderComputation` for efficiency |
/// | `Row<ERROR_BIT>` | Lazy thunk | Use `ErrorComputation` for efficiency |
/// | Combined | Boxed thunk | Some allocation overhead |
///
/// For maximum efficiency with single effects, use the specialized
/// computation types in the `effects` module directly.
///
/// # Example
///
/// ```rust
/// use ordofp_core::nexus::prelude::*;
///
/// // Pure computation - no overhead
/// let pure_comp: Eff<Pure, i32> = pure(42);
/// assert_eq!(pure_comp.run_pure(), 42);
/// ```
pub struct Eff<R: EffectRow, A> {
    /// The internal representation, selected at compile time.
    pub(crate) inner: EffInner<R, A>,
    /// Phantom to satisfy the compiler.
    _marker: PhantomData<R>,
}

/// Internal representation of Eff, specialized per effect row pattern.
pub(crate) enum EffInner<R: EffectRow, A> {
    /// Pure value (no effects).
    Pure(A),
    /// Lazy thunk for deferred computation.
    Lazy(LazyThunk<R, A>),
}

/// A lazily evaluated computation.
pub(crate) struct LazyThunk<R: EffectRow, A> {
    /// The thunk that produces the value.
    thunk: Option<Box<dyn FnOnce() -> Eff<R, A>>>,
}

impl<R: EffectRow, A> LazyThunk<R, A> {
    fn new<F: FnOnce() -> Eff<R, A> + 'static>(f: F) -> Self {
        LazyThunk {
            thunk: Some(Box::new(f)),
        }
    }

    fn force(mut self) -> Eff<R, A> {
        (self.thunk.take().expect("Thunk already forced"))()
    }
}

// =============================================================================
// Eff Constructors
// =============================================================================

impl<A> Eff<Pure, A> {
    /// Create a pure computation.
    ///
    /// This is the most efficient representation - just the value itself.
    #[inline(always)]
    pub const fn pure(value: A) -> Self {
        Eff {
            inner: EffInner::<Pure, A>::Pure(value),
            _marker: PhantomData,
        }
    }

    /// Run a pure computation, extracting the value.
    ///
    /// Since pure computations have no effects, we can run them directly.
    #[inline(always)]
    pub fn run_pure(self) -> A {
        match self.inner {
            EffInner::Pure(a) => a,
            EffInner::Lazy(_) => unreachable!("Pure computations cannot be lazy"),
        }
    }
}

impl<R: EffectRow, A> Eff<R, A> {
    /// Wrap a plain value in an effectful computation without allocating.
    ///
    /// The resulting `Eff<R, A>` is stored as an inline `Pure` variant — no
    /// boxing, no closure, no heap allocation.  It is semantically equivalent
    /// to `pure` (for `R = Pure`) or `Eff::lift`, but works for **any** effect
    /// row `R`, making it the preferred constructor when you already have a
    /// concrete row type inferred from context.
    ///
    /// # When to use `from_value` vs `pure` vs `lift`
    ///
    /// | Constructor | Effect row | Allocation |
    /// |-------------|-----------|------------|
    /// | `Eff::pure(v)` | Must be `Pure` | None |
    /// | `Eff::from_value(v)` | Any `R` | None |
    /// | `Eff::lift(v)` | Any `R` | None (alias for `from_value`) |
    ///
    /// Prefer `from_value` inside handler implementations and generic code where
    /// the effect row is a type parameter, and `pure` in call-sites where the
    /// computation is known to have no effects.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::nexus::prelude::*;
    ///
    /// // Works for any effect row — no effects are actually performed.
    /// let a: Eff<StateEff, i32> = Eff::from_value(42);
    /// let (result, state) = run_state(a, 0);
    /// assert_eq!(result, 42);
    /// assert_eq!(state, 0); // state was never touched
    /// ```
    #[inline(always)]
    pub fn from_value(value: A) -> Self {
        Eff {
            inner: EffInner::<R, A>::Pure(value),
            _marker: PhantomData,
        }
    }

    /// Create a lazily-evaluated `Eff` that defers execution until it is run.
    ///
    /// The closure `f` is wrapped in a `LazyThunk` and is not called until the
    /// computation is forced by a handler (e.g. `run_pure`, `run_state`, …).
    /// This is the primary way to build computations that should not execute their
    /// effects at construction time.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use ordofp_core::nexus::prelude::*;
    ///
    /// // no_run: `run_pure` only handles the `Pure` variant; forcing a
    /// // `Lazy`-built computation isn't wired up yet, so running this panics.
    /// let comp: Eff<Pure, i32> = Eff::lazy(|| Eff::from_value(42));
    /// assert_eq!(comp.run_pure(), 42);
    /// ```
    pub fn lazy<F: FnOnce() -> Eff<R, A> + 'static>(f: F) -> Self {
        Eff {
            inner: EffInner::<R, A>::Lazy(LazyThunk::new(f)),
            _marker: PhantomData,
        }
    }
}

// =============================================================================
// Eff Functor
// =============================================================================

impl<R: EffectRow + 'static, A: 'static> Eff<R, A> {
    /// Map a function over the result of this computation.
    ///
    /// This is the Functor `fmap` operation. It transforms the result
    /// without changing the effect row.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::nexus::prelude::*;
    ///
    /// let comp = pure(21).map(|x| x * 2);
    /// assert_eq!(comp.run_pure(), 42);
    /// ```
    #[inline]
    pub fn map<B, F: FnOnce(A) -> B + 'static>(self, f: F) -> Eff<R, B> {
        match self.inner {
            EffInner::Pure(a) => Eff::from_value(f(a)),
            EffInner::Lazy(thunk) => Eff::lazy(move || thunk.force().map(f)),
        }
    }

    /// Replace the result of this computation with a constant value, ignoring the original result.
    ///
    /// This is the `const_map` or `$>` operation from functional programming. It runs all the
    /// effects of `self` but discards the produced value, substituting `value` instead.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::nexus::prelude::*;
    ///
    /// // Computation produces 21 but we replace it with 99.
    /// let comp = pure(21).map_const(99);
    /// assert_eq!(comp.run_pure(), 99);
    /// ```
    #[inline]
    pub fn map_const<B: 'static>(self, value: B) -> Eff<R, B> {
        self.map(move |_| value)
    }

    /// Discard the result value and replace it with `()`.
    ///
    /// Equivalent to `self.map_const(())`. Useful when only the effects of a
    /// computation matter and the produced value is irrelevant — for example,
    /// after a `put` or `tell` whose result you want to normalise to `()`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::nexus::prelude::*;
    ///
    /// let comp: Eff<StateEff, i32> = Eff::from_value(42);
    /// let voided: Eff<StateEff, ()> = comp.void();
    /// let (result, _state) = run_state(voided, 0);
    /// assert_eq!(result, ());
    /// ```
    #[inline]
    pub fn void(self) -> Eff<R, ()> {
        self.map_const(())
    }
}

// =============================================================================
// Eff Applicative
// =============================================================================

impl<R: EffectRow + 'static, A: 'static> Eff<R, A> {
    /// Lift a pure value into Eff.
    ///
    /// This is equivalent to `pure` but works for any effect row.
    #[inline(always)]
    pub fn lift(value: A) -> Self {
        Eff::from_value(value)
    }
}

// =============================================================================
// Eff Monad
// =============================================================================

impl<R: EffectRow + 'static, A: 'static> Eff<R, A> {
    /// Sequence two computations, passing the result of the first to the second.
    ///
    /// This is the monadic bind (>>=) operation.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::nexus::prelude::*;
    ///
    /// let comp = pure(21).and_then(|x| pure(x * 2));
    /// assert_eq!(comp.run_pure(), 42);
    /// ```
    pub fn and_then<B, F: FnOnce(A) -> Eff<R, B> + 'static>(self, f: F) -> Eff<R, B> {
        match self.inner {
            EffInner::Pure(a) => f(a),
            EffInner::Lazy(thunk) => Eff::lazy(move || thunk.force().and_then(f)),
        }
    }

    /// Sequence two computations, discarding the result of the first.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::nexus::prelude::*;
    ///
    /// let comp = pure(1).then(|| pure(42));
    /// assert_eq!(comp.run_pure(), 42);
    /// ```
    pub fn then<B, F: FnOnce() -> Eff<R, B> + 'static>(self, f: F) -> Eff<R, B> {
        self.and_then(move |_| f())
    }

    /// Flatten a nested `Eff<R, Eff<R, A>>` into a single `Eff<R, A>`.
    ///
    /// This is the monadic `join` operation. It is equivalent to
    /// `self.and_then(|inner| inner)`, and is useful when a computation
    /// produces another computation as its result (e.g. after a `map` that
    /// returns an `Eff`).
    ///
    /// # Type Constraints
    ///
    /// The result type `A` must implement `Into<Eff<R, A>>`, which is
    /// satisfied whenever `A` is itself an `Eff<R, _>` with the same effect
    /// row `R`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::nexus::prelude::*;
    ///
    /// // `flatten`'s bound is `A: Into<Eff<R, A>>` — the produced value must
    /// // know how to re-embed itself as a computation of the *same* type.
    /// // Provide that locally for a small payload type.
    /// struct Wrapped(i32);
    ///
    /// impl Into<Eff<Pure, Wrapped>> for Wrapped {
    ///     fn into(self) -> Eff<Pure, Wrapped> {
    ///         Eff::from_value(self)
    ///     }
    /// }
    ///
    /// let comp: Eff<Pure, Wrapped> = Eff::from_value(Wrapped(42));
    /// let flat: Eff<Pure, Wrapped> = comp.flatten();
    /// assert_eq!(flat.run_pure().0, 42);
    /// ```
    pub fn flatten(self) -> Eff<R, A>
    where
        A: Into<Eff<R, A>>,
    {
        self.and_then(core::convert::Into::into)
    }
}

// =============================================================================
// State Effect Operations
// =============================================================================

/// Marker type for the State effect.
#[derive(Copy, Clone, Debug)]
pub struct State<S>(PhantomData<S>);

impl<S> EffectMarker for State<S> {
    const BIT: u128 = super::row::STATE_BIT;
    const NAME: &'static str = "State";
}

/// Type alias for state effect row.
pub type StateEff = Row<{ super::row::STATE_BIT }>;

/// Read the current state without modifying it.
///
/// This is the State effect's read primitive. The computation produces a
/// clone of the current state value. Must be run inside a [`StateHandler`]
/// (e.g. via [`run_state`]).
///
/// [`StateHandler`]: crate::nexus::handler::StateHandler
/// [`run_state`]: crate::nexus::handler::run_state
///
/// # Type Parameters
///
/// * `S` – The state type managed by the handler. Must implement [`Clone`]
///   so the value can be returned while leaving the state intact.
///
/// # Example
///
/// ```rust,no_run
/// use ordofp_core::nexus::prelude::*;
/// use ordofp_core::nexus::get;
///
/// // no_run: `get` is an unimplemented stub (see "Panics" below) — forcing
/// // it via `run_state` always panics.
/// let comp = get::<i32>();
/// let (value, state) = run_state(comp, 42);
/// assert_eq!(value, 42);
/// assert_eq!(state, 42); // state is unchanged
/// ```
///
/// # Panics
///
/// **Stub:** the returned computation always panics when forced — the handler
/// infrastructure that would interpret `get` does not exist yet. Use
/// `effects::state::StatefulComputation` for working state effects.
pub fn get<S: Clone + 'static>() -> Eff<StateEff, S> {
    Eff::lazy(|| {
        // This would be implemented with proper handler infrastructure
        crate::cold_panic!("get() requires proper handler - use State handler")
    })
}

/// Replace the current state with a new value.
///
/// This is the State effect's write primitive. The new state `value` takes
/// effect immediately when the computation is run by a [`StateHandler`].
/// The computation itself produces `()` — use [`get`] afterwards if you need
/// to observe the updated state.
///
/// [`StateHandler`]: crate::nexus::handler::StateHandler
///
/// # Type Parameters
///
/// * `S` – The state type managed by the handler.
///
/// # Example
///
/// ```rust,no_run
/// use ordofp_core::nexus::prelude::*;
/// use ordofp_core::nexus::put;
///
/// // no_run: `put` is an unimplemented stub (see "Panics" below) — forcing
/// // it via `run_state` always panics.
/// let comp = put(42_i32);
/// let ((), final_state) = run_state(comp, 0);
/// assert_eq!(final_state, 42);
/// ```
///
/// # Panics
///
/// **Stub:** the returned computation always panics when forced — the handler
/// infrastructure that would interpret `put` does not exist yet. Use
/// `effects::state::StatefulComputation` for working state effects.
pub fn put<S: 'static>(_value: S) -> Eff<StateEff, ()> {
    Eff::lazy(|| crate::cold_panic!("put() requires proper handler - use State handler"))
}

/// Apply a function to transform the current state in place.
///
/// This is the State effect's read-modify-write primitive. The function `f`
/// receives the current state and returns the new state. The computation
/// produces `()`. Must be run inside a [`StateHandler`] (e.g. via
/// [`run_state`]).
///
/// This is equivalent to `get().and_then(|s| put(f(s)))` but expressed as a
/// single primitive.
///
/// [`StateHandler`]: crate::nexus::handler::StateHandler
/// [`run_state`]: crate::nexus::handler::run_state
///
/// # Type Parameters
///
/// * `S` – The state type managed by the handler.
/// * `F` – A one-shot closure that maps the old state to the new state.
///
/// # Example
///
/// ```rust,no_run
/// use ordofp_core::nexus::prelude::*;
/// use ordofp_core::nexus::modify;
///
/// // no_run: `modify` is an unimplemented stub (see "Panics" below) —
/// // forcing it via `run_state` always panics.
/// let comp = modify(|n: i32| n + 1);
/// let ((), new_state) = run_state(comp, 41);
/// assert_eq!(new_state, 42);
/// ```
///
/// # Panics
///
/// **Stub:** the returned computation always panics when forced — the handler
/// infrastructure that would interpret `modify` does not exist yet. Use
/// `effects::state::StatefulComputation` for working state effects.
pub fn modify<S: 'static, F: FnOnce(S) -> S + 'static>(_f: F) -> Eff<StateEff, ()> {
    Eff::lazy(|| crate::cold_panic!("modify() requires proper handler - use State handler"))
}

// =============================================================================
// Reader Effect Operations
// =============================================================================

/// Marker type for the Reader effect.
#[derive(Copy, Clone, Debug)]
pub struct Reader<E>(PhantomData<E>);

impl<E> EffectMarker for Reader<E> {
    const BIT: u128 = super::row::READER_BIT;
    const NAME: &'static str = "Reader";
}

/// Type alias for reader effect row.
pub type ReaderEff = Row<{ super::row::READER_BIT }>;

/// Read the entire environment value.
///
/// This is the Reader effect's primary primitive. The computation produces a
/// clone of the environment supplied to the handler. Must be run inside a
/// [`ReaderHandler`] (e.g. via [`run_reader`]).
///
/// [`ReaderHandler`]: crate::nexus::handler::ReaderHandler
/// [`run_reader`]: crate::nexus::handler::run_reader
///
/// # Type Parameters
///
/// * `E` – The environment type threaded through by the Reader effect. Must
///   implement [`Clone`] so the value can be returned while leaving the
///   environment available for subsequent `ask` calls.
///
/// # Example
///
/// ```rust,no_run
/// use ordofp_core::nexus::prelude::*;
/// use ordofp_core::nexus::ask;
///
/// // no_run: `ask` is an unimplemented stub (see "Panics" below) — forcing
/// // it via `run_reader` always panics.
/// let comp = ask::<u16>();
/// assert_eq!(run_reader(comp, &8080_u16), 8080);
/// ```
///
/// # Panics
///
/// **Stub:** the returned computation always panics when forced — the handler
/// infrastructure that would interpret `ask` does not exist yet. Use
/// `effects::reader::ReaderComputation` for working reader effects.
pub fn ask<E: Clone + 'static>() -> Eff<ReaderEff, E> {
    Eff::lazy(|| crate::cold_panic!("ask() requires proper handler - use Reader handler"))
}

/// Extract a value from the environment by applying a projection function.
///
/// This is a convenience combinator over [`ask`]: instead of reading the whole
/// environment and mapping afterwards, `asks` lets you project directly in one
/// step.  It is sometimes called `asks` or `reader` in Haskell literature.
///
/// # Type Parameters
///
/// * `E` – The environment type threaded through by the Reader effect.
/// * `A` – The projected value type returned by the computation.
/// * `F` – A one-shot closure that receives a shared reference to `E` and
///   produces a value of type `A`.
///
/// # Example
///
/// ```rust
/// use ordofp_core::nexus::prelude::*;
/// use ordofp_core::nexus::asks;
///
/// #[derive(Clone)]
/// struct Config { port: u16 }
///
/// fn get_port() -> Eff<ReaderEff, u16> {
///     asks(|cfg: &Config| cfg.port)
/// }
/// ```
///
/// # Panics
///
/// **Stub:** the returned computation always panics when forced — the handler
/// infrastructure that would interpret `asks` does not exist yet. Use
/// `effects::reader::ReaderComputation` for working reader effects.
pub fn asks<E: Clone + 'static, A: 'static, F: FnOnce(&E) -> A + 'static>(
    _f: F,
) -> Eff<ReaderEff, A> {
    Eff::lazy(|| crate::cold_panic!("asks() requires proper handler - use Reader handler"))
}

// =============================================================================
// Error Effect Operations
// =============================================================================

/// Marker type for the Error effect.
#[derive(Copy, Clone, Debug)]
pub struct Error<E>(PhantomData<E>);

impl<E> EffectMarker for Error<E> {
    const BIT: u128 = super::row::ERROR_BIT;
    const NAME: &'static str = "Error";
}

/// Type alias for error effect row.
pub type ErrorEff = Row<{ super::row::ERROR_BIT }>;

/// Lift a successful value into an Error-effect computation.
///
/// This is the success constructor for the Error effect, equivalent to
/// `pure` / `Ok` in the error-handling context. The error type `E` is a
/// phantom parameter: it appears only in the effect row so that the
/// computation can be sequenced with [`err`] calls that produce the same
/// error type without requiring an actual error value here.
///
/// # Type Parameters
///
/// * `E` – The error type that *could* be raised; constrains which [`err`]
///   values can be sequenced with this computation.
/// * `A` – The value type of the successful result.
///
/// # Example
///
/// ```rust
/// use ordofp_core::nexus::prelude::*;
/// use ordofp_core::nexus::ok;
///
/// let comp: Eff<ErrorEff, i32> = ok::<String, i32>(42);
/// assert_eq!(run_error::<String, i32>(comp), Ok(42));
/// ```
pub fn ok<E: 'static, A: 'static>(value: A) -> Eff<ErrorEff, A> {
    Eff::from_value(value)
}

/// Raise an error inside an Error-effect computation.
///
/// This is the failure constructor for the Error effect — the effectful
/// equivalent of `Err(e)` / `throw`. When run by an [`ErrorHandler`]
/// (e.g. via [`run_error`]), the computation short-circuits immediately and
/// returns `Err(error)` without evaluating any subsequent `and_then` steps.
///
/// [`ErrorHandler`]: crate::nexus::handler::ErrorHandler
/// [`run_error`]: crate::nexus::handler::run_error
///
/// # Type Parameters
///
/// * `E` – The error type to raise. Must match the error type expected by the
///   surrounding [`ErrorHandler`].
/// * `A` – The (phantom) value type the computation would have produced on
///   success. This allows `err` to be sequenced with any `Eff<ErrorEff, A>`
///   regardless of what `A` is.
///
/// # Example
///
/// ```rust,no_run
/// use ordofp_core::nexus::prelude::*;
/// use ordofp_core::nexus::err;
///
/// // no_run: `err` is an unimplemented stub (see "Panics" below) — forcing
/// // it via `run_error` always panics, so it never actually reaches `Err`.
/// let comp: Eff<ErrorEff, i32> = err::<String, i32>("oops".to_string());
/// assert_eq!(run_error::<String, i32>(comp), Err("oops".to_string()));
/// ```
///
/// # Panics
///
/// **Stub:** the returned computation always panics when forced — the handler
/// infrastructure that would interpret `err` does not exist yet (the error
/// value is discarded). Use `effects::error::ErrorComputation` for working
/// error effects.
pub fn err<E: 'static, A: 'static>(_error: E) -> Eff<ErrorEff, A> {
    Eff::lazy(|| crate::cold_panic!("err() requires proper handler - use Error handler"))
}

// =============================================================================
// IO Effect Operations
// =============================================================================

/// Marker type for the IO effect.
#[derive(Copy, Clone, Debug)]
pub struct IO;

impl EffectMarker for IO {
    const BIT: u128 = super::row::IO_BIT;
    const NAME: &'static str = "IO";
}

/// Type alias for IO effect row.
pub type IoEff = Row<{ super::row::IO_BIT }>;

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pure_eff() {
        let eff = Eff::<Pure, i32>::pure(42);
        assert_eq!(eff.run_pure(), 42);
    }

    #[test]
    fn test_pure_map() {
        let eff = Eff::<Pure, i32>::pure(21).map(|x| x * 2);
        assert_eq!(eff.run_pure(), 42);
    }

    #[test]
    fn test_pure_and_then() {
        let eff = Eff::<Pure, i32>::pure(21).and_then(|x| Eff::pure(x * 2));
        assert_eq!(eff.run_pure(), 42);
    }

    #[test]
    fn test_pure_chain() {
        let eff = Eff::<Pure, i32>::pure(10)
            .map(|x| x + 5)
            .and_then(|x| Eff::pure(x * 2))
            .map(|x| x + 12);
        assert_eq!(eff.run_pure(), 42);
    }

    #[test]
    fn test_lift() {
        let eff: Eff<Pure, i32> = Eff::lift(42);
        assert_eq!(eff.run_pure(), 42);
    }

    #[test]
    fn test_map_const() {
        let eff = Eff::<Pure, i32>::pure(100).map_const(42);
        assert_eq!(eff.run_pure(), 42);
    }

    #[test]
    fn test_void() {
        let eff = Eff::<Pure, i32>::pure(42).void();
        assert_eq!(eff.run_pure(), ());
    }

    #[test]
    fn test_then() {
        let eff = Eff::<Pure, i32>::pure(1).then(|| Eff::pure(42));
        assert_eq!(eff.run_pure(), 42);
    }
}
