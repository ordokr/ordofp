//! Applicative Probatum with error accumulation.
//!
//! This replaces the legacy `HList` accumulator with a semigroupal `Probatum<E, A>`
//! that collects multiple errors instead of short-circuiting like `Result`.

use alloc::vec::Vec;
use core::fmt;

use crate::typeclasses::{Applicatio, Apply, Functor};

#[cfg(feature = "Probatum-smallvec")]
use smallvec::SmallVec;

// Inline capacity 4 is measured-optimal for the end-to-end error-heavy
// workload: [E;4]
// never spills for the consumer shape (≤1 error per validator, 4 validators)
// and halves the enum vs [E;8] (~280 → ~152 B with a String-bearing E) —
// paired e2e: error-heavy −5.5…−7%, steady neutral, checksums bit-identical.
// [E;2] was tried and reverted: 3-error cells spill at N=2, putting a
// reachable heap-spill path inside the inlined map2 merge. Don't resize in
// either direction without re-measuring (cargo run -p xtask -- perf-guard, plus paired
// hyperfine sessions to control for machine drift).
#[cfg(feature = "Probatum-smallvec")]
type ErrorBuf<E> = SmallVec<[E; 4]>;
#[cfg(not(feature = "Probatum-smallvec"))]
type ErrorBuf<E> = Vec<E>;

/// Error returned when an iterator offered to form an invalid validation is empty.
///
/// Empty input is not a validation failure. Callers must decide how to handle it;
/// no implicit success, fabricated error, or panic is used.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ErroresVacui;

/// English alias for `ErroresVacui`.
pub use ErroresVacui as EmptyErrors;

impl fmt::Display for ErroresVacui {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("an invalid validation requires at least one error")
    }
}

/// A nonempty error collection used by `Probatum::Invalid`.
///
/// The buffer is private: safe construction requires at least one error.
/// This wrapper retains the exact layout of the selected `ErrorBuf` backing
/// store (`Vec` by default, four-inline-element `SmallVec` when opted in).
/// It offers read-only slice access; merging consumes the other collection
/// rather than temporarily leaving another live value empty.
#[repr(transparent)]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct ErroresNonVacui<E>(ErrorBuf<E>);

/// English alias for `ErroresNonVacui`.
pub use ErroresNonVacui as NonEmptyErrors;

impl<E> ErroresNonVacui<E> {
    /// Create a one-error collection.
    #[inline]
    pub fn one(error: E) -> Self {
        Self(core::iter::once(error).collect())
    }

    /// Reject empty input rather than constructing `Invalid([])`.
    ///
    /// # Errors
    /// Returns `EmptyErrors` if the iterator yields no values.
    #[inline]
    pub fn try_from_iter<I: IntoIterator<Item = E>>(iter: I) -> Result<Self, EmptyErrors> {
        let errors: ErrorBuf<E> = iter.into_iter().collect();
        if errors.is_empty() {
            Err(EmptyErrors)
        } else {
            Ok(Self(errors))
        }
    }

    /// Borrow the errors in their original order.
    #[inline]
    #[must_use]
    pub fn as_slice(&self) -> &[E] {
        self.0.as_ref()
    }

    /// Number of accumulated errors (always at least one).
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.as_slice().len()
    }

    /// Always false: a value of this type can never contain zero errors.
    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        false
    }

    /// Merge two nonempty lists, preserving left-to-right order.
    #[inline]
    #[must_use]
    pub fn concat(mut self, other: Self) -> Self {
        self.0.extend(other.0);
        self
    }

    /// Map each error; one input always produces one output.
    #[inline]
    #[must_use]
    pub fn map<E2, F>(self, f: F) -> NonEmptyErrors<E2>
    where
        F: FnMut(E) -> E2,
    {
        ErroresNonVacui(self.0.into_iter().map(f).collect())
    }
}

