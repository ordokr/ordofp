//! Length-Indexed Vectors - Vectum
//!
//! > *"In mathematica, vectorem est quantitas cum magnitudine et directione."*
//! > — In mathematics, a vector is a quantity with magnitude and direction.
//!
//! This module provides length-indexed vectors where the length is tracked
//! at the type level using Peano numbers.
//!
//! # Benefits
//!
//! - **Compile-time length safety**: Operations like `head` don't need `Option`
//! - **Type-safe concatenation**: Length is computed at compile time
//! - **Guaranteed invariants**: Empty/non-empty distinction in the type system
//!
//! # Example
//!
//! ```rust
//! use ordofp_core::dependent::vect::Vectum;
//! use ordofp_core::dependent::peano::{Zero, Succ, N1, N2, N3};
//! use ordofp_core::dependent::FromArray;
//!
//! // Create from array - length is inferred
//! let v: Vectum<i32, N3> = Vectum::from_array([1, 2, 3]);
//!
//! // Head is guaranteed to exist
//! assert_eq!(*v.caput(), 1);
//!
//! // Tail is one element shorter
//! let tail: Vectum<i32, N2> = v.cauda();
//! assert_eq!(tail.len(), 2);
//! ```

use super::peano::{Additio, Naturalis, NonNihil, Praecessor, Succ, Zero};
use alloc::vec::Vec;
use core::marker::PhantomData;

// =============================================================================
// Vectum - Length-Indexed Vector
// =============================================================================

/// A vector with compile-time tracked length.
///
/// # Latin Etymology
/// *Vectum* means "that which carries" - a container that carries elements.
///
/// # Type Parameters
///
/// - `T`: The element type
/// - `N`: The length as a type-level Peano number
///
/// # Example
///
/// ```rust
/// use ordofp_core::dependent::vect::Vectum;
/// use ordofp_core::dependent::peano::N3;
/// use ordofp_core::dependent::FromArray;
///
/// let v: Vectum<i32, N3> = Vectum::from_array([1, 2, 3]);
/// assert_eq!(v.len(), 3);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Vectum<T, N: Naturalis> {
    data: Vec<T>,
    _len: PhantomData<N>,
}

// =============================================================================
// Construction
// =============================================================================

impl<T> Vectum<T, Zero> {
    /// Create an empty vector.
    ///
    /// # Latin Etymology
    /// *Vacuus* means "empty".
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::dependent::vect::Vectum;
    /// use ordofp_core::dependent::peano::Zero;
    ///
    /// let v: Vectum<i32, Zero> = Vectum::vacuus();
    /// assert!(v.is_empty());
    /// ```
    #[inline]
    pub fn vacuus() -> Self {
        Vectum {
            data: Vec::new(),
            _len: PhantomData,
        }
    }

    /// Alias for `vacuus`.
    #[inline]
    pub fn empty() -> Self {
        Self::vacuus()
    }
}

impl<T, N: Naturalis> Vectum<T, N> {
    /// Get the runtime length.
    #[inline]
    pub fn len(&self) -> usize {
        N::VALUE
    }

    /// Check if empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        N::VALUE == 0
    }

    /// Get the type-level length as a runtime value.
    #[inline]
    pub fn type_len() -> usize {
        N::VALUE
    }

    /// Convert to a standard Vec, consuming self.
    ///
    /// # Latin Etymology
    /// *Resolvo* means "to unfasten, release".
    #[inline]
    pub fn resolvere(self) -> Vec<T> {
        self.data
    }

    /// Get a reference to the underlying Vec.
    #[inline]
    pub fn as_vec(&self) -> &Vec<T> {
        &self.data
    }

    /// Get a reference to the underlying slice.
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        &self.data
    }
}

// =============================================================================
// From Array
// =============================================================================

/// Helper trait for array-to-vectum conversion.
pub trait FromArray<T, const N: usize>: Sized {
    /// Create a Vectum from an array.
    fn from_array(arr: [T; N]) -> Self;
}

impl<T> FromArray<T, 0> for Vectum<T, Zero> {
    fn from_array(_arr: [T; 0]) -> Self {
        Vectum::vacuus()
    }
}

