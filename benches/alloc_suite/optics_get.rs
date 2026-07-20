//! Measures the clone overhead of composed-lens reads.
//!
//! `ComposedAspectus::get` clones the whole intermediate `A` (`outer.get`
//! returns owned), then extracts `B` and drops `A`. For a production LMS
//! pattern like `course_lens.compose(&name_lens).get(&enrollment)` that clones
//! the entire `Course` per read just to read its `name`. This bench quantifies that vs the
//! zero-clone floor (direct field borrow) so the fix can be sized.

use criterion::{Criterion, criterion_group};
use ordofp_core::optics::{AspectusRef, aspectus};
use std::hint::black_box;

#[derive(Clone)]
struct Course {
    _id: u64,
    name: String,
    _description: String,
    _tags: Vec<String>,
}

#[derive(Clone)]
struct Enrollment {
    _student: String,
    course: Course,
    _grade: f64,
}

fn sample() -> Enrollment {
    Enrollment {
        _student: "Alice Student".to_string(),
        course: Course {
            _id: 42,
            name: "Advanced Functional Programming".to_string(),
            _description: "A fairly long course description that makes the \
                           intermediate Course struct non-trivial to clone."
                .to_string(),
            _tags: vec![
                "rust".to_string(),
                "fp".to_string(),
                "monads".to_string(),
                "optics".to_string(),
            ],
        },
        _grade: 95.0,
    }
}

fn bench_composed_get(c: &mut Criterion) {
    let mut group = c.benchmark_group("optics_get");
    let enrollment = sample();

    // Current API: composed get clones the whole Course intermediate + the name.
    let course_lens = aspectus(
        |e: &Enrollment| e.course.clone(),
        |e: &Enrollment, course: Course| Enrollment {
            course,
            _student: e._student.clone(),
            _grade: e._grade,
        },
    );
    let name_lens = aspectus(
        |c: &Course| c.name.clone(),
        |c: &Course, name: String| Course {
            name,
            _id: c._id,
            _description: c._description.clone(),
            _tags: c._tags.clone(),
        },
    );
    let course_name = course_lens.compose(&name_lens);

    group.bench_function("composed_lens_get", |b| {
        b.iter(|| black_box(course_name.get(black_box(&enrollment))));
    });

    // NEW: composed borrowing aspectus — reads &name through &course with zero clones.
    let course_ref = AspectusRef::new(|e: &Enrollment| &e.course);
    let name_ref = AspectusRef::new(|c: &Course| &c.name);
    let course_name_ref = course_ref.compose(&name_ref);
    group.bench_function("composed_ref_get", |b| {
        b.iter(|| black_box(course_name_ref.get(black_box(&enrollment))));
    });

    // Zero-clone floor: direct nested field borrow (what a borrowing lens achieves).
    group.bench_function("direct_field_borrow", |b| {
        b.iter(|| black_box(&black_box(&enrollment).course.name));
    });

    // Single-level (non-composed) get clones just the focus String.
    group.bench_function("single_lens_get_string", |b| {
        b.iter(|| black_box(name_lens.get(black_box(&enrollment.course))));
    });

    group.finish();
}

criterion_group!(optics, bench_composed_get);
