//! End-to-end macro-bench mirroring a production LMS application's
//! grade-calculator helper — the #1 ordofp surface
//! (`validated::Probatum`, 21 call sites): multi-error grade-input validation.
//!
//! The real pattern combines several `Probatum<GradeInputError, ()>` validators
//! via `map2` so ALL errors are accumulated in one pass (better UX than
//! `Result`'s short-circuit). Validators + error type mirror that production
//! helper verbatim (incl. `rust_decimal::Decimal`).
//!
//! Verdict question: does Probatum's applicative `map2` accumulation cost more
//! than a hand-written `Vec`-accumulating validator with identical (collect-all)
//! semantics? Happy path (the ~99% case: a valid grade) is the clean comparison
//! — both should be allocation-free. On the error path Probatum's
//! `SmallVec<[E;8]>` (feature `Probatum-smallvec`, which the production consumer enables) stays
//! INLINE for ≤8 errors, where the naive `Vec` accumulator heap-allocates.

use criterion::{Criterion, criterion_group};
use ordofp::validated::Probatum as Validated;
use rust_decimal::Decimal;
use std::hint::black_box;

// ─── faithful copy of calculator_helper.rs error type + validators ──────────

#[derive(Clone, Debug)]
// The String payloads mirror the consumer's error type (and their format! cost).
enum GradeInputError {
    InvalidPointsPossible(String),
    ScoreOutOfRange(String),
    MissingGradingScheme,
}

// Mirrors the consumer's user-facing rendering; also the read path for the
// message payloads (the bench only black_box'es the Probatum).
impl core::fmt::Display for GradeInputError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidPointsPossible(msg) | Self::ScoreOutOfRange(msg) => f.write_str(msg),
            Self::MissingGradingScheme => f.write_str("missing grading scheme"),
        }
    }
}

fn validate_input_not_empty(input: &str) -> Validated<GradeInputError, ()> {
    // Empty / special-case inputs are acceptable (cleared grade, "ex", "mi").
    let _ = input.trim();
    Validated::valid(())
}

fn validate_points_possible(pp: Option<Decimal>) -> Validated<GradeInputError, ()> {
    if let Some(pp) = pp
        && pp <= Decimal::ZERO
    {
        return Validated::invalid(GradeInputError::InvalidPointsPossible(format!(
            "Points possible must be positive, got {pp}"
        )));
    }
    Validated::valid(())
}

fn validate_score_range(score: Decimal, pp: Option<Decimal>) -> Validated<GradeInputError, ()> {
    if score < Decimal::ZERO {
        return Validated::invalid(GradeInputError::ScoreOutOfRange(format!(
            "Score cannot be negative: {score}"
        )));
    }
    if let Some(pp) = pp
        && score > pp
        && pp > Decimal::ZERO
    {
        return Validated::invalid(GradeInputError::ScoreOutOfRange(format!(
            "Score {score} exceeds points possible {pp}"
        )));
    }
    Validated::valid(())
}

fn validate_entry_mode(has_scheme: bool, needs_scheme: bool) -> Validated<GradeInputError, ()> {
    if needs_scheme && !has_scheme {
        return Validated::invalid(GradeInputError::MissingGradingScheme);
    }
    Validated::valid(())
}

/// The real combine-and-accumulate pattern (map2 chain → collects ALL errors).
fn validate_all(
    input: &str,
    score: Decimal,
    pp: Option<Decimal>,
    has_scheme: bool,
    needs_scheme: bool,
) -> Validated<GradeInputError, ()> {
    Validated::valid(())
        .map2(validate_input_not_empty(input), |(), ()| ())
        .map2(validate_points_possible(pp), |(), ()| ())
        .map2(validate_score_range(score, pp), |(), ()| ())
        .map2(validate_entry_mode(has_scheme, needs_scheme), |(), ()| ())
}

// ─── hand-written Vec-accumulating floor (same collect-all semantics) ────────

fn validate_all_handwritten(
    _input: &str,
    score: Decimal,
    pp: Option<Decimal>,
    has_scheme: bool,
    needs_scheme: bool,
) -> Result<(), Vec<GradeInputError>> {
    let mut errs: Vec<GradeInputError> = Vec::new();
    if let Some(pp) = pp
        && pp <= Decimal::ZERO
    {
        errs.push(GradeInputError::InvalidPointsPossible(format!(
            "Points possible must be positive, got {pp}"
        )));
    }
    if score < Decimal::ZERO {
        errs.push(GradeInputError::ScoreOutOfRange(format!(
            "Score cannot be negative: {score}"
        )));
    } else if let Some(pp) = pp
        && score > pp
        && pp > Decimal::ZERO
    {
        errs.push(GradeInputError::ScoreOutOfRange(format!(
            "Score {score} exceeds points possible {pp}"
        )));
    }
    if needs_scheme && !has_scheme {
        errs.push(GradeInputError::MissingGradingScheme);
    }
    if errs.is_empty() { Ok(()) } else { Err(errs) }
}

// ─── benches ─────────────────────────────────────────────────────────────────

fn bench(c: &mut Criterion) {
    let mut g = c.benchmark_group("grade_validation");
    let valid = (Decimal::new(8500, 2), Some(Decimal::new(10000, 2))); // 85.00 / 100.00
    let two_err = (Decimal::new(-500, 2), Some(Decimal::ZERO)); // negative score + zero pp

    // Happy path — the ~99% case. Both should be allocation-free.
    g.bench_function("probatum_all_valid", |b| {
        b.iter(|| {
            black_box(validate_all(
                black_box("85"),
                black_box(valid.0),
                black_box(valid.1),
                true,
                false,
            ))
        });
    });
    g.bench_function("handwritten_all_valid", |b| {
        b.iter(|| {
            black_box(validate_all_handwritten(
                black_box("85"),
                black_box(valid.0),
                black_box(valid.1),
                true,
                false,
            ))
        });
    });

    // Two-error path. format! dominates both; Probatum's SmallVec stays inline.
    g.bench_function("probatum_two_errors", |b| {
        b.iter(|| {
            black_box(validate_all(
                black_box(""),
                black_box(two_err.0),
                black_box(two_err.1),
                false,
                true,
            ))
        });
    });
    g.bench_function("handwritten_two_errors", |b| {
        b.iter(|| {
            black_box(validate_all_handwritten(
                black_box(""),
                black_box(two_err.0),
                black_box(two_err.1),
                false,
                true,
            ))
        });
    });

    g.finish();
}

criterion_group!(grade_validation, bench);
