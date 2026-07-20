//! Effect Operations - Performing and Lifting Effects
//!
//! This module provides the core operations for working with effects:
//! - `pure` - Lift a value into an effectful computation
//! - `perform` - Perform an effect operation
//!
//! # Zero-Cost Design
//!
//! These operations are designed to compile away when possible:
//! - `pure(x)` for `Pure` row compiles to just `x`
//! - Effect operations are inlined by handlers

use super::effect::Eff;
use super::row::{EffectRow, Pure};

// =============================================================================
// Pure Operations
// =============================================================================

/// Lift a pure value into an effectful computation.
///
/// This is the monadic `return` / applicative `pure` operation.
/// When the effect row is `Pure`, this compiles to just the value.
///
/// # Example
///
/// ```rust
/// use ordofp_core::nexus::prelude::*;
///
/// let comp: Eff<Pure, i32> = pure(42);
/// assert_eq!(comp.run_pure(), 42);
/// ```
#[inline(always)]
pub fn pure<A>(value: A) -> Eff<Pure, A> {
    Eff::pure(value)
}

/// Lift a value into any effect row.
///
/// Unlike `pure`, this works for any effect row, not just `Pure`.
#[inline(always)]
pub fn lift<R: EffectRow, A>(value: A) -> Eff<R, A> {
    Eff::from_value(value)
}

// =============================================================================
// Effect Performance
// =============================================================================

/// Perform an effect operation.
///
/// This is the core primitive for invoking effects. The actual
/// implementation is provided by effect handlers.
///
/// # Example
///
/// ```rust
/// use ordofp_core::nexus::prelude::*;
///
/// // Get the current state
/// fn get<S: Clone>() -> Eff<StateEff, S> {
///     perform(StateOp::<S>::Get)
/// }
///
/// let _comp: Eff<StateEff, i32> = get::<i32>();
/// ```
pub fn perform<R: EffectRow, Op, A>(_op: Op) -> Eff<R, A> {
    // The actual implementation would use effect handlers
    // This is a placeholder that demonstrates the API
    Eff::lazy(|| crate::cold_panic!("perform requires an effect handler"))
}

// =============================================================================
// Combinators
// =============================================================================

/// Sequence two computations, returning the result of the second.
///
/// # Example
///
/// ```rust
/// use ordofp_core::nexus::prelude::*;
///
/// let comp = sequence(pure(1), pure(42));
/// assert_eq!(comp.run_pure(), 42);
/// ```
#[inline]
pub fn sequence<R: EffectRow + 'static, A: 'static, B: 'static>(
    first: Eff<R, A>,
    second: Eff<R, B>,
) -> Eff<R, B> {
    first.and_then(move |_| second)
}

/// Execute a computation only for its effects, discarding the result.
///
/// # Example
///
/// ```rust
/// use ordofp_core::nexus::prelude::*;
/// use ordofp_core::nexus::put;
///
/// // `put` is a handler stub (see its own docs), so the resulting
/// // computation is constructed but intentionally not run here.
/// let comp: Eff<StateEff, ()> = discard(put(42));
/// let _ = comp; // State would be updated, but we get () back, once run
/// ```
#[inline]
pub fn discard<R: EffectRow + 'static, A: 'static>(eff: Eff<R, A>) -> Eff<R, ()> {
    eff.void()
}

/// Apply a function in an effectful context.
///
/// # Example
///
/// ```rust
/// use ordofp_core::nexus::prelude::*;
///
/// let add = |x: i32, y: i32| x + y;
/// let result = apply2(pure(20), pure(22), add);
/// assert_eq!(result.run_pure(), 42);
/// ```
#[inline]
pub fn apply2<R: EffectRow + 'static, A: 'static, B: 'static, C: 'static, F>(
    a: Eff<R, A>,
    b: Eff<R, B>,
    f: F,
) -> Eff<R, C>
where
    F: FnOnce(A, B) -> C + 'static,
{
    a.and_then(move |a_val| b.map(move |b_val| f(a_val, b_val)))
}

/// Tuple two effectful computations.
///
/// # Example
///
/// ```rust
/// use ordofp_core::nexus::prelude::*;
///
/// let comp = tuple(pure(1), pure(2));
/// assert_eq!(comp.run_pure(), (1, 2));
/// ```
#[inline]
pub fn tuple<R: EffectRow + 'static, A: 'static, B: 'static>(
    a: Eff<R, A>,
    b: Eff<R, B>,
) -> Eff<R, (A, B)> {
    apply2(a, b, |a, b| (a, b))
}

