//! Enhanced Error Messages for Effect Mismatches
//!
//! Provides clear, actionable error messages when effect constraints fail.

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;

// =============================================================================
// Effect Mismatch Errors
// =============================================================================

/// Types of effect-related errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectErrorKind {
    /// An effect is missing from the row.
    MissingEffect {
        /// Name of the effect the computation requires.
        required: String,
        /// Effects actually present in the row; empty when the
        /// computation is pure.
        available: Vec<String>,
    },
    /// An effect is present but shouldn't be.
    UnexpectedEffect {
        /// Name of the effect that should not be present.
        effect: String,
        /// Why the effect is disallowed in this context.
        reason: String,
    },
    /// Effect types don't match.
    TypeMismatch {
        /// Rendered type the constraint expected.
        expected: String,
        /// Rendered type that was actually found.
        found: String,
    },
    /// Handler doesn't cover all effects.
    UnhandledEffect {
        /// Name of the effect left unhandled.
        effect: String,
        /// Name of the handler that fails to cover it.
        handler: String,
    },
    /// Row types are incompatible.
    RowMismatch {
        /// Rendered effect row the constraint expected.
        expected_row: String,
        /// Rendered effect row that was actually found.
        found_row: String,
    },
}

/// An enhanced error message for effect-related issues.
#[derive(Debug, Clone)]
pub struct EffectError {
    /// The kind of error.
    pub kind: EffectErrorKind,
    /// Location information (file, line, column).
    pub location: Option<ErrorLocation>,
    /// Suggestions for fixing the error.
    pub suggestions: Vec<Suggestion>,
    /// Related information.
    pub notes: Vec<String>,
}

impl EffectError {
    /// Create a new effect error.
    #[inline]
    pub fn new(kind: EffectErrorKind) -> Self {
        EffectError {
            kind,
            location: None,
            suggestions: Vec::new(),
            notes: Vec::new(),
        }
    }

    /// Add location information.
    pub fn with_location(mut self, location: ErrorLocation) -> Self {
        self.location = Some(location);
        self
    }

    /// Add a suggestion.
    pub fn with_suggestion(mut self, suggestion: Suggestion) -> Self {
        self.suggestions.push(suggestion);
        self
    }

    /// Add a note.
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    /// Create a "missing effect" error.
    pub fn missing_effect(required: impl Into<String>, available: Vec<String>) -> Self {
        EffectError::new(EffectErrorKind::MissingEffect {
            required: required.into(),
            available,
        })
    }

    /// Create an "unexpected effect" error.
    pub fn unexpected_effect(effect: impl Into<String>, reason: impl Into<String>) -> Self {
        EffectError::new(EffectErrorKind::UnexpectedEffect {
            effect: effect.into(),
            reason: reason.into(),
        })
    }

    /// Create a "type mismatch" error.
    pub fn type_mismatch(expected: impl Into<String>, found: impl Into<String>) -> Self {
        EffectError::new(EffectErrorKind::TypeMismatch {
            expected: expected.into(),
            found: found.into(),
        })
    }

    /// Create an "unhandled effect" error.
    pub fn unhandled_effect(effect: impl Into<String>, handler: impl Into<String>) -> Self {
        EffectError::new(EffectErrorKind::UnhandledEffect {
            effect: effect.into(),
            handler: handler.into(),
        })
    }

    /// Format as a primary error message.
    pub fn primary_message(&self) -> String {
        match &self.kind {
            EffectErrorKind::MissingEffect {
                required,
                available,
            } => {
                if available.is_empty() {
                    format!("effect `{required}` is required but the computation is pure")
                } else {
                    let available_str = available.join(", ");
                    format!(
                        "effect `{required}` is required but not in effect row: [{available_str}]"
                    )
                }
            }
            EffectErrorKind::UnexpectedEffect { effect, reason } => {
                format!("unexpected effect `{effect}`: {reason}")
            }
            EffectErrorKind::TypeMismatch { expected, found } => {
                format!("effect type mismatch: expected `{expected}`, found `{found}`")
            }
            EffectErrorKind::UnhandledEffect { effect, handler } => {
                format!("effect `{effect}` is not handled by `{handler}`")
            }
            EffectErrorKind::RowMismatch {
                expected_row,
                found_row,
            } => {
                format!(
                    "incompatible effect rows:\n  expected: {expected_row}\n  found: {found_row}"
                )
            }
        }
    }
}

impl fmt::Display for EffectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "error: {}", self.primary_message())?;

        if let Some(ref loc) = self.location {
            writeln!(f, "  --> {}:{}:{}", loc.file, loc.line, loc.column)?;
        }

        for note in &self.notes {
            writeln!(f, "  note: {note}")?;
        }

        for suggestion in &self.suggestions {
            writeln!(f, "  help: {suggestion}")?;
        }

        Ok(())
    }
}

// =============================================================================
// Error Location
// =============================================================================

/// Location of an error in source code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorLocation {
    /// File path.
    pub file: String,
    /// Line number (1-indexed).
    pub line: u32,
    /// Column number (1-indexed).
    pub column: u32,
}

impl ErrorLocation {
    /// Create a new error location.
    pub fn new(file: impl Into<String>, line: u32, column: u32) -> Self {
        ErrorLocation {
            file: file.into(),
            line,
            column,
        }
    }
}

// =============================================================================
// Suggestions
// =============================================================================

/// A suggestion for fixing an error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Suggestion {
    /// The suggestion message.
    pub message: String,
    /// Optional code snippet to insert/replace.
    pub code: Option<String>,
    /// Whether this is a primary suggestion.
    pub is_primary: bool,
}