/// Implements `FromArray<T, $n>` for the matching type-level length.
/// The impl set deliberately stops at N = 8 (pinned by
/// `core/tests/dependent_characterization.rs`).
macro_rules! impl_from_array {
    ($($n:literal => $nat:ty),+ $(,)?) => {
        $(impl<T> FromArray<T, $n> for Vectum<T, $nat> {
            fn from_array(arr: [T; $n]) -> Self {
                Vectum {
                    data: arr.into(),
                    _len: PhantomData,
                }
            }
        })+
    };
}

impl_from_array!(
    1 => Succ<Zero>,
    2 => Succ<Succ<Zero>>,
    3 => Succ<Succ<Succ<Zero>>>,
    4 => Succ<Succ<Succ<Succ<Zero>>>>,
    5 => Succ<Succ<Succ<Succ<Succ<Zero>>>>>,
    6 => Succ<Succ<Succ<Succ<Succ<Succ<Zero>>>>>>,
    7 => Succ<Succ<Succ<Succ<Succ<Succ<Succ<Zero>>>>>>>,
    8 => Succ<Succ<Succ<Succ<Succ<Succ<Succ<Succ<Zero>>>>>>>>,
);

impl<T> Vectum<T, Succ<Zero>> {
    /// Create a singleton vector.
    ///
    /// # Latin Etymology
    /// *Singulus* means "one at a time, single".
    #[inline]
    pub fn singulus(value: T) -> Self {
        Vectum {
            data: alloc::vec![value],
            _len: PhantomData,
        }
    }
}

// =============================================================================
// Non-Empty Operations
// =============================================================================

impl<T, N: NonNihil> Vectum<T, N> {
    /// Get the first element.
    ///
    /// This is safe because the type system guarantees the vector is non-empty.
    ///
    /// # Latin Etymology
    /// *Caput* means "head".
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::dependent::vect::Vectum;
    /// use ordofp_core::dependent::peano::N3;
    /// use ordofp_core::dependent::FromArray;
    ///
    /// let v: Vectum<i32, N3> = Vectum::from_array([1, 2, 3]);
    /// assert_eq!(*v.caput(), 1);
    /// ```
    #[inline]
    pub fn caput(&self) -> &T {
        // SAFETY: NonNihil guarantees at least one element
        &self.data[0]
    }

    /// Get a mutable reference to the first element.
    #[inline]
    pub fn caput_mut(&mut self) -> &mut T {
        &mut self.data[0]
    }

    /// Get the last element.
    ///
    /// # Latin Etymology
    /// *Ultimus* means "last, final".
    ///
    /// # Panics
    ///
    /// Panics only if the type-level `NonNihil` invariant (length >= 1) is
    /// violated by the underlying storage, which indicates a bug in this
    /// crate.
    #[inline]
    pub fn ultimus(&self) -> &T {
        self.data
            .last()
            .expect("NonNihil guarantees at least one element")
    }

    /// Get a mutable reference to the last element.
    ///
    /// # Panics
    ///
    /// Panics only if the type-level `NonNihil` invariant (length >= 1) is
    /// violated by the underlying storage, which indicates a bug in this
    /// crate.
    #[inline]
    pub fn ultimus_mut(&mut self) -> &mut T {
        self.data
            .last_mut()
            .expect("NonNihil guarantees at least one element")
    }
}

impl<T: Clone, N: NonNihil + Praecessor> Vectum<T, N>
where
    N::Prior: Naturalis,
{
    /// Get the tail (all elements except the first).
    ///
    /// # Latin Etymology
    /// *Cauda* means "tail".
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::dependent::vect::Vectum;
    /// use ordofp_core::dependent::peano::{N2, N3};
    /// use ordofp_core::dependent::FromArray;
    ///
    /// let v: Vectum<i32, N3> = Vectum::from_array([1, 2, 3]);
    /// let tail: Vectum<i32, N2> = v.cauda();
    /// assert_eq!(tail.len(), 2);
    /// ```
    #[inline]
    pub fn cauda(&self) -> Vectum<T, N::Prior> {
        Vectum {
            data: self.data[1..].to_vec(),
            _len: PhantomData,
        }
    }

    /// Get all elements except the last.
    ///
    /// # Latin Etymology
    /// *Initium* means "beginning".
    #[inline]
    pub fn initium(&self) -> Vectum<T, N::Prior> {
        Vectum {
            data: self.data[..self.data.len() - 1].to_vec(),
            _len: PhantomData,
        }
    }
}

