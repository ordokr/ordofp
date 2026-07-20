//! `ParLinearis` - Linear pairs and tensor products
//!
//! > *"Duo simul consumenda"*
//! > — Two to be consumed together. (Neo-Latin)
//!
//! This module provides linear pair types corresponding to linear logic's
//! multiplicative and additive products.

use core::fmt;

// =============================================================================
// ParLinearis - Tensor Product (⊗)
// =============================================================================

/// A linear pair (tensor product) where both components must be used.
///
/// In linear logic, `A ⊗ B` (tensor product) means "use both A and B".
/// This is the multiplicative conjunction - both resources are available
/// and both must be consumed.
///
/// # Latin Etymology
///
/// *Par* = pair, equal
///
/// # Example
///
/// ```rust
/// use ordofp_core::quantitative::ParLinearis;
///
/// let pair = ParLinearis::new(42, "hello");
///
/// // Must consume both components
/// let (n, s) = pair.split();
/// assert_eq!(n, 42);
/// assert_eq!(s, "hello");
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ParLinearis<A, B> {
    first: A,
    second: B,
}

impl<A, B> ParLinearis<A, B> {
    /// Create a new linear pair.
    ///
    /// Both components must eventually be consumed.
    #[inline]
    pub const fn new(first: A, second: B) -> Self {
        ParLinearis { first, second }
    }

    /// Split the pair, consuming it and returning both components.
    ///
    /// This is the elimination rule for tensor products.
    #[inline]
    pub fn split(self) -> (A, B) {
        (self.first, self.second)
    }

    /// Consume the pair with a function that takes both components.
    #[inline]
    pub fn consume_with<C, F>(self, f: F) -> C
    where
        F: FnOnce(A, B) -> C,
    {
        f(self.first, self.second)
    }

    /// Map a function over the first component.
    #[inline]
    pub fn map_first<C, F>(self, f: F) -> ParLinearis<C, B>
    where
        F: FnOnce(A) -> C,
    {
        ParLinearis::new(f(self.first), self.second)
    }

    /// Map a function over the second component.
    #[inline]
    pub fn map_second<C, F>(self, f: F) -> ParLinearis<A, C>
    where
        F: FnOnce(B) -> C,
    {
        ParLinearis::new(self.first, f(self.second))
    }

    /// Map functions over both components.
    #[inline]
    pub fn bimap<C, D, F, G>(self, f: F, g: G) -> ParLinearis<C, D>
    where
        F: FnOnce(A) -> C,
        G: FnOnce(B) -> D,
    {
        ParLinearis::new(f(self.first), g(self.second))
    }

    /// Swap the components.
    #[inline]
    pub fn swap(self) -> ParLinearis<B, A> {
        ParLinearis::new(self.second, self.first)
    }
}

impl<A, B, C> ParLinearis<A, ParLinearis<B, C>> {
    /// Associate a nested tensor to the left.
    ///
    /// `A ⊗ (B ⊗ C) → (A ⊗ B) ⊗ C`
    ///
    /// The mirror of [`ParLinearis::assoc_right`].
    #[inline]
    pub fn assoc_left(self) -> ParLinearis<ParLinearis<A, B>, C> {
        let (a, bc) = self.split();
        let (b, c) = bc.split();
        ParLinearis::new(ParLinearis::new(a, b), c)
    }
}

impl<A, B, C> ParLinearis<ParLinearis<A, B>, C> {
    /// Associate a nested tensor to the right.
    ///
    /// `(A ⊗ B) ⊗ C → A ⊗ (B ⊗ C)`
    #[inline]
    pub fn assoc_right(self) -> ParLinearis<A, ParLinearis<B, C>> {
        let (ab, c) = self.split();
        let (a, b) = ab.split();
        ParLinearis::new(a, ParLinearis::new(b, c))
    }
}

impl<A: Default, B: Default> Default for ParLinearis<A, B> {
    #[inline]
    fn default() -> Self {
        ParLinearis::new(A::default(), B::default())
    }
}

impl<A: fmt::Debug, B: fmt::Debug> fmt::Debug for ParLinearis<A, B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ParLinearis")
            .field("first", &self.first)
            .field("second", &self.second)
            .finish()
    }
}

impl<A: fmt::Display, B: fmt::Display> fmt::Display for ParLinearis<A, B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({} ⊗ {})", self.first, self.second)
    }
}

