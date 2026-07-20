//! Variatio - Extensible Variant Type
//!
//! > *"Variatio delectat."*
//! > — Variety delights. (Cicero)
//!
//! This module provides `Variatio` - an extensible variant (sum type) that
//! supports row polymorphism, the dual of extensible records.

extern crate alloc;

use alloc::boxed::Box;
use core::any::TypeId;
use core::fmt;
use core::marker::PhantomData;

use crate::hlist::{Coniunctio, HList, Nihil};

// =============================================================================
// Variatio - Extensible Variant
// =============================================================================

/// An extensible variant type with row polymorphism support.
///
/// `Variatio` is the dual of `Registrum` - while a record contains
/// all fields simultaneously, a variant contains exactly one case.
///
/// # Latin Etymology
///
/// *Variatio* = variation, variety
///
/// # Type Parameters
///
/// * `R` - The row type listing all possible cases
///
/// # Example
///
/// ```rust
/// use ordofp_core::hlist::{Coniunctio, Nihil};
/// use ordofp_core::labelled::chars::*;
/// use ordofp_core::rows::{Casus, Variatio, inject};
///
/// type Success = (Ls, Lu, Lc, Lc, Le, Ls, Ls);
/// type Error = (Le, Lr, Lr, Lo, Lr);
///
/// type ResultRow = Coniunctio<Casus<Success, i32>, Coniunctio<Casus<Error, String>, Nihil>>;
///
/// // Inject a value into the variant
/// let result: Variatio<ResultRow> = inject::<Success, _, _, _>(42);
///
/// // Match on the variant
/// let message = result
///     .on::<Success, i32, _, _, _>(|n| format!("Success: {}", n))
///     .on::<Error, String, _, _>(|e| format!("Error: {}", e))
///     .exhaust();
///
/// assert_eq!(message, "Success: 42");
/// ```
pub struct Variatio<R> {
    /// The type tag identifying which case is active
    tag: TypeId,
    /// The actual value, type-erased
    value: Box<dyn core::any::Any + Send + Sync>,
    /// Phantom data for the row type
    _row: PhantomData<R>,
}

// =============================================================================
// Case Membership - Marker Trait
// =============================================================================

/// Marker trait indicating a row contains a specific case.
///
/// This uses type-level indices to avoid overlapping impls.
///
/// # Latin Etymology
///
/// *Habet* = has
/// *Casum* = case
pub trait HabetCasum<Label, Value, Index>: Sized {}

/// Marker for head position.
pub struct CasusHic;

/// Marker for tail position.
pub struct CasusIbi<T>(PhantomData<T>);

/// Implementation for when the case is at the head.
impl<Label, Value, Tail: HList> HabetCasum<Label, Value, CasusHic>
    for Coniunctio<(Label, Value), Tail>
{
}

/// Implementation for when the case is in the tail.
impl<Label, Value, Head, Tail, TailIndex> HabetCasum<Label, Value, CasusIbi<TailIndex>>
    for Coniunctio<Head, Tail>
where
    Tail: HabetCasum<Label, Value, TailIndex>,
{
}

// =============================================================================
// Injection
// =============================================================================

/// Inject a value into a variant at a specific case.
///
/// # Type Parameters
///
/// * `Label` - The case label
/// * `R` - The row type
/// * `Index` - Index for case lookup (inferred)
///
/// # Example
///
/// ```rust
/// use ordofp_core::hlist::{Coniunctio, Nihil};
/// use ordofp_core::labelled::chars::*;
/// use ordofp_core::rows::{Casus, Variatio, inject};
///
/// type Success = (Ls, Lu, Lc, Lc, Le, Ls, Ls);
/// type Error = (Le, Lr, Lr, Lo, Lr);
/// type Cases = Coniunctio<Casus<Success, &'static str>, Coniunctio<Casus<Error, &'static str>, Nihil>>;
///
/// let success: Variatio<Cases> = inject::<Success, _, _, _>("value");
/// assert!(success.is::<Success>());
/// ```
#[inline]
pub fn inject<Label: 'static, Value: Send + Sync + 'static, R, Index>(value: Value) -> Variatio<R>
where
    R: HabetCasum<Label, Value, Index>,
{
    Variatio {
        tag: TypeId::of::<Label>(),
        value: Box::new(value),
        _row: PhantomData,
    }
}

// =============================================================================
// Matching
// =============================================================================

impl<R> Variatio<R> {
    /// Get the type tag of the current case.
    #[inline]
    pub fn tag(&self) -> TypeId {
        self.tag
    }

