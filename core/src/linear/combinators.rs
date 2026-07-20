//! Linear Combinators - Utilities for working with linear values
//!
//! > *"Compositio linearis"*
//! > — Linear composition. (Neo-Latin)
//!
//! This module provides combinators for manipulating linear values,
//! including pair operations, currying, and consumption utilities.

use super::Linearis;

/// Create a linear pair from two linear values.
///
/// # Example
///
/// ```rust
/// use ordofp_core::linear::{Linearis, linear_pair};
///
/// let x = Linearis::new(1);
/// let y = Linearis::new("hello");
/// let pair = linear_pair(x, y);
/// assert_eq!(pair.consume(), (1, "hello"));
/// ```
#[inline]
pub fn linear_pair<A, B>(a: Linearis<A>, b: Linearis<B>) -> Linearis<(A, B)> {
    Linearis::new((a.consume(), b.consume()))
}

/// Extract the first element of a linear pair.
///
/// The second element is consumed and discarded.
///
/// # Example
///
/// ```rust
/// use ordofp_core::linear::{Linearis, linear_first};
///
/// let pair = Linearis::new((1, "hello"));
/// let first = linear_first(pair);
/// assert_eq!(first.consume(), 1);
/// ```
#[inline]
pub fn linear_first<A, B>(pair: Linearis<(A, B)>) -> Linearis<A> {
    let (a, _) = pair.consume();
    Linearis::new(a)
}

/// Extract the second element of a linear pair.
///
/// The first element is consumed and discarded.
///
/// # Example
///
/// ```rust
/// use ordofp_core::linear::{Linearis, linear_second};
///
/// let pair = Linearis::new((1, "hello"));
/// let second = linear_second(pair);
/// assert_eq!(second.consume(), "hello");
/// ```
#[inline]
pub fn linear_second<A, B>(pair: Linearis<(A, B)>) -> Linearis<B> {
    let (_, b) = pair.consume();
    Linearis::new(b)
}

/// Swap the elements of a linear pair.
///
/// # Example
///
/// ```rust
/// use ordofp_core::linear::{Linearis, linear_swap};
///
/// let pair = Linearis::new((1, "hello"));
/// let swapped = linear_swap(pair);
/// assert_eq!(swapped.consume(), ("hello", 1));
/// ```
#[inline]
pub fn linear_swap<A, B>(pair: Linearis<(A, B)>) -> Linearis<(B, A)> {
    let (a, b) = pair.consume();
    Linearis::new((b, a))
}

/// A curried linear function wrapper.
///
/// This struct holds a partially applied curried function.
pub struct CurriedLinear<A, B, C, F>
where
    F: FnOnce(Linearis<(A, B)>) -> C,
{
    f: F,
    a: Linearis<A>,
    _phantom: core::marker::PhantomData<(B, C)>,
}

impl<A, B, C, F> CurriedLinear<A, B, C, F>
where
    F: FnOnce(Linearis<(A, B)>) -> C,
{
    /// Apply the second argument to complete the curried function.
    #[inline]
    pub fn apply(self, b: Linearis<B>) -> C {
        let pair = linear_pair(self.a, b);
        (self.f)(pair)
    }
}

/// Curry a linear function taking a pair.
///
/// Transforms `Linearis<(A, B)> -> C` into a two-step application.
/// Returns a closure that, when given the first argument, returns
/// a `CurriedLinear` that can be applied to the second argument.
///
/// # Example
///
/// ```rust
/// use ordofp_core::linear::{Linearis, linear_curry};
///
/// let add_pair = |pair: Linearis<(i32, i32)>| {
///     let (a, b) = pair.consume();
///     a + b
/// };
///
/// let curried = linear_curry(add_pair);
/// let partial = curried(Linearis::new(1));
/// let result = partial.apply(Linearis::new(2));
/// assert_eq!(result, 3);
/// ```
pub fn linear_curry<A, B, C, F>(f: F) -> impl FnOnce(Linearis<A>) -> CurriedLinear<A, B, C, F>
where
    F: FnOnce(Linearis<(A, B)>) -> C,
{
    move |a: Linearis<A>| CurriedLinear {
        f,
        a,
        _phantom: core::marker::PhantomData,
    }
}

/// Uncurry a linear function.
///
/// Transforms `Linearis<A> -> Linearis<B> -> C` into `Linearis<(A, B)> -> C`.
///
/// # Example
///
/// ```rust
/// use ordofp_core::linear::{Linearis, linear_uncurry};
///
/// let add = |a: Linearis<i32>| move |b: Linearis<i32>| a.consume() + b.consume();
///
/// let uncurried = linear_uncurry(add);
/// let result = uncurried(Linearis::new((1, 2)));
/// assert_eq!(result, 3);
/// ```
pub fn linear_uncurry<A, B, C, F, G>(f: F) -> impl FnOnce(Linearis<(A, B)>) -> C
where
    F: FnOnce(Linearis<A>) -> G,
    G: FnOnce(Linearis<B>) -> C,
{
    move |pair: Linearis<(A, B)>| {
        let (a, b) = pair.consume();
        f(Linearis::new(a))(Linearis::new(b))
    }
}

