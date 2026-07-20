//! # Composable Left Folds (Hadolint Pattern)
//!
//! This module implements the `Fold` abstraction from Gabriel Gonzalez's
//! `foldl` library, adapted for Rust's ownership model. This pattern is
//! used extensively in Hadolint for composable linting rules.
//!
//! ## Overview
//!
//! A `Fold` encapsulates:
//! - A step function that processes each element
//! - An initial state
//! - An extract function that produces the final result
//!
//! ## Example
//!
//! ```rust
//! use ordofp_core::foldl::{Fold, sum, length, mean};
//!
//! let numbers = vec![1, 2, 3, 4, 5];
//! assert_eq!(sum().run(numbers.iter().copied()), 15);
//! ```
//!
//! ## Hadolint-style Rules
//!
//! For linting/validation use cases, see `simple_rule` and `custom_rule`.

use core::marker::PhantomData;

#[cfg(feature = "alloc")]
use alloc::{boxed::Box, string::String, vec::Vec};

/// A composable left fold.
///
/// The fold processes elements of type `A`, maintains internal state of type `S`,
/// and produces a final result of type `B`.
///
/// Unlike Haskell's `Fold`, this uses generics rather than existentials,
/// exposing the state type for full composability.
pub struct Fold<A, B, S, Step, Extract>
where
    Step: Fn(S, A) -> S,
    Extract: Fn(S) -> B,
{
    step: Step,
    initial: S,
    extract: Extract,
    _marker: PhantomData<fn(A) -> B>,
}

/// A [`Fold`] whose step and extract functions are plain function pointers —
/// the shape returned by the capture-free constructors in this module
/// (`head`, `last`, `minimum`, `maximum`, `mean`, `to_vec`).
///
/// Capture-free closures coerce to `fn` pointers, which keeps the
/// five-parameter `Fold` type nameable in return signatures.
pub type FnFold<A, B, S> = Fold<A, B, S, fn(S, A) -> S, fn(S) -> B>;

/// A collector [`Fold`]: accumulates into a `Vec<B>` with a capturing step
/// and an identity (function-pointer) extract — the shape returned by
/// `filter` and `map_collect`.
#[cfg(feature = "alloc")]
pub type VecFold<A, B, Step> = Fold<A, Vec<B>, Vec<B>, Step, fn(Vec<B>) -> Vec<B>>;

impl<A, B, S, Step, Extract> Fold<A, B, S, Step, Extract>
where
    Step: Fn(S, A) -> S,
    Extract: Fn(S) -> B,
{
    /// Create a new `Fold` from step function, initial state, and extract function.
    #[inline]
    pub fn new(step: Step, initial: S, extract: Extract) -> Self {
        Fold {
            step,
            initial,
            extract,
            _marker: PhantomData,
        }
    }

    /// Run the fold over an iterator.
    ///
    /// The inner loop is the standard `Iterator::fold`, which the compiler
    /// lowers to an iterative loop — no recursion, no stack-overflow risk.
    #[inline]
    pub fn run<I>(self, iter: I) -> B
    where
        I: IntoIterator<Item = A>,
    {
        let state = iter.into_iter().fold(self.initial, &self.step);
        (self.extract)(state)
    }

    /// Run the fold over a slice (requires Clone for state).
    #[cfg(feature = "alloc")]
    #[inline]
    pub fn run_slice(&self, slice: &[A]) -> B
    where
        A: Clone,
        S: Clone,
    {
        let state = slice.iter().cloned().fold(self.initial.clone(), &self.step);
        (self.extract)(state)
    }
}

// ============================================================================
// Common Folds
// ============================================================================

/// Sum all numeric elements.
pub fn sum<A>() -> Fold<A, A, A, impl Fn(A, A) -> A, impl Fn(A) -> A>
where
    A: core::ops::Add<Output = A> + Default + Copy,
{
    Fold::new(|acc, x| acc + x, A::default(), |s| s)
}

/// Product of all numeric elements (for i32).
pub fn product_i32() -> Fold<i32, i32, i32, impl Fn(i32, i32) -> i32, impl Fn(i32) -> i32> {
    Fold::new(|acc, x| acc * x, 1, |s| s)
}