impl<E> core::ops::Deref for ErroresNonVacui<E> {
    type Target = [E];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<E> IntoIterator for ErroresNonVacui<E> {
    type Item = E;
    type IntoIter = <ErrorBuf<E> as IntoIterator>::IntoIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// A validation result that can accumulate multiple errors.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(bound(
        serialize = "E: serde::Serialize, A: serde::Serialize",
        deserialize = "E: serde::Deserialize<'de>, A: serde::Deserialize<'de>"
    ))
)]
pub enum Probatum<E, A> {
    /// Successful validation.
    Valid(A),
    /// One or more errors.
    Invalid(#[cfg_attr(feature = "serde", serde(with = "error_buf_serde"))] NonEmptyErrors<E>),
}

#[cfg(feature = "serde")]
mod error_buf_serde {
    use super::NonEmptyErrors;
    use alloc::vec::Vec;
    use serde::{Deserialize, Serialize};

    /// Keep the existing flat array wire shape for nonempty invalid results.
    pub(super) fn serialize<S, E>(buf: &NonEmptyErrors<E>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
        E: serde::Serialize,
    {
        buf.as_slice().serialize(serializer)
    }

    /// Reject serialized invalid results with no errors.
    pub(super) fn deserialize<'de, D, E>(deserializer: D) -> Result<NonEmptyErrors<E>, D::Error>
    where
        D: serde::Deserializer<'de>,
        E: serde::Deserialize<'de>,
    {
        let values = Vec::<E>::deserialize(deserializer)?;
        NonEmptyErrors::try_from_iter(values).map_err(serde::de::Error::custom)
    }
}

impl<E, A> Probatum<E, A> {
    /// Construct a valid value.
    pub const fn valid(a: A) -> Self {
        Self::Valid(a)
    }

    /// Construct a single error.
    pub fn invalid(err: E) -> Self {
        Self::Invalid(NonEmptyErrors::one(err))
    }

    /// Construct an invalid result from a nonempty iterable.
    ///
    /// # Errors
    /// Returns `EmptyErrors` for an empty iterator; no invalid result is made.
    pub fn invalid_many<I: IntoIterator<Item = E>>(iter: I) -> Result<Self, EmptyErrors> {
        NonEmptyErrors::try_from_iter(iter).map(Self::Invalid)
    }

    /// Returns true if this is `Valid`.
    pub const fn is_valid(&self) -> bool {
        matches!(self, Self::Valid(_))
    }

    /// Returns true if this is `Invalid`.
    pub const fn is_invalid(&self) -> bool {
        !self.is_valid()
    }

    /// Borrow the success value.
    pub const fn value(&self) -> Option<&A> {
        match self {
            Self::Valid(a) => Some(a),
            Self::Invalid(_) => None,
        }
    }

    /// Borrow the accumulated errors.
    pub fn errors(&self) -> Option<&[E]> {
        match self {
            Self::Invalid(es) => Some(es.as_slice()),
            Self::Valid(_) => None,
        }
    }

    /// Convert into a `Result`.
    ///
    /// # Errors
    ///
    /// Returns `Err` carrying the accumulated error buffer when this is
    /// `Invalid`; every collected error is preserved, in order.
    pub fn into_result(self) -> Result<A, NonEmptyErrors<E>> {
        match self {
            Self::Valid(a) => Ok(a),
            Self::Invalid(es) => Err(es),
        }
    }

    /// Finish accumulating independent errors, then check a domain constructor.
    ///
    /// Unlike `map`, the conversion can fail with a single dependent
    /// invariant error. The `TryFrom` implementation is called exactly once
    /// for `Valid` and never for `Invalid`. Earlier errors are preserved
    /// unchanged and in order. No `Clone` bound is required.
    ///
    /// Use this after building a plain DTO with validation and optics.
    /// The domain type's `TryFrom<DTO>` must enforce its own invariants;
    /// this method cannot make an unchecked constructor safe.
    ///
    /// # Example
    ///
    /// ```
    /// use ordofp_core::validated::Probatum;
    ///
    /// #[derive(Debug, PartialEq, Eq)]
    /// struct Positive(i32);
    ///
    /// impl TryFrom<i32> for Positive {
    ///     type Error = &'static str;
    ///
    ///     fn try_from(value: i32) -> Result<Self, Self::Error> {
    ///         if value > 0 { Ok(Self(value)) } else { Err("nonpositive") }
    ///     }
    /// }
    ///
    /// let validated = Probatum::<&str, i32>::valid(4);
    /// let domain: Probatum<&str, Positive> = validated.try_construct();
    /// assert_eq!(domain, Probatum::valid(Positive(4)));
    /// ```
    #[inline]
    #[must_use]
    pub fn try_construct<B>(self) -> Probatum<E, B>
    where
        B: TryFrom<A, Error = E>,
    {
        match self {
            Self::Valid(dto) => Probatum::from(B::try_from(dto)),
            Self::Invalid(errors) => Probatum::Invalid(errors),
        }
    }

