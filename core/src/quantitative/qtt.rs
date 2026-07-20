//! Qtt - Quantitative Type Theory wrapper
//!
//! > *"Quantitas determinat usum"*
//! > — Quantity determines use. (Neo-Latin)
//!
//! This module provides the `Qtt` wrapper type that encodes multiplicity
//! at the type level, enabling compile-time tracking of value usage.

use core::fmt;
use core::marker::PhantomData;
use core::ops::Deref;

use super::multiplicitas::{Multiplicitas, Nihil, Omega, Semel, Usage};

// =============================================================================
// Qtt - The Core QTT Wrapper
// =============================================================================

/// A value with explicit multiplicity annotation.
///
/// `Qtt<A, M>` wraps a value of type `A` with a type-level multiplicity `M`.
/// The multiplicity constrains how the value can be used:
///
/// - `Qtt<A, Nihil>`: Compile-time-only intent — *meant* to be erased and
///   unused. Note this is advisory: `consume`/`consume_with` are available
///   for every multiplicity (each represents one use), so `Nihil` does not
///   actually prevent consumption; it only forbids duplication.
/// - `Qtt<A, Semel>`: Must be used exactly once
/// - `Qtt<A, Omega>`: Can be used any number of times
///
/// # Type Parameters
///
/// * `A` - The wrapped value type
/// * `M` - The multiplicity marker (`Nihil`, `Semel`, or `Omega`)
///
/// # Example
///
/// ```rust
/// use ordofp_core::quantitative::{Qtt, Semel, Omega};
///
/// // A linear value
/// let linear: Qtt<i32, Semel> = Qtt::new(42);
/// let result = linear.consume(); // Must consume exactly once
///
/// // An unrestricted value
/// let free: Qtt<i32, Omega> = Qtt::new(42);
/// let copy1 = free.dup(); // Can duplicate
/// let copy2 = free.dup();
/// ```
pub struct Qtt<A, M: Usage> {
    value: A,
    _multiplicity: PhantomData<M>,
}

impl<A, M: Usage> Qtt<A, M> {
    /// Create a new quantitative value with the given multiplicity.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::quantitative::{Qtt, Semel};
    ///
    /// let linear: Qtt<i32, Semel> = Qtt::new(42);
    /// ```
    #[inline]
    pub const fn new(value: A) -> Self {
        Qtt {
            value,
            _multiplicity: PhantomData,
        }
    }

    /// Get the runtime multiplicity value.
    #[inline]
    pub const fn multiplicity(&self) -> Multiplicitas {
        M::VALUE
    }

    /// Check if this value can be discarded without use.
    #[inline]
    pub const fn can_discard(&self) -> bool {
        M::ALLOWS_DISCARD
    }

    /// Check if this value can be duplicated.
    #[inline]
    pub const fn can_dup(&self) -> bool {
        M::ALLOWS_DUP
    }

    /// Consume the value, returning the inner value.
    ///
    /// This is always available regardless of multiplicity,
    /// representing one use of the value.
    #[inline]
    pub fn consume(self) -> A {
        self.value
    }

    /// Map a function over the value, preserving multiplicity.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::quantitative::{Qtt, Semel};
    ///
    /// let x: Qtt<i32, Semel> = Qtt::new(5);
    /// let y: Qtt<i32, Semel> = x.fmap(|n| n * 2);
    /// assert_eq!(y.consume(), 10);
    /// ```
    #[inline]
    pub fn fmap<B, F>(self, f: F) -> Qtt<B, M>
    where
        F: FnOnce(A) -> B,
    {
        Qtt::new(f(self.value))
    }

    /// Apply a function to the value, consuming it.
    #[inline]
    pub fn consume_with<B, F>(self, f: F) -> B
    where
        F: FnOnce(A) -> B,
    {
        f(self.value)
    }

    /// Convert to an Option, always returning Some.
    #[inline]
    pub fn into_option(self) -> Option<A> {
        Some(self.value)
    }

