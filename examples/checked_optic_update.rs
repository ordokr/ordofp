//! Checked domain reconstruction after validating a composed optic update.
//!
//! The lens modifies only a plain, possibly invalid DTO. The final
//! `TryFrom` conversion is the sole constructor for the invariant-bearing domain
//! type. A failed parse or failed domain check cannot alter the borrowed source.

use ordofp::vernacular::{Lens, Validated};

#[derive(Debug, PartialEq, Eq)]
enum Problem {
    NotUnsigned,
    StartAfterEnd,
}

// Plain drafts are transport shapes, not proof of a valid time window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct WindowDraft {
    start: u32,
    end: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct BatchDraft {
    window: WindowDraft,
    id: u32,
}

// Private fields prevent callers from constructing an invalid domain object.
#[derive(Debug, PartialEq, Eq)]
struct CheckedBatch {
    start: u32,
    end: u32,
    id: u32,
}

impl TryFrom<BatchDraft> for CheckedBatch {
    type Error = Problem;

    fn try_from(draft: BatchDraft) -> Result<Self, Self::Error> {
        if draft.window.start > draft.window.end {
            return Err(Problem::StartAfterEnd);
        }
        Ok(Self {
            start: draft.window.start,
            end: draft.window.end,
            id: draft.id,
        })
    }
}

impl CheckedBatch {
    // An explicit snapshot, not a generic, unchecked setter on CheckedBatch.
    const fn draft(&self) -> BatchDraft {
        BatchDraft {
            window: WindowDraft {
                start: self.start,
                end: self.end,
            },
            id: self.id,
        }
    }
}

fn update_start(source: &CheckedBatch, proposed: &str) -> Validated<Problem, CheckedBatch> {
    let outer = Lens::new(
        |draft: &BatchDraft| draft.window,
        |draft: &BatchDraft, window| BatchDraft {
            window,
            id: draft.id,
        },
    );
    let inner = Lens::new(
        |window: &WindowDraft| window.start,
        |window: &WindowDraft, start| WindowDraft {
            start,
            end: window.end,
        },
    );
    let composed = outer.compose(&inner);

    // This setter operates on a plain DTO, never on CheckedBatch.
    let candidate = composed.modify_validated(&source.draft(), |_| {
        proposed.parse::<u32>().map_or_else(
            |_| Validated::invalid(Problem::NotUnsigned),
            Validated::valid,
        )
    });

    // Dependent invariants are checked only after independent parsing passed.
    candidate.try_construct::<CheckedBatch>()
}

fn initial_batch() -> CheckedBatch {
    CheckedBatch::try_from(BatchDraft {
        window: WindowDraft { start: 2, end: 6 },
        id: 19,
    })
    .expect("initial domain value must satisfy its checked constructor")
}

fn main() {
    let source = initial_batch();
    let updated = update_start(&source, "4")
        .into_result()
        .expect("valid update");
    assert_eq!(
        updated,
        CheckedBatch {
            start: 4,
            end: 6,
            id: 19,
        }
    );
    assert_eq!(update_start(&source, "9").errors(), Some(&[Problem::StartAfterEnd][..]));
    assert_eq!(update_start(&source, "bad").errors(), Some(&[Problem::NotUnsigned][..]));
    assert_eq!(
        source,
        CheckedBatch {
            start: 2,
            end: 6,
            id: 19,
        },
        "the original is never changed"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source() -> CheckedBatch {
        initial_batch()
    }

    #[test]
    fn checked_update_preserves_unrelated_fields_and_original() {
        let input = source();
        let updated = update_start(&input, "4")
            .into_result()
            .expect("valid update");
        assert_eq!(updated.start, 4);
        assert_eq!(updated.end, 6);
        assert_eq!(updated.id, 19);
        assert_eq!(input, source());
    }

    #[test]
    fn invalid_input_never_constructs_a_domain_value() {
        let input = source();
        assert_eq!(
            update_start(&input, "bad").errors(),
            Some(&[Problem::NotUnsigned][..])
        );
        assert_eq!(input, source());
    }

    #[test]
    fn cross_field_failure_rejects_composed_update() {
        let input = source();
        assert_eq!(
            update_start(&input, "9").errors(),
            Some(&[Problem::StartAfterEnd][..])
        );
        assert_eq!(input, source());
    }

    #[test]
    fn equal_bounds_are_a_valid_domain_update() {
        let input = source();
        let accepted = update_start(&input, "6")
            .into_result()
            .expect("equal bounds are allowed");
        assert_eq!(accepted.start, accepted.end);
        assert_eq!(input, source());
    }
}