/// Execute a list of computations in order, collecting all results into a `Vec`.
///
/// Each computation is sequenced left-to-right. The effects of each computation
/// are observed before the next one begins, preserving ordering semantics.
///
/// # Example
///
/// ```rust
/// use ordofp_core::nexus::prelude::*;
///
/// let comps = vec![pure(1), pure(2), pure(3)];
/// let comp = sequence_vec(comps);
/// assert_eq!(comp.run_pure(), vec![1, 2, 3]);
/// ```
#[inline]
pub fn sequence_vec<R: EffectRow + 'static, A: 'static>(
    effs: alloc::vec::Vec<Eff<R, A>>,
) -> Eff<R, alloc::vec::Vec<A>> {
    // Pre-allocate the result vector with the exact number of elements we expect
    // to collect, avoiding reallocation as each computation's result is pushed.
    let n = effs.len();
    effs.into_iter().fold(
        Eff::from_value(alloc::vec::Vec::with_capacity(n)),
        |acc, eff| {
            acc.and_then(move |mut vec| {
                eff.map(move |a| {
                    vec.push(a);
                    vec
                })
            })
        },
    )
}

/// Map an effectful function over a list, collecting all results into a `Vec`.
///
/// Applies `f` to each element of `items` in order, sequencing the resulting
/// computations left-to-right. This is the `traverse` from the `Traversable`
/// type class — equivalent to `map` followed by `sequence_vec`.
///
/// # Example
///
/// ```rust
/// use ordofp_core::nexus::prelude::*;
///
/// let result = traverse(vec![1, 2, 3], |x| pure(x * 2));
/// assert_eq!(result.run_pure(), vec![2, 4, 6]);
/// ```
#[inline]
pub fn traverse<R: EffectRow + 'static, A: 'static, B: 'static, F>(
    items: alloc::vec::Vec<A>,
    f: F,
) -> Eff<R, alloc::vec::Vec<B>>
where
    F: Fn(A) -> Eff<R, B> + 'static,
{
    let effs: alloc::vec::Vec<Eff<R, B>> = items.into_iter().map(f).collect();
    sequence_vec(effs)
}

// =============================================================================
// Conditional Combinators
// =============================================================================

/// Execute a computation only if the condition is true.
///
/// # Example
///
/// ```rust
/// use ordofp_core::nexus::prelude::*;
///
/// let comp = when(true, || pure(42));
/// assert_eq!(comp.run_pure(), Some(42));
///
/// let comp = when(false, || pure(42));
/// assert_eq!(comp.run_pure(), None);
/// ```
#[inline]
pub fn when<R: EffectRow + 'static, A: 'static, F>(condition: bool, f: F) -> Eff<R, Option<A>>
where
    F: FnOnce() -> Eff<R, A> + 'static,
{
    if condition {
        f().map(Some)
    } else {
        Eff::from_value(None)
    }
}

/// Execute a computation unless the condition is true.
///
/// This is the dual of [`when`]: the computation runs only when `condition`
/// is `false`, wrapping the result in `Some`. When `condition` is `true`,
/// the computation is skipped and `None` is returned immediately.
///
/// # Example
///
/// ```rust
/// use ordofp_core::nexus::prelude::*;
///
/// let comp = unless(false, || pure(42));
/// assert_eq!(comp.run_pure(), Some(42));
///
/// let comp = unless(true, || pure(42));
/// assert_eq!(comp.run_pure(), None);
/// ```
#[inline]
pub fn unless<R: EffectRow + 'static, A: 'static, F>(condition: bool, f: F) -> Eff<R, Option<A>>
where
    F: FnOnce() -> Eff<R, A> + 'static,
{
    when(!condition, f)
}

/// Choose between two computations based on a condition.
///
/// Eagerly selects `then_branch` when `condition` is `true`, otherwise
/// `else_branch`. Both branches must produce the same value type.
///
/// # Example
///
/// ```rust
/// use ordofp_core::nexus::prelude::*;
///
/// let x = 10;
/// let result = if_else(x > 5, pure("big"), pure("small"));
/// assert_eq!(result.run_pure(), "big");
/// ```
#[inline]
pub fn if_else<R: EffectRow + 'static, A: 'static>(
    condition: bool,
    then_branch: Eff<R, A>,
    else_branch: Eff<R, A>,
) -> Eff<R, A> {
    if condition { then_branch } else { else_branch }
}