    /// Check if this variant holds a specific case.
    #[inline]
    pub fn is<Label: 'static>(&self) -> bool {
        self.tag == TypeId::of::<Label>()
    }

    /// Try to extract the value if it matches the given case.
    ///
    /// Returns `Some(value)` if the variant holds this case, `None` otherwise.
    #[inline]
    pub fn try_get<Label: 'static, Value: 'static>(&self) -> Option<&Value> {
        if self.is::<Label>() {
            self.value.downcast_ref()
        } else {
            None
        }
    }

    /// Start a pattern match on this variant.
    ///
    /// Returns a `MatchBuilder` that can be used to handle each case.
    #[inline]
    pub fn match_on(self) -> MatchBuilder<R, Nihil> {
        MatchBuilder {
            variant: self,
            _handled: PhantomData,
        }
    }

    /// Handle a single case and continue matching.
    ///
    /// This is a convenience method combining `match_on` with a single case.
    ///
    /// The `Value` type given here must match the row's declared type for
    /// `Label` — the same `HabetCasum<Label, Value, _>` row-membership bound
    /// that `inject` carries. Matching a label with the wrong `Value` type
    /// is a compile error, not a runtime `downcast` panic:
    ///
    /// ```compile_fail,E0277
    /// use ordofp_core::hlist::{Coniunctio, Nihil};
    /// use ordofp_core::labelled::chars::*;
    /// use ordofp_core::rows::{Casus, Variatio, inject};
    ///
    /// type Success = (Ls, Lu, Lc, Lc, Le, Ls, Ls);
    /// type Error = (Le, Lr, Lr, Lo, Lr);
    ///
    /// type ResultRow = Coniunctio<Casus<Success, i32>, Coniunctio<Casus<Error, String>, Nihil>>;
    ///
    /// let v: Variatio<ResultRow> = inject::<Success, _, _, _>(42i32);
    ///
    /// // BUG: `Success` is declared with value type `i32` in `ResultRow`, but
    /// // this asks to match it as a `String`. This must not compile.
    /// let _ = v.on::<Success, String, _, _, _>(|s: String| s.len());
    /// ```
    ///
    /// # Panics
    ///
    /// Panics only if the stored value fails to downcast to `Value` after
    /// the tag matched — the `HabetCasum` bound ties each label to exactly
    /// one value type, so such a panic indicates a bug in this crate.
    #[inline]
    pub fn on<Label, Value, F, T, Index>(self, f: F) -> CaseResult<R, T>
    where
        Label: 'static,
        Value: 'static,
        F: FnOnce(Value) -> T,
        R: HabetCasum<Label, Value, Index>,
    {
        if self.is::<Label>() {
            let value = *self
                .value
                .downcast::<Value>()
                .expect("unreachable: HabetCasum bound guarantees the value type for this label");
            CaseResult::Matched(f(value))
        } else {
            CaseResult::Unmatched(Variatio {
                tag: self.tag,
                value: self.value,
                _row: PhantomData,
            })
        }
    }
}

// =============================================================================
// Match Builder
// =============================================================================

/// A builder for pattern matching on variants.
///
/// **No compile-time exhaustiveness:** despite the `Handled` type parameter,
/// nothing tracks which cases have been handled — the phantom is never
/// advanced, so missing cases are *not* a compile error. Exhaustiveness is
/// enforced only at **runtime**: finish a chain with
/// [`CaseResult::otherwise`]/[`CaseResult::otherwise_with`] for a fallback,
/// or [`CaseResult::exhaust`], which panics on an unmatched variant.
pub struct MatchBuilder<R, Handled> {
    variant: Variatio<R>,
    _handled: PhantomData<Handled>,
}

impl<R, Handled> MatchBuilder<R, Handled> {
    /// Handle a case in the pattern match.
    #[inline]
    pub fn case<Label, Value, F, T, Index>(self, f: F) -> CaseResult<R, T>
    where
        Label: 'static,
        Value: 'static,
        F: FnOnce(Value) -> T,
        R: HabetCasum<Label, Value, Index>,
    {
        self.variant.on::<Label, Value, F, T, Index>(f)
    }
}

// =============================================================================
// Case Result
// =============================================================================

/// The result of matching a single case.
pub enum CaseResult<R, T> {
    /// The case matched and produced a result.
    Matched(T),
    /// The case didn't match; the variant still needs handling.
    Unmatched(Variatio<R>),
}

impl<R, T> CaseResult<R, T> {
    /// Handle another case.
    #[inline]
    pub fn on<Label, Value, F, Index>(self, f: F) -> CaseResult<R, T>
    where
        Label: 'static,
        Value: 'static,
        F: FnOnce(Value) -> T,
        R: HabetCasum<Label, Value, Index>,
    {
        match self {
            CaseResult::Matched(t) => CaseResult::Matched(t),
            CaseResult::Unmatched(v) => v.on::<Label, Value, F, T, Index>(f),
        }
    }