    /// Convert to a Result, always returning Ok.
    ///
    /// # Errors
    ///
    /// Never returns `Err`; the error type parameter exists only to satisfy
    /// Result-based call sites. The usage obligation is discharged exactly
    /// once, as the quantity annotation requires.
    #[inline]
    pub fn into_result<E>(self) -> Result<A, E> {
        Ok(self.value)
    }
}

// =============================================================================
// Erased (Nihil) Operations
// =============================================================================

impl<A> Qtt<A, Nihil> {
    /// Create an erased value.
    ///
    /// Erased values exist only at compile time for type checking
    /// and are removed during compilation.
    #[inline]
    pub const fn erased(value: A) -> Self {
        Qtt::new(value)
    }

    /// Witness that erased values can be freely discarded.
    #[inline]
    pub fn discard(self) {
        // Value is simply dropped
    }

    /// Witness that erased values can be "duplicated".
    ///
    /// Since erased values don't exist at runtime, duplication
    /// is trivial.
    #[inline]
    pub fn phantom_dup(&self) -> Qtt<(), Nihil>
    where
        A: Copy,
    {
        Qtt::new(())
    }
}

// =============================================================================
// Linear (Semel) Operations
// =============================================================================

impl<A> Qtt<A, Semel> {
    /// Create a linear value.
    ///
    /// Linear values must be used exactly once.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::quantitative::{Qtt, Semel};
    ///
    /// let linear = Qtt::<i32, Semel>::linear(42);
    /// let result = linear.consume(); // Must use exactly once
    /// ```
    #[inline]
    pub const fn linear(value: A) -> Self {
        Qtt::new(value)
    }

    /// Split a linear value into two parts using a function.
    ///
    /// This is the linear equivalent of pattern matching on a pair.
    #[inline]
    pub fn split<B, C, F>(self, f: F) -> (Qtt<B, Semel>, Qtt<C, Semel>)
    where
        F: FnOnce(A) -> (B, C),
    {
        let (b, c) = f(self.value);
        (Qtt::new(b), Qtt::new(c))
    }

    /// Chain linear computations (monadic bind).
    #[inline]
    pub fn bind_linear<B, F>(self, f: F) -> Qtt<B, Semel>
    where
        F: FnOnce(A) -> Qtt<B, Semel>,
    {
        f(self.value)
    }

    /// Sequence two linear values, keeping the second.
    #[inline]
    pub fn then_linear<B>(self, other: Qtt<B, Semel>) -> Qtt<B, Semel> {
        let _ = self.value;
        other
    }

    /// Convert a linear value to unrestricted if the type allows cloning.
    #[inline]
    pub fn relax(self) -> Qtt<A, Omega>
    where
        A: Clone,
    {
        Qtt::new(self.value)
    }
}

// =============================================================================
// Unrestricted (Omega) Operations
// =============================================================================

impl<A> Qtt<A, Omega> {
    /// Create an unrestricted value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::quantitative::{Qtt, Omega};
    ///
    /// let free = Qtt::<i32, Omega>::unrestricted(42);
    /// let copy = free.dup();
    /// ```
    #[inline]
    pub const fn unrestricted(value: A) -> Self {
        Qtt::new(value)
    }

    /// Duplicate the value.
    ///
    /// Only available for unrestricted values.
    #[inline]
    pub fn dup(&self) -> Qtt<A, Omega>
    where
        A: Clone,
    {
        Qtt::new(self.value.clone())
    }

    /// Discard the value without using it.
    ///
    /// Only available for unrestricted values.
    #[inline]
    pub fn discard(self) {
        // Value is simply dropped
    }

    /// Get a reference to the inner value.
    #[inline]
    pub fn get_ref(&self) -> &A {
        &self.value
    }

    /// Restrict an unrestricted value to linear usage.
    ///
    /// This "forgets" that the value could be duplicated.
    #[inline]
    pub fn restrict(self) -> Qtt<A, Semel> {
        Qtt::new(self.value)
    }