/// Product of all numeric elements (for f64).
pub fn product_f64() -> Fold<f64, f64, f64, impl Fn(f64, f64) -> f64, impl Fn(f64) -> f64> {
    Fold::new(|acc, x| acc * x, 1.0, |s| s)
}

/// Count elements.
pub fn length<A>() -> Fold<A, usize, usize, impl Fn(usize, A) -> usize, impl Fn(usize) -> usize> {
    Fold::new(|count, _: A| count + 1, 0, |s| s)
}

/// Find the first element (if any).
pub fn head<A>() -> FnFold<A, Option<A>, Option<A>> {
    Fold::new(|acc: Option<A>, x| acc.or(Some(x)), None, |s| s)
}

/// Find the last element (if any).
pub fn last<A>() -> FnFold<A, Option<A>, Option<A>> {
    Fold::new(|_, x| Some(x), None, |s| s)
}

/// Check if all elements satisfy a predicate.
pub fn all<A, P>(
    predicate: P,
) -> Fold<A, bool, bool, impl Fn(bool, A) -> bool, impl Fn(bool) -> bool>
where
    P: Fn(&A) -> bool,
{
    Fold::new(move |acc, x| acc && predicate(&x), true, |s| s)
}

/// Check if any element satisfies a predicate.
pub fn any<A, P>(
    predicate: P,
) -> Fold<A, bool, bool, impl Fn(bool, A) -> bool, impl Fn(bool) -> bool>
where
    P: Fn(&A) -> bool,
{
    Fold::new(move |acc, x| acc || predicate(&x), false, |s| s)
}

/// Find the minimum element.
pub fn minimum<A>() -> FnFold<A, Option<A>, Option<A>>
where
    A: Ord,
{
    Fold::new(
        |acc: Option<A>, x| match acc {
            None => Some(x),
            Some(min) => Some(if x < min { x } else { min }),
        },
        None,
        |s| s,
    )
}

/// Find the maximum element.
pub fn maximum<A>() -> FnFold<A, Option<A>, Option<A>>
where
    A: Ord,
{
    Fold::new(
        |acc: Option<A>, x| match acc {
            None => Some(x),
            Some(max) => Some(if x > max { x } else { max }),
        },
        None,
        |s| s,
    )
}

/// Compute the mean of numeric elements.
pub fn mean() -> FnFold<f64, Option<f64>, (f64, usize)> {
    Fold::new(
        |(sum, count): (f64, usize), x: f64| (sum + x, count + 1),
        (0.0, 0),
        |(sum, count)| {
            if count == 0 {
                None
            } else {
                Some(sum / count as f64)
            }
        },
    )
}

/// Collect all elements into a Vec.
#[cfg(feature = "alloc")]
pub fn to_vec<A>() -> FnFold<A, Vec<A>, Vec<A>> {
    Fold::new(
        |mut v: Vec<A>, x| {
            v.push(x);
            v
        },
        Vec::new(),
        |s| s,
    )
}

/// Filter elements by a predicate, then collect.
#[cfg(feature = "alloc")]
pub fn filter<A, P>(predicate: P) -> VecFold<A, A, impl Fn(Vec<A>, A) -> Vec<A>>
where
    P: Fn(&A) -> bool,
{
    Fold::new(
        move |mut v: Vec<A>, x| {
            if predicate(&x) {
                v.push(x);
            }
            v
        },
        Vec::new(),
        |s| s,
    )
}

/// Map a function over elements before collecting.
#[cfg(feature = "alloc")]
pub fn map_collect<A, B, F>(f: F) -> VecFold<A, B, impl Fn(Vec<B>, A) -> Vec<B>>
where
    F: Fn(A) -> B,
{
    Fold::new(
        move |mut v: Vec<B>, x| {
            v.push(f(x));
            v
        },
        Vec::new(),
        |s| s,
    )
}

// ============================================================================
// Hadolint-style Rule System
// ============================================================================

/// A failure from a rule check.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(feature = "alloc")]
pub struct Failure<Code> {
    /// Line number (1-indexed).
    pub line: usize,
    /// Error code identifying the rule.
    pub code: Code,
    /// Human-readable message.
    pub message: String,
}

