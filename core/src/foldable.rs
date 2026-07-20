//! Foldable type class for data structures that can be folded.
//!
//! A `Foldable` represents a data structure that can be reduced to a single
//! value by combining its elements.
//!
//! # Example
//!
//! ```rust
//! use ordofp_core::foldable::Foldable;
//!
//! let v = vec![1, 2, 3, 4, 5];
//!
//! // Left fold (reduce from left to right)
//! let sum = v.fold_left(0, |acc, x| acc + x);
//! assert_eq!(sum, 15);
//!
//! // Right fold (reduce from right to left)
//! let list = v.fold_right(Vec::new(), |x, mut acc| {
//!     acc.insert(0, *x);
//!     acc
//! });
//! assert_eq!(list, vec![1, 2, 3, 4, 5]);
//! ```

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// A type class for data structures that can be folded.
///
/// Foldable provides two fundamental operations:
/// - `fold_left`: Reduces elements from left to right
/// - `fold_right`: Reduces elements from right to left
///
/// Many other operations can be derived from these two.
pub trait Foldable {
    /// The element type contained in this foldable structure.
    type Elem;

    /// Left-associative fold of a structure.
    ///
    /// Reduces the structure from left to right using the combining function.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::foldable::Foldable;
    ///
    /// let v = vec![1, 2, 3];
    /// let result = v.fold_left(0, |acc, x| acc + x);
    /// assert_eq!(result, 6);
    /// ```
    fn fold_left<B, F>(&self, init: B, f: F) -> B
    where
        F: FnMut(B, &Self::Elem) -> B;

    /// Right-associative fold of a structure.
    ///
    /// Reduces the structure from right to left using the combining function.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::foldable::Foldable;
    ///
    /// let v = vec!["a", "b", "c"];
    /// let result = v.fold_right(String::new(), |x, acc| format!("{}{}", x, acc));
    /// assert_eq!(result, "abc");
    /// ```
    fn fold_right<B, F>(&self, init: B, f: F) -> B
    where
        F: FnMut(&Self::Elem, B) -> B;

    /// Returns `true` if the structure is empty.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::foldable::Foldable;
    ///
    /// let empty: Vec<i32> = vec![];
    /// let non_empty = vec![1, 2, 3];
    ///
    /// assert!(empty.is_empty_foldable());
    /// assert!(!non_empty.is_empty_foldable());
    /// ```
    #[inline]
    fn is_empty_foldable(&self) -> bool {
        self.fold_left(true, |_, _| false)
    }

    /// Returns the number of elements in the structure.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::foldable::Foldable;
    ///
    /// let v = vec![1, 2, 3, 4, 5];
    /// assert_eq!(v.length(), 5);
    /// ```
    #[inline]
    fn length(&self) -> usize {
        self.fold_left(0, |acc, _| acc + 1)
    }

    /// Returns `true` if all elements satisfy the predicate.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::foldable::Foldable;
    ///
    /// let v = vec![2, 4, 6, 8];
    /// assert!(v.all(|x| x % 2 == 0));
    /// assert!(!v.all(|x| *x > 5));
    /// ```
    #[inline]
    fn all<F>(&self, mut pred: F) -> bool
    where
        F: FnMut(&Self::Elem) -> bool,
    {
        self.fold_left(true, |acc, x| acc && pred(x))
    }

    /// Returns `true` if any element satisfies the predicate.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::foldable::Foldable;
    ///
    /// let v = vec![1, 2, 3, 4, 5];
    /// assert!(v.any(|x| *x > 3));
    /// assert!(!v.any(|x| *x > 10));
    /// ```
    #[inline]
    fn any<F>(&self, mut pred: F) -> bool
    where
        F: FnMut(&Self::Elem) -> bool,
    {
        self.fold_left(false, |acc, x| acc || pred(x))
    }

    /// Returns `true` if the structure contains an element equal to `elem`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::foldable::Foldable;
    ///
    /// let v = vec![1, 2, 3, 4, 5];
    /// assert!(v.contains_elem(&3));
    /// assert!(!v.contains_elem(&10));
    /// ```
    #[inline]
    fn contains_elem(&self, elem: &Self::Elem) -> bool
    where
        Self::Elem: PartialEq,
    {
        self.any(|x| x == elem)
    }

