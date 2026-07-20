//! Tail recursion optimization for stack-safe recursive computations.
//!
//! The `TailRec` trait and `tail_rec` function provide a way to perform
//! tail-recursive computations without stack overflow, by converting
//! recursion into iteration.
//!
//! # Example
//!
//! ```rust
//! use ordofp_core::tailrec::{TailRec, RecursionState, tail_rec};
//!
//! // Factorial using tail recursion
//! let factorial = tail_rec((5, 1), |(n, acc)| {
//!     if n == 0 {
//!         RecursionState::Done(acc)
//!     } else {
//!         RecursionState::Continue((n - 1, acc * n))
//!     }
//! });
//! assert_eq!(factorial, 120);
//!
//! // Using the trait method
//! let countdown = 10.rec(|x| {
//!     if x == 0 {
//!         RecursionState::Done("done")
//!     } else {
//!         RecursionState::Continue(x - 1)
//!     }
//! });
//! assert_eq!(countdown, "done");
//! ```

/// Represents the state of a tail-recursive computation.
///
/// - `Continue(next)`: Continue recursion with the next value
/// - `Done(result)`: Recursion is complete with the final result
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecursionState<Done, Continue> {
    /// Continue the recursion with a new input value.
    Continue(Continue),
    /// The recursion is complete with this result.
    Done(Done),
}

impl<D, C> RecursionState<D, C> {
    /// Returns `true` if this is a `Done` variant.
    #[inline]
    pub fn is_done(&self) -> bool {
        matches!(self, RecursionState::Done(_))
    }

    /// Returns `true` if this is a `Continue` variant.
    #[inline]
    pub fn is_continue(&self) -> bool {
        matches!(self, RecursionState::Continue(_))
    }

    /// Maps the `Done` value using the provided function.
    #[inline]
    pub fn map_done<D2, F>(self, f: F) -> RecursionState<D2, C>
    where
        F: FnOnce(D) -> D2,
    {
        match self {
            RecursionState::Done(d) => RecursionState::Done(f(d)),
            RecursionState::Continue(c) => RecursionState::Continue(c),
        }
    }

    /// Maps the `Continue` value using the provided function.
    #[inline]
    pub fn map_continue<C2, F>(self, f: F) -> RecursionState<D, C2>
    where
        F: FnOnce(C) -> C2,
    {
        match self {
            RecursionState::Done(d) => RecursionState::Done(d),
            RecursionState::Continue(c) => RecursionState::Continue(f(c)),
        }
    }
}

/// A trait for types that support tail-recursive operations.
///
/// This trait is automatically implemented for all `Sized` types.
///
/// # Example
///
/// ```rust
/// use ordofp_core::tailrec::{TailRec, RecursionState};
///
/// // Count down from 10 to 0
/// let result = 10.rec(|x| {
///     if x == 0 {
///         RecursionState::Done(x)
///     } else {
///         RecursionState::Continue(x - 1)
///     }
/// });
/// assert_eq!(result, 0);
/// ```
pub trait TailRec<Output> {
    /// Execute a tail-recursive function, converting recursion to iteration.
    ///
    /// The function `iterate` is called repeatedly until it returns
    /// `RecursionState::Done`, at which point the final value is returned.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::tailrec::{TailRec, RecursionState};
    ///
    /// // Sum numbers from 1 to n
    /// let sum = (10, 0).rec(|(n, acc)| {
    ///     if n == 0 {
    ///         RecursionState::Done(acc)
    ///     } else {
    ///         RecursionState::Continue((n - 1, acc + n))
    ///     }
    /// });
    /// assert_eq!(sum, 55);
    /// ```
    #[inline]
    fn rec<F>(self, iterate: F) -> Output
    where
        Self: Sized,
        F: Fn(Self) -> RecursionState<Output, Self>,
    {
        let mut state = iterate(self);
        loop {
            match state {
                RecursionState::Done(output) => return output,
                RecursionState::Continue(next) => state = iterate(next),
            }
        }
    }

