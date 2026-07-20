//! Applicative Probatum with error accumulation.
//!
//! This replaces the legacy `HList` accumulator with a semigroupal `Probatum<E, A>`
//! that collects multiple errors instead of short-circuiting like `Result`.

use alloc::vec::Vec;
use core::fmt::Debug;

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
    Invalid(#[cfg_attr(feature = "serde", serde(with = "error_buf_serde"))] ErrorBuf<E>),
}

#[cfg(feature = "serde")]
mod error_buf_serde {
    use super::ErrorBuf;
    use alloc::vec::Vec;
    use serde::{Deserialize, Serialize};

    /// Serialize an `ErrorBuf` as a flat sequence of errors.
    ///
    /// Used by `#[serde(with = "error_buf_serde")]` on `ErrorBuf` fields.
    pub(crate) fn serialize<S, E>(buf: &ErrorBuf<E>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
        E: serde::Serialize,
    {
        buf.as_slice().serialize(serializer)
    }

    /// Deserialize an `ErrorBuf` from a flat sequence of errors.
    ///
    /// Used by `#[serde(with = "error_buf_serde")]` on `ErrorBuf` fields.
    pub(crate) fn deserialize<'de, D, E>(deserializer: D) -> Result<ErrorBuf<E>, D::Error>
    where
        D: serde::Deserializer<'de>,
        E: serde::Deserialize<'de>,
    {
        let v = Vec::<E>::deserialize(deserializer)?;
        Ok(v.into_iter().collect())
    }
}

impl<E, A> Probatum<E, A> {
    /// Construct a valid value.
    pub fn valid(a: A) -> Self {
        Probatum::Valid(a)
    }

    /// Construct a single error.
    pub fn invalid(err: E) -> Self {
        Probatum::Invalid(core::iter::once(err).collect())
    }

    /// Construct from many errors.
    pub fn invalid_many<I: IntoIterator<Item = E>>(iter: I) -> Self {
        let buf: ErrorBuf<E> = iter.into_iter().collect();
        Probatum::Invalid(buf)
    }

    /// Returns true if this is `Valid`.
    pub fn is_valid(&self) -> bool {
        matches!(self, Probatum::Valid(_))
    }

    /// Returns true if this is `Invalid`.
    pub fn is_invalid(&self) -> bool {
        !self.is_valid()
    }

    /// Borrow the success value.
    pub fn value(&self) -> Option<&A> {
        match self {
            Probatum::Valid(a) => Some(a),
            _ => None,
        }
    }

    /// Borrow the accumulated errors.
    pub fn errors(&self) -> Option<&[E]> {
        match self {
            Probatum::Invalid(es) => Some(es.as_slice()),
            _ => None,
        }
    }

    /// Convert into a `Result`.
    ///
    /// # Errors
    ///
    /// Returns `Err` carrying the accumulated error buffer when this is
    /// `Invalid`; every collected error is preserved, in order.
    pub fn into_result(self) -> Result<A, ErrorBuf<E>> {
        match self {
            Probatum::Valid(a) => Ok(a),
            Probatum::Invalid(es) => Err(es),
        }
    }

    /// Combine two Probatum values with a pure function.
    pub fn map2<B, C, F>(self, vb: Probatum<E, B>, mut f: F) -> Probatum<E, C>
    where
        F: FnMut(A, B) -> C,
    {
        match (self, vb) {
            (Probatum::Valid(a), Probatum::Valid(b)) => Probatum::Valid(f(a, b)),
            (Probatum::Invalid(mut e1), Probatum::Invalid(e2)) => {
                e1.extend(e2);
                Probatum::Invalid(e1)
            }
            (Probatum::Invalid(e), _) | (_, Probatum::Invalid(e)) => Probatum::Invalid(e),
        }
    }

    /// Collect an iterator of Probatum values into a single Probatum Vec.
    pub fn collect<I, T>(iter: I) -> Probatum<E, Vec<T>>
    where
        I: IntoIterator<Item = Probatum<E, T>>,
    {
        let iter = iter.into_iter();
        // Pre-size from the iterator's hint: in the all-valid case `values`
        // ends up with exactly this many items, so the upper bound is tight
        // (no over-allocation) and we avoid the realloc cascade of Vec::new().
        let (lower, upper) = iter.size_hint();
        let mut values = Vec::with_capacity(upper.unwrap_or(lower));
        let mut errors: Option<ErrorBuf<E>> = None;
        for item in iter {
            match item {
                Probatum::Valid(v) => {
                    if errors.is_none() {
                        values.push(v);
                    }
                }
                Probatum::Invalid(mut es) => {
                    if let Some(ref mut acc) = errors {
                        acc.append(&mut es);
                    } else {
                        errors = Some(es);
                    }
                }
            }
        }

        if let Some(es) = errors {
            Probatum::Invalid(es)
        } else {
            Probatum::Valid(values)
        }
    }

