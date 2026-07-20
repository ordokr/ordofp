//! Type Hints for IDE Integration
//!
//! Provides inlay hints and type annotations for IDE features like
//! rust-analyzer integration.

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

// =============================================================================
// Inlay Hints
// =============================================================================

/// Types of inlay hints.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InlayHintKind {
    /// Type annotation hint.
    TypeAnnotation,
    /// Parameter name hint.
    ParameterName,
    /// Effect row hint.
    EffectRow,
    /// Chaining hint (for method chains).
    Chaining,
}

/// An inlay hint to display in the IDE.
#[derive(Debug, Clone)]
pub struct InlayHint {
    /// The kind of hint.
    pub kind: InlayHintKind,
    /// The position in the source (byte offset).
    pub position: usize,
    /// The label to display.
    pub label: String,
    /// Whether a space should be rendered before the label.
    pub padding_left: bool,
    /// Whether a space should be rendered after the label.
    pub padding_right: bool,
}

impl InlayHint {
    /// Create a new type annotation hint.
    pub fn type_annotation(position: usize, type_label: impl Into<String>) -> Self {
        InlayHint {
            kind: InlayHintKind::TypeAnnotation,
            position,
            label: format!(": {}", type_label.into()),
            padding_left: false,
            padding_right: false,
        }
    }

    /// Create a new parameter hint.
    pub fn parameter(position: usize, param_name: impl Into<String>) -> Self {
        InlayHint {
            kind: InlayHintKind::ParameterName,
            position,
            label: format!("{}: ", param_name.into()),
            padding_left: false,
            padding_right: false,
        }
    }

    /// Create an effect row hint.
    pub fn effect_row(position: usize, effects: impl Into<String>) -> Self {
        InlayHint {
            kind: InlayHintKind::EffectRow,
            position,
            label: format!("/* {} */", effects.into()),
            padding_left: true,
            padding_right: true,
        }
    }

    /// Create a chaining hint.
    pub fn chaining(position: usize, return_type: impl Into<String>) -> Self {
        InlayHint {
            kind: InlayHintKind::Chaining,
            position,
            label: return_type.into(),
            padding_left: true,
            padding_right: false,
        }
    }

    /// Set padding.
    pub fn with_padding(mut self, left: bool, right: bool) -> Self {
        self.padding_left = left;
        self.padding_right = right;
        self
    }
}

// =============================================================================
// Effect Inference Display
// =============================================================================

/// Represents inferred effects for display.
#[derive(Debug, Clone)]
pub struct InferredEffects {
    /// The effects inferred for a computation.
    pub effects: Vec<String>,
    /// Whether the inference is complete.
    pub is_complete: bool,
    /// Any effects that couldn't be inferred.
    pub unknown: Vec<String>,
}

impl InferredEffects {
    /// Create a new inferred effects display.
    pub fn new() -> Self {
        InferredEffects {
            effects: Vec::new(),
            is_complete: true,
            unknown: Vec::new(),
        }
    }

    /// Add an inferred effect.
    pub fn with_effect(mut self, effect: impl Into<String>) -> Self {
        self.effects.push(effect.into());
        self
    }

    /// Mark as incomplete.
    pub fn incomplete(mut self) -> Self {
        self.is_complete = false;
        self
    }

    /// Add an unknown effect.
    pub fn with_unknown(mut self, unknown: impl Into<String>) -> Self {
        self.unknown.push(unknown.into());
        self.is_complete = false;
        self
    }

    /// Format for display.
    pub fn display(&self) -> String {
        if self.effects.is_empty() && self.unknown.is_empty() {
            return "Pure".to_string();
        }

        let total = self.effects.len() + self.unknown.len();
        let mut parts: Vec<String> = Vec::with_capacity(total);
        parts.extend(self.effects.iter().cloned());
        for u in &self.unknown {
            parts.push(format!("?{u}"));
        }

        parts.join(" | ")
    }

    /// Format as an inlay hint.
    pub fn as_hint(&self, position: usize) -> InlayHint {
        InlayHint::effect_row(position, self.display())
    }
}

impl Default for InferredEffects {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Type Simplification
// =============================================================================

/// Simplify complex types for display.
pub struct TypeSimplifier {
    /// Maximum nesting depth before truncating.
    pub max_depth: usize,
    /// Whether to use vernacular names.
    pub use_vernacular: bool,
    /// Whether to abbreviate long type names.
    pub abbreviate: bool,
}

impl TypeSimplifier {
    /// Create a new simplifier with defaults.
    pub fn new() -> Self {
        TypeSimplifier {
            max_depth: 3,
            use_vernacular: true,
            abbreviate: true,
        }
    }

    /// Set maximum depth.
    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    /// Enable/disable vernacular names.
    pub fn with_vernacular(mut self, enabled: bool) -> Self {
        self.use_vernacular = enabled;
        self
    }