impl<A, B> From<(A, B)> for ParLinearis<A, B> {
    #[inline]
    fn from((a, b): (A, B)) -> Self {
        ParLinearis::new(a, b)
    }
}

impl<A, B> From<ParLinearis<A, B>> for (A, B) {
    #[inline]
    fn from(pair: ParLinearis<A, B>) -> Self {
        pair.split()
    }
}

// =============================================================================
// TensorLinearis - Named Tensor Product
// =============================================================================

/// Type alias for tensor product with explicit linear marker.
pub type TensorLinearis<A, B> = ParLinearis<A, B>;

// =============================================================================
// WithLinearis - Additive Product (&)
// =============================================================================

/// An additive product (with) where exactly one component is used.
///
/// In linear logic, `A & B` (with) means "choose to use either A or B".
/// This is the additive conjunction - both resources are available
/// but only one can be consumed.
///
/// # Latin Etymology
///
/// *Cum* = with
///
/// # Example
///
/// ```rust
/// use ordofp_core::quantitative::WithLinearis;
///
/// // Must choose one; each choice consumes the whole pair, so a fresh
/// // `WithLinearis` is needed per choice.
/// let choice_a = WithLinearis::new(42, "hello");
/// let left = choice_a.choose_left(); // Consumes the whole thing
/// assert_eq!(left, 42);
///
/// let choice_b = WithLinearis::new(42, "hello");
/// let right = choice_b.choose_right(); // Alternative
/// assert_eq!(right, "hello");
/// ```
pub struct WithLinearis<A, B> {
    left: A,
    right: B,
}

impl<A, B> WithLinearis<A, B> {
    /// Create a new additive product.
    ///
    /// Exactly one component must eventually be chosen and consumed.
    #[inline]
    pub const fn new(left: A, right: B) -> Self {
        WithLinearis { left, right }
    }

    /// Choose the left component, discarding the right.
    #[inline]
    pub fn choose_left(self) -> A {
        self.left
        // right is dropped
    }

    /// Choose the right component, discarding the left.
    #[inline]
    pub fn choose_right(self) -> B {
        self.right
        // left is dropped
    }

    /// Choose based on a boolean: true = left, false = right.
    #[inline]
    pub fn choose(self, left: bool) -> AdditiveChoice<A, B> {
        if left {
            AdditiveChoice::Left(self.left)
        } else {
            AdditiveChoice::Right(self.right)
        }
    }

    /// Project the left component (non-consuming peek).
    ///
    /// Note: This breaks strict linearity but is useful for inspection.
    #[inline]
    pub fn project_left(&self) -> &A {
        &self.left
    }

    /// Project the right component (non-consuming peek).
    #[inline]
    pub fn project_right(&self) -> &B {
        &self.right
    }

    /// Map a function over the left component.
    #[inline]
    pub fn map_left<C, F>(self, f: F) -> WithLinearis<C, B>
    where
        F: FnOnce(A) -> C,
    {
        WithLinearis::new(f(self.left), self.right)
    }

    /// Map a function over the right component.
    #[inline]
    pub fn map_right<C, F>(self, f: F) -> WithLinearis<A, C>
    where
        F: FnOnce(B) -> C,
    {
        WithLinearis::new(self.left, f(self.right))
    }

    /// Map functions over both components (lazy).
    #[inline]
    pub fn bimap<C, D, F, G>(self, f: F, g: G) -> WithLinearis<C, D>
    where
        F: FnOnce(A) -> C,
        G: FnOnce(B) -> D,
    {
        WithLinearis::new(f(self.left), g(self.right))
    }

    /// Swap the components.
    #[inline]
    pub fn swap(self) -> WithLinearis<B, A> {
        WithLinearis::new(self.right, self.left)
    }
}

impl<A: Clone, B: Clone> Clone for WithLinearis<A, B> {
    #[inline]
    fn clone(&self) -> Self {
        WithLinearis::new(self.left.clone(), self.right.clone())
    }
}

impl<A: fmt::Debug, B: fmt::Debug> fmt::Debug for WithLinearis<A, B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WithLinearis")
            .field("left", &self.left)
            .field("right", &self.right)
            .finish()
    }
}

