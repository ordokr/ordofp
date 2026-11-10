//! Validate independent fields with typed locations before checked domain construction.
//!
//! Error paths can be prefixed with row indices without cloning error payloads.

use ordofp::vernacular::{IntoValidated, Located, PathSegment, Validated};

#[derive(Debug, PartialEq, Eq)]
enum Problem {
    Missing,
    NotUnsigned,
    StartAfterEnd,
}

#[derive(Debug, Clone, Copy)]
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
    type Error = Located<Problem>;

    fn try_from(parsed: ParsedWindow) -> Result<Self, Self::Error> {
        if parsed.start > parsed.end {
            Err(Located::field("window", Problem::StartAfterEnd))
        } else {
            Ok(Self {
                start: parsed.start,
                end: parsed.end,
            })
        }
    }
}

impl Window {
    const fn bounds(&self) -> (u32, u32) {
        (self.start, self.end)
    }

    // Borrow the source; a failed replacement leaves the original unchanged.
    fn with_start(&self, start: u32) -> Result<Self, Located<Problem>> {
        Self::try_from(ParsedWindow {
            start,
            end: self.end,
        })
    }
}

fn parse_endpoint(value: &str, field: &'static str) -> Validated<Located<Problem>, u32> {
    let value = value.trim();
    if value.is_empty() {
        return Validated::invalid(Located::field(field, Problem::Missing));
    }
    value.parse::<u32>().map_or_else(
        |_| Validated::invalid(Located::field(field, Problem::NotUnsigned)),
        Validated::valid,
    )
}

fn validate_window(raw: RawWindow<'_>) -> Validated<Located<Problem>, Window> {
    // Independent parsing accumulates errors in start, end order.
    let parsed =
        parse_endpoint(raw.start, "start").map2(parse_endpoint(raw.end, "end"), |start, end| {
            ParsedWindow { start, end }
        });

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

    // Add collection context to both errors, preserving their own field paths.
    let nested = rejected.map_err(|issue| issue.at_index(4).at_field("rows"));
    assert_eq!(
        nested.errors().unwrap()[0].path(),
        &[
            PathSegment::Field("rows".into()),
            PathSegment::Index(4),
            PathSegment::Field("start".into()),
        ]
    );
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
                Located::field("start", Problem::NotUnsigned),
                Located::field("end", Problem::Missing),
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
            &[Located::field("window", Problem::StartAfterEnd)]
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
            &[Located::field("start", Problem::Missing)]
        );
    }

    #[test]
    fn batch_context_preserves_error_order_and_typed_segments() {
        let result = validate_window(RawWindow {
            start: "bad",
            end: "",
        })
        .map_err(|issue| issue.at_index(4).at_field("rows"));
        let errors = result.errors().expect("two located failures");
        assert_eq!(errors.len(), 2);
        assert_eq!(
            errors[0].path(),
            &[
                PathSegment::Field("rows".into()),
                PathSegment::Index(4),
                PathSegment::Field("start".into()),
            ]
        );
        assert_eq!(
            errors[1].path(),
            &[
                PathSegment::Field("rows".into()),
                PathSegment::Index(4),
                PathSegment::Field("end".into()),
            ]
        );
        assert_eq!(errors[0].error(), &Problem::NotUnsigned);
        assert_eq!(errors[1].error(), &Problem::Missing);
    }
}
