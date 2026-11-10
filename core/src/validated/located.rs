//! Typed, owned locations for validation diagnostics.
//!
//! An error path is ordered from outermost to innermost. Named fields and
//! indexed elements remain distinct in memory and on the wire. The payload
//! stays generic and does not need to implement `Clone` or `Display`.

use alloc::string::String;
use alloc::vec::Vec;

/// One segment of an owned validation-error path.
///
/// `Field` and `Index` are distinct in serialized data, so the string
/// "4" is never confused with collection index 4.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SegmentumViae {
    /// A named field, retaining its exact name.
    Field(String),
    /// A zero-based collection index.
    Index(usize),
}

/// English alias for `SegmentumViae`.
pub use SegmentumViae as PathSegment;

/// An application-defined error associated with an owned structural path.
///
/// An empty path denotes the root. Locations do not change how the payload
/// is interpreted. Prefix operations consume `self` without cloning `E`.
///
/// # Example
///
/// ```
/// use ordofp_core::validated::{Located, PathSegment};
///
/// let issue = Located::field("amount", "not positive")
///     .at_index(4)
///     .at_field("rows");
/// assert_eq!(
///     issue.path(),
///     &[
///         PathSegment::Field("rows".into()),
///         PathSegment::Index(4),
///         PathSegment::Field("amount".into()),
///     ]
/// );
/// assert_eq!(issue.error(), &"not positive");
/// ```
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Locatus<E> {
    path: Vec<SegmentumViae>,
    error: E,
}

/// English alias for `Locatus`.
pub use Locatus as Located;

impl<E> Locatus<E> {
    /// Attach a diagnostic to the root, with no path segments.
    #[inline]
    #[must_use]
    pub const fn root(error: E) -> Self {
        Self {
            path: Vec::new(),
            error,
        }
    }

    /// Attach a diagnostic to one named field.
    #[inline]
    #[must_use]
    pub fn field(name: impl Into<String>, error: E) -> Self {
        Self {
            path: alloc::vec![SegmentumViae::Field(name.into())],
            error,
        }
    }

    /// Attach a diagnostic to one collection index.
    #[inline]
    #[must_use]
    pub fn index(index: usize, error: E) -> Self {
        Self {
            path: alloc::vec![SegmentumViae::Index(index)],
            error,
        }
    }

    /// Prepend an outer named field; the payload is moved, never cloned.
    #[inline]
    #[must_use]
    pub fn at_field(mut self, name: impl Into<String>) -> Self {
        self.path.insert(0, SegmentumViae::Field(name.into()));
        self
    }

    /// Prepend an outer collection index; the payload is moved, never cloned.
    #[inline]
    #[must_use]
    pub fn at_index(mut self, index: usize) -> Self {
        self.path.insert(0, SegmentumViae::Index(index));
        self
    }

    /// Borrow the structural path, from outermost to innermost.
    #[inline]
    #[must_use]
    pub fn path(&self) -> &[SegmentumViae] {
        &self.path
    }

    /// Borrow the original application error without requiring Display.
    #[inline]
    #[must_use]
    pub const fn error(&self) -> &E {
        &self.error
    }

    /// Consume the diagnostic, preserving both its path and payload.
    #[inline]
    #[must_use]
    pub fn into_parts(self) -> (Vec<SegmentumViae>, E) {
        (self.path, self.error)
    }

    /// Transform the payload once, preserving the location without cloning it.
    #[inline]
    #[must_use]
    pub fn map_error<F, E2>(self, f: F) -> Locatus<E2>
    where
        F: FnOnce(E) -> E2,
    {
        Locatus {
            path: self.path,
            error: f(self.error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validated::Probatum;
    #[cfg(feature = "serde")]
    use alloc::string::ToString;

    #[derive(Debug, PartialEq, Eq)]
    struct NonClone(u32);

    #[test]
    fn field_index_prefixes_order_outside_in_without_cloning() {
        let issue = Located::field("amount", NonClone(9))
            .at_index(4)
            .at_field("rows");
        assert_eq!(
            issue.path(),
            &[
                PathSegment::Field("rows".into()),
                PathSegment::Index(4),
                PathSegment::Field("amount".into()),
            ]
        );
        let (path, error) = issue.into_parts();
        assert_eq!(path.len(), 3);
        assert_eq!(error, NonClone(9));
    }

    #[test]
    fn root_index_and_map_preserve_typed_error() {
        let root = Located::root(NonClone(2));
        assert!(root.path().is_empty());
        let indexed = Located::index(3, NonClone(7))
            .at_field("items")
            .map_error(|issue| issue.0 + 1);
        assert_eq!(indexed.error(), &8);
        assert_eq!(
            indexed.path(),
            &[PathSegment::Field("items".into()), PathSegment::Index(3)]
        );
    }

    #[test]
    fn validation_accumulation_preserves_locations_and_error_order() {
        let start: Probatum<Located<NonClone>, ()> =
            Probatum::invalid(Located::field("start", NonClone(1)));
        let end: Probatum<Located<NonClone>, ()> =
            Probatum::invalid(Located::field("end", NonClone(2)));
        let combined = start.map2(end, |(), ()| ());
        let errors = combined.errors().expect("two failures");
        assert_eq!(errors.len(), 2);
        assert_eq!(errors[0].path(), &[PathSegment::Field("start".into())]);
        assert_eq!(errors[1].path(), &[PathSegment::Field("end".into())]);
        assert_eq!(errors[0].error(), &NonClone(1));
        assert_eq!(errors[1].error(), &NonClone(2));
    }

    #[test]
    fn map_err_prefixes_each_issue_without_changing_payload() {
        let errors: Probatum<Located<NonClone>, ()> = Probatum::invalid_many([
            Located::field("id", NonClone(1)),
            Located::field("age", NonClone(2)),
        ])
        .expect("two input errors");
        let prefixed = errors.map_err(|issue| issue.at_index(2).at_field("users"));
        let issues = prefixed.errors().expect("invalid");
        assert_eq!(
            issues[0].path(),
            &[
                PathSegment::Field("users".into()),
                PathSegment::Index(2),
                PathSegment::Field("id".into()),
            ]
        );
        assert_eq!(issues[1].error(), &NonClone(2));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_roundtrips_root_and_nested_locations() {
        let nested = Located::field("amount", "out of range".to_string())
            .at_index(4)
            .at_field("rows");
        let json = serde_json::to_string(&nested).expect("serialize");
        assert_eq!(
            json,
            r#"{"path":[{"Field":"rows"},{"Index":4},{"Field":"amount"}],"error":"out of range"}"#
        );
        let decoded: Located<String> = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded, nested);

        let root = Located::root("batch".to_string());
        let json = serde_json::to_string(&root).expect("serialize root");
        assert_eq!(json, r#"{"path":[],"error":"batch"}"#);
        let decoded: Located<String> = serde_json::from_str(&json).expect("deserialize root");
        assert_eq!(decoded, root);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_distinguishes_numeric_field_name_from_index() {
        let issue = Located::field("4", "message".to_string()).at_index(4);
        let serialized = serde_json::to_string(&issue).expect("serialize");
        assert!(serialized.contains(r#"{"Index":4}"#));
        assert!(serialized.contains(r#"{"Field":"4"}"#));
        let decoded: Located<String> = serde_json::from_str(&serialized).expect("deserialize");
        assert_eq!(decoded, issue);
    }
}