// =============================================================================
// Prepend and Append
// =============================================================================

impl<T, N: Naturalis> Vectum<T, N> {
    /// Prepend an element, incrementing the length type.
    ///
    /// # Latin Etymology
    /// *Praepono* means "to place before".
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::dependent::vect::Vectum;
    /// use ordofp_core::dependent::peano::{Zero, N1};
    ///
    /// let v: Vectum<i32, Zero> = Vectum::vacuus();
    /// let v2: Vectum<i32, N1> = v.praepono(42);
    /// assert_eq!(*v2.caput(), 42);
    /// ```
    #[inline]
    pub fn praepono(self, elem: T) -> Vectum<T, Succ<N>> {
        let mut new_data = Vec::with_capacity(N::VALUE + 1);
        new_data.push(elem);
        new_data.extend(self.data);
        Vectum {
            data: new_data,
            _len: PhantomData,
        }
    }

    /// Alias for `praepono`.
    #[inline]
    pub fn cons(self, elem: T) -> Vectum<T, Succ<N>> {
        self.praepono(elem)
    }

    /// Append an element, incrementing the length type.
    ///
    /// # Latin Etymology
    /// *Appono* means "to place near, add".
    #[inline]
    pub fn appono(mut self, elem: T) -> Vectum<T, Succ<N>> {
        self.data.push(elem);
        Vectum {
            data: self.data,
            _len: PhantomData,
        }
    }

    /// Alias for `appono`.
    #[inline]
    pub fn snoc(self, elem: T) -> Vectum<T, Succ<N>> {
        self.appono(elem)
    }
}

// =============================================================================
// Concatenation
// =============================================================================

impl<T, N: Naturalis> Vectum<T, N> {
    /// Concatenate two vectors.
    ///
    /// The resulting length is the sum of the two lengths.
    ///
    /// # Latin Etymology
    /// *Concatenare* means "to link together".
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::dependent::vect::Vectum;
    /// use ordofp_core::dependent::peano::{N2, N3, N5};
    /// use ordofp_core::dependent::FromArray;
    ///
    /// let v1: Vectum<i32, N2> = Vectum::from_array([1, 2]);
    /// let v2: Vectum<i32, N3> = Vectum::from_array([3, 4, 5]);
    /// let v3: Vectum<i32, N5> = v1.concatenare(v2);
    /// assert_eq!(v3.len(), 5);
    /// ```
    #[inline]
    pub fn concatenare<M: Naturalis>(
        mut self,
        other: Vectum<T, M>,
    ) -> Vectum<T, <N as Additio<M>>::Summa>
    where
        N: Additio<M>,
        <N as Additio<M>>::Summa: Naturalis,
    {
        self.data.reserve(M::VALUE);
        self.data.extend(other.data);
        Vectum {
            data: self.data,
            _len: PhantomData,
        }
    }

    /// Alias for `concatenare`.
    #[inline]
    pub fn append<M: Naturalis>(self, other: Vectum<T, M>) -> Vectum<T, <N as Additio<M>>::Summa>
    where
        N: Additio<M>,
        <N as Additio<M>>::Summa: Naturalis,
    {
        self.concatenare(other)
    }
}

// =============================================================================
// Mapping and Transformations
// =============================================================================

impl<T, N: Naturalis> Vectum<T, N> {
    /// Map a function over all elements.
    ///
    /// Length is preserved.
    ///
    /// # Latin Etymology
    /// *Mutare* means "to change".
    #[inline]
    pub fn mutare<U, F>(self, f: F) -> Vectum<U, N>
    where
        F: FnMut(T) -> U,
    {
        Vectum {
            data: self.data.into_iter().map(f).collect(),
            _len: PhantomData,
        }
    }

    /// Alias for `mutare`.
    #[inline]
    pub fn map<U, F>(self, f: F) -> Vectum<U, N>
    where
        F: FnMut(T) -> U,
    {
        self.mutare(f)
    }

    /// Map with index.
    #[inline]
    pub fn mutare_cum_indice<U, F>(self, mut f: F) -> Vectum<U, N>
    where
        F: FnMut(usize, T) -> U,
    {
        Vectum {
            data: self
                .data
                .into_iter()
                .enumerate()
                .map(|(i, t)| f(i, t))
                .collect(),
            _len: PhantomData,
        }
    }