#[cfg(feature = "alloc")]
impl<Code> Failure<Code> {
    /// Create a new failure.
    #[inline]
    pub fn new(line: usize, code: Code, message: impl Into<String>) -> Self {
        Failure {
            line,
            code,
            message: message.into(),
        }
    }
}

/// A validation rule that processes items and produces failures.
///
/// This is equivalent to Hadolint's `Rule` type:
/// ```haskell
/// type Rule args = Fold (Linenumber, Instruction args) [RuleCheck]
/// ```
/// Per-item check function used by [`Rule`].
#[cfg(feature = "alloc")]
type RuleChecker<A, Code> = Box<dyn FnMut(usize, &A) -> Option<Failure<Code>>>;

/// A validation rule: a fold over `(line, item)` pairs that accumulates
/// [`Failure`]s for every item its checker flags.
///
/// Build one with [`simple_rule`] (stateless per-item predicate), then
/// drive it with [`Rule::run`] or [`Rule::run_items`]. For checks that
/// need context across items, use [`StatefulRule`] instead.
#[cfg(feature = "alloc")]
pub struct Rule<A, Code> {
    failures: Vec<Failure<Code>>,
    checker: RuleChecker<A, Code>,
}

#[cfg(feature = "alloc")]
impl<A, Code> Rule<A, Code> {
    /// Run the rule over indexed items.
    pub fn run<I>(mut self, iter: I) -> Vec<Failure<Code>>
    where
        I: IntoIterator<Item = (usize, A)>,
    {
        for (line, item) in iter {
            if let Some(failure) = (self.checker)(line, &item) {
                self.failures.push(failure);
            }
        }
        self.failures
    }

    /// Run the rule over items (auto-indexing from 1).
    pub fn run_items<I>(self, iter: I) -> Vec<Failure<Code>>
    where
        I: IntoIterator<Item = A>,
    {
        self.run(iter.into_iter().enumerate().map(|(i, x)| (i + 1, x)))
    }
}

/// Create a simple rule that checks each item independently.
///
/// Equivalent to Hadolint's `simpleRule`:
/// ```haskell
/// simpleRule :: code -> (a -> Bool) -> Rule a
/// ```
///
/// # Example
///
/// ```rust
/// use ordofp_core::foldl::{simple_rule, Failure};
///
/// let rule = simple_rule("E001", "Value too large", |x: &i32| *x > 100);
/// let failures = rule.run_items(vec![50, 150, 75, 200]);
/// assert_eq!(failures.len(), 2);
/// assert_eq!(failures[0].line, 2);
/// assert_eq!(failures[1].line, 4);
/// ```
#[cfg(feature = "alloc")]
pub fn simple_rule<A, Code, P>(
    code: Code,
    message: impl Into<String> + Clone + 'static,
    predicate: P,
) -> Rule<A, Code>
where
    Code: Clone + 'static,
    P: Fn(&A) -> bool + 'static,
{
    let msg = message.into();
    Rule {
        failures: Vec::new(),
        checker: Box::new(move |line, item| {
            if predicate(item) {
                Some(Failure::new(line, code.clone(), msg.clone()))
            } else {
                None
            }
        }),
    }
}

/// A stateful rule that can accumulate context across items.
///
/// Equivalent to Hadolint's `customRule`:
/// ```haskell
/// customRule :: state -> (state -> (line, a) -> (state, Maybe RuleCheck)) -> Rule a
/// ```
/// Per-item check function with mutable state, used by [`StatefulRule`].
#[cfg(feature = "alloc")]
type StatefulChecker<A, Code, State> =
    Box<dyn FnMut(&mut State, usize, &A) -> Option<Failure<Code>>>;

/// A validation rule whose checker threads mutable `State` across
/// items, so a failure can depend on what has been seen so far.
///
/// The stateful counterpart of [`Rule`]; build one with
/// [`custom_rule`] and drive it with [`StatefulRule::run`] or
/// [`StatefulRule::run_items`].
#[cfg(feature = "alloc")]
pub struct StatefulRule<A, Code, State> {
    state: State,
    failures: Vec<Failure<Code>>,
    checker: StatefulChecker<A, Code, State>,
}