/// Consume both values of a linear pair, combining them.
///
/// # Example
///
/// ```rust
/// use ordofp_core::linear::{Linearis, consume_both};
///
/// let pair = Linearis::new((1, 2));
/// let sum = consume_both(pair, |a, b| a + b);
/// assert_eq!(sum, 3);
/// ```
#[inline]
pub fn consume_both<A, B, C, F>(pair: Linearis<(A, B)>, f: F) -> C
where
    F: FnOnce(A, B) -> C,
{
    let (a, b) = pair.consume();
    f(a, b)
}

/// Consume one of two linear values based on a condition.
///
/// # Example
///
/// ```rust
/// use ordofp_core::linear::{Linearis, consume_either};
///
/// let left = Linearis::new(1);
/// let right = Linearis::new(2);
///
/// // Consume left, discard right
/// let result = consume_either(true, left, right, |n| n * 10);
/// assert_eq!(result, 10);
/// ```
#[inline]
pub fn consume_either<A, B, F>(use_left: bool, left: Linearis<A>, right: Linearis<A>, f: F) -> B
where
    F: FnOnce(A) -> B,
{
    if use_left {
        let _ = right.consume();
        f(left.consume())
    } else {
        let _ = left.consume();
        f(right.consume())
    }
}

/// Apply a linear function to a linear value.
///
/// # Example
///
/// ```rust
/// use ordofp_core::linear::{Linearis, linear_apply};
///
/// let f = Linearis::new(|x: i32| x * 2);
/// let x = Linearis::new(5);
/// let result = linear_apply(f, x);
/// assert_eq!(result.consume(), 10);
/// ```
#[inline]
pub fn linear_apply<A, B, F>(f: Linearis<F>, a: Linearis<A>) -> Linearis<B>
where
    F: FnOnce(A) -> B,
{
    Linearis::new(f.consume()(a.consume()))
}

/// Compose two linear functions.
///
/// # Example
///
/// ```rust
/// use ordofp_core::linear::linear_compose;
///
/// let f = |x: i32| x + 1;
/// let g = |x: i32| x * 2;
///
/// let composed = linear_compose(f, g);
/// assert_eq!(composed(5), 12); // g(f(5)) = (5 + 1) * 2 = 12
/// ```
#[inline]
pub fn linear_compose<A, B, C, F, G>(f: F, g: G) -> impl FnOnce(A) -> C
where
    F: FnOnce(A) -> B,
    G: FnOnce(B) -> C,
{
    move |a| g(f(a))
}

/// Flip the arguments of a two-argument linear function.
///
/// # Example
///
/// ```rust
/// use ordofp_core::linear::linear_flip;
///
/// let sub = |a: i32, b: i32| a - b;
/// let flipped = linear_flip(sub);
/// assert_eq!(flipped(1, 5), 4); // 5 - 1 = 4
/// ```
#[inline]
pub fn linear_flip<A, B, C, F>(f: F) -> impl FnOnce(B, A) -> C
where
    F: FnOnce(A, B) -> C,
{
    move |b, a| f(a, b)
}

/// Create a constant linear function.
///
/// Returns a function that always returns the given value,
/// ignoring its argument.
///
/// # Example
///
/// ```rust
/// use ordofp_core::linear::{Linearis, linear_const};
///
/// let always_42 = linear_const(Linearis::new(42));
/// let result = always_42(Linearis::new("ignored"));
/// assert_eq!(result.consume(), 42);
/// ```
#[inline]
pub fn linear_const<A, B>(a: Linearis<A>) -> impl FnOnce(Linearis<B>) -> Linearis<A> {
    move |b| {
        let _ = b.consume();
        a
    }
}

/// Sequence two linear values, keeping the first.
///
/// # Example
///
/// ```rust
/// use ordofp_core::linear::{Linearis, linear_seq_first};
///
/// let a = Linearis::new(1);
/// let b = Linearis::new(2);
/// let result = linear_seq_first(a, b);
/// assert_eq!(result.consume(), 1);
/// ```
#[inline]
pub fn linear_seq_first<A, B>(a: Linearis<A>, b: Linearis<B>) -> Linearis<A> {
    let _ = b.consume();
    a
}

/// Sequence two linear values, keeping the second.
///
/// # Example
///
/// ```rust
/// use ordofp_core::linear::{Linearis, linear_seq_second};
///
/// let a = Linearis::new(1);
/// let b = Linearis::new(2);
/// let result = linear_seq_second(a, b);
/// assert_eq!(result.consume(), 2);
/// ```
#[inline]
pub fn linear_seq_second<A, B>(a: Linearis<A>, b: Linearis<B>) -> Linearis<B> {
    let _ = a.consume();
    b
}

