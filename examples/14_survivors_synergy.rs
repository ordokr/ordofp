//! Survivors Synergy Example: Deep Multi-Subsystem Integration
//!
//! Demonstrates the combination of `OrdoFP`'s core survivors:
//! 1. Heterogeneous Field Validation (`HList` + `Validated` + `Universalis`)
//! 2. Deep Struct Modification with Optics (`Lens` + `Validated`)
//! 3. Batch Collection Validation (`IteratorValidateExt::validate_all`)
//! 4. Zero-Cost Monoidal Aggregation (`Semigroup` / `Monoid`)
//!
//! # Running
//!
//! ```bash
//! cargo run --example 14_survivors_synergy
//! ```

use ordofp::hlist::HListSequenceValidated;
use ordofp::monoid::Unitas;
use ordofp::optics::{Aspectus, aspectus};
use ordofp::semigroup::Compositio;
use ordofp::universalis::from_universalis;
use ordofp::validated::{IntoValidated, IteratorValidateExt, Validated};
use ordofp::{Universalis, hlist};

#[derive(Universalis, Clone, Debug, PartialEq)]
struct StudentProfile {
    name: String,
    age: u32,
    score: u32,
    email: String,
}

// ─── 1. Field Validators (producing Validated results) ───────────────────────

fn validate_name(name: &str) -> Validated<String, String> {
    if name.trim().len() >= 2 {
        Validated::Valid(name.trim().to_string())
    } else {
        Err(format!("Name '{name}' is too short (min 2 chars)")).into_validated()
    }
}

fn validate_age(age_str: &str) -> Validated<String, u32> {
    match age_str.parse::<u32>() {
        Ok(age) if (5..=100).contains(&age) => Validated::Valid(age),
        Ok(age) => Err(format!("Age {age} is outside school range (5..=100)")).into_validated(),
        Err(_) => Err(format!("Invalid age format: '{age_str}'")).into_validated(),
    }
}

fn validate_score(score_str: &str) -> Validated<String, u32> {
    match score_str.parse::<u32>() {
        Ok(score) if score <= 100 => Validated::Valid(score),
        Ok(score) => Err(format!("Score {score} exceeds maximum (100)")).into_validated(),
        Err(_) => Err(format!("Invalid score format: '{score_str}'")).into_validated(),
    }
}

fn validate_email(email: &str) -> Validated<String, String> {
    if email.contains('@') && email.contains('.') {
        Validated::Valid(email.to_lowercase())
    } else {
        Err(format!("Invalid email format: '{email}'")).into_validated()
    }
}

// ─── 2. Domain Monoid for Aggregating Stats ──────────────────────────────────

#[derive(Debug, Clone, PartialEq, Default)]
struct ClassSummary {
    total_students: usize,
    total_score: u32,
    passing_students: usize,
}

impl Compositio for ClassSummary {
    fn combine(&self, other: &Self) -> Self {
        Self {
            total_students: self.total_students + other.total_students,
            total_score: self.total_score + other.total_score,
            passing_students: self.passing_students + other.passing_students,
        }
    }
}

impl Unitas for ClassSummary {
    fn empty() -> Self {
        Self::default()
    }
}

impl ClassSummary {
    fn from_student(s: &StudentProfile) -> Self {
        Self {
            total_students: 1,
            total_score: s.score,
            passing_students: usize::from(s.score >= 60),
        }
    }

    fn average_score(&self) -> f64 {
        if self.total_students == 0 {
            0.0
        } else {
            f64::from(self.total_score)
                / f64::from(u32::try_from(self.total_students).expect("student count fits in u32"))
        }
    }
}