#[cfg(feature = "alloc")]
impl<A, Code, State> StatefulRule<A, Code, State> {
    /// Run the stateful rule.
    pub fn run<I>(mut self, iter: I) -> Vec<Failure<Code>>
    where
        I: IntoIterator<Item = (usize, A)>,
    {
        for (line, item) in iter {
            if let Some(failure) = (self.checker)(&mut self.state, line, &item) {
                self.failures.push(failure);
            }
        }
        self.failures
    }

    /// Run over items with auto-indexing.
    pub fn run_items<I>(self, iter: I) -> Vec<Failure<Code>>
    where
        I: IntoIterator<Item = A>,
    {
        self.run(iter.into_iter().enumerate().map(|(i, x)| (i + 1, x)))
    }
}

/// Create a custom rule with state.
///
/// # Example
///
/// ```rust
/// use ordofp_core::foldl::{custom_rule, Failure};
///
/// // Track if we've seen a header
/// let rule = custom_rule(
///     false, // initial state: no header seen
///     |seen_header: &mut bool, line, item: &String| {
///         if item.starts_with("# ") {
///             *seen_header = true;
///             None
///         } else if !*seen_header && !item.is_empty() {
///             Some(Failure::new(line, "E002", "Content before header"))
///         } else {
///             None
///         }
///     }
/// );
/// ```
#[cfg(feature = "alloc")]
pub fn custom_rule<A, Code, State, Check>(
    initial_state: State,
    checker: Check,
) -> StatefulRule<A, Code, State>
where
    Check: FnMut(&mut State, usize, &A) -> Option<Failure<Code>> + 'static,
{
    StatefulRule {
        state: initial_state,
        failures: Vec::new(),
        checker: Box::new(checker),
    }
}

// ============================================================================
// Fold Combinators
// ============================================================================

/// Product fold returned by [`combine`]: runs over pairs of states and
/// produces a pair of results.
pub type PairFold<A, B1, B2, S1, S2, Step, Extract> = Fold<A, (B1, B2), (S1, S2), Step, Extract>;

/// Trait alias for the step-function shape of a [`PairFold`].
///
/// Blanket-implemented for every matching closure; exists so `combine` can
/// name its opaque return type without spelling out the nested `Fn` sugar.
pub trait PairStep<S1, S2, A>: Fn((S1, S2), A) -> (S1, S2) {}
impl<S1, S2, A, T: Fn((S1, S2), A) -> (S1, S2)> PairStep<S1, S2, A> for T {}

/// Trait alias for the extract-function shape of a [`PairFold`].
pub trait PairExtract<S1, S2, B1, B2>: Fn((S1, S2)) -> (B1, B2) {}
impl<S1, S2, B1, B2, T: Fn((S1, S2)) -> (B1, B2)> PairExtract<S1, S2, B1, B2> for T {}

/// Run two folds in parallel, combining their results.
pub fn combine<A, B1, B2, S1, S2, Step1, Extract1, Step2, Extract2>(
    fold1: Fold<A, B1, S1, Step1, Extract1>,
    fold2: Fold<A, B2, S2, Step2, Extract2>,
) -> PairFold<A, B1, B2, S1, S2, impl PairStep<S1, S2, A>, impl PairExtract<S1, S2, B1, B2>>
where
    A: Clone,
    Step1: Fn(S1, A) -> S1,
    Extract1: Fn(S1) -> B1,
    Step2: Fn(S2, A) -> S2,
    Extract2: Fn(S2) -> B2,
{
    Fold::new(
        move |(s1, s2): (S1, S2), a: A| ((fold1.step)(s1, a.clone()), (fold2.step)(s2, a)),
        (fold1.initial, fold2.initial),
        move |(s1, s2)| ((fold1.extract)(s1), (fold2.extract)(s2)),
    )
}

