//! `NonEmpty` - A non-empty list type.
//!
//! `NonEmpty<T>` guarantees at least one element, making operations like
//! `head`, `first`, and `last` always safe (no Option needed).
//!
//! # Example
//!
//! ```rust
//! use ordofp_core::nonempty::NonEmpty;
//!
//! // Create a non-empty list
//! let nel = NonEmpty::new(1, vec![2, 3, 4]);
//! assert_eq!(nel.head(), &1);
//! assert_eq!(nel.last(), &4);
//! assert_eq!(nel.len(), 4);
//!
//! // From a single element
//! let single = NonEmpty::singleton(42);
//! assert_eq!(single.head(), &42);
//! assert_eq!(single.len(), 1);
//!
//! // Map over all elements
//! let doubled = nel.map(|x| x * 2);
//! assert_eq!(doubled.to_vec(), vec![2, 4, 6, 8]);
//! ```

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A non-empty list guaranteeing at least one element.
///
/// This is useful when you need to ensure a collection always has at least
/// one item, allowing `head()` and `last()` to be total functions.
///
/// The derived `Deserialize` performs no extra validation beyond the shape
/// of its fields (`head: T`, `tail: Vec<T>`); unlike the tree-shaped `pfds`
/// containers, there is no forgeable structural invariant here — any `head`
/// plus `tail` is already a valid `NonEmpty`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg(feature = "alloc")]
pub struct NonEmpty<T> {
    /// The first (head) element, always present.
    head: T,
    /// The remaining elements (may be empty).
    tail: Vec<T>,
}

#[cfg(feature = "alloc")]
impl<T> NonEmpty<T> {
    /// Create a new `NonEmpty` list with a head and tail.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::nonempty::NonEmpty;
    ///
    /// let nel = NonEmpty::new(1, vec![2, 3]);
    /// assert_eq!(nel.head(), &1);
    /// assert_eq!(nel.tail(), &[2, 3]);
    /// ```
    #[inline]
    pub fn new(head: T, tail: Vec<T>) -> Self {
        NonEmpty { head, tail }
    }

    /// Create a `NonEmpty` with a single element.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::nonempty::NonEmpty;
    ///
    /// let nel = NonEmpty::singleton(42);
    /// assert_eq!(nel.len(), 1);
    /// assert_eq!(nel.head(), &42);
    /// ```
    #[inline]
    pub fn singleton(value: T) -> Self {
        NonEmpty {
            head: value,
            tail: Vec::new(),
        }
    }

    /// Try to create a `NonEmpty` from a Vec.
    ///
    /// Returns `None` if the Vec is empty.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::nonempty::NonEmpty;
    ///
    /// let nel = NonEmpty::from_vec(vec![1, 2, 3]);
    /// assert!(nel.is_some());
    ///
    /// let empty = NonEmpty::<i32>::from_vec(vec![]);
    /// assert!(empty.is_none());
    /// ```
    pub fn from_vec(mut vec: Vec<T>) -> Option<Self> {
        if vec.is_empty() {
            None
        } else {
            let head = vec.remove(0);
            Some(NonEmpty { head, tail: vec })
        }
    }

    /// Get a reference to the head (first element).
    ///
    /// This is always safe since `NonEmpty` guarantees at least one element.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::nonempty::NonEmpty;
    ///
    /// let nel = NonEmpty::new(1, vec![2, 3]);
    /// assert_eq!(nel.head(), &1);
    /// ```
    #[inline]
    pub fn head(&self) -> &T {
        &self.head
    }

    /// Get a mutable reference to the head.
    #[inline]
    pub fn head_mut(&mut self) -> &mut T {
        &mut self.head
    }

    /// Get a reference to the tail (all elements after the head).
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::nonempty::NonEmpty;
    ///
    /// let nel = NonEmpty::new(1, vec![2, 3]);
    /// assert_eq!(nel.tail(), &[2, 3]);
    /// ```
    #[inline]
    pub fn tail(&self) -> &[T] {
        &self.tail
    }

    /// Get a reference to the last element.
    ///
    /// This is always safe since `NonEmpty` guarantees at least one element.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::nonempty::NonEmpty;
    ///
    /// let nel = NonEmpty::new(1, vec![2, 3]);
    /// assert_eq!(nel.last(), &3);
    ///
    /// let single = NonEmpty::singleton(42);
    /// assert_eq!(single.last(), &42);
    /// ```
    #[inline]
    pub fn last(&self) -> &T {
        self.tail.last().unwrap_or(&self.head)
    }