/// Duplicate a linear value if the inner type is Clone.
///
/// This "escapes" linearity by cloning, which is only valid
/// for types that can be safely duplicated.
///
/// # Example
///
/// ```rust
/// use ordofp_core::linear::{Linearis, linear_dup};
///
/// let x = Linearis::new(42);
/// let (a, b) = linear_dup(x);
/// assert_eq!(a.consume(), 42);
/// assert_eq!(b.consume(), 42);
/// ```
#[inline]
pub fn linear_dup<A: Clone>(x: Linearis<A>) -> (Linearis<A>, Linearis<A>) {
    let value = x.consume();
    (Linearis::new(value.clone()), Linearis::new(value))
}

/// Discard a linear value explicitly.
///
/// This function makes discarding a linear value explicit,
/// which can improve code clarity.
///
/// # Example
///
/// ```rust
/// use ordofp_core::linear::{Linearis, linear_discard};
///
/// let x = Linearis::new(42);
/// linear_discard(x); // Explicitly discarded
/// ```
#[inline]
pub fn linear_discard<A>(x: Linearis<A>) {
    let _ = x.consume();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_pair() {
        let x = Linearis::new(1);
        let y = Linearis::new("hello");
        let pair = linear_pair(x, y);
        assert_eq!(pair.consume(), (1, "hello"));
    }

    #[test]
    fn test_linear_first() {
        let pair = Linearis::new((1, "hello"));
        let first = linear_first(pair);
        assert_eq!(first.consume(), 1);
    }

    #[test]
    fn test_linear_second() {
        let pair = Linearis::new((1, "hello"));
        let second = linear_second(pair);
        assert_eq!(second.consume(), "hello");
    }

    #[test]
    fn test_linear_swap() {
        let pair = Linearis::new((1, "hello"));
        let swapped = linear_swap(pair);
        assert_eq!(swapped.consume(), ("hello", 1));
    }

    #[test]
    fn test_linear_curry() {
        let add_pair = |pair: Linearis<(i32, i32)>| {
            let (a, b) = pair.consume();
            a + b
        };

        let curried = linear_curry(add_pair);
        let partial = curried(Linearis::new(1));
        let result = partial.apply(Linearis::new(2));
        assert_eq!(result, 3);
    }

    #[test]
    fn test_linear_uncurry() {
        let add = |a: Linearis<i32>| move |b: Linearis<i32>| a.consume() + b.consume();

        let uncurried = linear_uncurry(add);
        let result = uncurried(Linearis::new((1, 2)));
        assert_eq!(result, 3);
    }

    #[test]
    fn test_consume_both() {
        let pair = Linearis::new((1, 2));
        let sum = consume_both(pair, |a, b| a + b);
        assert_eq!(sum, 3);
    }

    #[test]
    fn test_consume_either() {
        let left = Linearis::new(1);
        let right = Linearis::new(2);

        let result = consume_either(true, left, right, |n| n * 10);
        assert_eq!(result, 10);

        let left2 = Linearis::new(1);
        let right2 = Linearis::new(2);
        let result2 = consume_either(false, left2, right2, |n| n * 10);
        assert_eq!(result2, 20);
    }

    #[test]
    fn test_linear_apply() {
        let f = Linearis::new(|x: i32| x * 2);
        let x = Linearis::new(5);
        let result = linear_apply(f, x);
        assert_eq!(result.consume(), 10);
    }

    #[test]
    fn test_linear_compose() {
        let f = |x: i32| x + 1;
        let g = |x: i32| x * 2;

        let composed = linear_compose(f, g);
        assert_eq!(composed(5), 12);
    }

    #[test]
    fn test_linear_flip() {
        let sub = |a: i32, b: i32| a - b;
        let flipped = linear_flip(sub);
        assert_eq!(flipped(1, 5), 4);
    }

    #[test]
    fn test_linear_const() {
        let always_42 = linear_const(Linearis::new(42));
        let result = always_42(Linearis::new("ignored"));
        assert_eq!(result.consume(), 42);
    }

    #[test]
    fn test_linear_seq_first() {
        let a = Linearis::new(1);
        let b = Linearis::new(2);
        let result = linear_seq_first(a, b);
        assert_eq!(result.consume(), 1);
    }

    #[test]
    fn test_linear_seq_second() {
        let a = Linearis::new(1);
        let b = Linearis::new(2);
        let result = linear_seq_second(a, b);
        assert_eq!(result.consume(), 2);
    }

    #[test]
    fn test_linear_dup() {
        let x = Linearis::new(42);
        let (a, b) = linear_dup(x);
        assert_eq!(a.consume(), 42);
        assert_eq!(b.consume(), 42);
    }

    #[test]
    fn test_linear_discard() {
        let x = Linearis::new(42);
        linear_discard(x);
        // No assertion - just verifying it compiles and runs
    }
}