/// Pre-map input elements before folding.
pub fn premap<A, B, C, S, Step, Extract, F>(
    fold: Fold<B, C, S, Step, Extract>,
    f: F,
) -> Fold<A, C, S, impl Fn(S, A) -> S, Extract>
where
    Step: Fn(S, B) -> S,
    Extract: Fn(S) -> C,
    F: Fn(A) -> B,
{
    Fold::new(move |s, a| (fold.step)(s, f(a)), fold.initial, fold.extract)
}

/// Post-map the fold result.
pub fn postmap<A, B, C, S, Step, Extract, F>(
    fold: Fold<A, B, S, Step, Extract>,
    f: F,
) -> Fold<A, C, S, Step, impl Fn(S) -> C>
where
    Step: Fn(S, A) -> S,
    Extract: Fn(S) -> B,
    F: Fn(B) -> C,
{
    Fold::new(fold.step, fold.initial, move |s| f((fold.extract)(s)))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_sum() {
        let result = sum().run(vec![1, 2, 3, 4, 5]);
        assert_eq!(result, 15);
    }

    #[test]
    fn test_sum_empty() {
        let result: i32 = sum().run(vec![]);
        assert_eq!(result, 0);
    }

    #[test]
    fn test_length() {
        let result = length().run(vec!["a", "b", "c"]);
        assert_eq!(result, 3);
    }

    #[test]
    fn test_length_empty() {
        let result = length::<i32>().run(vec![]);
        assert_eq!(result, 0);
    }

    #[test]
    fn test_head() {
        let result = head().run(vec![1, 2, 3]);
        assert_eq!(result, Some(1));
    }

    #[test]
    fn test_head_empty() {
        let result = head::<i32>().run(vec![]);
        assert_eq!(result, None);
    }

    #[test]
    fn test_last() {
        let result = last().run(vec![1, 2, 3]);
        assert_eq!(result, Some(3));
    }

    #[test]
    fn test_last_empty() {
        // last() starts with None and never runs the step, so an empty iterator
        // must return None rather than panicking or producing a stale value.
        let result = last::<i32>().run(vec![]);
        assert_eq!(result, None);
    }

    #[test]
    fn test_minimum() {
        let result = minimum().run(vec![3, 1, 4, 1, 5]);
        assert_eq!(result, Some(1));
    }

    #[test]
    fn test_maximum() {
        let result = maximum().run(vec![3, 1, 4, 1, 5]);
        assert_eq!(result, Some(5));
    }

    #[test]
    fn test_mean() {
        let result = mean().run(vec![2.0, 4.0, 6.0]);
        assert_eq!(result, Some(4.0));
    }

    #[test]
    fn test_mean_empty() {
        let result = mean().run(Vec::<f64>::new());
        assert_eq!(result, None);
    }

    #[test]
    fn test_all() {
        let result = all(|x: &i32| *x > 0).run(vec![1, 2, 3]);
        assert!(result);

        let result = all(|x: &i32| *x > 0).run(vec![1, -2, 3]);
        assert!(!result);
    }

    #[test]
    fn test_any() {
        let result = any(|x: &i32| *x < 0).run(vec![1, 2, 3]);
        assert!(!result);

        let result = any(|x: &i32| *x < 0).run(vec![1, -2, 3]);
        assert!(result);
    }

    #[test]
    fn test_to_vec() {
        let result = to_vec().run(vec![1, 2, 3]);
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[test]
    fn test_filter() {
        let result = filter(|x: &i32| *x % 2 == 0).run(vec![1, 2, 3, 4, 5]);
        assert_eq!(result, vec![2, 4]);
    }

    #[test]
    fn test_map_collect() {
        let result = map_collect(|x: i32| x * 2).run(vec![1, 2, 3]);
        assert_eq!(result, vec![2, 4, 6]);
    }

    #[test]
    fn test_simple_rule() {
        let rule = simple_rule("E001", "Too large", |x: &i32| *x > 100);
        let failures = rule.run_items(vec![50, 150, 75, 200]);

        assert_eq!(failures.len(), 2);
        assert_eq!(failures[0].line, 2);
        assert_eq!(failures[0].code, "E001");
        assert_eq!(failures[1].line, 4);
    }

    #[test]
    fn test_simple_rule_no_failures() {
        let rule = simple_rule("E001", "Too large", |x: &i32| *x > 100);
        let failures = rule.run_items(vec![10, 20, 30]);

        assert!(failures.is_empty());
    }

    #[test]
    fn test_custom_rule() {
        // Rule: detect duplicates
        let rule = custom_rule::<i32, &str, Vec<i32>, _>(vec![], |seen, line, item| {
            if seen.contains(item) {
                Some(Failure::new(line, "E002", "Duplicate value"))
            } else {
                seen.push(*item);
                None
            }
        });

        let failures = rule.run_items(vec![1, 2, 3, 2, 4, 1]);

        assert_eq!(failures.len(), 2);
        assert_eq!(failures[0].line, 4); // First duplicate of 2
        assert_eq!(failures[1].line, 6); // First duplicate of 1
    }

    #[test]
    fn test_combine_sum_and_length() {
        let sum_fold = sum::<i32>();
        let len_fold = length::<i32>();
        let combined = combine(sum_fold, len_fold);

        let (total, count) = combined.run(vec![10, 20, 30]);
        assert_eq!(total, 60);
        assert_eq!(count, 3);
    }

    #[test]
    fn test_premap() {
        // Sum the lengths of strings
        let fold = premap(sum::<usize>(), |s: &str| s.len());
        let result = fold.run(vec!["hello", "world", "!"]);
        assert_eq!(result, 11); // 5 + 5 + 1
    }

    #[test]
    fn test_postmap() {
        // Sum, then check if positive
        let fold = postmap(sum::<i32>(), |s| s > 0);

        let result = fold.run(vec![1, 2, 3]);
        assert!(result);

        let fold = postmap(sum::<i32>(), |s| s > 0);
        let result = fold.run(vec![-10, 2, 3]);
        assert!(!result);
    }

    #[test]
    fn test_failure_equality() {
        let f1: Failure<&str> = Failure::new(1, "E001", "Test");
        let f2: Failure<&str> = Failure::new(1, "E001", "Test");
        let f3: Failure<&str> = Failure::new(2, "E001", "Test");

        assert_eq!(f1, f2);
        assert_ne!(f1, f3);
    }

    #[test]
    fn test_run_slice() {
        let fold = sum::<i32>();
        let data = [1, 2, 3, 4, 5];
        let result = fold.run_slice(&data);
        assert_eq!(result, 15);
    }

    #[test]
    fn test_product_i32_empty() {
        // Multiplicative identity: product over zero elements must be 1.
        let result: i32 = product_i32().run(vec![]);
        assert_eq!(
            result, 1,
            "product of empty sequence is the multiplicative identity 1"
        );
    }

    #[test]
    fn test_product_i32_contains_zero() {
        // A zero anywhere in the sequence must collapse the entire product to 0.
        let result = product_i32().run(vec![2, 3, 0, 5, 7]);
        assert_eq!(result, 0, "product containing a zero element must be 0");
    }

    /// `all` on an empty collection is vacuously true (no element violates the predicate).
    /// `any` on an empty collection is vacuously false (no element satisfies the predicate).
    /// These are the identity-element edge cases for the respective boolean monoids.
    #[test]
    fn test_product_f64_empty_is_identity_and_zero_collapses() {
        // An empty sequence must return the multiplicative identity 1.0, mirroring
        // the analogous test for product_i32.  product_f64 is otherwise untested.
        let empty: f64 = product_f64().run(vec![]);
        assert_eq!(empty, 1.0, "product_f64 of empty sequence must be 1.0");

        // A single 0.0 anywhere in the sequence collapses the entire product to 0.0.
        let with_zero = product_f64().run(vec![2.0, 3.0, 0.0, 5.0]);
        assert_eq!(with_zero, 0.0, "product_f64 containing 0.0 must be 0.0");
    }

    #[test]
    fn test_all_any_empty_collection() {
        // all starts with true and never applies the predicate → vacuously true
        assert!(
            all(|_: &i32| false).run(vec![]),
            "all on an empty collection must be vacuously true"
        );

        // any starts with false and never applies the predicate → vacuously false
        assert!(
            !any(|_: &i32| true).run(vec![]),
            "any on an empty collection must be vacuously false"
        );
    }
}