    /// Map a function over the valid value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::validated::Probatum;
    ///
    /// let v = Probatum::<&str, i32>::Valid(10);
    /// assert_eq!(v.map(|x| x * 2), Probatum::Valid(20));
    /// ```
    pub fn map<B, F>(self, f: F) -> Probatum<E, B>
    where
        F: FnOnce(A) -> B,
    {
        match self {
            Self::Valid(a) => Probatum::Valid(f(a)),
            Self::Invalid(e) => Probatum::Invalid(e),
        }
    }

    /// Map a function over each accumulated error.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::validated::Probatum;
    ///
    /// let v = Probatum::<&str, i32>::invalid("bad");
    /// assert_eq!(v.map_err(|s| s.to_uppercase()).errors().unwrap(), &["BAD".to_string()]);
    /// ```
    pub fn map_err<E2, F>(self, f: F) -> Probatum<E2, A>
    where
        F: FnMut(E) -> E2,
    {
        match self {
            Self::Valid(a) => Probatum::Valid(a),
            Self::Invalid(es) => Probatum::Invalid(es.map(f)),
        }
    }

    /// Combine two Probatum values with a pure function.
    pub fn map2<B, C, F>(self, vb: Probatum<E, B>, f: F) -> Probatum<E, C>
    where
        F: FnOnce(A, B) -> C,
    {
        match (self, vb) {
            (Self::Valid(a), Probatum::Valid(b)) => Probatum::Valid(f(a, b)),
            (Self::Invalid(e1), Probatum::Invalid(e2)) => Probatum::Invalid(e1.concat(e2)),
            (Self::Invalid(e), _) | (_, Probatum::Invalid(e)) => Probatum::Invalid(e),
        }
    }

    /// Collect an iterator of Probatum values into a single Probatum Vec.
    pub fn collect<I, T>(iter: I) -> Probatum<E, Vec<T>>
    where
        I: IntoIterator<Item = Probatum<E, T>>,
    {
        let mut iter = iter.into_iter();
        // Pre-size from the iterator's hint: in the all-valid case `values`
        // ends up with exactly this many items, so the upper bound is tight
        // (no over-allocation) and we avoid the realloc cascade of Vec::new().
        let (lower, upper) = iter.size_hint();
        let mut values = Vec::with_capacity(upper.unwrap_or(lower));
        while let Some(item) = iter.next() {
            match item {
                Probatum::Valid(v) => values.push(v),
                Probatum::Invalid(mut es) => {
                    drop(values);
                    for item in iter {
                        if let Probatum::Invalid(es2) = item {
                            es = es.concat(es2);
                        }
                    }
                    return Probatum::Invalid(es);
                }
            }
        }
        Probatum::Valid(values)
    }

    /// Lift two Probatum values with a combining function.
    pub fn lift2<B, C, F>(f: F, va: Self, vb: Probatum<E, B>) -> Probatum<E, C>
    where
        F: FnOnce(A, B) -> C,
    {
        va.map2(vb, f)
    }

    /// Lift three Probatum values with a combining function.
    pub fn map3<B, C, D, F>(self, vb: Probatum<E, B>, vc: Probatum<E, C>, f: F) -> Probatum<E, D>
    where
        F: FnOnce(A, B, C) -> D,
    {
        self.map2(vb, |a, b| (a, b))
            .map2(vc, |(a, b), c| f(a, b, c))
    }

    /// Lift three owned Probatum values with a combining function.
    pub fn lift3<B, C, D, F>(
        f: F,
        va: Self,
        vb: Probatum<E, B>,
        vc: Probatum<E, C>,
    ) -> Probatum<E, D>
    where
        F: FnOnce(A, B, C) -> D,
    {
        va.map2(vb, |a, b| (a, b)).map2(vc, |(a, b), c| f(a, b, c))
    }

    /// Alias for `lift2` (cats/zio-style naming).
    pub fn map2_alias<B, C, F>(va: Self, vb: Probatum<E, B>, f: F) -> Probatum<E, C>
    where
        F: FnOnce(A, B) -> C,
    {
        Self::lift2(f, va, vb)
    }

