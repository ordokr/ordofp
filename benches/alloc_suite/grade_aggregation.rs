//! End-to-end macro-bench mirroring a production LMS application's
//! grade-aggregation module — the grade-statistics Semigroup/Monoid fold,
//! the most ordofp-central real downstream path.
//!
//! `GradeStatistics`, `WeightedGrade`, and the `calculate_statistics` /
//! `weighted_average` helpers mirror that production grading path verbatim
//! (incl. its `rust_decimal::Decimal` grade type, so per-call arithmetic cost
//! is faithful).
//! They are wired to *this crate's* `Compositio`/`Unitas` traits — the actual
//! library code under test.
//!
//! Verdict question (never measured before; previously dismissed by inspection):
//! the Semigroup fold builds a fresh `Self` every step
//! (`fold(empty, |acc, g| acc.combine(&g))`). Does that per-step construction
//! cost more than a hand-written in-place fold? If `semigroup_fold ≈
//! handwritten_inplace`, the abstraction is zero-cost and there is NO library
//! lever (a `combine_mut`/`combine_into` addition would buy nothing). If it is
//! materially slower, that gap sizes a real (additive, non-breaking) lever.

use criterion::{BenchmarkId, Criterion, criterion_group};
use ordofp_core::typeclasses::{Compositio as Semigroup, Unitas as Monoid};
use rust_decimal::Decimal;
use std::hint::black_box;

// ─── mirrors the production LMS aggregation module verbatim ─────────────────

#[derive(Debug, Clone, PartialEq)]
struct GradeStatistics {
    count: u64,
    sum: Decimal,
    sum_of_squares: Decimal,
    min: Option<Decimal>,
    max: Option<Decimal>,
}

impl GradeStatistics {
    fn from_score(score: Decimal) -> Self {
        Self {
            count: 1,
            sum: score,
            sum_of_squares: score * score,
            min: Some(score),
            max: Some(score),
        }
    }
}

impl Default for GradeStatistics {
    fn default() -> Self {
        Self {
            count: 0,
            sum: Decimal::ZERO,
            sum_of_squares: Decimal::ZERO,
            min: None,
            max: None,
        }
    }
}

impl Semigroup for GradeStatistics {
    fn combine(&self, other: &Self) -> Self {
        Self {
            count: self.count + other.count,
            sum: self.sum + other.sum,
            sum_of_squares: self.sum_of_squares + other.sum_of_squares,
            min: match (self.min, other.min) {
                (Some(a), Some(b)) => Some(a.min(b)),
                (Some(a), None) => Some(a),
                (None, Some(b)) => Some(b),
                (None, None) => None,
            },
            max: match (self.max, other.max) {
                (Some(a), Some(b)) => Some(a.max(b)),
                (Some(a), None) => Some(a),
                (None, Some(b)) => Some(b),
                (None, None) => None,
            },
        }
    }
}

impl Monoid for GradeStatistics {
    fn empty() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone, PartialEq)]
struct WeightedGrade {
    weighted_sum: Decimal,
    weight_sum: Decimal,
}

impl WeightedGrade {
    fn new(score: Decimal, weight: Decimal) -> Self {
        Self {
            weighted_sum: score * weight,
            weight_sum: weight,
        }
    }
    fn weighted_average(&self) -> Option<Decimal> {
        if self.weight_sum == Decimal::ZERO {
            None
        } else {
            Some(self.weighted_sum / self.weight_sum)
        }
    }
}

impl Default for WeightedGrade {
    fn default() -> Self {
        Self {
            weighted_sum: Decimal::ZERO,
            weight_sum: Decimal::ZERO,
        }
    }
}

impl Semigroup for WeightedGrade {
    fn combine(&self, other: &Self) -> Self {
        Self {
            weighted_sum: self.weighted_sum + other.weighted_sum,
            weight_sum: self.weight_sum + other.weight_sum,
        }
    }
}

impl Monoid for WeightedGrade {
    fn empty() -> Self {
        Self::default()
    }
}

/// Real consumer helper (the production `calculate_statistics`).
fn calculate_statistics(scores: &[Decimal]) -> GradeStatistics {
    scores
        .iter()
        .map(|s| GradeStatistics::from_score(*s))
        .fold(GradeStatistics::empty(), |acc, s| acc.combine(&s))
}

/// Real consumer helper (the production `weighted_average`).
fn weighted_average(grades: &[(Decimal, Decimal)]) -> Option<Decimal> {
    grades
        .iter()
        .map(|(score, weight)| WeightedGrade::new(*score, *weight))
        .fold(WeightedGrade::empty(), |acc, g| acc.combine(&g))
        .weighted_average()
}

// ─── hand-written in-place floor (no per-step Self construction) ─────────────

fn calculate_statistics_inplace(scores: &[Decimal]) -> GradeStatistics {
    let mut acc = GradeStatistics::empty();
    for &s in scores {
        acc.count += 1;
        acc.sum += s;
        acc.sum_of_squares += s * s;
        acc.min = Some(match acc.min {
            Some(m) => m.min(s),
            None => s,
        });
        acc.max = Some(match acc.max {
            Some(m) => m.max(s),
            None => s,
        });
    }
    acc
}

fn weighted_average_inplace(grades: &[(Decimal, Decimal)]) -> Option<Decimal> {
    let mut weighted_sum = Decimal::ZERO;
    let mut weight_sum = Decimal::ZERO;
    for &(score, weight) in grades {
        weighted_sum += score * weight;
        weight_sum += weight;
    }
    if weight_sum == Decimal::ZERO {
        None
    } else {
        Some(weighted_sum / weight_sum)
    }
}

// ─── benches ─────────────────────────────────────────────────────────────────

fn realistic_scores(n: usize) -> Vec<Decimal> {
    // Grades in [0.00, 100.00] with 2-decimal precision (scale = 2), like a gradebook.
    (0..n)
        .map(|i| Decimal::new(((i as i64 * 137) % 10001).abs(), 2))
        .collect()
}

fn bench(c: &mut Criterion) {
    let mut g = c.benchmark_group("grade_aggregation");

    for &n in &[30usize, 100, 1000, 10_000] {
        let scores = realistic_scores(n);
        let weighted: Vec<(Decimal, Decimal)> = scores
            .iter()
            .map(|&s| (s, Decimal::new(((s.mantissa() % 5) + 1) as i64, 0)))
            .collect();

        // The real consumer pattern: map→Semigroup fold (fresh Self per step).
        g.bench_with_input(
            BenchmarkId::new("stats_semigroup_fold", n),
            &scores,
            |b, s| b.iter(|| black_box(calculate_statistics(black_box(s)))),
        );
        // The floor: hand-written in-place fold.
        g.bench_with_input(
            BenchmarkId::new("stats_handwritten_inplace", n),
            &scores,
            |b, s| b.iter(|| black_box(calculate_statistics_inplace(black_box(s)))),
        );

        g.bench_with_input(
            BenchmarkId::new("weighted_semigroup_fold", n),
            &weighted,
            |b, w| b.iter(|| black_box(weighted_average(black_box(w)))),
        );
        g.bench_with_input(
            BenchmarkId::new("weighted_handwritten_inplace", n),
            &weighted,
            |b, w| b.iter(|| black_box(weighted_average_inplace(black_box(w)))),
        );
    }

    g.finish();
}

criterion_group!(grade_aggregation, bench);