    /// Get the number of elements.
    ///
    /// Always returns at least 1.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::nonempty::NonEmpty;
    ///
    /// let nel = NonEmpty::new(1, vec![2, 3]);
    /// assert_eq!(nel.len(), 3);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        1 + self.tail.len()
    }

    /// Always `false`: a `NonEmpty` is guaranteed to hold at least one element.
    ///
    /// Provided so generic code written against `len`/`is_empty` pairs works
    /// unchanged; the type system makes the `true` case unrepresentable.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::nonempty::NonEmpty;
    ///
    /// let nel = NonEmpty::singleton(1);
    /// assert!(!nel.is_empty());
    /// ```
    #[inline]
    pub const fn is_empty(&self) -> bool {
        false
    }

    /// Check if this contains only one element.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::nonempty::NonEmpty;
    ///
    /// let single = NonEmpty::singleton(1);
    /// assert!(single.is_singleton());
    ///
    /// let multiple = NonEmpty::new(1, vec![2]);
    /// assert!(!multiple.is_singleton());
    /// ```
    #[inline]
    pub fn is_singleton(&self) -> bool {
        self.tail.is_empty()
    }

    /// Convert to a Vec.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::nonempty::NonEmpty;
    ///
    /// let nel = NonEmpty::new(1, vec![2, 3]);
    /// assert_eq!(nel.to_vec(), vec![1, 2, 3]);
    /// ```
    #[inline]
    pub fn to_vec(self) -> Vec<T> {
        let mut result = Vec::with_capacity(self.len());
        result.push(self.head);
        result.extend(self.tail);
        result
    }

    /// Convert to a Vec by reference.
    #[inline]
    pub fn as_vec(&self) -> Vec<T>
    where
        T: Clone,
    {
        let mut result = Vec::with_capacity(self.len());
        result.push(self.head.clone());
        result.extend(self.tail.iter().cloned());
        result
    }

    /// Push an element to the end.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::nonempty::NonEmpty;
    ///
    /// let nel = NonEmpty::singleton(1).push(2).push(3);
    /// assert_eq!(nel.to_vec(), vec![1, 2, 3]);
    /// ```
    #[inline]
    pub fn push(mut self, value: T) -> Self {
        self.tail.push(value);
        self
    }

    /// Prepend an element to the front.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::nonempty::NonEmpty;
    ///
    /// let nel = NonEmpty::singleton(2).prepend(1);
    /// assert_eq!(nel.to_vec(), vec![1, 2]);
    /// ```
    pub fn prepend(self, value: T) -> Self {
        let mut new_tail = Vec::with_capacity(self.len());
        new_tail.push(self.head);
        new_tail.extend(self.tail);
        NonEmpty {
            head: value,
            tail: new_tail,
        }
    }

    /// Map a function over all elements.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::nonempty::NonEmpty;
    ///
    /// let nel = NonEmpty::new(1, vec![2, 3]);
    /// let doubled = nel.map(|x| x * 2);
    /// assert_eq!(doubled.to_vec(), vec![2, 4, 6]);
    /// ```
    #[inline]
    pub fn map<U, F>(self, mut f: F) -> NonEmpty<U>
    where
        F: FnMut(T) -> U,
    {
        let tail_len = self.tail.len();
        NonEmpty {
            head: f(self.head),
            tail: {
                let mut v = Vec::with_capacity(tail_len);
                v.extend(self.tail.into_iter().map(f));
                v
            },
        }
    }

    /// Map a function over all elements by reference.
    #[inline]
    pub fn map_ref<U, F>(&self, mut f: F) -> NonEmpty<U>
    where
        F: FnMut(&T) -> U,
    {
        NonEmpty {
            head: f(&self.head),
            tail: {
                let mut v = Vec::with_capacity(self.tail.len());
                v.extend(self.tail.iter().map(f));
                v
            },
        }
    }

    /// Fold from the left.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::nonempty::NonEmpty;
    ///
    /// let nel = NonEmpty::new(1, vec![2, 3, 4]);
    /// let sum = nel.fold(0, |acc, x| acc + x);
    /// assert_eq!(sum, 10);
    /// ```
    #[inline]
    pub fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        F: FnMut(B, T) -> B,
    {
        let acc = f(init, self.head);
        self.tail.into_iter().fold(acc, f)
    }

    /// Fold from the left by reference.
    #[inline]
    pub fn fold_ref<B, F>(&self, init: B, mut f: F) -> B
    where
        F: FnMut(B, &T) -> B,
    {
        let acc = f(init, &self.head);
        self.tail.iter().fold(acc, f)
    }

    /// Reduce without an initial value (uses head as initial).
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::nonempty::NonEmpty;
    ///
    /// let nel = NonEmpty::new(1, vec![2, 3, 4]);
    /// let sum = nel.reduce(|acc, x| acc + x);
    /// assert_eq!(sum, 10);
    /// ```
    #[inline]
    pub fn reduce<F>(self, f: F) -> T
    where
        F: FnMut(T, T) -> T,
    {
        self.tail.into_iter().fold(self.head, f)
    }

    /// Concatenate with another `NonEmpty`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::nonempty::NonEmpty;
    ///
    /// let a = NonEmpty::new(1, vec![2]);
    /// let b = NonEmpty::new(3, vec![4]);
    /// let combined = a.concat(b);
    /// assert_eq!(combined.to_vec(), vec![1, 2, 3, 4]);
    /// ```
    #[inline]
    pub fn concat(self, other: NonEmpty<T>) -> NonEmpty<T> {
        let mut tail = self.tail;
        tail.reserve(1 + other.tail.len());
        tail.push(other.head);
        tail.extend(other.tail);
        NonEmpty {
            head: self.head,
            tail,
        }
    }

    /// Reverse the list.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::nonempty::NonEmpty;
    ///
    /// let nel = NonEmpty::new(1, vec![2, 3]);
    /// let reversed = nel.reverse();
    /// assert_eq!(reversed.to_vec(), vec![3, 2, 1]);
    /// ```
    ///
    /// # Panics
    ///
    /// Panics only if the non-emptiness invariant is violated (the
    /// `unwrap` re-wraps a vector that necessarily holds at least the
    /// head); reaching it would indicate a bug in this crate.
    pub fn reverse(self) -> NonEmpty<T> {
        let mut all = self.to_vec();
        all.reverse();
        // Safe because NonEmpty always has at least one element
        NonEmpty::from_vec(all).unwrap()
    }

    /// Get an iterator over references.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        core::iter::once(&self.head).chain(self.tail.iter())
    }

    /// Get an element by index.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::nonempty::NonEmpty;
    ///
    /// let nel = NonEmpty::new(1, vec![2, 3]);
    /// assert_eq!(nel.get(0), Some(&1));
    /// assert_eq!(nel.get(1), Some(&2));
    /// assert_eq!(nel.get(10), None);
    /// ```
    #[inline]
    pub fn get(&self, index: usize) -> Option<&T> {
        if index == 0 {
            Some(&self.head)
        } else {
            self.tail.get(index - 1)
        }
    }

    /// Filter elements, returning None if all elements are filtered out.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::nonempty::NonEmpty;
    ///
    /// let nel = NonEmpty::new(1, vec![2, 3, 4]);
    /// let evens = nel.filter(|x| x % 2 == 0);
    /// assert_eq!(evens.map(|n| n.to_vec()), Some(vec![2, 4]));
    ///
    /// let nel2 = NonEmpty::new(1, vec![3, 5]);
    /// let evens2 = nel2.filter(|x| x % 2 == 0);
    /// assert!(evens2.is_none());
    /// ```
    pub fn filter<F>(self, mut pred: F) -> Option<NonEmpty<T>>
    where
        F: FnMut(&T) -> bool,
    {
        // Consume `self` into a single Vec instead of `to_vec().into_iter()
        // .filter().collect()`, which allocated two Vecs (the full-size
        // intermediate from to_vec, then the filtered result). One allocation,
        // no clones, sized to the upstream length.
        let mut all = Vec::with_capacity(1 + self.tail.len());
        if pred(&self.head) {
            all.push(self.head);
        }
        for x in self.tail {
            if pred(&x) {
                all.push(x);
            }
        }
        NonEmpty::from_vec(all)
    }

    /// `FlatMap` over the `NonEmpty`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::nonempty::NonEmpty;
    ///
    /// let nel = NonEmpty::new(1, vec![2]);
    /// let expanded = nel.flat_map(|x| NonEmpty::new(x, vec![x * 10]));
    /// assert_eq!(expanded.to_vec(), vec![1, 10, 2, 20]);
    /// ```
    #[inline]
    pub fn flat_map<U, F>(self, mut f: F) -> NonEmpty<U>
    where
        F: FnMut(T) -> NonEmpty<U>,
    {
        let first = f(self.head);
        // Stream tail results directly into the accumulator without an
        // intermediate Vec<NonEmpty<U>> — halves the number of allocations.
        let head = first.head;
        let mut tail = first.tail;
        for item in self.tail {
            let nel = f(item);
            tail.reserve(1 + nel.tail.len());
            tail.push(nel.head);
            tail.extend(nel.tail);
        }
        NonEmpty { head, tail }
    }

    // ========================================================================
    // Comonad operations
    // ========================================================================

    /// Extract the focused element (Comonad extract).
    ///
    /// For `NonEmpty`, this returns the head element. This is dual to
    /// Monad's `pure`/`return`.
    ///
    /// # Laws
    /// - `extract(duplicate(w)) == w`
    /// - `w.extend(extract) == w`
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::nonempty::NonEmpty;
    ///
    /// let nel = NonEmpty::new(1, vec![2, 3]);
    /// assert_eq!(nel.extract(), &1);
    /// ```
    #[inline]
    pub fn extract(&self) -> &T {
        &self.head
    }

    /// Duplicate the structure (Comonad duplicate).
    ///
    /// Creates a `NonEmpty` of `NonEmptys` where each position focuses on
    /// a different element. Each sub-NonEmpty starts at a different
    /// position in the original list (all suffixes including the original).
    ///
    /// # Laws
    /// - `duplicate(duplicate(w)) == duplicate(w).map(duplicate)`
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::nonempty::NonEmpty;
    ///
    /// let nel = NonEmpty::new(1, vec![2, 3]);
    /// let dup = nel.duplicate();
    ///
    /// // First element is the whole list focused on 1
    /// assert_eq!(dup.head().as_vec(), vec![1, 2, 3]);
    /// // Second element is the suffix starting at 2
    /// assert_eq!(dup.tail()[0].as_vec(), vec![2, 3]);
    /// // Third element is just [3]
    /// assert_eq!(dup.tail()[1].as_vec(), vec![3]);
    /// ```
    ///
    /// # Panics
    ///
    /// Panics only if the non-emptiness invariant is violated (each
    /// `unwrap` wraps a suffix taken from a loop that only runs while
    /// the slice is non-empty); reaching it would indicate a bug in
    /// this crate.
    pub fn duplicate(&self) -> NonEmpty<NonEmpty<T>>
    where
        T: Clone,
    {
        // Generate all suffixes (tails) of the list
        let mut suffixes = Vec::with_capacity(self.tail.len());
        let mut current_tail = self.tail.as_slice();

        while !current_tail.is_empty() {
            let suffix = NonEmpty::from_vec(current_tail.to_vec()).unwrap();
            suffixes.push(suffix);
            current_tail = &current_tail[1..];
        }

        NonEmpty {
            head: self.clone(),
            tail: suffixes,
        }
    }

    /// Extend a function over the structure (Comonad extend).
    ///
    /// Applies a function that extracts a value from each "focus position"
    /// of the `NonEmpty`. This is dual to Monad's `flat_map`/`bind`.
    ///
    /// # Laws
    /// - `w.extend(f).extend(g) == w.extend(|wa| g(&wa.extend(f)))`
    /// - `w.extend(extract) == w`
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::nonempty::NonEmpty;
    ///
    /// let nel = NonEmpty::new(1, vec![2, 3, 4]);
    ///
    /// // Sum of each suffix
    /// let sums = nel.extend(|suffix| suffix.fold_ref(0, |acc, x| acc + x));
    /// assert_eq!(sums.to_vec(), vec![10, 9, 7, 4]);
    /// // 10 = 1+2+3+4, 9 = 2+3+4, 7 = 3+4, 4 = 4
    /// ```
    pub fn extend<U, F>(&self, f: F) -> NonEmpty<U>
    where
        T: Clone,
        F: Fn(&NonEmpty<T>) -> U,
    {
        self.duplicate().map(|nel| f(&nel))
    }

    /// Coflatmap is an alias for extend (following Haskell conventions).
    #[inline]
    pub fn coflatmap<U, F>(&self, f: F) -> NonEmpty<U>
    where
        T: Clone,
        F: Fn(&NonEmpty<T>) -> U,
    {
        self.extend(f)
    }

    // ========================================================================
    // Zipper-style operations
    // ========================================================================

    /// Create all rotations of the list.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::nonempty::NonEmpty;
    ///
    /// let nel = NonEmpty::new(1, vec![2, 3]);
    /// let rotations = nel.rotations();
    ///
    /// assert_eq!(rotations.head().as_vec(), vec![1, 2, 3]);
    /// assert_eq!(rotations.tail()[0].as_vec(), vec![2, 3, 1]);
    /// assert_eq!(rotations.tail()[1].as_vec(), vec![3, 1, 2]);
    /// ```
    ///
    /// # Panics
    ///
    /// Panics only if the non-emptiness invariant is violated (each
    /// `unwrap` wraps a rotation of length `len() >= 1`); reaching it
    /// would indicate a bug in this crate.
    pub fn rotations(&self) -> NonEmpty<NonEmpty<T>>
    where
        T: Clone,
    {
        let n = self.len();
        let all_items = self.as_vec();

        let mut rotations = Vec::with_capacity(n - 1);
        for i in 1..n {
            let mut rotated = Vec::with_capacity(n);
            for j in 0..n {
                rotated.push(all_items[(i + j) % n].clone());
            }
            rotations.push(NonEmpty::from_vec(rotated).unwrap());
        }

        NonEmpty {
            head: self.clone(),
            tail: rotations,
        }
    }

    /// Apply a function that uses surrounding context.
    ///
    /// For each position, provides access to: (elements before, current, elements after).
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::nonempty::NonEmpty;
    ///
    /// let nel = NonEmpty::new(1, vec![2, 3]);
    /// let with_neighbors = nel.with_context(|before, current, after| {
    ///     format!("{:?} < {} > {:?}", before, current, after)
    /// });
    ///
    /// assert_eq!(with_neighbors.head(), "[] < 1 > [2, 3]");
    /// ```
    pub fn with_context<U, F>(&self, f: F) -> NonEmpty<U>
    where
        T: Clone,
        F: Fn(&[T], &T, &[T]) -> U,
    {
        let all = self.as_vec();
        let n = all.len();

        let head_result = f(&[], &all[0], &all[1..]);
        let tail_results: Vec<U> = (1..n)
            .map(|i| f(&all[..i], &all[i], &all[i + 1..]))
            .collect();

        NonEmpty {
            head: head_result,
            tail: tail_results,
        }
    }
}