    /// Finds the first element satisfying a predicate.
    ///
    /// Note: Default implementation uses `fold_left` with cloning.
    /// Types with iteration can provide more efficient implementations.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::foldable::Foldable;
    ///
    /// let v = vec![1, 2, 3, 4, 5];
    /// assert_eq!(v.find_elem(|x| *x > 3), Some(4));
    /// assert_eq!(v.find_elem(|x| *x > 10), None);
    /// ```
    #[inline]
    fn find_elem<F>(&self, mut pred: F) -> Option<Self::Elem>
    where
        Self::Elem: Clone,
        F: FnMut(&Self::Elem) -> bool,
    {
        self.fold_left(None, |acc, x| {
            if acc.is_some() {
                acc
            } else if pred(x) {
                Some(x.clone())
            } else {
                None
            }
        })
    }

    /// Finds the maximum element.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::foldable::Foldable;
    ///
    /// let v = vec![3, 1, 4, 1, 5];
    /// assert_eq!(v.maximum(), Some(5));
    ///
    /// let empty: Vec<i32> = vec![];
    /// assert_eq!(empty.maximum(), None);
    /// ```
    #[inline]
    fn maximum(&self) -> Option<Self::Elem>
    where
        Self::Elem: Ord + Clone,
    {
        self.fold_left(None, |acc: Option<Self::Elem>, x| match acc {
            None => Some(x.clone()),
            Some(max) => Some(if x > &max { x.clone() } else { max }),
        })
    }

    /// Finds the minimum element.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::foldable::Foldable;
    ///
    /// let v = vec![3, 1, 4, 1, 5];
    /// assert_eq!(v.minimum(), Some(1));
    ///
    /// let empty: Vec<i32> = vec![];
    /// assert_eq!(empty.minimum(), None);
    /// ```
    #[inline]
    fn minimum(&self) -> Option<Self::Elem>
    where
        Self::Elem: Ord + Clone,
    {
        self.fold_left(None, |acc: Option<Self::Elem>, x| match acc {
            None => Some(x.clone()),
            Some(min) => Some(if x < &min { x.clone() } else { min }),
        })
    }

    /// Checks if the structure is sorted in ascending order.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::foldable::Foldable;
    ///
    /// let sorted = vec![1, 2, 3, 4, 5];
    /// assert!(sorted.is_sorted_foldable());
    ///
    /// let unsorted = vec![1, 3, 2, 4, 5];
    /// assert!(!unsorted.is_sorted_foldable());
    /// ```
    #[inline]
    fn is_sorted_foldable(&self) -> bool
    where
        Self::Elem: Ord + Clone,
    {
        self.fold_left((true, None::<Self::Elem>), |(is_sorted, prev), curr| {
            if is_sorted {
                match prev {
                    None => (true, Some(curr.clone())),
                    Some(p) => (&p <= curr, Some(curr.clone())),
                }
            } else {
                (false, Some(curr.clone()))
            }
        })
        .0
    }

    /// Sums all elements using addition.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::foldable::Foldable;
    ///
    /// let v = vec![1, 2, 3, 4, 5];
    /// assert_eq!(v.sum_elems(), 15);
    /// ```
    #[inline]
    fn sum_elems(&self) -> Self::Elem
    where
        Self::Elem: Clone + core::ops::Add<Output = Self::Elem> + Default,
    {
        self.fold_left(Self::Elem::default(), |acc, x| acc + x.clone())
    }

    /// Multiplies all elements.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::foldable::Foldable;
    ///
    /// let v = vec![1, 2, 3, 4];
    /// assert_eq!(v.product_elems(), 24);
    /// ```
    #[inline]
    fn product_elems(&self) -> Self::Elem
    where
        Self::Elem: Clone + core::ops::Mul<Output = Self::Elem> + From<u8>,
    {
        self.fold_left(Self::Elem::from(1u8), |acc, x| acc * x.clone())
    }
}

// Implementation for Vec
#[cfg(feature = "alloc")]
impl<A> Foldable for Vec<A> {
    type Elem = A;

    #[inline]
    fn fold_left<B, F>(&self, init: B, f: F) -> B
    where
        F: FnMut(B, &Self::Elem) -> B,
    {
        self.iter().fold(init, f)
    }

