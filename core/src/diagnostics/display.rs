//! Effect Type Display Formatting
//!
//! Provides human-readable formatting for complex effect types.

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;

// =============================================================================
// Effect Display Trait
// =============================================================================

/// Trait for types that can be displayed in a user-friendly format.
///
/// This is used by IDE tools to show effect types in hover information
/// and error messages.
pub trait EffectDisplay {
    /// Format the type for display in IDE tooltips.
    fn display_for_ide(&self) -> String;

    /// Format the type for display in error messages.
    fn display_for_error(&self) -> String;

    /// Get a short description suitable for inline hints.
    fn short_description(&self) -> String;
}

// =============================================================================
// Effect Row Display
// =============================================================================

/// Represents a formatted effect row for display purposes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectRowDisplay {
    /// The effects in this row.
    pub effects: Vec<EffectInfo>,
    /// Whether the row is open (has a tail variable).
    pub is_open: bool,
    /// The tail variable name if open.
    pub tail_var: Option<String>,
}

impl EffectRowDisplay {
    /// Create an empty row.
    #[inline]
    pub fn empty() -> Self {
        EffectRowDisplay {
            effects: Vec::new(),
            is_open: false,
            tail_var: None,
        }
    }

    /// Create a closed row with specific effects.
    #[inline]
    pub fn closed(effects: Vec<EffectInfo>) -> Self {
        EffectRowDisplay {
            effects,
            is_open: false,
            tail_var: None,
        }
    }

    /// Create an open row with a tail variable.
    #[inline]
    pub fn open(effects: Vec<EffectInfo>, tail_var: impl Into<String>) -> Self {
        EffectRowDisplay {
            effects,
            is_open: true,
            tail_var: Some(tail_var.into()),
        }
    }

    /// Add an effect to the row.
    #[inline]
    pub fn with_effect(mut self, effect: EffectInfo) -> Self {
        self.effects.push(effect);
        self
    }
}

impl fmt::Display for EffectRowDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.effects.is_empty() && !self.is_open {
            return write!(f, "Pure");
        }

        let mut effect_strs: Vec<String> = Vec::with_capacity(self.effects.len());
        effect_strs.extend(self.effects.iter().map(ToString::to_string));

        let effects_part = effect_strs.join(" | ");

        if self.is_open {
            if let Some(ref tail) = self.tail_var {
                if self.effects.is_empty() {
                    write!(f, "{tail}")
                } else {
                    write!(f, "{effects_part} | {tail}")
                }
            } else {
                write!(f, "{effects_part} | ...")
            }
        } else {
            write!(f, "{effects_part}")
        }
    }
}

// =============================================================================
// Effect Information
// =============================================================================

/// Information about a single effect for display purposes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectInfo {
    /// The effect name (e.g., "State", "IO", "Error").
    pub name: String,
    /// Type parameters for the effect.
    pub type_params: Vec<String>,
    /// Human-readable description.
    pub description: Option<String>,
    /// Whether this is a core effect or user-defined.
    pub is_core_effect: bool,
}

impl EffectInfo {
    /// Create a new effect info.
    pub fn new(name: impl Into<String>) -> Self {
        EffectInfo {
            name: name.into(),
            type_params: Vec::new(),
            description: None,
            is_core_effect: false,
        }
    }

    /// Add a type parameter.
    pub fn with_param(mut self, param: impl Into<String>) -> Self {
        self.type_params.push(param.into());
        self
    }

    /// Set the description.
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Mark as a core effect.
    pub fn core(mut self) -> Self {
        self.is_core_effect = true;
        self
    }

    /// Common core effects.
    pub fn io() -> Self {
        Self::new("IO")
            .with_description("Performs input/output operations")
            .core()
    }

    /// Create an [`EffectInfo`] for the `State<S>` core effect.
    ///
    /// `state_type` is the display name of the state type `S` (e.g. `"i32"`).
    pub fn state(state_type: impl Into<String>) -> Self {
        Self::new("State")
            .with_param(state_type)
            .with_description("Manages mutable state")
            .core()
    }

    /// Create an [`EffectInfo`] for the `Error<E>` core effect.
    ///
    /// `error_type` is the display name of the error type `E` (e.g. `"String"`).
    pub fn error(error_type: impl Into<String>) -> Self {
        Self::new("Error")
            .with_param(error_type)
            .with_description("May fail with an error")
            .core()
    }

    /// Create an [`EffectInfo`] for the `Reader<E>` core effect.
    ///
    /// `env_type` is the display name of the environment type `E`.
    pub fn reader(env_type: impl Into<String>) -> Self {
        Self::new("Reader")
            .with_param(env_type)
            .with_description("Reads from an environment")
            .core()
    }

    /// Create an [`EffectInfo`] for the `Writer<W>` core effect.
    ///
    /// `log_type` is the display name of the log/monoidal accumulator type `W`.
    pub fn writer(log_type: impl Into<String>) -> Self {
        Self::new("Writer")
            .with_param(log_type)
            .with_description("Writes to a log")
            .core()
    }