    /// Execute a tail-recursive function using references.
    ///
    /// Similar to `rec`, but the iterate function receives a reference
    /// to the current state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::tailrec::{TailRec, RecursionState};
    ///
    /// let result = 5.rec_ref(|&x| {
    ///     if x == 0 {
    ///         RecursionState::Done(x)
    ///     } else {
    ///         RecursionState::Continue(x - 1)
    ///     }
    /// });
    /// assert_eq!(result, 0);
    /// ```
    #[inline]
    fn rec_ref<F>(&self, iterate: F) -> Output
    where
        Self: Sized + Clone,
        F: Fn(&Self) -> RecursionState<Output, Self>,
    {
        let mut state = iterate(self);
        loop {
            match state {
                RecursionState::Done(output) => return output,
                RecursionState::Continue(next) => state = iterate(&next),
            }
        }
    }
}

// Blanket implementation for all sized types
impl<T: Sized, Output> TailRec<Output> for T {}

/// Execute a tail-recursive function without using the trait.
///
/// This is a standalone function that performs the same operation as
/// `TailRec::rec`, but doesn't require importing the trait.
///
/// # Example
///
/// ```rust
/// use ordofp_core::tailrec::{tail_rec, RecursionState};
///
/// // Fibonacci using accumulator pattern
/// let fib = tail_rec((10, 0, 1), |(n, a, b)| {
///     if n == 0 {
///         RecursionState::Done(a)
///     } else {
///         RecursionState::Continue((n - 1, b, a + b))
///     }
/// });
/// assert_eq!(fib, 55);
/// ```
#[inline]
pub fn tail_rec<Input, Output, F>(input: Input, iterate: F) -> Output
where
    F: Fn(Input) -> RecursionState<Output, Input>,
{
    let mut state = iterate(input);
    loop {
        match state {
            RecursionState::Done(output) => return output,
            RecursionState::Continue(next) => state = iterate(next),
        }
    }
}