impl<A: fmt::Display, B: fmt::Display> fmt::Display for WithLinearis<A, B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({} & {})", self.left, self.right)
    }
}

// =============================================================================
// AdditiveChoice - Result of choosing from With
// =============================================================================

/// The result of choosing from an additive product.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AdditiveChoice<A, B> {
    /// The left component was chosen.
    Left(A),
    /// The right component was chosen.
    Right(B),
}

impl<A, B> AdditiveChoice<A, B> {
    /// Check if the left was chosen.
    #[inline]
    pub const fn is_left(&self) -> bool {
        matches!(self, AdditiveChoice::Left(_))
    }

    /// Check if the right was chosen.
    #[inline]
    pub const fn is_right(&self) -> bool {
        matches!(self, AdditiveChoice::Right(_))
    }

    /// Get the left value if present.
    #[inline]
    pub fn left(self) -> Option<A> {
        match self {
            AdditiveChoice::Left(a) => Some(a),
            AdditiveChoice::Right(_) => None,
        }
    }

    /// Get the right value if present.
    #[inline]
    pub fn right(self) -> Option<B> {
        match self {
            AdditiveChoice::Left(_) => None,
            AdditiveChoice::Right(b) => Some(b),
        }
    }

    /// Map functions over both variants.
    #[inline]
    pub fn bimap<C, D, F, G>(self, f: F, g: G) -> AdditiveChoice<C, D>
    where
        F: FnOnce(A) -> C,
        G: FnOnce(B) -> D,
    {
        match self {
            AdditiveChoice::Left(a) => AdditiveChoice::Left(f(a)),
            AdditiveChoice::Right(b) => AdditiveChoice::Right(g(b)),
        }
    }

    /// Fold the choice into a single value.
    #[inline]
    pub fn fold<C, F, G>(self, f: F, g: G) -> C
    where
        F: FnOnce(A) -> C,
        G: FnOnce(B) -> C,
    {
        match self {
            AdditiveChoice::Left(a) => f(a),
            AdditiveChoice::Right(b) => g(b),
        }
    }
}