// =============================================================================
// Looping Combinators
// =============================================================================

/// Repeat a computation `n` times, collecting the results into a `Vec`.
///
/// The computation `eff` is mapped once and its single result cloned `n` times.
/// This means any effects embedded in `eff` are observed exactly once, not `n`
/// times — use [`traverse`] with a repeated input if per-iteration effects are
/// required.
///
/// # Arguments
///
/// * `n` - Number of times to replicate the result. If `0`, returns an empty `Vec`.
/// * `eff` - The effectful computation whose result is replicated.
///
/// # Example
///
/// ```rust
/// use ordofp_core::nexus::prelude::*;
///
/// let comp: Eff<Pure, Vec<i32>> = replicate(3, pure(7));
/// assert_eq!(comp.run_pure(), vec![7, 7, 7]);
///
/// let empty: Eff<Pure, Vec<i32>> = replicate(0, pure(99));
/// assert_eq!(empty.run_pure(), vec![]);
/// ```
#[inline]
pub fn replicate<R: EffectRow + 'static, A: Clone + 'static>(
    n: usize,
    eff: Eff<R, A>,
) -> Eff<R, alloc::vec::Vec<A>> {
    // For pure computations, we can just clone the result
    eff.map(move |a| alloc::vec![a; n])
}

/// Iterate a stateful computation until a predicate is satisfied.
///
/// Starting from `initial`, repeatedly applies `step` to produce the next
/// value until `predicate` returns `true`. Returns the first value that
/// satisfies the predicate without applying `step` again.
///
/// # Example
///
/// ```rust
/// use ordofp_core::nexus::prelude::*;
///
/// // Count up to 5
/// let comp: Eff<Pure, i32> = iterate_until(0, |&n| n >= 5, |n| pure(n + 1));
/// assert_eq!(comp.run_pure(), 5);
/// ```
pub fn iterate_until<R: EffectRow + 'static, A: 'static, P, F>(
    initial: A,
    predicate: P,
    step: F,
) -> Eff<R, A>
where
    P: Fn(&A) -> bool + 'static,
    F: Fn(A) -> Eff<R, A> + 'static,
{
    if predicate(&initial) {
        Eff::from_value(initial)
    } else {
        step(initial).and_then(move |next| iterate_until(next, predicate, step))
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pure_op() {
        let comp = pure(42);
        assert_eq!(comp.run_pure(), 42);
    }

    #[test]
    fn test_lift() {
        let comp: Eff<Pure, i32> = lift(42);
        assert_eq!(comp.run_pure(), 42);
    }

    #[test]
    fn test_sequence() {
        let comp = sequence(pure(1), pure(42));
        assert_eq!(comp.run_pure(), 42);
    }

    #[test]
    fn test_discard() {
        let comp = discard(pure(42));
        assert_eq!(comp.run_pure(), ());
    }

    #[test]
    fn test_apply2() {
        let comp = apply2(pure(20), pure(22), |a, b| a + b);
        assert_eq!(comp.run_pure(), 42);
    }

    #[test]
    fn test_tuple() {
        let comp = tuple(pure(1), pure(2));
        assert_eq!(comp.run_pure(), (1, 2));
    }

    #[test]
    fn test_when_true() {
        let comp = when(true, || pure(42));
        assert_eq!(comp.run_pure(), Some(42));
    }

    #[test]
    fn test_when_false() {
        let comp = when(false, || pure(42));
        assert_eq!(comp.run_pure(), None);
    }

    #[test]
    fn test_unless() {
        let comp = unless(false, || pure(42));
        assert_eq!(comp.run_pure(), Some(42));
    }

    #[test]
    fn test_if_else() {
        let comp = if_else(true, pure(42), pure(0));
        assert_eq!(comp.run_pure(), 42);
    }

    #[test]
    fn test_sequence_vec() {
        let effs = alloc::vec![pure(1), pure(2), pure(3)];
        let comp = sequence_vec(effs);
        assert_eq!(comp.run_pure(), alloc::vec![1, 2, 3]);
    }

    #[test]
    fn test_traverse() {
        let items = alloc::vec![1, 2, 3];
        let comp = traverse(items, |x| pure(x * 2));
        assert_eq!(comp.run_pure(), alloc::vec![2, 4, 6]);
    }

    #[test]
    fn test_replicate() {
        let comp = replicate(3, pure(42));
        assert_eq!(comp.run_pure(), alloc::vec![42, 42, 42]);
    }
}