    /// Alias for `lift3`.
    pub fn map3_alias<B, C, D, F>(
        va: Self,
        vb: Probatum<E, B>,
        vc: Probatum<E, C>,
        f: F,
    ) -> Probatum<E, D>
    where
        F: FnOnce(A, B, C) -> D,
    {
        Self::lift3(f, va, vb, vc)
    }

    /// Convert from an Option using a provided error when None.
    pub fn from_option(opt: Option<A>, err: E) -> Self {
        opt.map_or_else(|| Self::invalid(err), |a| Self::Valid(a))
    }

    /// Sequence an iterator of Probatum values.
    pub fn sequence<I, T>(iter: I) -> Probatum<E, Vec<T>>
    where
        I: IntoIterator<Item = Probatum<E, T>>,
    {
        Self::collect(iter)
    }

    /// Apply a Probatum function to a Probatum value (delegates to `Apply::apply`).
    pub fn ap<B, F>(self, vf: Probatum<E, F>) -> Probatum<E, B>
    where
        F: FnMut(A) -> B,
    {
        use crate::typeclasses::Apply;
        self.apply(vf)
    }
}

/// Convert a `Result` into a `Probatum`.
pub trait IntoProbatum<T, E> {
    /// Converts this value into a [`Probatum`], mapping `Ok(v)` to `Valid(v)`
    /// and `Err(e)` to `Invalid` with a single accumulated error.
    fn into_probatum(self) -> Probatum<E, T>;
}

impl<T, E> IntoProbatum<T, E> for Result<T, E> {
    fn into_probatum(self) -> Probatum<E, T> {
        match self {
            Ok(v) => Probatum::Valid(v),
            Err(e) => Probatum::invalid(e),
        }
    }
}

/// English alias trait for `IntoProbatum`, providing `.into_validated()`.
pub trait IntoValidated<T, E> {
    /// Converts this value into a [`Probatum`]/[`Validated`], mapping `Ok(v)` to `Valid(v)`
    /// and `Err(e)` to `Invalid` with a single accumulated error.
    fn into_validated(self) -> Probatum<E, T>;
}

impl<T, E> IntoValidated<T, E> for Result<T, E> {
    #[inline]
    fn into_validated(self) -> Probatum<E, T> {
        self.into_probatum()
    }
}

impl<T, E> From<Result<T, E>> for Probatum<E, T> {
    fn from(res: Result<T, E>) -> Self {
        IntoProbatum::into_probatum(res)
    }
}

/// Extension trait for iterators to validate elements and accumulate all errors into a `Probatum`.
pub trait IteratorValidateExt: Iterator + Sized {
    /// Applies a validating function to each element and accumulates all errors.
    ///
    /// Unlike `Iterator::map().collect::<Result<Vec<_>, _>>()`, which short-circuits
    /// on the first failure, `validate_all` collects **all** validation errors across
    /// the entire iterator into a single `Probatum`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::validated::{IteratorValidateExt, Probatum, IntoProbatum};
    ///
    /// let inputs = vec!["1", "2", "bad1", "4", "bad2"];
    /// let parsed: Probatum<String, Vec<i32>> = inputs.into_iter().validate_all(|s| {
    ///     s.parse::<i32>()
    ///         .map_err(|_| format!("invalid integer: {s}"))
    ///         .into_probatum()
    /// });
    ///
    /// assert!(parsed.is_invalid());
    /// assert_eq!(parsed.errors().unwrap().len(), 2);
    /// ```
    fn validate_all<E, B, F>(self, f: F) -> Probatum<E, Vec<B>>
    where
        F: FnMut(Self::Item) -> Probatum<E, B>,
    {
        Probatum::<E, B>::collect(self.map(f))
    }
}

impl<I: Iterator> IteratorValidateExt for I {}

impl<E, A> Functor for Probatum<E, A> {
    type Inner = A;
    type Target<T> = Probatum<E, T>;