    /// Simplify a type string.
    pub fn simplify(&self, type_str: &str) -> String {
        let mut result = type_str.to_string();

        // Remove common module prefixes
        let prefixes = [
            "ordofp_core::",
            "ordofp::",
            "effects::",
            "core::option::",
            "core::result::",
            "alloc::string::",
            "alloc::vec::",
            "std::",
        ];

        for prefix in prefixes {
            result = result.replace(prefix, "");
        }

        // Apply vernacular translations if enabled
        // Note: More specific patterns must come before less specific ones
        if self.use_vernacular {
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
                ("Coniunctio", "::"),
                ("Nihil", "∅"),
                ("Disiunctio", "|"),
                ("Absurdum", "!"),
                ("Aut::Sinister", "Left"),
                ("Aut::Dexter", "Right"),
            ];

            for (latin, english) in translations {
                result = result.replace(latin, english);
            }
        }

        // Abbreviate if enabled
        if self.abbreviate {
            // Common abbreviations
            result = result.replace("String", "Str");
            result = result.replace("Vector", "Vec");
        }

        result
    }

    /// Simplify an effect row type.
    pub fn simplify_effect_row(&self, row_type: &str) -> String {
        let simplified = self.simplify(row_type);

        // Parse RowExtensio chains and format as pipe-separated
        // RowExtensio<E, RowExtensio<E2, RowVacuus>> -> E | E2

        if simplified.contains('+') || simplified.contains("RowExtensio") {
            // Already has row syntax or needs parsing
            simplified
                .replace("+<", " | ")
                .replace(", ∅>", "")
                .replace('<', "(")
                .replace('>', ")")
                .trim_end_matches(" | ∅")
                .to_string()
        } else {
            simplified
        }
    }
}

impl Default for TypeSimplifier {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Hover Information
// =============================================================================

/// Information to display on hover.
#[derive(Debug, Clone)]
pub struct HoverInfo {
    /// The type signature.
    pub signature: String,
    /// Documentation summary.
    pub doc_summary: Option<String>,
    /// Effect information.
    pub effects: Option<InferredEffects>,
    /// Additional details.
    pub details: Vec<String>,
}

impl HoverInfo {
    /// Create new hover info.
    pub fn new(signature: impl Into<String>) -> Self {
        HoverInfo {
            signature: signature.into(),
            doc_summary: None,
            effects: None,
            details: Vec::new(),
        }
    }

    /// Add documentation summary.
    pub fn with_doc(mut self, doc: impl Into<String>) -> Self {
        self.doc_summary = Some(doc.into());
        self
    }

    /// Add effect information.
    pub fn with_effects(mut self, effects: InferredEffects) -> Self {
        self.effects = Some(effects);
        self
    }

    /// Add a detail.
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.details.push(detail.into());
        self
    }

    /// Format for display.
    pub fn format(&self) -> String {
        let capacity = 1
            + usize::from(self.effects.is_some())
            + usize::from(self.doc_summary.is_some())
            + self.details.len();
        let mut lines = Vec::with_capacity(capacity);

        // Signature
        lines.push(format!("```rust\n{}\n```", self.signature));

        // Effects
        if let Some(ref effects) = self.effects {
            lines.push(format!("**Effects:** {}", effects.display()));
        }

        // Documentation
        if let Some(ref doc) = self.doc_summary {
            lines.push(format!("\n{doc}"));
        }

        // Details
        for detail in &self.details {
            lines.push(format!("- {detail}"));
        }

        lines.join("\n")
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inlay_hint_type() {
        let hint = InlayHint::type_annotation(10, "i32");
        assert_eq!(hint.label, ": i32");
        assert_eq!(hint.kind, InlayHintKind::TypeAnnotation);
    }

    #[test]
    fn test_inlay_hint_parameter() {
        let hint = InlayHint::parameter(5, "config");
        assert_eq!(hint.label, "config: ");
    }

    #[test]
    fn test_inlay_hint_effect() {
        let hint = InlayHint::effect_row(20, "IO | State<Config>");
        assert!(hint.label.contains("IO"));
        assert!(hint.label.contains("State<Config>"));
    }

    #[test]
    fn test_inferred_effects() {
        let effects = InferredEffects::new()
            .with_effect("IO")
            .with_effect("State<Config>");

        assert_eq!(effects.display(), "IO | State<Config>");
    }

    #[test]
    fn test_inferred_effects_pure() {
        let effects = InferredEffects::new();
        assert_eq!(effects.display(), "Pure");
    }

    #[test]
    fn test_type_simplifier() {
        let simplifier = TypeSimplifier::new();

        assert_eq!(
            simplifier.simplify("ordofp_core::effects::Computatio"),
            "Computation"
        );
        assert_eq!(simplifier.simplify("StatusEffectus"), "State");
    }

    #[test]
    fn test_hover_info() {
        let info = HoverInfo::new("fn fetch() -> Eff<IO | Error<AppError>, Response>")
            .with_effects(
                InferredEffects::new()
                    .with_effect("IO")
                    .with_effect("Error<AppError>"),
            )
            .with_doc("Fetches data from the server.");

        let formatted = info.format();
        assert!(formatted.contains("fetch()"));
        assert!(formatted.contains("IO"));
        assert!(formatted.contains("Fetches data"));
    }
}