impl<A: fmt::Display, B: fmt::Display> fmt::Display for AdditiveChoice<A, B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AdditiveChoice::Left(a) => write!(f, "Left({a})"),
            AdditiveChoice::Right(b) => write!(f, "Right({b})"),
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    #[test]
    fn assoc_left_reassociates_and_round_trips() {
        let nested = ParLinearis::new(1, ParLinearis::new("b", 3.0)); // A ⊗ (B ⊗ C)
        let left = nested.assoc_left(); // (A ⊗ B) ⊗ C
        let ((a, b), c) = {
            let (ab, c) = left.split();
            (ab.split(), c)
        };
        assert_eq!((a, b, c), (1, "b", 3.0));

        // Round trip: assoc_right undoes assoc_left.
        let back = ParLinearis::new(ParLinearis::new(1, "b"), 3.0).assoc_right();
        let (a2, bc) = back.split();
        let (b2, c2) = bc.split();
        assert_eq!((a2, b2, c2), (1, "b", 3.0));
    }

    #[test]
    fn test_par_linearis_new_split() {
        let pair = ParLinearis::new(42, "hello");
        let (n, s) = pair.split();
        assert_eq!(n, 42);
        assert_eq!(s, "hello");
    }

    #[test]
    fn test_par_linearis_consume_with() {
        let pair = ParLinearis::new(5, 10);
        let result = pair.consume_with(|a, b| a + b);
        assert_eq!(result, 15);
    }

    #[test]
    fn test_par_linearis_map_first() {
        let pair = ParLinearis::new(5, "hello");
        let mapped = pair.map_first(|x| x * 2);
        let (n, s) = mapped.split();
        assert_eq!(n, 10);
        assert_eq!(s, "hello");
    }

    #[test]
    fn test_par_linearis_map_second() {
        let pair = ParLinearis::new(42, 5);
        let mapped = pair.map_second(|x| x * 2);
        let (n, m) = mapped.split();
        assert_eq!(n, 42);
        assert_eq!(m, 10);
    }

    #[test]
    fn test_par_linearis_bimap() {
        let pair = ParLinearis::new(5, 10);
        let mapped = pair.bimap(|x| x * 2, |y| y + 1);
        let (a, b) = mapped.split();
        assert_eq!(a, 10);
        assert_eq!(b, 11);
    }

    #[test]
    fn test_par_linearis_swap() {
        let pair = ParLinearis::new(42, "hello");
        let swapped = pair.swap();
        let (s, n) = swapped.split();
        assert_eq!(s, "hello");
        assert_eq!(n, 42);
    }

    #[test]
    fn test_par_linearis_from_tuple() {
        let pair: ParLinearis<i32, &str> = (42, "hello").into();
        let (n, s) = pair.split();
        assert_eq!(n, 42);
        assert_eq!(s, "hello");
    }

    #[test]
    fn test_par_linearis_into_tuple() {
        let pair = ParLinearis::new(42, "hello");
        let tuple: (i32, &str) = pair.into();
        assert_eq!(tuple, (42, "hello"));
    }

    #[test]
    fn test_par_linearis_display() {
        let pair = ParLinearis::new(42, "hello");
        let s = alloc::format!("{pair}");
        assert_eq!(s, "(42 ⊗ hello)");
    }

    #[test]
    fn test_par_linearis_assoc_right() {
        let nested = ParLinearis::new(ParLinearis::new(1, 2), 3);
        let assoc = nested.assoc_right();
        let (a, bc) = assoc.split();
        let (b, c) = bc.split();
        assert_eq!(a, 1);
        assert_eq!(b, 2);
        assert_eq!(c, 3);
    }

    // WithLinearis tests

    #[test]
    fn test_with_linearis_choose_left() {
        let choice = WithLinearis::new(42, "hello");
        let left = choice.choose_left();
        assert_eq!(left, 42);
    }

    #[test]
    fn test_with_linearis_choose_right() {
        let choice = WithLinearis::new(42, "hello");
        let right = choice.choose_right();
        assert_eq!(right, "hello");
    }

    #[test]
    fn test_with_linearis_choose_bool() {
        let choice1 = WithLinearis::new(42, "hello");
        let result1 = choice1.choose(true);
        assert!(result1.is_left());

        let choice2 = WithLinearis::new(42, "hello");
        let result2 = choice2.choose(false);
        assert!(result2.is_right());
    }

    #[test]
    fn test_with_linearis_project() {
        let choice = WithLinearis::new(42, "hello");
        assert_eq!(*choice.project_left(), 42);
        assert_eq!(*choice.project_right(), "hello");
    }

    #[test]
    fn test_with_linearis_map() {
        let choice = WithLinearis::new(5, 10);
        let mapped = choice.map_left(|x| x * 2).map_right(|x| x + 1);
        assert_eq!(mapped.choose_left(), 10);
    }

    #[test]
    fn test_with_linearis_swap() {
        let choice = WithLinearis::new(42, "hello");
        let swapped = choice.swap();
        assert_eq!(swapped.choose_left(), "hello");
    }

    #[test]
    fn test_with_linearis_display() {
        let choice = WithLinearis::new(42, "hello");
        let s = alloc::format!("{choice}");
        assert_eq!(s, "(42 & hello)");
    }

    // AdditiveChoice tests

    #[test]
    fn test_additive_choice_left() {
        let choice: AdditiveChoice<i32, &str> = AdditiveChoice::Left(42);
        assert!(choice.is_left());
        assert!(!choice.is_right());
        assert_eq!(choice.left(), Some(42));
    }

    #[test]
    fn test_additive_choice_right() {
        let choice: AdditiveChoice<i32, &str> = AdditiveChoice::Right("hello");
        assert!(!choice.is_left());
        assert!(choice.is_right());
        assert_eq!(choice.right(), Some("hello"));
    }

    #[test]
    fn test_additive_choice_bimap() {
        let choice: AdditiveChoice<i32, i32> = AdditiveChoice::Left(5);
        let mapped = choice.bimap(|x| x * 2, |x| x + 1);
        assert_eq!(mapped.left(), Some(10));
    }

    #[test]
    fn test_additive_choice_fold() {
        let left: AdditiveChoice<i32, &str> = AdditiveChoice::Left(42);
        let result = left.fold(|n| n.to_string(), std::string::ToString::to_string);
        assert_eq!(result, "42");

        let right: AdditiveChoice<i32, &str> = AdditiveChoice::Right("hello");
        let result = right.fold(|n| n.to_string(), std::string::ToString::to_string);
        assert_eq!(result, "hello");
    }
}