    fn map<B, F>(self, mut f: F) -> Self::Target<B>
    where
        F: FnMut(Self::Inner) -> B,
    {
        match self {
            Self::Valid(a) => Probatum::Valid(f(a)),
            Self::Invalid(e) => Probatum::Invalid(e),
        }
    }
}

impl<E, A> Apply for Probatum<E, A> {
    fn apply<B, F>(self, ff: Probatum<E, F>) -> Probatum<E, B>
    where
        F: FnMut(A) -> B,
    {
        match (ff, self) {
            (Probatum::Valid(mut f), Self::Valid(a)) => Probatum::Valid(f(a)),
            (Probatum::Invalid(e1), Self::Invalid(e2)) => Probatum::Invalid(e1.concat(e2)),
            (Probatum::Invalid(e), _) | (_, Self::Invalid(e)) => Probatum::Invalid(e),
        }
    }

    fn ap_left<B>(self, fb: Self::Target<B>) -> Self::Target<Self::Inner>
    where
        Self: Sized,
        Self::Inner: Clone,
        Self::Target<B>: Apply<Inner = B, Target<Self::Inner> = Self::Target<Self::Inner>>,
        Self::Target<fn(B) -> Self::Inner>:
            Apply<Inner = fn(B) -> Self::Inner, Target<Self::Inner> = Self::Target<Self::Inner>>,
    {
        self.map2(fb, |a, _| a)
    }

    fn ap_right<B>(self, fb: Self::Target<B>) -> Self::Target<B>
    where
        Self: Sized,
        Self::Target<B>: Apply<Inner = B, Target<B> = Self::Target<B>> + Clone,
        Self::Target<fn(Self::Inner) -> B>:
            Apply<Inner = fn(Self::Inner) -> B, Target<B> = Self::Target<B>>,
    {
        self.map2(fb, |_, b| b)
    }
}

impl<E, A> Applicatio for Probatum<E, A> {
    #[inline]
    fn pure(a: A) -> Self {
        Self::Valid(a)
    }

