//! Specialization Markers
//!
//! Type-level markers to guide monomorphization and specialization.

use core::marker::PhantomData;

// =============================================================================
// Specialization Tags
// =============================================================================

/// Marker trait for types that should be specialized.
///
/// When a type implements this trait, Universalis code using it
/// may be more aggressively specialized by the compiler.
pub trait Specialize: Sized {
    /// A unique tag identifying this specialization.
    type Tag;
}

/// Default specialization - no special handling.
pub struct DefaultSpec;

/// Inline specialization - functions should be inlined.
pub struct InlineSpec;

/// No-inline specialization - functions should not be inlined.
pub struct NoInlineSpec;

/// Small size specialization - optimized for small types.
pub struct SmallSpec;

/// Large size specialization - optimized for large types.
pub struct LargeSpec;

// =============================================================================
// Monomorphization Markers
// =============================================================================

/// A marker that forces monomorphization of a Universalis type.
///
/// By including this marker in a type, you ensure that distinct
/// type parameters produce distinct monomorphizations.
///
/// # Example
///
/// ```rust
/// use ordofp_core::specialization::Mono;
/// use core::marker::PhantomData;
///
/// struct Handler<E, M = Mono<E>> {
///     _error: PhantomData<E>,
///     _marker: M,
/// }
///
/// // Handler<ErrorA> and Handler<ErrorB> are guaranteed
/// // to be different monomorphizations.
/// struct ErrorA;
/// struct ErrorB;
///
/// let _a: Handler<ErrorA> = Handler { _error: PhantomData, _marker: Mono::new() };
/// let _b: Handler<ErrorB> = Handler { _error: PhantomData, _marker: Mono::new() };
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Mono<T>(PhantomData<fn() -> T>);

impl<T> Mono<T> {
    /// Create a new monomorphization marker.
    pub const fn new() -> Self {
        Mono(PhantomData)
    }
}

impl<T> Default for Mono<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// A marker that prevents monomorphization sharing.
///
/// Even if two types have the same representation, this marker
/// ensures they get separate code generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Distinct<T, const ID: u64 = 0>(PhantomData<T>);

impl<T, const ID: u64> Distinct<T, ID> {
    /// Create a new distinct marker.
    pub const fn new() -> Self {
        Distinct(PhantomData)
    }
}

impl<T, const ID: u64> Default for Distinct<T, ID> {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Effect Specialization Markers
// =============================================================================

/// Marker for pure computations (no effects).
///
/// Functions marked with this can be more aggressively optimized
/// since they have no side effects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pure;

/// Marker for stateful computations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stateful<S>(PhantomData<S>);

impl<S> Stateful<S> {
    /// Construct a new `Stateful` marker.
    ///
    /// This is a zero-cost constructor; the returned value is a
    /// `PhantomData` wrapper with no runtime representation.
    pub const fn new() -> Self {
        Stateful(PhantomData)
    }
}

impl<S> Default for Stateful<S> {
    fn default() -> Self {
        Self::new()
    }
}

/// Marker for effectful computations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Effectful<R>(PhantomData<R>);

impl<R> Effectful<R> {
    /// Construct a new `Effectful` marker.
    ///
    /// This is a zero-cost constructor; the returned value is a
    /// `PhantomData` wrapper with no runtime representation.
    pub const fn new() -> Self {
        Effectful(PhantomData)
    }
}

impl<R> Default for Effectful<R> {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Fusion Markers
// =============================================================================

/// Marker indicating a type supports fusion.
///
/// Fusion combines multiple operations into a single pass,
/// eliminating intermediate allocations.
pub trait Fusible {
    /// The fused representation.
    type Fused;