    /// Provide a default value for any unmatched cases.
    #[inline]
    pub fn otherwise(self, default: T) -> T {
        match self {
            CaseResult::Matched(t) => t,
            CaseResult::Unmatched(_) => default,
        }
    }

    /// Provide a default value computed from a function.
    #[inline]
    pub fn otherwise_with<F: FnOnce() -> T>(self, f: F) -> T {
        match self {
            CaseResult::Matched(t) => t,
            CaseResult::Unmatched(_) => f(),
        }
    }

    /// Assert that all cases have been handled.
    ///
    /// Panics if the variant is unmatched.
    #[inline]
    pub fn exhaust(self) -> T {
        match self {
            CaseResult::Matched(t) => t,
            CaseResult::Unmatched(_) => crate::cold_panic!("non-exhaustive match on Variatio"),
        }
    }
}

// =============================================================================
// Extend Variant Row
// =============================================================================

/// Trait for extending a variant's possible cases.
///
/// # Latin Etymology
///
/// *Extendo Casum* = extend case
pub trait ExtendoCasum<Label, Value>: Sized {
    /// The extended row type.
    type Output;

    /// Widen this variant to accept additional cases.
    fn widen(self) -> Variatio<Self::Output>;
}

impl<R, Label, Value> ExtendoCasum<Label, Value> for Variatio<R> {
    type Output = Coniunctio<(Label, Value), R>;

    #[inline]
    fn widen(self) -> Variatio<Self::Output> {
        Variatio {
            tag: self.tag,
            value: self.value,
            _row: PhantomData,
        }
    }
}

// =============================================================================
// Debug Implementation
// =============================================================================

impl<R> fmt::Debug for Variatio<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Variatio {{ tag: {:?}, ... }}", self.tag)
    }
}

// =============================================================================
// Helper Types for Case Tuples
// =============================================================================

/// A case in a variant row.
///
/// This is a type-level pair of (Label, `ValueType`).
pub type Casus<Label, Value> = (Label, Value);

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::labelled::chars::*;
    use alloc::string::String;

    type Success = (Ls, Lu, Lc, Lc, Le, Ls, Ls);
    type Error = (Le, Lr, Lr, Lo, Lr);

    type ResultRow = Coniunctio<Casus<Success, i32>, Coniunctio<Casus<Error, String>, Nihil>>;

    #[test]
    fn test_inject() {
        let v: Variatio<ResultRow> = inject::<Success, _, _, _>(42);
        assert!(v.is::<Success>());
        assert!(!v.is::<Error>());
    }

    #[test]
    fn test_try_get() {
        let v: Variatio<ResultRow> = inject::<Success, _, _, _>(42i32);
        assert_eq!(v.try_get::<Success, i32>(), Some(&42));
        assert_eq!(v.try_get::<Error, String>(), None);
    }

    #[test]
    fn test_on_matched() {
        let v: Variatio<ResultRow> = inject::<Success, _, _, _>(42i32);
        let result = v.on::<Success, i32, _, _, _>(|n| n * 2).otherwise(0);
        assert_eq!(result, 84);
    }

    #[test]
    fn test_on_unmatched() {
        let v: Variatio<ResultRow> = inject::<Error, _, _, _>(String::from("oops"));
        let result = v.on::<Success, i32, _, _, _>(|n| n * 2).otherwise(-1);
        assert_eq!(result, -1);
    }

    #[test]
    fn test_multiple_cases() {
        let v1: Variatio<ResultRow> = inject::<Success, _, _, _>(42i32);
        let v2: Variatio<ResultRow> = inject::<Error, _, _, _>(String::from("oops"));

        let r1 = v1
            .on::<Success, i32, _, _, _>(|n| alloc::format!("ok: {n}"))
            .on::<Error, String, _, _>(|e| alloc::format!("err: {e}"))
            .exhaust();

        let r2 = v2
            .on::<Success, i32, _, _, _>(|n| alloc::format!("ok: {n}"))
            .on::<Error, String, _, _>(|e| alloc::format!("err: {e}"))
            .exhaust();

        assert_eq!(r1, "ok: 42");
        assert_eq!(r2, "err: oops");
    }

    #[test]
    fn test_widen() {
        type NewError = (Ln, Le, Lw, DoubleUnderscore, Le, Lr, Lr);

        let v: Variatio<ResultRow> = inject::<Success, _, _, _>(42i32);
        let widened: Variatio<Coniunctio<Casus<NewError, bool>, ResultRow>> = v.widen();

        assert!(widened.is::<Success>());
    }

    #[test]
    fn test_otherwise_with() {
        let v: Variatio<ResultRow> = inject::<Error, _, _, _>(String::from("oops"));
        let result = v.on::<Success, i32, _, _, _>(|n| n).otherwise_with(|| 999);
        assert_eq!(result, 999);
    }
}