fn main() {
    println!("=== OrdoFP Survivors Synergy Showcase ===\n");

    // ─── Pipeline Step 1: Heterogeneous Validation & Struct Synthesis ────────
    println!("1. Heterogeneous Validation via HList + Validated + Universalis");
    println!("---------------------------------------------------------------");

    // Valid raw input
    let raw_fields_valid = hlist![
        validate_name("Alice Johnson"),
        validate_age("16"),
        validate_score("94"),
        validate_email("alice@school.edu"),
    ];

    // Sequence the HList of Validated values into a Validated HList, then convert to StudentProfile!
    let student_1: Validated<String, StudentProfile> =
        raw_fields_valid.sequence_validated().map(from_universalis);

    println!("   Valid Student: {student_1:?}");
    assert!(student_1.is_valid());

    // Invalid raw input (multiple errors accumulated simultaneously)
    let raw_fields_invalid = hlist![
        validate_name("X"),          // too short
        validate_age("not_an_age"),  // invalid number
        validate_score("150"),       // exceeds 100
        validate_email("bad_email"), // invalid email
    ];

    let student_invalid: Validated<String, StudentProfile> = raw_fields_invalid
        .sequence_validated()
        .map(from_universalis);

    println!(
        "   Invalid Student Errors (all 4 collected): {:?}",
        student_invalid.errors().unwrap()
    );
    assert_eq!(student_invalid.errors().unwrap().len(), 4);

    // ─── Pipeline Step 2: Deep Field Optics Modification with Validation ─────
    println!("\n2. Deep Struct Modification via Lens + Validated");
    println!("------------------------------------------------");

    let score_lens: Aspectus<StudentProfile, u32, _, _> = aspectus(
        |s: &StudentProfile| s.score,
        |s: &StudentProfile, score| StudentProfile { score, ..s.clone() },
    );

    let alice = student_1.into_result().unwrap();

    // Give Alice 5 bonus points, validating it does not exceed 100
    let bonus_pass = score_lens.modify_validated(&alice, |score| {
        let new_score = score + 5;
        if new_score <= 100 {
            Validated::Valid(new_score)
        } else {
            Err(format!("Bonus puts score {new_score} over 100")).into_validated()
        }
    });
    println!("   Alice with valid bonus: {bonus_pass:?}");
    assert_eq!(bonus_pass.value().unwrap().score, 99);

    // Give Alice 10 bonus points (94 + 10 = 104 > 100 -> fails with error)
    let bonus_fail = score_lens.modify_validated(&alice, |score| {
        let new_score = score + 10;
        if new_score <= 100 {
            Validated::Valid(new_score)
        } else {
            Err(format!("Bonus puts score {new_score} over 100")).into_validated()
        }
    });
    println!("   Alice with invalid bonus: {bonus_fail:?}");
    assert!(bonus_fail.is_invalid());

    // ─── Pipeline Step 3: Batch Record Validation with validate_all ──────────
    println!("\n3. Batch Record Validation with IteratorValidateExt");
    println!("---------------------------------------------------");

    let raw_roster = vec![
        ("Bob Smith", "17", "88", "bob@school.edu"),
        ("Carol White", "16", "92", "carol@school.edu"),
        ("David Brown", "15", "74", "david@school.edu"),
    ];

    let validated_roster: Validated<String, Vec<StudentProfile>> =
        raw_roster.into_iter().validate_all(|(n, a, s, e)| {
            hlist![
                validate_name(n),
                validate_age(a),
                validate_score(s),
                validate_email(e),
            ]
            .sequence_validated()
            .map(from_universalis)
        });

    println!(
        "   Validated Class Roster: {} students",
        validated_roster.value().unwrap().len()
    );

    // ─── Pipeline Step 4: Monoidal Aggregation over Validated Results ────────
    println!("\n4. Monoidal Summary Aggregation (Semigroup / Monoid)");
    println!("-----------------------------------------------------");

    let students = validated_roster.into_result().unwrap();
    let class_summary = students
        .iter()
        .map(ClassSummary::from_student)
        .fold(ClassSummary::empty(), |acc, item| acc.combine(&item));

    println!("   Total Students: {}", class_summary.total_students);
    println!("   Average Score:  {:.2}", class_summary.average_score());
    println!("   Passing:        {}", class_summary.passing_students);

    assert_eq!(class_summary.total_students, 3);
    assert_eq!(class_summary.passing_students, 3);

    println!("\n=== Survivors Synergy Completed Successfully! ===");
}