#[cfg(feature = "alloc")]
impl<T> IntoIterator for NonEmpty<T> {
    type Item = T;
    type IntoIter = alloc::vec::IntoIter<T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        // Build the Vec directly here rather than going through to_vec() so
        // that the compiler sees a single allocation path and can inline/elide it.
        let mut v = Vec::with_capacity(1 + self.tail.len());
        v.push(self.head);
        v.extend(self.tail);
        v.into_iter()
    }
}

#[cfg(feature = "alloc")]
impl<'a, T> IntoIterator for &'a NonEmpty<T> {
    type Item = &'a T;
    type IntoIter = core::iter::Chain<core::iter::Once<&'a T>, core::slice::Iter<'a, T>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        core::iter::once(&self.head).chain(self.tail.iter())
    }
}

/// Macro for creating `NonEmpty` lists.
///
/// # Example
///
/// ```rust
/// use ordofp_core::{nonempty, nonempty::NonEmpty};
///
/// let nel = nonempty![1, 2, 3];
/// assert_eq!(nel.as_vec(), vec![1, 2, 3]);
///
/// let single = nonempty![42];
/// assert_eq!(single.head(), &42);
/// ```
#[macro_export]
macro_rules! nonempty {
    ($head:expr) => {
        $crate::nonempty::NonEmpty::singleton($head)
    };
    ($head:expr, $($tail:expr),+ $(,)?) => {
        $crate::nonempty::NonEmpty::new($head, $crate::__alloc::vec![$($tail),+])
    };
}

