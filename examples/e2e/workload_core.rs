//! Shared workload core for the end-to-end perf drivers (`e2e_workload` timed
//! driver, `e2e_allocs` allocation-count driver). Not auto-discovered as an
//! example (lives in a subdirectory); included via `#[path]` by both drivers.
//!
//! Models one term-end grade-processing batch shaped like a production LMS
//! application, with surface proportions measured from that real workload:
//!
//! 1. `validate` — per-grade `Probatum` multi-error validation (`map2` chain),
//!    the #1 downstream surface; validators are the faithful copies used by
//!    `benches/grade_validation.rs`.
//! 2. `route` — per-student error routing: `NonEmpty` error lists in a
//!    `Disiunctio` coproduct, `Display`-rendered at the boundary (the cold /
//!    error path; scales with `--error-pct`).
//! 3. `aggregate` — per-student and per-course `Compositio`/`Unitas` folds of
//!    `GradeStatistics` / `WeightedGrade` over `rust_decimal::Decimal`, the
//!    faithful copies used by `benches/grade_aggregation.rs`.
//! 4. `optics` — per-enrollment nested read through a composed cloning
//!    `Aspectus` (the pattern the production consumer imports).
//! 5. `pfds` — a small persistent `OrdMap` course index: per-course inserts,
//!    per-student `&str` lookups (the Borrow-generic path).
//!
//! Phases run phase-major (not request-major) so each phase can be timed with
//! two `Instant` calls per rep; the per-grade work is identical either way.
//!
//! Determinism: xorshift64* with a fixed seed. Each rep recomputes the same
//! checksum from value-bearing results (counts, rendered-error bytes, Decimal
//! mantissae, name lengths); drivers assert rep checksums are identical, and
//! the checksum must stay IDENTICAL across optimization commits
//! (behavior-preservation evidence).

use std::fmt::Write as _;
use std::time::Instant;

use ordofp::validated::Probatum;
use ordofp_core::disiunctio::Disiunctio;
use ordofp_core::nonempty::NonEmpty;
use ordofp_core::optics::aspectus;
use ordofp_core::pfds::OrdMap;
use ordofp_core::typeclasses::{Compositio, Unitas};
use rust_decimal::Decimal;

// ─── configuration ───────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct Config {
    pub courses: usize,
    pub students: usize,
    pub assignments: usize,
    pub reps: usize,
    /// Percentage (0–100) of grade cells generated invalid.
    pub error_pct: u64,
    pub seed: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            courses: 40,
            students: 150,
            assignments: 20,
            reps: 60,
            error_pct: 3,
            seed: 0xC0FFEE0D0F,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// Realistic steady-state mix (~3% invalid input).
    Steady,
    /// Error-path stress (50% invalid input unless --error-pct overrides).
    ErrorHeavy,
    /// Parse args and exit immediately (process/start-up cost floor).
    Startup,
}

fn usage(msg: &str) -> ! {
    eprintln!("error: {msg}");
    eprintln!(
        "usage: --mode steady|error-heavy|startup --courses N --students N \
         --assignments N --reps N --error-pct N --seed N"
    );
    std::process::exit(2)
}

fn num(args: &mut impl Iterator<Item = String>, flag: &str) -> u64 {
    let v = args
        .next()
        .unwrap_or_else(|| usage(&format!("{flag} needs a value")));
    v.parse()
        .unwrap_or_else(|_| usage(&format!("{flag} needs a number, got {v}")))
}

pub fn parse_args() -> (Config, Mode) {
    let mut cfg = Config::default();
    let mut mode = Mode::Steady;
    let mut error_pct_set = false;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--mode" => {
                let v = args.next().unwrap_or_else(|| usage("--mode needs a value"));
                mode = match v.as_str() {
                    "steady" => Mode::Steady,
                    "error-heavy" => Mode::ErrorHeavy,
                    "startup" => Mode::Startup,
                    other => usage(&format!("unknown mode {other}")),
                };
            }
            "--courses" => cfg.courses = num(&mut args, "--courses") as usize,
            "--students" => cfg.students = num(&mut args, "--students") as usize,
            "--assignments" => cfg.assignments = num(&mut args, "--assignments") as usize,
            "--reps" => cfg.reps = num(&mut args, "--reps") as usize,
            "--error-pct" => {
                cfg.error_pct = num(&mut args, "--error-pct").min(100);
                error_pct_set = true;
            }
            "--seed" => cfg.seed = num(&mut args, "--seed"),
            other => usage(&format!("unknown flag {other}")),
        }
    }
    if mode == Mode::ErrorHeavy && !error_pct_set {
        cfg.error_pct = 50;
    }
    (cfg, mode)
}

// ─── deterministic rng (xorshift64*) ─────────────────────────────────────────

pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed.max(1))
    }

    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

// ─── grade validation (faithful copy of benches/grade_validation.rs) ─────────

#[derive(Clone, Debug)]
pub enum GradeInputError {
    InvalidPointsPossible(String),
    ScoreOutOfRange(String),
    MissingGradingScheme,
}

impl core::fmt::Display for GradeInputError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidPointsPossible(msg) | Self::ScoreOutOfRange(msg) => f.write_str(msg),
            Self::MissingGradingScheme => f.write_str("missing grading scheme"),
        }
    }
}

fn validate_input_not_empty(input: &str) -> Probatum<GradeInputError, ()> {
    // Empty / special-case inputs are acceptable (cleared grade, "ex", "mi").
    let _ = input.trim();
    Probatum::valid(())
}

fn validate_points_possible(pp: Option<Decimal>) -> Probatum<GradeInputError, ()> {
    if let Some(pp) = pp
        && pp <= Decimal::ZERO
    {
        return Probatum::invalid(GradeInputError::InvalidPointsPossible(format!(
            "Points possible must be positive, got {pp}"
        )));
    }
    Probatum::valid(())
}

fn validate_score_range(score: Decimal, pp: Option<Decimal>) -> Probatum<GradeInputError, ()> {
    if score < Decimal::ZERO {
        return Probatum::invalid(GradeInputError::ScoreOutOfRange(format!(
            "Score cannot be negative: {score}"
        )));
    }
    if let Some(pp) = pp
        && score > pp
        && pp > Decimal::ZERO
    {
        return Probatum::invalid(GradeInputError::ScoreOutOfRange(format!(
            "Score {score} exceeds points possible {pp}"
        )));
    }
    Probatum::valid(())
}

fn validate_entry_mode(has_scheme: bool, needs_scheme: bool) -> Probatum<GradeInputError, ()> {
    if needs_scheme && !has_scheme {
        return Probatum::invalid(GradeInputError::MissingGradingScheme);
    }
    Probatum::valid(())
}

/// The real combine-and-accumulate pattern (map2 chain → collects ALL errors).
fn validate_all(
    input: &str,
    score: Decimal,
    pp: Option<Decimal>,
    has_scheme: bool,
    needs_scheme: bool,
) -> Probatum<GradeInputError, ()> {
    Probatum::valid(())
        .map2(validate_input_not_empty(input), |(), ()| ())
        .map2(validate_points_possible(pp), |(), ()| ())
        .map2(validate_score_range(score, pp), |(), ()| ())
        .map2(validate_entry_mode(has_scheme, needs_scheme), |(), ()| ())
}

// ─── grade aggregation (faithful copy of benches/grade_aggregation.rs) ───────