    /// Zip with another vector of the same length.
    ///
    /// # Latin Etymology
    /// *Coniungere* means "to join together".
    #[inline]
    pub fn coniungere<U>(self, other: Vectum<U, N>) -> Vectum<(T, U), N> {
        Vectum {
            data: self.data.into_iter().zip(other.data).collect(),
            _len: PhantomData,
        }
    }

    /// Alias for `coniungere`.
    #[inline]
    pub fn zip<U>(self, other: Vectum<U, N>) -> Vectum<(T, U), N> {
        self.coniungere(other)
    }

    /// Zip with a function.
    #[inline]
    pub fn zip_with<U, V, F>(self, other: Vectum<U, N>, mut f: F) -> Vectum<V, N>
    where
        F: FnMut(T, U) -> V,
    {
        Vectum {
            data: self
                .data
                .into_iter()
                .zip(other.data)
                .map(|(t, u)| f(t, u))
                .collect(),
            _len: PhantomData,
        }
    }
}

// =============================================================================
// Folding
// =============================================================================

impl<T, N: Naturalis> Vectum<T, N> {
    /// Left fold.
    ///
    /// # Latin Etymology
    /// *Plicare* means "to fold".
    #[inline]
    pub fn plicare_sinistrum<B, F>(self, init: B, f: F) -> B
    where
        F: FnMut(B, T) -> B,
    {
        self.data.into_iter().fold(init, f)
    }

    /// Alias for `plicare_sinistrum`.
    #[inline]
    pub fn foldl<B, F>(self, init: B, f: F) -> B
    where
        F: FnMut(B, T) -> B,
    {
        self.plicare_sinistrum(init, f)
    }

    /// Right fold.
    #[inline]
    pub fn plicare_dextrum<B, F>(self, init: B, mut f: F) -> B
    where
        F: FnMut(T, B) -> B,
    {
        self.data.into_iter().rev().fold(init, |acc, t| f(t, acc))
    }

    /// Alias for `plicare_dextrum`.
    #[inline]
    pub fn foldr<B, F>(self, init: B, f: F) -> B
    where
        F: FnMut(T, B) -> B,
    {
        self.plicare_dextrum(init, f)
    }
}

// =============================================================================
// Indexing
// =============================================================================

impl<T, N: Naturalis> Vectum<T, N> {
    /// Get element at index, returning None if out of bounds.
    ///
    /// For compile-time-safe indexing, use `at_type` with type-level indices.
    #[inline]
    pub fn get(&self, index: usize) -> Option<&T> {
        self.data.get(index)
    }

    /// Get mutable element at index.
    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.data.get_mut(index)
    }

    /// Get mutable references to multiple disjoint indices simultaneously.
    ///
    /// This uses `<[T]>::get_disjoint_mut()` (stable in Rust 1.86) to safely
    /// borrow multiple elements at once without runtime checks beyond index bounds.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::dependent::vect::Vectum;
    /// use ordofp_core::dependent::peano::N4;
    /// use ordofp_core::dependent::FromArray;
    ///
    /// let mut v: Vectum<i32, N4> = Vectum::from_array([10, 20, 30, 40]);
    /// if let Ok([a, c]) = v.get_disjoint_mut([0, 2]) {
    ///     *a = 100;
    ///     *c = 300;
    /// }
    /// assert_eq!(v.get(0), Some(&100));
    /// assert_eq!(v.get(2), Some(&300));
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `Err` if any indices are out of bounds or if any indices are duplicated.
    #[inline]
    pub fn get_disjoint_mut<const M: usize>(
        &mut self,
        indices: [usize; M],
    ) -> Result<[&mut T; M], core::slice::GetDisjointMutError> {
        self.data.get_disjoint_mut(indices)
    }
}

// =============================================================================
// Iteration
// =============================================================================

impl<T, N: Naturalis> IntoIterator for Vectum<T, N> {
    type Item = T;
    type IntoIter = alloc::vec::IntoIter<T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

impl<'a, T, N: Naturalis> IntoIterator for &'a Vectum<T, N> {
    type Item = &'a T;
    type IntoIter = core::slice::Iter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.data.iter()
    }
}

impl<'a, T, N: Naturalis> IntoIterator for &'a mut Vectum<T, N> {
    type Item = &'a mut T;
    type IntoIter = core::slice::IterMut<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.data.iter_mut()
    }
}