    #[inline]
    fn fold_right<B, F>(&self, init: B, mut f: F) -> B
    where
        F: FnMut(&Self::Elem, B) -> B,
    {
        self.iter().rev().fold(init, |acc, x| f(x, acc))
    }
}

// Implementation for slices
impl<A> Foldable for [A] {
    type Elem = A;

    #[inline]
    fn fold_left<B, F>(&self, init: B, f: F) -> B
    where
        F: FnMut(B, &Self::Elem) -> B,
    {
        self.iter().fold(init, f)
    }

    #[inline]
    fn fold_right<B, F>(&self, init: B, mut f: F) -> B
    where
        F: FnMut(&Self::Elem, B) -> B,
    {
        self.iter().rev().fold(init, |acc, x| f(x, acc))
    }
}

// Implementation for Option
impl<A> Foldable for Option<A> {
    type Elem = A;

    #[inline]
    fn fold_left<B, F>(&self, init: B, mut f: F) -> B
    where
        F: FnMut(B, &Self::Elem) -> B,
    {
        match self {
            Some(x) => f(init, x),
            None => init,
        }
    }

    #[inline]
    fn fold_right<B, F>(&self, init: B, mut f: F) -> B
    where
        F: FnMut(&Self::Elem, B) -> B,
    {
        match self {
            Some(x) => f(x, init),
            None => init,
        }
    }
}

// Implementation for Result
impl<A, E> Foldable for Result<A, E> {
    type Elem = A;

    #[inline]
    fn fold_left<B, F>(&self, init: B, mut f: F) -> B
    where
        F: FnMut(B, &Self::Elem) -> B,
    {
        match self {
            Ok(x) => f(init, x),
            Err(_) => init,
        }
    }