    /// Lift two Probatum values with a combining function.
    pub fn lift2<B, C, F>(f: F, va: Probatum<E, A>, vb: Probatum<E, B>) -> Probatum<E, C>
    where
        F: FnMut(A, B) -> C,
    {
        va.map2(vb, f)
    }

    /// Lift three Probatum values with a combining function.
    pub fn map3<B, C, D, F>(
        self,
        vb: Probatum<E, B>,
        vc: Probatum<E, C>,
        mut f: F,
    ) -> Probatum<E, D>
    where
        F: FnMut(A, B, C) -> D,
    {
        match (self, vb, vc) {
            (Probatum::Valid(a), Probatum::Valid(b), Probatum::Valid(c)) => {
                Probatum::Valid(f(a, b, c))
            }
            (va, vb, vc) => {
                let mut errors: ErrorBuf<E> = ErrorBuf::new();
                if let Probatum::Invalid(es) = va {
                    errors.extend(es);
                }
                if let Probatum::Invalid(es) = vb {
                    errors.extend(es);
                }
                if let Probatum::Invalid(es) = vc {
                    errors.extend(es);
                }
                Probatum::Invalid(errors)
            }
        }
    }

    /// Lift three Probatum values without consuming `self`.
    pub fn lift3<B, C, D, F>(
        mut f: F,
        va: Probatum<E, A>,
        vb: Probatum<E, B>,
        vc: Probatum<E, C>,
    ) -> Probatum<E, D>
    where
        F: FnMut(A, B, C) -> D,
    {
        va.map2(vb, |a, b| (a, b)).map2(vc, |(a, b), c| f(a, b, c))
    }

    /// Alias for `lift2` (cats/zio-style naming).
    pub fn map2_alias<B, C, F>(va: Probatum<E, A>, vb: Probatum<E, B>, f: F) -> Probatum<E, C>
    where
        F: FnMut(A, B) -> C,
    {
        Self::lift2(f, va, vb)
    }

    /// Alias for `lift3`.
    pub fn map3_alias<B, C, D, F>(
        va: Probatum<E, A>,
        vb: Probatum<E, B>,
        vc: Probatum<E, C>,
        f: F,
    ) -> Probatum<E, D>
    where
        F: FnMut(A, B, C) -> D,
    {
        Self::lift3(f, va, vb, vc)
    }

    /// Convert from an Option using a provided error when None.
    pub fn from_option(opt: Option<A>, err: E) -> Self {
        match opt {
            Some(a) => Probatum::Valid(a),
            None => Probatum::Invalid(core::iter::once(err).collect()),
        }
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
            Err(e) => Probatum::Invalid(core::iter::once(e).collect()),
        }
    }
}

impl<T, E> From<Result<T, E>> for Probatum<E, T> {
    fn from(res: Result<T, E>) -> Self {
        IntoProbatum::into_probatum(res)
    }
}

impl<E, A> Functor for Probatum<E, A> {
    type Inner = A;
    type Target<T> = Probatum<E, T>;

    fn map<B, F>(self, mut f: F) -> Self::Target<B>
    where
        F: FnMut(Self::Inner) -> B,
    {
        match self {
            Probatum::Valid(a) => Probatum::Valid(f(a)),
            Probatum::Invalid(e) => Probatum::Invalid(e),
        }
    }
}

impl<E, A> Apply for Probatum<E, A> {
    fn apply<B, F>(self, ff: Probatum<E, F>) -> Probatum<E, B>
    where
        F: FnMut(A) -> B,
    {
        match (ff, self) {
            (Probatum::Valid(mut f), Probatum::Valid(a)) => Probatum::Valid(f(a)),
            (Probatum::Invalid(mut e1), Probatum::Invalid(e2)) => {
                e1.extend(e2);
                Probatum::Invalid(e1)
            }
            (Probatum::Invalid(e), _) | (_, Probatum::Invalid(e)) => Probatum::Invalid(e),
        }
    }
}

impl<E, A> Applicatio for Probatum<E, A> {
    #[inline]
    fn pure(a: A) -> Self {
        Probatum::Valid(a)
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
            Errs(match (self.0, other.0) {
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
    fn invalid_many_with_empty_iterator_produces_invalid_with_no_errors() {
        // invalid_many collects its argument into an ErrorBuf without checking emptiness.
        // An empty iterator yields Invalid([]), which is still considered invalid even though
        // it carries zero error messages — callers must be aware of this edge case.
        let result: Probatum<&str, i32> = Probatum::invalid_many(core::iter::empty());
        assert!(result.is_invalid(), "invalid_many([]) must produce Invalid");
        assert_eq!(
            result.errors().expect("Invalid must expose Some(&[])"),
            &[] as &[&str],
            "error slice must be empty when no errors were provided"
        );
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
        let v1: Probatum<&str, i32> = Probatum::invalid_many(["e1a", "e1b"]);
        let v2: Probatum<&str, i32> = Probatum::invalid_many(["e2a", "e2b", "e2c"]);
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