    /// Erase an unrestricted value to zero multiplicity.
    #[inline]
    pub fn erase(self) -> Qtt<A, Nihil> {
        Qtt::new(self.value)
    }
}

// Clone only for Omega - use bitwise copy for Copy types, clone otherwise
// This implementation satisfies both Copy types (bitwise) and non-Copy Clone types,
// so it is deliberately broader than the Copy impl below.
#[allow(clippy::expl_impl_clone_on_copy)]
impl<A: Clone> Clone for Qtt<A, Omega> {
    #[inline]
    fn clone(&self) -> Self {
        Qtt::new(self.value.clone())
    }
}

// Copy only for Omega with Copy inner
impl<A: Copy> Copy for Qtt<A, Omega> {}

// =============================================================================
// Trait Implementations
// =============================================================================

impl<A: PartialEq, M: Usage> PartialEq for Qtt<A, M> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<A: Eq, M: Usage> Eq for Qtt<A, M> {}

impl<A: PartialOrd, M: Usage> PartialOrd for Qtt<A, M> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl<A: Ord, M: Usage> Ord for Qtt<A, M> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.value.cmp(&other.value)
    }
}

impl<A: core::hash::Hash, M: Usage> core::hash::Hash for Qtt<A, M> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl<A: Default, M: Usage> Default for Qtt<A, M> {
    #[inline]
    fn default() -> Self {
        Qtt::new(A::default())
    }
}

impl<A: fmt::Debug, M: Usage> fmt::Debug for Qtt<A, M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Qtt")
            .field("value", &self.value)
            .field("multiplicity", &M::VALUE)
            .finish()
    }
}

impl<A: fmt::Display, M: Usage> fmt::Display for Qtt<A, M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.value, M::VALUE)
    }
}

impl<A, M: Usage> From<A> for Qtt<A, M> {
    #[inline]
    fn from(value: A) -> Self {
        Qtt::new(value)
    }
}

// Deref only for Omega (unrestricted access)
impl<A> Deref for Qtt<A, Omega> {
    type Target = A;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

// =============================================================================
// Extension Trait
// =============================================================================

/// Extension trait for converting values into Qtt wrappers.
pub trait QttExt: Sized {
    /// Wrap as a linear value.
    #[inline]
    fn into_linear(self) -> Qtt<Self, Semel> {
        Qtt::linear(self)
    }

    /// Wrap as an unrestricted value.
    #[inline]
    fn into_unrestricted(self) -> Qtt<Self, Omega> {
        Qtt::unrestricted(self)
    }

    /// Wrap as an erased value.
    #[inline]
    fn into_erased(self) -> Qtt<Self, Nihil> {
        Qtt::erased(self)
    }
}

impl<T> QttExt for T {}
// =============================================================================

/// A linear value (multiplicity 1).
pub type QttLinearis<A> = Qtt<A, Semel>;

/// An erased value (multiplicity 0).
pub type QttErasum<A> = Qtt<A, Nihil>;

/// An unrestricted value (multiplicity ω).
pub type QttLiber<A> = Qtt<A, Omega>;

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::format;
    use alloc::string::ToString;

    #[test]
    fn test_linear_creation() {
        let x: Qtt<i32, Semel> = Qtt::linear(42);
        assert_eq!(x.multiplicity(), Multiplicitas::Semel);
        assert!(!x.can_discard());
        assert!(!x.can_dup());
    }

    #[test]
    fn test_linear_consume() {
        let x: Qtt<i32, Semel> = Qtt::linear(42);
        let result = x.consume();
        assert_eq!(result, 42);
    }

    #[test]
    fn test_linear_fmap() {
        let x: Qtt<i32, Semel> = Qtt::linear(5);
        let y = x.fmap(|n| n * 2);
        assert_eq!(y.consume(), 10);
    }

    #[test]
    fn test_linear_bind() {
        let x: Qtt<i32, Semel> = Qtt::linear(5);
        let y = x.bind_linear(|n| Qtt::linear(n + 10));
        assert_eq!(y.consume(), 15);
    }