    /// Whether fusion is beneficial for this type.
    const FUSION_BENEFICIAL: bool = true;
}

/// Marker for operations that should be fused.
#[derive(Debug, Clone, Copy)]
pub struct FuseWith<T>(PhantomData<T>);

impl<T> FuseWith<T> {
    /// Construct a new `FuseWith` marker.
    ///
    /// This is a zero-cost constructor; the returned value is a
    /// `PhantomData` wrapper with no runtime representation.
    pub const fn new() -> Self {
        FuseWith(PhantomData)
    }
}

impl<T> Default for FuseWith<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Marker to prevent fusion.
#[derive(Debug, Clone, Copy)]
pub struct NoFuse;

// =============================================================================
// Inlining Markers
// =============================================================================

/// Wrapper that suggests the contained function should be inlined.
pub struct InlineHint<F>(pub F);

impl<F> InlineHint<F> {
    /// Create a new inline hint wrapper.
    #[inline(always)]
    pub fn new(f: F) -> Self {
        InlineHint(f)
    }

    /// Call the wrapped function.
    #[inline(always)]
    pub fn call<A, R>(self, arg: A) -> R
    where
        F: FnOnce(A) -> R,
    {
        (self.0)(arg)
    }
}

/// Wrapper that suggests the contained function should NOT be inlined.
pub struct NoInlineHint<F>(pub F);

impl<F> NoInlineHint<F> {
    /// Create a new no-inline hint wrapper.
    #[inline(never)]
    pub fn new(f: F) -> Self {
        NoInlineHint(f)
    }

    /// Call the wrapped function.
    #[inline(never)]
    pub fn call<A, R>(self, arg: A) -> R
    where
        F: FnOnce(A) -> R,
    {
        (self.0)(arg)
    }
}

// =============================================================================
// Size Optimization Markers
// =============================================================================

/// Marker for small, trivially-copyable types.
///
/// Types marked as small may be passed by value rather than reference.
pub trait SmallType: Sized + Copy {
    /// Maximum size in bytes for a "small" type.
    const MAX_SIZE: usize = 16;

    /// Whether this type is actually small.
    #[inline]
    fn is_small() -> bool {
        core::mem::size_of::<Self>() <= Self::MAX_SIZE
    }
}

// Implement for primitive types
impl SmallType for u8 {}
impl SmallType for u16 {}
impl SmallType for u32 {}
impl SmallType for u64 {}
impl SmallType for u128 {}
impl SmallType for usize {}
impl SmallType for i8 {}
impl SmallType for i16 {}
impl SmallType for i32 {}
impl SmallType for i64 {}
impl SmallType for i128 {}
impl SmallType for isize {}
impl SmallType for f32 {}
impl SmallType for f64 {}
impl SmallType for bool {}
impl SmallType for char {}
impl<T> SmallType for *const T {}
impl<T> SmallType for *mut T {}

/// Marker for zero-sized types.
pub trait ZeroSized: Sized {
    /// Whether this type is zero-sized.
    #[inline]
    fn is_zst() -> bool {
        core::mem::size_of::<Self>() == 0
    }
}

impl ZeroSized for () {}
impl<T> ZeroSized for PhantomData<T> {}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "alloc")]
    use alloc::string::String;

    #[test]
    #[cfg(feature = "alloc")]
    fn test_mono_marker() {
        let _m1: Mono<i32> = Mono::new();
        let _m2: Mono<String> = Mono::default();
    }

    #[test]
    #[cfg(not(feature = "alloc"))]
    fn test_mono_marker() {
        let _m1: Mono<i32> = Mono::new();
    }

    #[test]
    fn test_distinct_marker() {
        let _d1: Distinct<i32, 1> = Distinct::new();
        let _d2: Distinct<i32, 2> = Distinct::new();
        // d1 and d2 are different types even though both wrap i32
    }

    #[test]
    fn test_effect_markers() {
        let _ = Pure;
        let _: Stateful<i32> = Stateful::new();
        let _: Effectful<()> = Effectful::new();
    }

    #[test]
    fn test_inline_hint() {
        let hint = InlineHint::new(|x: i32| x * 2);
        assert_eq!(hint.call(21), 42);
    }

    #[test]
    fn test_no_inline_hint() {
        let hint = NoInlineHint::new(|x: i32| x + 1);
        assert_eq!(hint.call(41), 42);
    }

    #[test]
    fn test_small_type() {
        assert!(u32::is_small());
        assert!(u64::is_small());
        assert!(f64::is_small());
    }

    #[test]
    fn test_zero_sized() {
        assert!(<()>::is_zst());
        assert!(<PhantomData<i32>>::is_zst());
    }
}
