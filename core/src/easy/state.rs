//! Easy State Management
//!
//! Simplified state handling that hides the `StateT` machinery.
//!
//! # Example
//!
//! ```rust
//! use ordofp_core::easy::*;
//!
//! // Simple counter
//! let final_count = run_with_state(0, |count| {
//!     *count += 1;
//!     *count += 1;
//!     *count
//! });
//! assert_eq!(final_count, 2);
//!
//! // State with modifications
//! let (result, final_state) = run_state(0, |count| {
//!     *count += 10;
//!     "done"
//! });
//! assert_eq!(result, "done");
//! assert_eq!(final_state, 10);
//! ```

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::cell::RefCell;

// =============================================================================
// Basic State Operations
// =============================================================================

/// Run a computation with mutable state, returning only the result.
///
/// # Example
///
/// ```rust
/// use ordofp_core::easy::run_with_state;
///
/// let result = run_with_state(0, |state| {
///     *state += 1;
///     *state * 2
/// });
/// assert_eq!(result, 2);
/// ```
#[inline]
pub fn run_with_state<S, A, F>(initial: S, computation: F) -> A
where
    F: FnOnce(&mut S) -> A,
{
    let mut state = initial;
    computation(&mut state)
}

/// Run a computation with state, returning both result and final state.
///
/// # Example
///
/// ```rust
/// use ordofp_core::easy::run_state;
///
/// let (result, final_state) = run_state(0, |state| {
///     *state += 10;
///     "done"
/// });
/// assert_eq!(result, "done");
/// assert_eq!(final_state, 10);
/// ```
#[inline]
pub fn run_state<S, A, F>(initial: S, computation: F) -> (A, S)
where
    F: FnOnce(&mut S) -> A,
{
    let mut state = initial;
    let result = computation(&mut state);
    (result, state)
}

/// Run a computation that only modifies state, returning the final state.
///
/// # Example
///
/// ```rust
/// use ordofp_core::easy::exec_state;
///
/// let final_state = exec_state(0, |state| {
///     *state += 1;
///     *state *= 2;
/// });
/// assert_eq!(final_state, 2);
/// ```
#[inline]
pub fn exec_state<S, F>(initial: S, computation: F) -> S
where
    F: FnOnce(&mut S),
{
    let mut state = initial;
    computation(&mut state);
    state
}

/// Run a computation with read-only state.
///
/// # Example
///
/// ```rust
/// use ordofp_core::easy::eval_state;
///
/// let result = eval_state(&42, |state| *state * 2);
/// assert_eq!(result, 84);
/// ```
#[inline]
pub fn eval_state<S, A, F>(state: &S, computation: F) -> A
where
    F: FnOnce(&S) -> A,
{
    computation(state)
}

// =============================================================================
// State Monad Style
// =============================================================================

/// A stateful computation that can be composed.
pub struct State<S, A> {
    run: Box<dyn FnOnce(S) -> (A, S)>,
}

impl<S: 'static, A: 'static> State<S, A> {
    /// Create a new stateful computation.
    #[inline]
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce(S) -> (A, S) + 'static,
    {
        State { run: Box::new(f) }
    }

    /// Run the computation with initial state.
    #[inline]
    pub fn run(self, initial: S) -> (A, S) {
        (self.run)(initial)
    }

    /// Run and return only the result.
    #[inline]
    pub fn eval(self, initial: S) -> A {
        self.run(initial).0
    }

    /// Run and return only the final state.
    #[inline]
    pub fn exec(self, initial: S) -> S {
        self.run(initial).1
    }

    /// Map over the result.
    #[inline]
    pub fn map<B: 'static, F>(self, f: F) -> State<S, B>
    where
        F: FnOnce(A) -> B + 'static,
    {
        State::new(move |s| {
            let (a, s2) = (self.run)(s);
            (f(a), s2)
        })
    }

    /// Chain with another stateful computation.
    #[inline]
    pub fn and_then<B: 'static, F>(self, f: F) -> State<S, B>
    where
        F: FnOnce(A) -> State<S, B> + 'static,
    {
        State::new(move |s| {
            let (a, s2) = (self.run)(s);
            f(a).run(s2)
        })
    }

    /// Sequence two computations, keeping the second result.
    #[inline]
    pub fn then<B: 'static>(self, next: State<S, B>) -> State<S, B> {
        State::new(move |s| {
            let (_, s2) = (self.run)(s);
            next.run(s2)
        })
    }
}

/// Create a pure stateful computation.
#[inline]
pub fn state_pure<S: 'static, A: 'static>(value: A) -> State<S, A> {
    State::new(move |s| (value, s))
}

/// Get the current state.
#[inline]
pub fn get<S: Clone + 'static>() -> State<S, S> {
    State::new(|s: S| (s.clone(), s))
}

/// Set the state.
#[inline]
pub fn put<S: 'static>(new_state: S) -> State<S, ()> {
    State::new(|_| ((), new_state))
}

/// Modify the state with a function.
#[inline]
pub fn modify<S: 'static, F>(f: F) -> State<S, ()>
where
    F: FnOnce(S) -> S + 'static,
{
    State::new(|s| ((), f(s)))
}

/// Get a value derived from the state.
#[inline]
pub fn gets<S: 'static, A: 'static, F>(f: F) -> State<S, A>
where
    F: FnOnce(&S) -> A + 'static,
{
    State::new(|s| {
        let a = f(&s);
        (a, s)
    })
}

// =============================================================================
// Stateful Collections
// =============================================================================