    #[inline]
    fn pure_target<T>(t: T) -> Probatum<E, T>
    where
        Probatum<E, T>: Applicatio<Inner = T>,
    {
        Probatum::Valid(t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::typeclasses::Compositio;

    #[derive(Clone, PartialEq, Eq, Debug, Hash)]
    struct Errs(&'static str);

    impl Compositio for Errs {
        fn combine(&self, other: &Self) -> Self {
            Self(match (self.0, other.0) {
                (a, b) if a == b => a,
                (a, b) => {
                    if a.len() >= b.len() {
                        a
                    } else {
                        b
                    }
                }
            })
        }
    }

    #[test]
    fn accumulates_errors() {
        let v1: Probatum<Errs, i32> = Probatum::invalid(Errs("a"));
        let v2: Probatum<Errs, i32> = Probatum::invalid(Errs("bb"));
        let combined = v1.map2(v2, |a, b| a + b);
        assert!(combined.is_invalid());
    }

    #[test]
    fn collects_values() {
        use alloc::vec;
        let items = vec![Probatum::valid(1), Probatum::valid(2)];
        let collected: Probatum<Errs, Vec<i32>> = Probatum::<Errs, i32>::collect(items);
        assert_eq!(
            collected
                .value()
                .expect("all items were valid so collect should return Valid"),
            &vec![1, 2]
        );
    }

    #[test]
    fn collect_accumulates_all_errors_from_mixed_iterator() {
        // collect must not short-circuit: errors from all invalid items must be accumulated
        // even when valid and invalid items are interleaved in the iterator.
        use alloc::vec;
        let items: Vec<Probatum<&str, i32>> = vec![
            Probatum::valid(1),
            Probatum::invalid("err_a"),
            Probatum::valid(2),
            Probatum::invalid("err_b"),
        ];
        let result = Probatum::<&str, i32>::collect(items);
        assert!(result.is_invalid(), "mixed iterator should yield Invalid");
        let errs = result.errors().expect("expected error list");
        assert_eq!(
            errs,
            &["err_a", "err_b"],
            "all errors should be accumulated"
        );
    }

    #[test]
    fn from_option_none_produces_single_error() {
        let result: Probatum<&str, i32> = Probatum::from_option(None, "missing value");
        assert!(result.is_invalid());
        assert_eq!(
            result
                .errors()
                .expect("from_option(None) should produce an Invalid with errors"),
            &["missing value"]
        );
    }

    #[test]
    fn collect_empty_iterator_produces_valid_empty_vec() {
        // An empty sequence has no errors, so collect must return Valid([]).
        let empty: alloc::vec::Vec<Probatum<&str, i32>> = alloc::vec::Vec::new();
        let result = Probatum::<&str, i32>::collect(empty);
        assert!(result.is_valid(), "empty iterator should yield Valid");
        assert_eq!(
            result.value().expect("expected Valid"),
            &alloc::vec::Vec::<i32>::new(),
            "Valid payload should be an empty Vec"
        );
    }

    #[test]
    fn map3_accumulates_errors_from_all_three_invalid_branches() {
        // map3 uses individual `if let` checks rather than the tuple-match used in
        // map2, so verify that errors from all three invalid inputs are accumulated
        // rather than the first one short-circuiting the others.
        let v1: Probatum<&str, i32> = Probatum::invalid("err1");
        let v2: Probatum<&str, i32> = Probatum::invalid("err2");
        let v3: Probatum<&str, i32> = Probatum::invalid("err3");
        let result = v1.map3(v2, v3, |a, b, c| a + b + c);
        assert!(result.is_invalid(), "all-invalid map3 should yield Invalid");
        let errs = result.errors().expect("expected error list");
        assert_eq!(errs.len(), 3, "all three errors should be accumulated");
        assert!(errs.contains(&"err1"));
        assert!(errs.contains(&"err2"));
        assert!(errs.contains(&"err3"));
    }

    #[test]
    fn invalid_many_rejects_empty_and_preserves_nonempty() {
        let empty = Probatum::<&str, i32>::invalid_many(core::iter::empty());
        assert_eq!(empty, Err(EmptyErrors));
        let one: Probatum<&str, i32> =
            Probatum::invalid_many(["reason"]).expect("one error is nonempty");
        assert_eq!(one.errors(), Some(&["reason"][..]));
    }

    #[test]
    fn buffer_layout_is_unchanged_by_nonempty_wrapper() {
        assert_eq!(
            core::mem::size_of::<NonEmptyErrors<u8>>(),
            core::mem::size_of::<ErrorBuf<u8>>()
        );
        assert_eq!(
            core::mem::align_of::<NonEmptyErrors<u8>>(),
            core::mem::align_of::<ErrorBuf<u8>>()
        );
    }

    #[test]
    fn nonclone_error_payloads_can_be_mapped_and_merged() {
        #[derive(Debug, PartialEq, Eq)]
        struct Owned(i32);
        let first = NonEmptyErrors::one(Owned(1));
        let second = NonEmptyErrors::one(Owned(2));
        let joined = first.concat(second).map(|e| e.0 * 10);
        assert_eq!(joined.as_slice(), &[10, 20]);
        assert_eq!(joined.len(), 2);
        assert!(!joined.is_empty());
    }

    #[test]
    fn into_result_converts_valid_to_ok_and_invalid_to_err() {
        // Valid must become Ok carrying the inner value.
        let ok: Probatum<&str, i32> = Probatum::valid(42);
        assert_eq!(ok.into_result(), Ok(42));

        // Invalid must become Err carrying the accumulated error buffer.
        let err: Probatum<&str, i32> = Probatum::invalid("boom");
        let result = err.into_result();
        assert!(result.is_err(), "Invalid should convert to Err");
        let errs = result.expect_err("into_result on an Invalid Probatum must yield Err");
        assert_eq!(errs.as_slice(), &["boom"]);
    }

    #[test]
    fn lift3_accumulates_errors_from_all_three_invalid_branches() {
        // lift3 is implemented via chained map2 calls rather than the direct
        // if-let checks used in map3, so verify independently that errors from
        // all three invalid inputs survive both map2 passes.
        let v1: Probatum<&str, i32> = Probatum::invalid("err1");
        let v2: Probatum<&str, i32> = Probatum::invalid("err2");
        let v3: Probatum<&str, i32> = Probatum::invalid("err3");
        let result = Probatum::lift3(|a, b, c| a + b + c, v1, v2, v3);
        assert!(
            result.is_invalid(),
            "all-invalid lift3 should yield Invalid"
        );
        let errs = result.errors().expect("expected error list");
        assert_eq!(
            errs.len(),
            3,
            "all three errors should be accumulated via chained map2"
        );
        assert!(errs.contains(&"err1"));
        assert!(errs.contains(&"err2"));
        assert!(errs.contains(&"err3"));
    }

    #[test]
    fn map_on_invalid_skips_function_and_preserves_errors() {
        // `map` (fmap) on an `Invalid` value must thread the error buffer through
        // untouched without ever invoking the mapping function.  This is the core
        // FP invariant: a failed context is opaque to pure transformations.
        let mut called = false;
        let v: Probatum<&str, i32> = Probatum::invalid("oops");
        let result: Probatum<&str, i32> = v.map(|n| {
            called = true;
            n * 2
        });
        assert!(!called, "mapping function must not be called for Invalid");
        assert!(result.is_invalid(), "result must remain Invalid");
        assert_eq!(
            result.errors().expect("Invalid must carry errors"),
            &["oops"],
            "original error must be preserved unchanged"
        );
    }

    #[test]
    fn map3_accumulates_errors_for_partial_invalid_inputs() {
        // The catch-all arm in map3 uses three separate `if let Invalid` checks,
        // so only the invalid branches contribute errors.  Verify each single-invalid
        // permutation: (I,V,V), (V,I,V), and (V,V,I) must each yield exactly one
        // error from the invalid position, and the valid values must be discarded.
        let err = |s| Probatum::<&str, i32>::invalid(s);
        let ok = |n| Probatum::<&str, i32>::valid(n);

        // Only first argument is invalid.
        let result = err("e1").map3(ok(2), ok(3), |a, b, c| a + b + c);
        assert!(result.is_invalid());
        let errs = result.errors().expect("should have errors");
        assert_eq!(errs, &["e1"], "(I,V,V) should carry only the first error");

        // Only second argument is invalid.
        let result = ok(1).map3(err("e2"), ok(3), |a, b, c| a + b + c);
        assert!(result.is_invalid());
        let errs = result.errors().expect("should have errors");
        assert_eq!(errs, &["e2"], "(V,I,V) should carry only the second error");

        // Only third argument is invalid.
        let result = ok(1).map3(ok(2), err("e3"), |a, b, c| a + b + c);
        assert!(result.is_invalid());
        let errs = result.errors().expect("should have errors");
        assert_eq!(errs, &["e3"], "(V,V,I) should carry only the third error");
    }

    #[test]
    fn map2_accumulates_all_errors_in_order_when_both_sides_have_multiple_errors() {
        // When both arguments carry multiple errors (built via invalid_many), map2 must
        // extend e1 with e2, preserving left-then-right ordering and the full count.
        let v1: Probatum<&str, i32> = Probatum::invalid_many(["e1a", "e1b"]).unwrap();
        let v2: Probatum<&str, i32> = Probatum::invalid_many(["e2a", "e2b", "e2c"]).unwrap();
        let result = v1.map2(v2, |a, b| a + b);
        assert!(result.is_invalid(), "both-invalid map2 must yield Invalid");
        let errs = result.errors().expect("Invalid must carry errors");
        assert_eq!(
            errs,
            &["e1a", "e1b", "e2a", "e2b", "e2c"],
            "left errors must precede right errors and all must be present"
        );
    }

    #[test]
    fn ap_accumulates_errors_from_both_invalid_function_and_value() {
        // `ap` delegates to `Apply::apply`, which has its own (Invalid, Invalid)
        // match arm independent of `map2`. Verify that errors from both the
        // invalid function container and the invalid value container are merged.
        let val: Probatum<&str, i32> = Probatum::invalid("val-err");
        let func: Probatum<&str, fn(i32) -> i32> = Probatum::invalid("func-err");
        let result: Probatum<&str, i32> = val.ap(func);
        assert!(result.is_invalid(), "both-invalid ap must yield Invalid");
        let errs = result.errors().expect("Invalid must carry errors");
        assert_eq!(
            errs.len(),
            2,
            "errors from both containers must be accumulated"
        );
        assert!(
            errs.contains(&"func-err"),
            "function container error must be present"
        );
        assert!(
            errs.contains(&"val-err"),
            "value container error must be present"
        );
    }

    #[test]
    fn from_result_ok_produces_valid_and_err_produces_invalid() {
        // The `From<Result<T, E>>` impl delegates to `IntoProbatum::into_probatum`.
        // Verify both branches: Ok must yield Valid carrying the inner value, and
        // Err must yield Invalid with a single-element error buffer.
        let ok_result: Result<i32, &str> = Ok(42);
        let from_ok: Probatum<&str, i32> = ok_result.into();
        assert!(from_ok.is_valid(), "Ok(v) must convert to Valid(v)");
        assert_eq!(from_ok.value().expect("Ok maps to Valid"), &42);

        let err_result: Result<i32, &str> = Err("bad");
        let from_err: Probatum<&str, i32> = err_result.into();
        assert!(from_err.is_invalid(), "Err(e) must convert to Invalid([e])");
        assert_eq!(
            from_err
                .errors()
                .expect("Err maps to Invalid with one error"),
            &["bad"],
        );
    }

    #[test]
    fn try_construct_calls_checker_only_after_validation_and_once() {
        use core::cell::Cell;

        struct Draft<'a> {
            value: i32,
            calls: &'a Cell<usize>,
        }
        #[derive(Debug, PartialEq, Eq)]
        struct Domain(i32);

        impl<'a> TryFrom<Draft<'a>> for Domain {
            type Error = &'static str;

            fn try_from(draft: Draft<'a>) -> Result<Self, Self::Error> {
                draft.calls.set(draft.calls.get() + 1);
                if draft.value >= 0 {
                    Ok(Self(draft.value))
                } else {
                    Err("negative value")
                }
            }
        }

        let calls = Cell::new(0);
        let parse_failed: Probatum<&str, Draft<'_>> =
            Probatum::invalid_many(["parse start", "parse end"]).expect("two independent errors");
        let result: Probatum<&str, Domain> = parse_failed.try_construct();
        assert_eq!(result.errors(), Some(&["parse start", "parse end"][..]));
        assert_eq!(
            calls.get(),
            0,
            "the constructor must not run after parse errors"
        );

        let accepted: Probatum<&str, Domain> = Probatum::valid(Draft {
            value: 3,
            calls: &calls,
        })
        .try_construct();
        assert_eq!(accepted, Probatum::valid(Domain(3)));
        assert_eq!(calls.get(), 1, "successful construction runs once");

        let rejected: Probatum<&str, Domain> = Probatum::valid(Draft {
            value: -1,
            calls: &calls,
        })
        .try_construct();
        assert_eq!(rejected.errors(), Some(&["negative value"][..]));
        assert_eq!(calls.get(), 2, "failed construction also runs exactly once");
    }

    #[test]
    fn try_construct_carries_nonclone_errors_without_reconstruction() {
        #[derive(Debug, PartialEq, Eq)]
        struct ErrorCode(u8);
        struct Draft(u8);
        #[derive(Debug, PartialEq, Eq)]
        struct Domain(u8);

        impl TryFrom<Draft> for Domain {
            type Error = ErrorCode;

            fn try_from(draft: Draft) -> Result<Self, Self::Error> {
                if draft.0 == 0 {
                    Err(ErrorCode(3))
                } else {
                    Ok(Self(draft.0))
                }
            }
        }

        let failed: Probatum<ErrorCode, Draft> =
            Probatum::invalid_many([ErrorCode(1), ErrorCode(2)]).expect("two errors");
        let domain: Probatum<ErrorCode, Domain> = failed.try_construct();
        assert_eq!(domain.errors(), Some(&[ErrorCode(1), ErrorCode(2)][..]));

        let invalid_domain: Probatum<ErrorCode, Domain> =
            Probatum::valid(Draft(0)).try_construct();
        assert_eq!(invalid_domain.errors(), Some(&[ErrorCode(3)][..]));
    }

    #[test]
    fn from_option_some_produces_valid_and_ignores_error() {
        // When the Option is Some, from_option must return Valid carrying the inner value
        // and must discard the provided fallback error — the error argument only applies
        // to the None branch.  Also verifies that `errors()` returns None on a Valid.
        let result: Probatum<&str, i32> = Probatum::from_option(Some(42), "should be ignored");
        assert!(result.is_valid(), "Some(_) must convert to Valid");
        assert_eq!(
            result
                .value()
                .expect("from_option(Some(v)) must yield Valid(v)"),
            &42,
        );
        assert!(
            result.errors().is_none(),
            "Valid must return None from errors()"
        );
    }
}