    #[inline]
    fn fold_right<B, F>(&self, init: B, mut f: F) -> B
    where
        F: FnMut(&Self::Elem, B) -> B,
    {
        match self {
            Ok(x) => f(x, init),
            Err(_) => init,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_vec_fold_left() {
        let v = vec![1, 2, 3, 4, 5];
        let sum = v.fold_left(0, |acc, x| acc + x);
        assert_eq!(sum, 15);
    }

    #[test]
    fn test_vec_fold_right() {
        let v = vec![1, 2, 3];
        // Building a string right-to-left
        let result = v.fold_right(alloc::string::String::new(), |x, acc| {
            alloc::format!("{x}{acc}")
        });
        assert_eq!(result, "123");
    }

    #[test]
    fn test_vec_is_empty() {
        let empty: Vec<i32> = vec![];
        let non_empty = vec![1, 2, 3];
        assert!(empty.is_empty_foldable());
        assert!(!non_empty.is_empty_foldable());
    }

    #[test]
    fn test_vec_length() {
        let v = vec![1, 2, 3, 4, 5];
        assert_eq!(v.length(), 5);

        let empty: Vec<i32> = vec![];
        assert_eq!(empty.length(), 0);
    }

    #[test]
    fn test_vec_all() {
        let v = vec![2, 4, 6, 8];
        assert!(v.all(|x| x % 2 == 0));
        assert!(!v.all(|x| *x > 5));

        let empty: Vec<i32> = vec![];
        assert!(empty.all(|_| false)); // vacuously true
    }

    #[test]
    fn test_vec_any() {
        let v = vec![1, 2, 3, 4, 5];
        assert!(v.any(|x| *x > 3));
        assert!(!v.any(|x| *x > 10));

        let empty: Vec<i32> = vec![];
        assert!(!empty.any(|_| true));
    }

    #[test]
    fn test_vec_contains_elem() {
        let v = vec![1, 2, 3, 4, 5];
        assert!(v.contains_elem(&3));
        assert!(!v.contains_elem(&10));
    }

    #[test]
    fn test_slice_fold_left() {
        let arr = [1, 2, 3, 4, 5];
        let sum = arr.fold_left(0, |acc, x| acc + x);
        assert_eq!(sum, 15);
    }

    #[test]
    fn test_option_fold_left() {
        let some = Some(5);
        let none: Option<i32> = None;

        assert_eq!(some.fold_left(0, |acc, x| acc + x), 5);
        assert_eq!(none.fold_left(0, |acc, x| acc + x), 0);
    }

    #[test]
    fn test_option_fold_right() {
        let some = Some(5);
        let none: Option<i32> = None;

        assert_eq!(some.fold_right(10, |x, acc| x + acc), 15);
        assert_eq!(none.fold_right(10, |x, acc| x + acc), 10);
    }

    #[test]
    fn test_option_is_empty() {
        let some = Some(5);
        let none: Option<i32> = None;

        assert!(!some.is_empty_foldable());
        assert!(none.is_empty_foldable());
    }

    #[test]
    fn test_result_fold_left() {
        let ok: Result<i32, &str> = Ok(5);
        let err: Result<i32, &str> = Err("error");

        assert_eq!(ok.fold_left(0, |acc, x| acc + x), 5);
        assert_eq!(err.fold_left(0, |acc, x| acc + x), 0);
    }

    #[test]
    fn test_result_fold_right() {
        let ok: Result<i32, &str> = Ok(5);
        let err: Result<i32, &str> = Err("error");

        assert_eq!(ok.fold_right(10, |x, acc| x + acc), 15);
        assert_eq!(err.fold_right(10, |x, acc| x + acc), 10);
    }

    #[test]
    fn test_fold_left_vs_right_order() {
        let v = vec![1, 2, 3];

        // Left fold: ((init - 1) - 2) - 3
        let left = v.fold_left(0, |acc, x| acc - x);
        assert_eq!(left, -6); // 0 - 1 - 2 - 3 = -6

        // Right fold: 1 - (2 - (3 - init))
        let right = v.fold_right(0, |x, acc| x - acc);
        assert_eq!(right, 2); // 1 - (2 - (3 - 0)) = 1 - (2 - 3) = 1 - (-1) = 2
    }

    #[test]
    fn test_product() {
        let v = vec![1, 2, 3, 4, 5];
        let product = v.fold_left(1, |acc, x| acc * x);
        assert_eq!(product, 120);
    }

    #[test]
    fn test_max() {
        let v = vec![3, 1, 4, 1, 5, 9, 2, 6];
        let max = v.fold_left(i32::MIN, |acc, &x| if x > acc { x } else { acc });
        assert_eq!(max, 9);
    }

    #[test]
    fn test_min() {
        let v = vec![3, 1, 4, 1, 5, 9, 2, 6];
        let min = v.fold_left(i32::MAX, |acc, &x| if x < acc { x } else { acc });
        assert_eq!(min, 1);
    }

    #[test]
    fn test_find_elem() {
        let v = vec![1, 2, 3, 4, 5];
        assert_eq!(v.find_elem(|x| *x > 3), Some(4));
        assert_eq!(v.find_elem(|x| *x > 10), None);

        let empty: Vec<i32> = vec![];
        assert_eq!(empty.find_elem(|_| true), None);
    }

    #[test]
    fn test_maximum() {
        let v = vec![3, 1, 4, 1, 5, 9, 2, 6];
        assert_eq!(v.maximum(), Some(9));

        let empty: Vec<i32> = vec![];
        assert_eq!(empty.maximum(), None);
    }

    #[test]
    fn test_minimum() {
        let v = vec![3, 1, 4, 1, 5, 9, 2, 6];
        assert_eq!(v.minimum(), Some(1));

        let empty: Vec<i32> = vec![];
        assert_eq!(empty.minimum(), None);
    }

    #[test]
    fn test_is_sorted_foldable() {
        let sorted = vec![1, 2, 3, 4, 5];
        assert!(sorted.is_sorted_foldable());

        let unsorted = vec![1, 3, 2, 4, 5];
        assert!(!unsorted.is_sorted_foldable());

        let empty: Vec<i32> = vec![];
        assert!(empty.is_sorted_foldable());

        let single = vec![42];
        assert!(single.is_sorted_foldable());
    }

    #[test]
    fn test_sum_elems() {
        let v = vec![1, 2, 3, 4, 5];
        assert_eq!(v.sum_elems(), 15);

        let empty: Vec<i32> = vec![];
        assert_eq!(empty.sum_elems(), 0);
    }

    #[test]
    fn test_product_elems() {
        let v = vec![1, 2, 3, 4];
        assert_eq!(v.product_elems(), 24);

        let empty: Vec<i32> = vec![];
        assert_eq!(empty.product_elems(), 1);
    }
}