/// Accumulate values into a vector.
///
/// # Example
///
/// ```rust
/// use ordofp_core::easy::collect_with_state;
///
/// let collected = collect_with_state(3, |i, acc| {
///     acc.push(i * 2);
/// });
/// assert_eq!(collected, vec![0, 2, 4]);
/// ```
pub fn collect_with_state<A, F>(count: usize, mut collector: F) -> Vec<A>
where
    F: FnMut(usize, &mut Vec<A>),
{
    let mut acc = Vec::new();
    for i in 0..count {
        collector(i, &mut acc);
    }
    acc
}

/// Fold with mutable accumulator.
///
/// # Example
///
/// ```rust
/// use ordofp_core::easy::fold_mut;
///
/// let sum = fold_mut(&[1, 2, 3, 4], 0, |acc, x| *acc += x);
/// assert_eq!(sum, 10);
/// ```
pub fn fold_mut<T, A, F>(items: &[T], initial: A, mut folder: F) -> A
where
    F: FnMut(&mut A, &T),
{
    let mut acc = initial;
    for item in items {
        folder(&mut acc, item);
    }
    acc
}

/// Map with access to mutable state.
///
/// # Example
///
/// ```rust
/// use ordofp_core::easy::map_with_state;
///
/// let (mapped, count) = map_with_state(&[1, 2, 3], 0, |x, count| {
///     *count += 1;
///     x * 2
/// });
/// assert_eq!(mapped, vec![2, 4, 6]);
/// assert_eq!(count, 3);
/// ```
pub fn map_with_state<T, S, U, F>(items: &[T], initial: S, mut mapper: F) -> (Vec<U>, S)
where
    F: FnMut(&T, &mut S) -> U,
{
    let mut state = initial;
    let mapped: Vec<U> = items.iter().map(|x| mapper(x, &mut state)).collect();
    (mapped, state)
}

// =============================================================================
// Scoped Local State
// =============================================================================

/// A scoped state cell for implicit state passing.
///
/// This is a plain `RefCell`, **not** thread-local storage — to keep one
/// instance per thread, place it in a `thread_local!` block yourself.
/// `run` is also **not re-entrant**: nesting `run` calls on the same cell
/// overwrites the outer state, and the state is not restored if the
/// closure panics.
pub struct LocalState<S> {
    cell: RefCell<Option<S>>,
}

impl<S> LocalState<S> {
    /// Create a new local state.
    pub const fn new() -> Self {
        LocalState {
            cell: RefCell::new(None),
        }
    }

    /// Run a computation with this local state.
    pub fn run<A, F>(&self, initial: S, f: F) -> A
    where
        F: FnOnce() -> A,
    {
        *self.cell.borrow_mut() = Some(initial);
        let result = f();
        *self.cell.borrow_mut() = None;
        result
    }

    /// Get a reference to the current state.
    ///
    /// # Panics
    /// Panics if called outside of a `run` context.
    pub fn with<A, F>(&self, f: F) -> A
    where
        F: FnOnce(&S) -> A,
    {
        let borrow = self.cell.borrow();
        f(borrow.as_ref().expect("LocalState: not in run context"))
    }

    /// Modify the current state.
    ///
    /// # Panics
    /// Panics if called outside of a `run` context.
    pub fn modify_with<A, F>(&self, f: F) -> A
    where
        F: FnOnce(&mut S) -> A,
    {
        let mut borrow = self.cell.borrow_mut();
        f(borrow.as_mut().expect("LocalState: not in run context"))
    }
}

impl<S> Default for LocalState<S> {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_with_state() {
        let result = run_with_state(0, |state| {
            *state += 1;
            *state * 2
        });
        assert_eq!(result, 2);
    }

    #[test]
    fn test_run_state() {
        let (result, final_state) = run_state(0, |state| {
            *state += 10;
            "done"
        });
        assert_eq!(result, "done");
        assert_eq!(final_state, 10);
    }

    #[test]
    fn test_exec_state() {
        let final_state = exec_state(0, |state| {
            *state += 1;
            *state *= 2;
        });
        assert_eq!(final_state, 2);
    }

    #[test]
    fn test_state_monad() {
        let comp = get::<i32>().and_then(|x| State::new(move |s| (x + 1, s + 10)));

        let (result, final_state) = comp.run(5);
        assert_eq!(result, 6); // x was 5, + 1 = 6
        assert_eq!(final_state, 15); // s was 5, + 10 = 15
    }

    #[test]
    fn test_modify() {
        let comp = modify(|x: i32| x * 2).then(get());

        let (result, _) = comp.run(5);
        assert_eq!(result, 10);
    }

    #[test]
    fn test_collect_with_state() {
        let collected = collect_with_state(3, |i, acc| {
            acc.push(i * 2);
        });
        assert_eq!(collected, alloc::vec![0, 2, 4]);
    }

    #[test]
    fn test_fold_mut() {
        let sum = fold_mut(&[1, 2, 3, 4], 0, |acc, x| *acc += x);
        assert_eq!(sum, 10);
    }

    #[test]
    fn test_map_with_state() {
        let (mapped, count) = map_with_state(&[1, 2, 3], 0, |x, count| {
            *count += 1;
            x * 2
        });
        assert_eq!(mapped, alloc::vec![2, 4, 6]);
        assert_eq!(count, 3);
    }

    #[test]
    fn test_local_state() {
        let state: LocalState<i32> = LocalState::new();

        let result = state.run(42, || state.with(|s| *s * 2));

        assert_eq!(result, 84);
    }
}
