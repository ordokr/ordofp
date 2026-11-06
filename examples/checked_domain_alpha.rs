//! 0.2.0 design probe: accumulate independent field errors, then check a domain invariant.
//!
//! This uses existing public OrdoFP APIs on main. The local LocatedIssue is a
//! temporary example type, not the proposed generic Located<E> API.

use ordofp::vernacular::{IntoValidated, Validated};

#[derive(Debug, PartialEq, Eq)]
enum Problem {
    Missing,
    NotUnsigned,
    StartAfterEnd,
}

#[derive(Debug, PartialEq, Eq)]
struct LocatedIssue {
    field: &'static str,
    problem: Problem,
}

#[derive(Debug)]
struct RawWindow<'a> {
    start: &'a str,
    end: &'a str,
}

#[derive(Debug)]
struct ParsedWindow {
    start: u32,
    end: u32,
}

// Private fields mean callers cannot bypass the checked conversion.
#[derive(Debug, PartialEq, Eq)]
struct Window {
    start: u32,
    end: u32,
}

impl TryFrom<ParsedWindow> for Window {
    type Error = LocatedIssue;

    fn try_from(parsed: ParsedWindow) -> Result<Self, Self::Error> {
        if parsed.start > parsed.end {
            Err(LocatedIssue {
                field: "window",
                problem: Problem::StartAfterEnd,
            })
        } else {
            Ok(Self {
                start: parsed.start,
                end: parsed.end,
            })
        }
    }
}

impl Window {
    fn bounds(&self) -> (u32, u32) {
        (self.start, self.end)
    }

    // Borrow the source; a failed replacement leaves the original unchanged.
    fn with_start(&self, start: u32) -> Result<Self, LocatedIssue> {
        Self::try_from(ParsedWindow {
            start,
            end: self.end,
        })
    }
}

fn parse_endpoint(value: &str, field: &'static str) -> Validated<LocatedIssue, u32> {
    let value = value.trim();
    if value.is_empty() {
        return Validated::invalid(LocatedIssue {
            field,
            problem: Problem::Missing,
        });
    }
    match value.parse::<u32>() {
        Ok(n) => Validated::valid(n),
        Err(_) => Validated::invalid(LocatedIssue {
            field,
            problem: Problem::NotUnsigned,
        }),
    }
}

fn validate_window(raw: RawWindow<'_>) -> Validated<LocatedIssue, Window> {
    // Independent parsing accumulates errors in start, end order.
    let parsed = parse_endpoint(raw.start, "start").map2(
        parse_endpoint(raw.end, "end"),
        |start, end| ParsedWindow { start, end },
    );

    // Cross-field validation runs only after both endpoints parse.
    match parsed {
        Validated::Valid(dto) => Window::try_from(dto).into_validated(),
        Validated::Invalid(errors) => Validated::Invalid(errors),
    }
}

fn main() {
    let accepted = validate_window(RawWindow {
        start: "2",
        end: "5",
    })
    .into_result()
    .expect("valid endpoints form a valid window");
    assert_eq!(accepted.bounds(), (2, 5));
    assert!(accepted.with_start(7).is_err());
    assert_eq!(accepted.bounds(), (2, 5));

    let rejected = validate_window(RawWindow {
        start: "bad",
        end: "",
    });
    assert_eq!(rejected.errors().expect("both fields rejected").len(), 2);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn independent_errors_accumulate_in_input_order() {
        let result = validate_window(RawWindow {
            start: "oops",
            end: "",
        });
        assert_eq!(
            result.errors().unwrap(),
            &[
                LocatedIssue {
                    field: "start",
                    problem: Problem::NotUnsigned,
                },
                LocatedIssue {
                    field: "end",
                    problem: Problem::Missing,
                },
            ]
        );
    }

    #[test]
    fn cross_field_invariant_runs_after_both_fields_succeed() {
        let result = validate_window(RawWindow {
            start: "12",
            end: "4",
        });
        assert_eq!(
            result.errors().unwrap(),
            &[LocatedIssue {
                field: "window",
                problem: Problem::StartAfterEnd,
            }]
        );
    }

    #[test]
    fn rejected_update_preserves_borrowed_source() {
        let source = validate_window(RawWindow {
            start: "2",
            end: "5",
        })
        .into_result()
        .unwrap();
        assert!(source.with_start(8).is_err());
        assert_eq!(source.bounds(), (2, 5));
        assert_eq!(source.with_start(3).unwrap().bounds(), (3, 5));
    }

    #[test]
    fn zero_and_equal_bounds_are_valid() {
        let zero = validate_window(RawWindow {
            start: "0",
            end: "0",
        })
        .into_result()
        .unwrap();
        assert_eq!(zero.bounds(), (0, 0));
    }

    #[test]
    fn field_failures_prevent_cross_field_reconstruction() {
        let result = validate_window(RawWindow {
            start: "",
            end: "1",
        });
        assert_eq!(
            result.errors().unwrap(),
            &[LocatedIssue {
                field: "start",
                problem: Problem::Missing,
            }]
        );
    }
}