/// Execute a tail-recursive function with mutable state.
///
/// This variant allows the iterate function to mutate external state
/// while performing the recursion.
///
/// # Example
///
/// ```rust
/// use ordofp_core::tailrec::{tail_rec_mut, RecursionState};
///
/// let mut log = Vec::new();
/// let result = tail_rec_mut(5, |x| {
///     log.push(x);
///     if x == 0 {
///         RecursionState::Done("done")
///     } else {
///         RecursionState::Continue(x - 1)
///     }
/// });
/// assert_eq!(result, "done");
/// assert_eq!(log, vec![5, 4, 3, 2, 1, 0]);
/// ```
#[inline]
pub fn tail_rec_mut<Input, Output, F>(input: Input, mut iterate: F) -> Output
where
    F: FnMut(Input) -> RecursionState<Output, Input>,
{
    let mut state = iterate(input);
    loop {
        match state {
            RecursionState::Done(output) => return output,
            RecursionState::Continue(next) => state = iterate(next),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_countdown() {
        let result = 10.rec(|x| {
            if x == 0 {
                RecursionState::Done(x)
            } else {
                RecursionState::Continue(x - 1)
            }
        });
        assert_eq!(result, 0);
    }

    #[test]
    fn test_factorial() {
        let result = tail_rec((5, 1), |(n, acc)| {
            if n == 0 {
                RecursionState::Done(acc)
            } else {
                RecursionState::Continue((n - 1, acc * n))
            }
        });
        assert_eq!(result, 120);
    }

    #[test]
    fn test_fibonacci() {
        let result = tail_rec((10, 0u64, 1u64), |(n, a, b)| {
            if n == 0 {
                RecursionState::Done(a)
            } else {
                RecursionState::Continue((n - 1, b, a + b))
            }
        });
        assert_eq!(result, 55);
    }

    #[test]
    fn test_sum_to_n() {
        let result = (100, 0).rec(|(n, acc)| {
            if n == 0 {
                RecursionState::Done(acc)
            } else {
                RecursionState::Continue((n - 1, acc + n))
            }
        });
        assert_eq!(result, 5050);
    }

    #[test]
    fn test_rec_ref() {
        let result = 5.rec_ref(|&x| {
            if x == 0 {
                RecursionState::Done(x)
            } else {
                RecursionState::Continue(x - 1)
            }
        });
        assert_eq!(result, 0);
    }

    #[test]
    fn test_large_recursion_no_overflow() {
        // This would overflow the stack with normal recursion
        let result = tail_rec(100_000, |x| {
            if x == 0 {
                RecursionState::Done(0)
            } else {
                RecursionState::Continue(x - 1)
            }
        });
        assert_eq!(result, 0);
    }

    #[test]
    fn test_with_option() {
        let result = Some(5).rec(|opt| match opt {
            Some(0) => RecursionState::Done(Some(0)),
            Some(n) => RecursionState::Continue(Some(n - 1)),
            None => RecursionState::Done(None),
        });
        assert_eq!(result, Some(0));
    }

    #[test]
    fn test_with_result() {
        let result: Result<i32, &str> = Ok(5).rec(|res| match res {
            Ok(0) => RecursionState::Done(Ok(0)),
            Ok(n) => RecursionState::Continue(Ok(n - 1)),
            Err(e) => RecursionState::Done(Err(e)),
        });
        assert_eq!(result, Ok(0));
    }

    #[test]
    fn test_tail_rec_mut() {
        let mut count = 0;
        let result = tail_rec_mut(5, |x| {
            count += 1;
            if x == 0 {
                RecursionState::Done(x)
            } else {
                RecursionState::Continue(x - 1)
            }
        });
        assert_eq!(result, 0);
        assert_eq!(count, 6); // Called 6 times: 5, 4, 3, 2, 1, 0
    }

    #[test]
    fn test_recursion_state_is_done() {
        let done: RecursionState<i32, i32> = RecursionState::Done(42);
        let cont: RecursionState<i32, i32> = RecursionState::Continue(42);
        assert!(done.is_done());
        assert!(!done.is_continue());
        assert!(!cont.is_done());
        assert!(cont.is_continue());
    }

    #[test]
    fn test_recursion_state_map_done() {
        let state: RecursionState<i32, i32> = RecursionState::Done(5);
        let mapped = state.map_done(|x| x * 2);
        assert_eq!(mapped, RecursionState::Done(10));

        let state: RecursionState<i32, i32> = RecursionState::Continue(5);
        let mapped = state.map_done(|x| x * 2);
        assert_eq!(mapped, RecursionState::Continue(5));
    }

    #[test]
    fn test_recursion_state_map_continue() {
        let state: RecursionState<i32, i32> = RecursionState::Continue(5);
        let mapped = state.map_continue(|x| x * 2);
        assert_eq!(mapped, RecursionState::Continue(10));

        let state: RecursionState<i32, i32> = RecursionState::Done(5);
        let mapped = state.map_continue(|x| x * 2);
        assert_eq!(mapped, RecursionState::Done(5));
    }

    #[test]
    fn test_gcd() {
        // Greatest common divisor using Euclidean algorithm
        let gcd = tail_rec((48, 18), |(a, b)| {
            if b == 0 {
                RecursionState::Done(a)
            } else {
                RecursionState::Continue((b, a % b))
            }
        });
        assert_eq!(gcd, 6);
    }

    #[test]
    fn test_power() {
        // Compute a^n using tail recursion
        let power = tail_rec((2i64, 10, 1i64), |(base, exp, acc)| {
            if exp == 0 {
                RecursionState::Done(acc)
            } else {
                RecursionState::Continue((base, exp - 1, acc * base))
            }
        });
        assert_eq!(power, 1024);
    }

    #[test]
    fn test_immediate_done_skips_loop() {
        // Edge case: the iterate function returns Done on the very first call,
        // so the Continue arm of the loop is never reached.  This exercises
        // the boundary where tail_rec, tail_rec_mut, and TailRec::rec must all
        // produce the correct output without entering any continuation step.
        let result = tail_rec(42, RecursionState::<i32, i32>::Done);
        assert_eq!(
            result, 42,
            "tail_rec must return immediately when Done on first call"
        );

        let result_mut = tail_rec_mut(99, RecursionState::<i32, i32>::Done);
        assert_eq!(
            result_mut, 99,
            "tail_rec_mut must return immediately when Done on first call"
        );

        let result_trait = 7.rec(RecursionState::<i32, i32>::Done);
        assert_eq!(
            result_trait, 7,
            "TailRec::rec must return immediately when Done on first call"
        );
    }
}