// ============================================================================
// Typeclass implementations
// ============================================================================

#[cfg(feature = "alloc")]
impl<T> crate::typeclasses::Functor for NonEmpty<T> {
    type Inner = T;
    type Target<U> = NonEmpty<U>;

    #[inline]
    fn map<B, F>(self, mut f: F) -> NonEmpty<B>
    where
        F: FnMut(T) -> B,
    {
        let tail_len = self.tail.len();
        NonEmpty {
            head: f(self.head),
            tail: {
                let mut v = Vec::with_capacity(tail_len);
                v.extend(self.tail.into_iter().map(f));
                v
            },
        }
    }
}

#[cfg(feature = "alloc")]
impl<T: Clone> crate::typeclasses::Apply for NonEmpty<T> {
    fn apply<B, F>(self, ff: NonEmpty<F>) -> NonEmpty<B>
    where
        NonEmpty<F>: crate::typeclasses::Apply<Inner = F, Target<B> = NonEmpty<B>>,
        F: FnMut(T) -> B,
    {
        // Cartesian product application (same semantics as Vec)
        let first_result = {
            let mut f = ff.head;
            let head = f(self.head.clone());
            let tail: Vec<B> = self.tail.iter().map(|a| f(a.clone())).collect();
            NonEmpty { head, tail }
        };
        let mut result = first_result;
        for mut f in ff.tail {
            result.tail.push(f(self.head.clone()));
            for a in &self.tail {
                result.tail.push(f(a.clone()));
            }
        }
        result
    }
}