    /// Create an [`EffectInfo`] for the `Async` core effect.
    pub fn async_effect() -> Self {
        Self::new("Async")
            .with_description("Performs asynchronous operations")
            .core()
    }
}

impl fmt::Display for EffectInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.type_params.is_empty() {
            write!(f, "{}", self.name)
        } else {
            write!(f, "{}<{}>", self.name, self.type_params.join(", "))
        }
    }
}

// =============================================================================
// Computation Type Display
// =============================================================================

/// Formatted display of a computation type.
#[derive(Debug, Clone)]
pub struct ComputationTypeDisplay {
    /// The effect row.
    pub effects: EffectRowDisplay,
    /// The return type.
    pub return_type: String,
}

impl ComputationTypeDisplay {
    /// Create a new computation type display.
    pub fn new(effects: EffectRowDisplay, return_type: impl Into<String>) -> Self {
        ComputationTypeDisplay {
            effects,
            return_type: return_type.into(),
        }
    }

    /// Create a pure computation (no effects).
    pub fn pure(return_type: impl Into<String>) -> Self {
        ComputationTypeDisplay {
            effects: EffectRowDisplay::empty(),
            return_type: return_type.into(),
        }
    }
}

impl fmt::Display for ComputationTypeDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Computation<{{ {} }}, {}>",
            self.effects, self.return_type
        )
    }
}

// =============================================================================
// Type Formatting Utilities
// =============================================================================

/// Format a type name for display.
pub fn format_type_name(type_name: &str) -> String {
    // Remove common prefixes and make more readable
    let cleaned = type_name
        .replace("ordofp_core::", "")
        .replace("ordofp::", "")
        .replace("effects::", "")
        .replace("::Effectus", "");

    // Handle common Latin->English translations for display
    // Note: More specific patterns must come before less specific ones
    let translations = [
        ("Computatio", "Computation"),
        ("StatusEffectus", "State"),
        ("ErrorEffectus", "Error"),
        ("ReaderEffectus", "Reader"),
        ("ScriptorEffectus", "Writer"),
        ("IoEffectus", "IO"),
        ("AsyncEffectus", "Async"),
        ("Effectus", "Effect"), // Universalis must come last
        ("RowVacuus", "∅"),
        ("RowExtensio", "+"),
    ];

    let mut result = cleaned;
    for (latin, english) in translations {
        result = result.replace(latin, english);
    }

    result
}

/// Format an effect row type for display.
pub fn format_effect_row(row_type: &str) -> String {
    // Parse and format effect row syntax
    // RowExtensio<State<S>, RowExtensio<IO, RowVacuus>> -> State<S> | IO

    let cleaned = format_type_name(row_type);

    // Simple parsing for common patterns
    if cleaned.contains('+') {
        // Already simplified
        cleaned
            .replace('+', "|")
            .replace("∅", "")
            .trim_end_matches(" |")
            .to_string()
    } else {
        cleaned
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effect_info_display() {
        let io = EffectInfo::io();
        assert_eq!(io.to_string(), "IO");

        let state = EffectInfo::state("Config");
        assert_eq!(state.to_string(), "State<Config>");

        let error = EffectInfo::error("AppError");
        assert_eq!(error.to_string(), "Error<AppError>");
    }

    #[test]
    fn test_effect_row_display_empty() {
        let row = EffectRowDisplay::empty();
        assert_eq!(row.to_string(), "Pure");
    }

    #[test]
    fn test_effect_row_display_single() {
        let row = EffectRowDisplay::closed(alloc::vec![EffectInfo::io()]);
        assert_eq!(row.to_string(), "IO");
    }

    #[test]
    fn test_effect_row_display_multiple() {
        let row = EffectRowDisplay::closed(alloc::vec![
            EffectInfo::io(),
            EffectInfo::state("Config"),
            EffectInfo::error("AppError"),
        ]);
        assert_eq!(row.to_string(), "IO | State<Config> | Error<AppError>");
    }

    #[test]
    fn test_effect_row_display_open() {
        let row = EffectRowDisplay::open(alloc::vec![EffectInfo::io()], "R");
        assert_eq!(row.to_string(), "IO | R");
    }

    #[test]
    fn test_computation_type_display() {
        let comp = ComputationTypeDisplay::new(
            EffectRowDisplay::closed(alloc::vec![EffectInfo::io()]),
            "Response",
        );
        assert_eq!(comp.to_string(), "Computation<{ IO }, Response>");
    }

    #[test]
    fn test_format_type_name() {
        assert_eq!(
            format_type_name("ordofp_core::effects::Computatio"),
            "Computation"
        );
        assert_eq!(format_type_name("StatusEffectus"), "State");
        assert_eq!(format_type_name("RowVacuus"), "∅");
    }
}