impl Suggestion {
    /// Create a new suggestion.
    pub fn new(message: impl Into<String>) -> Self {
        Suggestion {
            message: message.into(),
            code: None,
            is_primary: false,
        }
    }

    /// Add a code snippet.
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    /// Mark as primary suggestion.
    pub fn primary(mut self) -> Self {
        self.is_primary = true;
        self
    }

    // Common suggestions

    /// Suggest adding a handler.
    pub fn add_handler(effect: &str, handler: &str) -> Self {
        Suggestion::new(format!("add a handler for `{effect}`"))
            .with_code(format!("handle::<{handler}, _>(handler, computation)"))
    }

    /// Suggest extending the effect row.
    pub fn extend_row(effect: &str) -> Self {
        Suggestion::new(format!("add `{effect}` to the effect row constraint"))
            .with_code(format!("where R: HasEffect<{effect}>"))
    }

    /// Suggest using a different function.
    pub fn use_function(current: &str, suggested: &str) -> Self {
        Suggestion::new(format!(
            "consider using `{suggested}` instead of `{current}`"
        ))
    }

    /// Suggest lifting a pure value.
    pub fn lift_pure() -> Self {
        Suggestion::new("consider lifting the pure value into the effect context")
            .with_code("pure(value)")
    }
}

impl fmt::Display for Suggestion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)?;
        if let Some(ref code) = self.code {
            write!(f, "\n           {code}")?;
        }
        Ok(())
    }
}

// =============================================================================
// Error Builders
// =============================================================================

/// Builder for creating rich error messages.
pub struct ErrorBuilder {
    kind: EffectErrorKind,
    location: Option<ErrorLocation>,
    suggestions: Vec<Suggestion>,
    notes: Vec<String>,
}

impl ErrorBuilder {
    /// Start building an error.
    #[inline]
    pub fn new(kind: EffectErrorKind) -> Self {
        ErrorBuilder {
            kind,
            location: None,
            suggestions: Vec::new(),
            notes: Vec::new(),
        }
    }

    /// Add location.
    pub fn at(mut self, file: impl Into<String>, line: u32, column: u32) -> Self {
        self.location = Some(ErrorLocation::new(file, line, column));
        self
    }

    /// Add a suggestion.
    pub fn suggest(mut self, suggestion: Suggestion) -> Self {
        self.suggestions.push(suggestion);
        self
    }

    /// Add a note.
    pub fn note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    /// Build the error.
    pub fn build(self) -> EffectError {
        EffectError {
            kind: self.kind,
            location: self.location,
            suggestions: self.suggestions,
            notes: self.notes,
        }
    }
}

// =============================================================================
// Pre-built Error Messages
// =============================================================================

/// Create an error for when IO is used in a pure context.
pub fn error_io_in_pure_context() -> EffectError {
    EffectError::new(EffectErrorKind::UnexpectedEffect {
        effect: "IO".to_string(),
        reason: "cannot perform IO in a pure computation".to_string(),
    })
    .with_suggestion(
        Suggestion::new("use an effectful context")
            .with_code("Eff<R, A> where R: HasEffect<IoEffect>"),
    )
    .with_note("pure computations cannot have side effects")
}

/// Create an error for missing state handler.
pub fn error_missing_state_handler(state_type: &str) -> EffectError {
    EffectError::missing_effect(format!("State<{state_type}>"), Vec::new())
        .with_suggestion(Suggestion::add_handler(
            &format!("State<{state_type}>"),
            "StateHandler",
        ))
        .with_note("state effects must be handled before running a computation")
}

/// Create an error for unhandled effects.
pub fn error_effects_not_handled(effects: &[&str]) -> EffectError {
    EffectError::new(EffectErrorKind::RowMismatch {
        expected_row: "∅ (empty)".to_string(),
        found_row: effects.join(" | "),
    })
    .with_suggestion(Suggestion::new("handle all effects before running"))
    .with_note("computations can only be run when all effects are handled")
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_missing_effect_error() {
        let error = EffectError::missing_effect(
            "State<Config>",
            alloc::vec!["IO".to_string(), "Error<AppError>".to_string()],
        );

        let msg = error.primary_message();
        assert!(msg.contains("State<Config>"));
        assert!(msg.contains("IO"));
    }

    #[test]
    fn test_error_with_suggestion() {
        let error = EffectError::missing_effect("IO", Vec::new())
            .with_suggestion(Suggestion::extend_row("IO"));

        assert!(!error.suggestions.is_empty());
    }

    #[test]
    fn test_error_builder() {
        let error = ErrorBuilder::new(EffectErrorKind::TypeMismatch {
            expected: "State<u32>".to_string(),
            found: "State<i32>".to_string(),
        })
        .at("src/main.rs", 42, 10)
        .note("state types must match exactly")
        .suggest(Suggestion::new("change the state type"))
        .build();

        assert!(error.location.is_some());
        assert_eq!(error.notes.len(), 1);
        assert_eq!(error.suggestions.len(), 1);
    }

    #[test]
    fn test_error_display() {
        let error = EffectError::type_mismatch("IO", "State<u32>")
            .with_location(ErrorLocation::new("src/lib.rs", 10, 5))
            .with_note("these effect types are incompatible");

        let display = error.to_string();
        assert!(display.contains("error:"));
        assert!(display.contains("src/lib.rs:10:5"));
        assert!(display.contains("note:"));
    }
}