impl<T, N: Naturalis> Vectum<T, N> {
    /// Returns an iterator over references to elements.
    #[inline]
    pub fn iter(&self) -> core::slice::Iter<'_, T> {
        self.data.iter()
    }

    /// Returns an iterator over mutable references to elements.
    #[inline]
    pub fn iter_mut(&mut self) -> core::slice::IterMut<'_, T> {
        self.data.iter_mut()
    }
}

// =============================================================================
// Reverse
// =============================================================================

impl<T, N: Naturalis> Vectum<T, N> {
    /// Reverse the vector in place.
    ///
    /// # Latin Etymology
    /// *Invertere* means "to turn upside down".
    #[inline]
    pub fn invertere(&mut self) {
        self.data.reverse();
    }

    /// Create a reversed copy.
    #[inline]
    pub fn inversus(mut self) -> Vectum<T, N> {
        self.data.reverse();
        self
    }
}

// =============================================================================
// Replicate
// =============================================================================

/// Create a vector of N identical elements.
///
/// # Latin Etymology
/// *Replicare* means "to repeat".
#[inline]
pub fn replicare<T: Clone, N: Naturalis>(value: T) -> Vectum<T, N> {
    let mut data = Vec::with_capacity(N::VALUE);
    data.extend((0..N::VALUE).map(|_| value.clone()));
    Vectum {
        data,
        _len: PhantomData,
    }
}