#[cfg(feature = "alloc")]
impl<T: Clone> crate::typeclasses::Applicatio for NonEmpty<T> {
    fn pure(a: T) -> Self {
        NonEmpty::singleton(a)
    }

    fn pure_target<U>(u: U) -> NonEmpty<U>
    where
        NonEmpty<U>: crate::typeclasses::Applicatio<Inner = U>,
    {
        NonEmpty::singleton(u)
    }
}

#[cfg(feature = "alloc")]
impl<T: Clone> crate::typeclasses::Monad for NonEmpty<T> {
    #[inline]
    fn flat_map<B, F>(self, mut f: F) -> NonEmpty<B>
    where
        F: FnMut(T) -> NonEmpty<B>,
    {
        let first = f(self.head);
        let head = first.head;
        let mut tail = first.tail;
        for item in self.tail {
            let nel = f(item);
            tail.reserve(1 + nel.tail.len());
            tail.push(nel.head);
            tail.extend(nel.tail);
        }
        NonEmpty { head, tail }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_new() {
        let nel = NonEmpty::new(1, vec![2, 3]);
        assert_eq!(nel.head(), &1);
        assert_eq!(nel.tail(), &[2, 3]);
    }

    #[test]
    fn test_singleton() {
        let nel = NonEmpty::singleton(42);
        assert_eq!(nel.head(), &42);
        assert!(nel.tail().is_empty());
        assert!(nel.is_singleton());
    }

    #[test]
    fn test_from_vec() {
        let nel = NonEmpty::from_vec(vec![1, 2, 3]).expect("non-empty vec should produce Some");
        assert_eq!(nel.head(), &1);
        assert_eq!(nel.tail(), &[2, 3]);

        let empty = NonEmpty::<i32>::from_vec(vec![]);
        assert!(empty.is_none());
    }

    #[test]
    fn test_len() {
        let nel = NonEmpty::new(1, vec![2, 3, 4]);
        assert_eq!(nel.len(), 4);

        let single = NonEmpty::singleton(1);
        assert_eq!(single.len(), 1);
    }

    #[test]
    fn test_last() {
        let nel = NonEmpty::new(1, vec![2, 3]);
        assert_eq!(nel.last(), &3);

        let single = NonEmpty::singleton(42);
        assert_eq!(single.last(), &42);
    }

    #[test]
    fn test_to_vec() {
        let nel = NonEmpty::new(1, vec![2, 3]);
        assert_eq!(nel.to_vec(), vec![1, 2, 3]);
    }

    #[test]
    fn test_push() {
        let nel = NonEmpty::singleton(1).push(2).push(3);
        assert_eq!(nel.to_vec(), vec![1, 2, 3]);
    }

    #[test]
    fn test_prepend() {
        let nel = NonEmpty::singleton(3).prepend(2).prepend(1);
        assert_eq!(nel.to_vec(), vec![1, 2, 3]);
    }

    #[test]
    fn test_map() {
        let nel = NonEmpty::new(1, vec![2, 3]);
        let doubled = nel.map(|x| x * 2);
        assert_eq!(doubled.to_vec(), vec![2, 4, 6]);
    }

    #[test]
    fn test_fold() {
        let nel = NonEmpty::new(1, vec![2, 3, 4]);
        let sum = nel.fold(0, |acc, x| acc + x);
        assert_eq!(sum, 10);
    }

    #[test]
    fn test_reduce() {
        let nel = NonEmpty::new(1, vec![2, 3, 4]);
        let sum = nel.reduce(|acc, x| acc + x);
        assert_eq!(sum, 10);
    }

    #[test]
    fn test_reduce_singleton() {
        // A singleton's reduce must return the head without ever invoking the
        // combining function (the tail is empty, so the fold is a no-op).
        let nel = NonEmpty::singleton(42);
        let result = nel.reduce(|_acc, _x| panic!("combiner must not be called on singleton"));
        assert_eq!(result, 42);
    }

    #[test]
    fn test_concat() {
        let a = NonEmpty::new(1, vec![2]);
        let b = NonEmpty::new(3, vec![4]);
        let combined = a.concat(b);
        assert_eq!(combined.to_vec(), vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_reverse() {
        let nel = NonEmpty::new(1, vec![2, 3]);
        let reversed = nel.reverse();
        assert_eq!(reversed.to_vec(), vec![3, 2, 1]);
    }

    #[test]
    fn test_get() {
        let nel = NonEmpty::new(1, vec![2, 3]);
        assert_eq!(nel.get(0), Some(&1));
        assert_eq!(nel.get(1), Some(&2));
        assert_eq!(nel.get(2), Some(&3));
        assert_eq!(nel.get(3), None);
    }

    #[test]
    fn test_filter() {
        let nel = NonEmpty::new(1, vec![2, 3, 4]);
        let evens = nel.filter(|x| x % 2 == 0);
        assert_eq!(evens.map(super::NonEmpty::to_vec), Some(vec![2, 4]));
    }

    #[test]
    fn test_filter_all_out() {
        let nel = NonEmpty::new(1, vec![3, 5]);
        let evens = nel.filter(|x| x % 2 == 0);
        assert!(evens.is_none());
    }

    #[test]
    fn test_flat_map() {
        let nel = NonEmpty::new(1, vec![2]);
        let expanded = nel.flat_map(|x| NonEmpty::new(x, vec![x * 10]));
        assert_eq!(expanded.to_vec(), vec![1, 10, 2, 20]);
    }

    #[test]
    fn test_iter() {
        let nel = NonEmpty::new(1, vec![2, 3]);
        let collected: Vec<_> = nel.iter().copied().collect();
        assert_eq!(collected, vec![1, 2, 3]);
    }

    #[test]
    fn test_into_iter() {
        let nel = NonEmpty::new(1, vec![2, 3]);
        let collected: Vec<_> = nel.into_iter().collect();
        assert_eq!(collected, vec![1, 2, 3]);
    }

    // Comonad tests

    #[test]
    fn test_extract() {
        let nel = NonEmpty::new(1, vec![2, 3]);
        assert_eq!(nel.extract(), &1);

        let single = NonEmpty::singleton(42);
        assert_eq!(single.extract(), &42);
    }

    #[test]
    fn test_duplicate() {
        let nel = NonEmpty::new(1, vec![2, 3]);
        let dup = nel.duplicate();

        assert_eq!(dup.len(), 3);
        assert_eq!(dup.head().as_vec(), vec![1, 2, 3]);
        assert_eq!(dup.tail()[0].as_vec(), vec![2, 3]);
        assert_eq!(dup.tail()[1].as_vec(), vec![3]);
    }

    #[test]
    fn test_duplicate_singleton() {
        let nel = NonEmpty::singleton(42);
        let dup = nel.duplicate();

        assert_eq!(dup.len(), 1);
        assert_eq!(dup.head().as_vec(), vec![42]);
    }

    #[test]
    fn test_extend() {
        let nel = NonEmpty::new(1, vec![2, 3, 4]);

        // Sum of each suffix
        let sums = nel.extend(|suffix| suffix.fold_ref(0, |acc, x| acc + x));
        assert_eq!(sums.to_vec(), vec![10, 9, 7, 4]);
    }

    #[test]
    fn test_extend_head() {
        let nel = NonEmpty::new(1, vec![2, 3]);

        // Get head of each suffix
        let heads = nel.extend(|suffix| *suffix.head());
        assert_eq!(heads.to_vec(), vec![1, 2, 3]);
    }

    #[test]
    fn test_comonad_law_extract_duplicate() {
        // Law: extract(duplicate(w)) == w
        let nel = NonEmpty::new(1, vec![2, 3]);
        let dup = nel.duplicate();
        assert_eq!(dup.extract().as_vec(), nel.as_vec());
    }

    #[test]
    fn test_comonad_law_extend_extract() {
        // Law: w.extend(extract) == w
        let nel = NonEmpty::new(1, vec![2, 3]);
        let extended = nel.extend(|w| *w.extract());
        assert_eq!(extended.to_vec(), vec![1, 2, 3]);
    }

    #[test]
    fn test_rotations() {
        let nel = NonEmpty::new(1, vec![2, 3]);
        let rotations = nel.rotations();

        assert_eq!(rotations.len(), 3);
        assert_eq!(rotations.head().as_vec(), vec![1, 2, 3]);
        assert_eq!(rotations.tail()[0].as_vec(), vec![2, 3, 1]);
        assert_eq!(rotations.tail()[1].as_vec(), vec![3, 1, 2]);
    }

    #[test]
    fn test_with_context() {
        let nel = NonEmpty::new(1, vec![2, 3]);
        let contexts =
            nel.with_context(|before, current, after| (before.len(), *current, after.len()));

        assert_eq!(
            contexts.to_vec(),
            vec![
                (0, 1, 2), // [] < 1 > [2, 3]
                (1, 2, 1), // [1] < 2 > [3]
                (2, 3, 0), // [1, 2] < 3 > []
            ]
        );
    }

    #[test]
    fn test_with_context_singleton() {
        // A singleton has no neighbors: both before and after slices must be empty.
        let nel = NonEmpty::singleton(42);
        let contexts =
            nel.with_context(|before, current, after| (before.to_vec(), *current, after.to_vec()));

        assert_eq!(contexts.len(), 1);
        let (before, val, after) = contexts.head().clone();
        assert_eq!(before, Vec::<i32>::new());
        assert_eq!(val, 42);
        assert_eq!(after, Vec::<i32>::new());
    }

    #[test]
    fn test_rotations_singleton() {
        // A singleton has exactly one rotation (itself). The inner loop
        // `for i in 1..n` never executes when n == 1, so the result must
        // still be a valid NonEmpty containing only the original element.
        let nel = NonEmpty::singleton(7);
        let rotations = nel.rotations();
        assert_eq!(rotations.len(), 1);
        assert_eq!(rotations.head().as_vec(), vec![7]);
        assert!(rotations.tail().is_empty());
    }

    #[test]
    fn test_rotations_two_elements() {
        // Two-element list is the minimal non-trivial case: the inner loop
        // `for i in 1..n` executes exactly once, so we get exactly 2 rotations.
        // Verifies no off-by-one error in the loop bound.
        let nel = NonEmpty::new(1, vec![2]);
        let rotations = nel.rotations();
        assert_eq!(rotations.len(), 2);
        assert_eq!(rotations.head().as_vec(), vec![1, 2]);
        assert_eq!(rotations.tail()[0].as_vec(), vec![2, 1]);
    }

    #[test]
    fn test_filter_only_head_passes() {
        // Edge case: head passes the predicate but all tail elements are filtered
        // out. The result must be Some(singleton) with the original head value.
        let nel = NonEmpty::new(2, vec![3, 5, 7]);
        let result = nel.filter(|x| x % 2 == 0);
        let inner = result.expect("filter should return Some when head passes");
        assert_eq!(inner.head(), &2);
        assert!(
            inner.is_singleton(),
            "only the head should survive the filter"
        );
    }

    #[test]
    fn test_macro_single() {
        let nel = nonempty![42];
        assert_eq!(nel.head(), &42);
        assert!(nel.is_singleton());
    }

    #[test]
    fn test_macro_multiple() {
        let nel = nonempty![1, 2, 3];
        assert_eq!(nel.to_vec(), vec![1, 2, 3]);
    }

    #[test]
    fn test_map_ref_borrows_self() {
        // `map_ref` takes `&self` rather than consuming `self`, so the original
        // NonEmpty must remain usable after the call — unlike `map` which moves.
        let nel = NonEmpty::new(1, vec![2, 3]);
        let doubled = nel.map_ref(|x| x * 2);
        // The original is still accessible because map_ref only borrowed it.
        assert_eq!(nel.head(), &1, "original head must be unchanged");
        assert_eq!(doubled.to_vec(), vec![2, 4, 6]);
    }

    #[test]
    fn test_fold_ref_does_not_consume() {
        // `fold_ref` takes `&self`, so the list is still usable afterwards.
        let nel = NonEmpty::new(1, vec![2, 3, 4]);
        let sum = nel.fold_ref(0, |acc, x| acc + x);
        assert_eq!(sum, 10);
        // Confirm the original list is still intact after folding by reference.
        assert_eq!(nel.len(), 4, "list must not be consumed by fold_ref");
        assert_eq!(nel.head(), &1, "head must be unchanged after fold_ref");
    }

    #[test]
    fn test_coflatmap_matches_extend() {
        // `coflatmap` is specified as an alias for `extend`; both must produce
        // identical results for the same input and function.
        let nel = NonEmpty::new(1, vec![2, 3]);
        let sum = |w: &NonEmpty<i32>| w.fold_ref(0, |acc, x| acc + x);
        // Use separate closure instances to avoid a use-after-move.
        let via_extend = nel.extend(|w| sum(w));
        let via_coflatmap = nel.coflatmap(|w| sum(w));
        // Concrete values: suffix sums are [6, 5, 3].
        assert_eq!(via_extend.to_vec(), vec![6, 5, 3]);
        assert_eq!(
            via_coflatmap.to_vec(),
            vec![6, 5, 3],
            "coflatmap must return the same result as extend for the same function"
        );
    }

    #[test]
    fn test_head_mut_mutation_is_visible() {
        // `head_mut` returns an exclusive mutable reference to the head element.
        // Mutations through that reference must be reflected in all subsequent
        // reads of the structure — including `head()` and `to_vec()`.
        let mut nel = NonEmpty::new(1, vec![2, 3]);
        *nel.head_mut() = 10;
        assert_eq!(
            nel.head(),
            &10,
            "head() must reflect the mutation made via head_mut()"
        );
        assert_eq!(
            nel.to_vec(),
            vec![10, 2, 3],
            "to_vec() must show the mutated head and unchanged tail"
        );
    }

    #[test]
    fn test_as_vec_borrows_without_consuming() {
        // `as_vec` clones elements into a Vec without consuming `self`,
        // so the original must remain usable after the call.
        // Edge case: singleton — as_vec must yield a one-element Vec.
        let single = NonEmpty::singleton(7);
        assert_eq!(
            single.as_vec(),
            vec![7],
            "as_vec on a singleton must return a one-element Vec"
        );
        // `single` is still live — a second call must return an identical Vec.
        assert_eq!(
            single.as_vec(),
            vec![7],
            "as_vec must not consume self; calling it twice must give the same result"
        );

        // Multi-element case: full list must be reproduced in order.
        let nel = NonEmpty::new(1, vec![2, 3]);
        assert_eq!(
            nel.as_vec(),
            vec![1, 2, 3],
            "as_vec must return head followed by tail elements in order"
        );
        // Original is still accessible after as_vec.
        assert_eq!(
            nel.head(),
            &1,
            "as_vec must not mutate or consume the original"
        );
    }

    #[test]
    fn test_filter_head_filtered_out_tail_survives() {
        // Edge case: the head element fails the predicate but some tail elements
        // pass. The result must be Some, and the new head must be the first
        // surviving tail element — not the original head.
        let nel = NonEmpty::new(1, vec![2, 3, 4]);
        let result = nel.filter(|x| x % 2 == 0);
        let inner = result.expect("filter should return Some when tail elements survive");
        assert_eq!(
            inner.head(),
            &2,
            "new head must be the first surviving tail element"
        );
        assert_eq!(
            inner.to_vec(),
            vec![2, 4],
            "surviving elements must be in original order"
        );
    }
}