    #[test]
    fn test_unrestricted_creation() {
        let x: Qtt<i32, Omega> = Qtt::unrestricted(42);
        assert_eq!(x.multiplicity(), Multiplicitas::Omega);
        assert!(x.can_discard());
        assert!(x.can_dup());
    }

    #[test]
    fn test_unrestricted_dup() {
        let x: Qtt<i32, Omega> = Qtt::unrestricted(42);
        let y = x.dup();
        let z = x.dup();
        assert_eq!(y.consume(), 42);
        assert_eq!(z.consume(), 42);
        assert_eq!(x.consume(), 42);
    }

    #[test]
    fn test_unrestricted_discard() {
        let x: Qtt<i32, Omega> = Qtt::unrestricted(42);
        x.discard(); // Should compile and run
    }

    #[test]
    fn test_unrestricted_deref() {
        let x: Qtt<i32, Omega> = Qtt::unrestricted(42);
        assert_eq!(*x, 42);
    }

    #[test]
    fn test_erased_creation() {
        let x: Qtt<i32, Nihil> = Qtt::erased(42);
        assert_eq!(x.multiplicity(), Multiplicitas::Nihil);
        assert!(x.can_discard());
    }

    #[test]
    fn test_erased_discard() {
        let x: Qtt<i32, Nihil> = Qtt::erased(42);
        x.discard(); // Should compile and run
    }

    #[test]
    fn test_linear_relax() {
        let x: Qtt<i32, Semel> = Qtt::linear(42);
        let y: Qtt<i32, Omega> = x.relax();
        assert_eq!(y.multiplicity(), Multiplicitas::Omega);
        assert_eq!(y.consume(), 42);
    }

    #[test]
    fn test_unrestricted_restrict() {
        let x: Qtt<i32, Omega> = Qtt::unrestricted(42);
        let y: Qtt<i32, Semel> = x.restrict();
        assert_eq!(y.multiplicity(), Multiplicitas::Semel);
        assert_eq!(y.consume(), 42);
    }

    #[test]
    fn test_qtt_ext() {
        let x = 42.into_linear();
        assert_eq!(x.consume(), 42);

        let y = 42.into_unrestricted();
        assert_eq!(y.consume(), 42);

        let z = 42.into_erased();
        z.discard();
    }

    #[test]
    fn test_type_aliases() {
        let _linear: QttLinearis<i32> = Qtt::linear(42);
        let _erased: QttErasum<i32> = Qtt::erased(42);
        let _free: QttLiber<i32> = Qtt::unrestricted(42);
    }

    #[test]
    fn test_display() {
        let x: Qtt<i32, Semel> = Qtt::linear(42);
        assert_eq!(format!("{x}"), "42:1");

        let y: Qtt<i32, Omega> = Qtt::unrestricted(42);
        assert_eq!(format!("{y}"), "42:ω");
    }

    #[test]
    fn test_split() {
        let pair: Qtt<(i32, &str), Semel> = Qtt::linear((42, "hello"));
        let (a, b) = pair.split(|(n, s)| (n, s));
        assert_eq!(a.consume(), 42);
        assert_eq!(b.consume(), "hello");
    }

    #[test]
    fn test_chaining() {
        let result = Qtt::linear(5i32)
            .fmap(|x| x * 2)
            .fmap(|x| x + 1)
            .bind_linear(|x: i32| Qtt::linear(x.to_string()))
            .consume();

        assert_eq!(result, "11");
    }

    #[test]
    fn test_clone_omega() {
        let x: Qtt<i32, Omega> = Qtt::unrestricted(42);
        let y = x;
        assert_eq!(x.consume(), 42);
        assert_eq!(y.consume(), 42);
    }

    #[test]
    fn test_copy_omega() {
        let x: Qtt<i32, Omega> = Qtt::unrestricted(42);
        let y = x; // Copy
        assert_eq!(x.consume(), 42);
        assert_eq!(y.consume(), 42);
    }
}