#[derive(Debug, Clone, PartialEq)]
pub struct GradeStatistics {
    pub count: u64,
    pub sum: Decimal,
    pub sum_of_squares: Decimal,
    pub min: Option<Decimal>,
    pub max: Option<Decimal>,
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

impl Compositio for GradeStatistics {
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

impl Unitas for GradeStatistics {
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

impl Compositio for WeightedGrade {
    fn combine(&self, other: &Self) -> Self {
        Self {
            weighted_sum: self.weighted_sum + other.weighted_sum,
            weight_sum: self.weight_sum + other.weight_sum,
        }
    }
}

impl Unitas for WeightedGrade {
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

// ─── optics structs (shapes from benches/optics_get.rs) ──────────────────────

#[derive(Clone)]
pub struct Course {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
}

#[derive(Clone)]
pub struct Enrollment {
    pub student: String,
    pub course: Course,
    pub grade: f64,
}

const DESCRIPTION: &str = "A fairly long course description that makes the \
                           intermediate Course struct non-trivial to clone.";
const TAGS: [&str; 4] = ["rust", "fp", "monads", "optics"];

// ─── workload data + per-rep state ───────────────────────────────────────────

#[derive(Clone, Copy)]
pub struct GradeCell {
    score: Decimal,
    pp: Option<Decimal>,
    has_scheme: bool,
    needs_scheme: bool,
}

pub struct CourseData {
    pub name: String,
    /// Raw input strings, one per assignment (validated per student).
    inputs: Vec<String>,
    /// Per-assignment weights for the weighted-average fold.
    weights: Vec<Decimal>,
    /// Row-major `[student][assignment]` grade cells.
    cells: Vec<GradeCell>,
    pub enrollments: Vec<Enrollment>,
}

pub struct WorkloadData {
    pub courses: Vec<CourseData>,
    pub students: usize,
    pub assignments: usize,
}

fn gen_cell(rng: &mut Rng, error_pct: u64) -> GradeCell {
    let bad = rng.below(100) < error_pct;
    if !bad {
        return GradeCell {
            score: Decimal::new(rng.below(10_001) as i64, 2),
            pp: if rng.below(10) == 0 {
                None
            } else {
                Some(Decimal::new(10_000, 2))
            },
            has_scheme: true,
            needs_scheme: rng.below(4) == 0,
        };
    }
    // Invalid cell: a non-zero 3-bit mask picks 1–3 simultaneous error causes,
    // exercising Probatum's multi-error accumulation.
    let kinds = 1 + rng.below(7);
    let mut score = Decimal::new(rng.below(10_001) as i64, 2);
    let mut pp = Some(Decimal::new(10_000, 2));
    let mut has_scheme = true;
    let mut needs_scheme = false;
    if kinds & 1 != 0 {
        pp = Some(Decimal::ZERO); // InvalidPointsPossible
    }
    if kinds & 2 != 0 {
        score = Decimal::new(-(rng.below(5_000) as i64 + 1), 2); // ScoreOutOfRange
    }
    if kinds & 4 != 0 {
        has_scheme = false; // MissingGradingScheme
        needs_scheme = true;
    }
    GradeCell {
        score,
        pp,
        has_scheme,
        needs_scheme,
    }
}

pub fn generate(cfg: &Config) -> WorkloadData {
    let mut rng = Rng::new(cfg.seed);
    let courses = (0..cfg.courses)
        .map(|ci| {
            let name = format!("course_{ci:04}");
            let inputs: Vec<String> = (0..cfg.assignments)
                .map(|ai| format!("{}", 50 + (ai * 7) % 51))
                .collect();
            let weights: Vec<Decimal> = (0..cfg.assignments)
                .map(|_| Decimal::new((rng.below(5) + 1) as i64, 0))
                .collect();
            let cells: Vec<GradeCell> = (0..cfg.students * cfg.assignments)
                .map(|_| gen_cell(&mut rng, cfg.error_pct))
                .collect();
            let enrollments: Vec<Enrollment> = (0..cfg.students)
                .map(|si| Enrollment {
                    student: format!("student_{si:05}"),
                    course: Course {
                        id: ci as u64,
                        name: name.clone(),
                        description: DESCRIPTION.to_string(),
                        tags: TAGS.iter().map(|t| (*t).to_string()).collect(),
                    },
                    grade: 0.0,
                })
                .collect();
            CourseData {
                name,
                inputs,
                weights,
                cells,
                enrollments,
            }
        })
        .collect();
    WorkloadData {
        courses,
        students: cfg.students,
        assignments: cfg.assignments,
    }
}

/// Reused per-rep buffers, preallocated so the steady state adds no
/// driver-side allocations on the hot (valid) path.
pub struct State {
    /// Per-(course,student) valid scores.
    scores: Vec<Vec<Decimal>>,
    /// Per-(course,student) (score, weight) pairs for the weighted fold.
    pairs: Vec<Vec<(Decimal, Decimal)>>,
    /// Per-(course,student) accumulated validation errors.
    errs: Vec<Vec<GradeInputError>>,
    /// Per-course (course index, average) collected by the aggregate phase.
    course_avgs: Vec<(usize, Decimal)>,
    /// Reused boundary-rendering buffer for error messages.
    render_buf: String,
}

impl State {
    pub fn new(cfg: &Config) -> Self {
        let slots = cfg.courses * cfg.students;
        Self {
            scores: (0..slots)
                .map(|_| Vec::with_capacity(cfg.assignments))
                .collect(),
            pairs: (0..slots)
                .map(|_| Vec::with_capacity(cfg.assignments))
                .collect(),
            errs: (0..slots).map(|_| Vec::new()).collect(),
            course_avgs: Vec::with_capacity(cfg.courses),
            render_buf: String::new(),
        }
    }
}

// ─── the measured pipeline ───────────────────────────────────────────────────

pub const PHASES: [&str; 5] = ["validate", "route", "aggregate", "optics", "pfds"];

pub struct RepOutcome {
    pub checksum: u64,
    pub phase_ns: [u64; 5],
    pub valid_grades: u64,
    pub errored_students: u64,
    pub errors: u64,
}

pub fn run_rep(data: &WorkloadData, st: &mut State) -> RepOutcome {
    let s_count = data.students;
    let a_count = data.assignments;
    let mut cs: u64 = 0xcbf2_9ce4_8422_2325;
    let mut phase_ns = [0u64; 5];
    let mut valid_grades = 0u64;
    let mut errored_students = 0u64;
    let mut errors = 0u64;
    st.course_avgs.clear();

    // Phase 1: validate — per-grade Probatum map2 accumulation (#1 surface).
    let t = Instant::now();
    for (ci, course) in data.courses.iter().enumerate() {
        for si in 0..s_count {
            let slot = ci * s_count + si;
            st.scores[slot].clear();
            st.pairs[slot].clear();
            st.errs[slot].clear();
            for ai in 0..a_count {
                let cell = &course.cells[si * a_count + ai];
                let v = validate_all(
                    &course.inputs[ai],
                    cell.score,
                    cell.pp,
                    cell.has_scheme,
                    cell.needs_scheme,
                );
                match v {
                    Probatum::Valid(()) => {
                        valid_grades += 1;
                        st.scores[slot].push(cell.score);
                        st.pairs[slot].push((cell.score, course.weights[ai]));
                    }
                    Probatum::Invalid(es) => st.errs[slot].extend(es),
                }
            }
        }
    }
    phase_ns[0] = t.elapsed().as_nanos() as u64;

    // Phase 2: route — per-student NonEmpty/Disiunctio error coproduct +
    // boundary Display rendering (the explicit error/cold path).
    let t = Instant::now();
    for ci in 0..data.courses.len() {
        for si in 0..s_count {
            let slot = ci * s_count + si;
            let routed: Disiunctio<NonEmpty<GradeInputError>, usize> = if st.errs[slot].is_empty() {
                Disiunctio::Dexter(st.scores[slot].len())
            } else {
                let mut drained = st.errs[slot].drain(..);
                let head = drained.next().expect("checked non-empty");
                let tail: Vec<GradeInputError> = drained.collect();
                Disiunctio::Sinister(NonEmpty::new(head, tail))
            };
            match routed {
                Disiunctio::Dexter(n) => cs = cs.wrapping_add(n as u64),
                Disiunctio::Sinister(ne) => {
                    errored_students += 1;
                    errors += ne.len() as u64;
                    st.render_buf.clear();
                    for e in ne.iter() {
                        let _ = write!(st.render_buf, "{e}; ");
                    }
                    cs = cs.wrapping_add(st.render_buf.len() as u64).rotate_left(7);
                }
            }
        }
    }
    phase_ns[1] = t.elapsed().as_nanos() as u64;

    // Phase 3: aggregate — Semigroup/Monoid folds (per-student stats fold,
    // per-course combine, per-student weighted average).
    let t = Instant::now();
    for ci in 0..data.courses.len() {
        let mut course_stats = GradeStatistics::empty();
        for si in 0..s_count {
            let slot = ci * s_count + si;
            let student_stats = calculate_statistics(&st.scores[slot]);
            course_stats = course_stats.combine(&student_stats);
            if let Some(avg) = weighted_average(&st.pairs[slot]) {
                cs = cs.wrapping_add(avg.mantissa() as u64);
            }
        }
        if course_stats.count > 0 {
            let avg = course_stats.sum / Decimal::from(course_stats.count);
            st.course_avgs.push((ci, avg));
            cs = cs
                .wrapping_add(course_stats.count)
                .wrapping_add(course_stats.sum.mantissa() as u64);
        }
    }
    phase_ns[2] = t.elapsed().as_nanos() as u64;

    // Phase 4: optics — per-enrollment nested read via composed cloning
    // Aspectus (the consumer's import).
    let t = Instant::now();
    let course_lens = aspectus(
        |e: &Enrollment| e.course.clone(),
        |e: &Enrollment, course: Course| Enrollment {
            course,
            student: e.student.clone(),
            grade: e.grade,
        },
    );
    let name_lens = aspectus(
        |c: &Course| c.name.clone(),
        |c: &Course, name: String| Course {
            name,
            id: c.id,
            description: c.description.clone(),
            tags: c.tags.clone(),
        },
    );
    let course_name = course_lens.compose(&name_lens);
    for course in &data.courses {
        for e in &course.enrollments {
            let name = course_name.get(e);
            cs = cs.wrapping_add(name.len() as u64);
        }
    }
    phase_ns[3] = t.elapsed().as_nanos() as u64;

    // Phase 5: pfds — persistent OrdMap course index: per-course insert
    // (owned String key), per-student &str lookup (Borrow-generic path).
    let t = Instant::now();
    let mut index: OrdMap<String, Decimal> = OrdMap::new();
    for (ci, avg) in &st.course_avgs {
        index = index.insert(data.courses[*ci].name.clone(), *avg);
    }
    for course in &data.courses {
        for _ in 0..s_count {
            if let Some(avg) = index.get(course.name.as_str()) {
                cs = cs.wrapping_add(avg.mantissa() as u64 & 0xFFFF);
            }
        }
    }
    phase_ns[4] = t.elapsed().as_nanos() as u64;

    RepOutcome {
        checksum: cs,
        phase_ns,
        valid_grades,
        errored_students,
        errors,
    }
}