/// Create a vector using a function from index to value.
#[inline]
pub fn tabulare<T, N: Naturalis, F>(mut f: F) -> Vectum<T, N>
where
    F: FnMut(usize) -> T,
{
    let mut data = Vec::with_capacity(N::VALUE);
    data.extend((0..N::VALUE).map(&mut f));
    Vectum {
        data,
        _len: PhantomData,
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dependent::peano::{N1, N2, N3, N4, N5};
    use alloc::vec;

    #[test]
    fn test_vacuus() {
        let v: Vectum<i32, Zero> = Vectum::vacuus();
        assert!(v.is_empty());
        assert_eq!(v.len(), 0);
    }

    #[test]
    fn test_singulus() {
        let v: Vectum<i32, N1> = Vectum::singulus(42);
        assert_eq!(v.len(), 1);
        assert_eq!(*v.caput(), 42);
    }

    #[test]
    fn test_from_array() {
        let v: Vectum<i32, N3> = Vectum::from_array([1, 2, 3]);
        assert_eq!(v.len(), 3);
        assert_eq!(*v.caput(), 1);
        assert_eq!(*v.ultimus(), 3);
    }

    #[test]
    fn test_cauda() {
        let v: Vectum<i32, N3> = Vectum::from_array([1, 2, 3]);
        let tail: Vectum<i32, N2> = v.cauda();
        assert_eq!(tail.len(), 2);
        assert_eq!(*tail.caput(), 2);
    }

    #[test]
    fn test_initium() {
        let v: Vectum<i32, N3> = Vectum::from_array([1, 2, 3]);
        let init: Vectum<i32, N2> = v.initium();
        assert_eq!(init.len(), 2);
        assert_eq!(*init.ultimus(), 2);
    }

    #[test]
    fn test_praepono() {
        let v: Vectum<i32, N2> = Vectum::from_array([2, 3]);
        let v2: Vectum<i32, N3> = v.praepono(1);
        assert_eq!(v2.len(), 3);
        assert_eq!(*v2.caput(), 1);
    }

    #[test]
    fn test_appono() {
        let v: Vectum<i32, N2> = Vectum::from_array([1, 2]);
        let v2: Vectum<i32, N3> = v.appono(3);
        assert_eq!(v2.len(), 3);
        assert_eq!(*v2.ultimus(), 3);
    }

    #[test]
    fn test_concatenare() {
        let v1: Vectum<i32, N2> = Vectum::from_array([1, 2]);
        let v2: Vectum<i32, N3> = Vectum::from_array([3, 4, 5]);
        let v3: Vectum<i32, N5> = v1.concatenare(v2);
        assert_eq!(v3.len(), 5);
        assert_eq!(*v3.caput(), 1);
        assert_eq!(*v3.ultimus(), 5);
    }

    #[test]
    fn test_mutare() {
        let v: Vectum<i32, N3> = Vectum::from_array([1, 2, 3]);
        let v2: Vectum<i32, N3> = v.mutare(|x| x * 2);
        assert_eq!(*v2.caput(), 2);
        assert_eq!(*v2.ultimus(), 6);
    }

    #[test]
    fn test_coniungere() {
        let v1: Vectum<i32, N2> = Vectum::from_array([1, 2]);
        let v2: Vectum<char, N2> = Vectum::from_array(['a', 'b']);
        let zipped: Vectum<(i32, char), N2> = v1.coniungere(v2);
        assert_eq!(*zipped.caput(), (1, 'a'));
        assert_eq!(*zipped.ultimus(), (2, 'b'));
    }

    #[test]
    fn test_foldl() {
        let v: Vectum<i32, N3> = Vectum::from_array([1, 2, 3]);
        let sum = v.foldl(0, |acc, x| acc + x);
        assert_eq!(sum, 6);
    }

    #[test]
    fn test_foldr() {
        let v: Vectum<i32, N3> = Vectum::from_array([1, 2, 3]);
        // foldr: 1 - (2 - (3 - 0)) = 1 - (2 - 3) = 1 - (-1) = 2
        let result = v.foldr(0, |x, acc| x - acc);
        assert_eq!(result, 2);
    }

    #[test]
    fn test_inversus() {
        let v: Vectum<i32, N3> = Vectum::from_array([1, 2, 3]);
        let reversed = v.inversus();
        assert_eq!(*reversed.caput(), 3);
        assert_eq!(*reversed.ultimus(), 1);
    }

    #[test]
    fn test_replicare() {
        let v: Vectum<i32, N4> = replicare(42);
        assert_eq!(v.len(), 4);
        for elem in &v {
            assert_eq!(*elem, 42);
        }
    }

    #[test]
    fn test_tabulare() {
        let v: Vectum<usize, N5> = tabulare(|i| i * i);
        assert_eq!(v.as_slice(), &[0, 1, 4, 9, 16]);
    }

    #[test]
    fn test_into_iter() {
        let v: Vectum<i32, N3> = Vectum::from_array([1, 2, 3]);
        let collected: Vec<i32> = v.into_iter().collect();
        assert_eq!(collected, vec![1, 2, 3]);
    }

    #[test]
    fn test_resolvere() {
        let v: Vectum<i32, N3> = Vectum::from_array([1, 2, 3]);
        let vec: Vec<i32> = v.resolvere();
        assert_eq!(vec, vec![1, 2, 3]);
    }

    #[test]
    fn test_get_disjoint_mut() {
        // Test get_disjoint_mut (stable 1.86) - safe multi-mutable-borrow from slices
        let mut v: Vectum<i32, N4> = Vectum::from_array([10, 20, 30, 40]);

        // Get two disjoint mutable references
        let [first, third] = v
            .get_disjoint_mut([0, 2])
            .expect("indices 0 and 2 are valid and disjoint within a 4-element array");
        *first = 100;
        *third = 300;

        assert_eq!(v.get(0), Some(&100));
        assert_eq!(v.get(1), Some(&20)); // Unchanged
        assert_eq!(v.get(2), Some(&300));
        assert_eq!(v.get(3), Some(&40)); // Unchanged
    }

    #[test]
    fn test_get_disjoint_mut_out_of_bounds() {
        let mut v: Vectum<i32, N3> = Vectum::from_array([1, 2, 3]);

        // Index out of bounds should fail
        let result = v.get_disjoint_mut([0, 5]);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_disjoint_mut_duplicate_indices() {
        let mut v: Vectum<i32, N4> = Vectum::from_array([10, 20, 30, 40]);

        // Duplicate indices should fail (not disjoint)
        let result = v.get_disjoint_mut([1, 1]);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_disjoint_mut_four_elements() {
        let mut v: Vectum<i32, N5> = Vectum::from_array([1, 2, 3, 4, 5]);

        // Get four disjoint mutable references
        let [a, b, c, d] = v
            .get_disjoint_mut([0, 1, 3, 4])
            .expect("indices 0, 1, 3, 4 are valid and disjoint within a 5-element array");
        *a = 10;
        *b = 20;
        *c = 40;
        *d = 50;

        assert_eq!(v.as_slice(), &[10, 20, 3, 40, 50]);
    }
}
