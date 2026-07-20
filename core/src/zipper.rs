//! Zipper: A cursor into a list with cheap focus operations
//!
//! A Zipper is a functional data structure that provides efficient
//! focus-based navigation and modification. It represents a position
//! within a data structure with direct access at the cursor.
//!
//! Based on Huet's original paper "Functional Pearl: The Zipper" (1997)
//! and `XMonad`'s `StackSet` implementation.
//!
//! # Complexity note
//!
//! This implementation is `Vec`-backed. Reading/replacing the focus is
//! O(1), and operations touching only the *left* list (`push`/`pop` at its
//! end) are O(1) — but operations that touch the front of the *right* list
//! (`focus_next`, `focus_prev`, `swap_right`, …) use `remove(0)` /
//! `insert(0)` and are therefore **O(n)** in the number of elements to the
//! right, unlike the O(1) of a classical two-stack zipper.
//!
//! # Example
//!
//! ```
//! use ordofp_core::zipper::Zipper;
//!
//! // Create a zipper with focus on 2
//! let z = Zipper::new(2, vec![1], vec![3, 4]);
//!
//! assert_eq!(z.focus(), &2);
//! assert_eq!(z.clone().to_vec(), vec![1, 2, 3, 4]);
//!
//! // Move focus
//! let z = z.focus_next().unwrap();
//! assert_eq!(z.focus(), &3);
//!
//! let z = z.focus_prev().unwrap();
//! assert_eq!(z.focus(), &2);
//! ```

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A Zipper is a cursor into a non-empty sequence.
///
/// It tracks:
/// - `focus`: the currently focused element
/// - `left`: elements to the left, in **natural order** (the immediate left
///   neighbor is the *last* element, so it pops off the end in O(1))
/// - `right`: elements to the right, in natural order (the immediate right
///   neighbor is the *first* element — accessing it uses `remove(0)`, which
///   is O(n); see the module-level complexity note)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg(feature = "alloc")]
pub struct Zipper<A> {
    /// The focused element
    focus: A,
    /// Elements to the left, in natural order (nearest neighbor last)
    left: Vec<A>,
    /// Elements to the right, in natural order (nearest neighbor first)
    right: Vec<A>,
}

#[cfg(feature = "alloc")]
impl<A> Zipper<A> {
    /// Create a new Zipper with the given focus, left elements (in order),
    /// and right elements.
    ///
    /// # Example
    ///
    /// ```
    /// use ordofp_core::zipper::Zipper;
    ///
    /// // Creates: [1] <2> [3, 4] where <2> is focused
    /// let z = Zipper::new(2, vec![1], vec![3, 4]);
    /// assert_eq!(z.focus(), &2);
    /// ```
    #[inline]
    pub fn new(focus: A, left: Vec<A>, right: Vec<A>) -> Self {
        // Left is stored with nearest neighbor at the end for O(1) pop
        // User provides [1, 2] meaning "1 then 2 to the left of focus"
        // We store as-is because pop() from [1, 2] gives us 2 (nearest)
        Zipper { focus, left, right }
    }

    /// Create a Zipper from a non-empty slice, focusing on the first element.
    ///
    /// Returns `None` if the slice is empty.
    ///
    /// # Example
    ///
    /// ```
    /// use ordofp_core::zipper::Zipper;
    ///
    /// let z = Zipper::from_slice(&[1, 2, 3]).unwrap();
    /// assert_eq!(z.focus(), &1);
    /// ```
    pub fn from_slice(slice: &[A]) -> Option<Self>
    where
        A: Clone,
    {
        match slice {
            [] => None,
            [first, rest @ ..] => Some(Zipper {
                focus: first.clone(),
                left: Vec::new(),
                // Allocate exactly the capacity needed for the right side.
                right: {
                    let mut v = Vec::with_capacity(rest.len());
                    v.extend_from_slice(rest);
                    v
                },
            }),
        }
    }

    /// Create a Zipper from a Vec, focusing on the first element.
    ///
    /// Returns `None` if the Vec is empty.
    ///
    /// # Example
    ///
    /// ```
    /// use ordofp_core::zipper::Zipper;
    ///
    /// let z = Zipper::from_vec(vec![1, 2, 3]).unwrap();
    /// assert_eq!(z.focus(), &1);
    /// assert_eq!(z.to_vec(), vec![1, 2, 3]);
    /// ```
    pub fn from_vec(mut vec: Vec<A>) -> Option<Self> {
        if vec.is_empty() {
            None
        } else {
            // from_vec is one-time construction (not a hot path); the O(n)
            // shift of remove(0) is fine and keeps `right` in natural order.
            let focus = vec.remove(0);
            Some(Zipper {
                focus,
                left: Vec::new(),
                right: vec,
            })
        }
    }

    /// Get a reference to the focused element.
    ///
    /// # Example
    ///
    /// ```
    /// use ordofp_core::zipper::Zipper;
    ///
    /// let z = Zipper::singleton(42);
    /// assert_eq!(z.focus(), &42);
    /// ```
    #[inline]
    pub fn focus(&self) -> &A {
        &self.focus
    }

    /// Get a mutable reference to the focused element.
    #[inline]
    pub fn focus_mut(&mut self) -> &mut A {
        &mut self.focus
    }

    /// Create a Zipper with a single focused element.
    ///
    /// # Example
    ///
    /// ```
    /// use ordofp_core::zipper::Zipper;
    ///
    /// let z = Zipper::singleton(42);
    /// assert_eq!(z.len(), 1);
    /// ```
    #[inline]
    pub fn singleton(a: A) -> Self {
        Zipper {
            focus: a,
            left: Vec::new(),
            right: Vec::new(),
        }
    }

    /// Move focus to the next (right) element.
    ///
    /// Returns `None` if there are no elements to the right.
    ///
    /// # Example
    ///
    /// ```
    /// use ordofp_core::zipper::Zipper;
    ///
    /// let z = Zipper::from_vec(vec![1, 2, 3]).unwrap();
    /// let z = z.focus_next().unwrap();
    /// assert_eq!(z.focus(), &2);
    /// ```
    #[inline]
    pub fn focus_next(mut self) -> Option<Self> {
        if self.right.is_empty() {
            None
        } else {
            let new_focus = self.right.remove(0);
            self.left.push(self.focus);
            self.focus = new_focus;
            Some(self)
        }
    }

    /// Move focus to the previous (left) element.
    ///
    /// Returns `None` if there are no elements to the left.
    ///
    /// # Example
    ///
    /// ```
    /// use ordofp_core::zipper::Zipper;
    ///
    /// let z = Zipper::new(2, vec![1], vec![3]);
    /// let z = z.focus_prev().unwrap();
    /// assert_eq!(z.focus(), &1);
    /// ```
    #[inline]
    pub fn focus_prev(mut self) -> Option<Self> {
        self.left.pop().map(|new_focus| {
            self.right.insert(0, self.focus);
            self.focus = new_focus;
            self
        })
    }

    /// Move focus to the next element, wrapping around to the first if at the end.
    ///
    /// # Example
    ///
    /// ```
    /// use ordofp_core::zipper::Zipper;
    ///
    /// let z = Zipper::from_vec(vec![1, 2, 3]).unwrap();
    /// let z = z.focus_next_wrap().focus_next_wrap().focus_next_wrap();
    /// assert_eq!(z.focus(), &1); // wrapped around
    /// ```
    /// # Panics
    ///
    /// Panics only if the internal zipper invariant is violated (the
    /// `expect` sits in the branch where `right` was just checked
    /// non-empty, so `focus_next` always succeeds); reaching it would
    /// indicate a bug in this crate.
    #[inline]
    pub fn focus_next_wrap(self) -> Self {
        if self.right.is_empty() {
            // Wrap around: combine left and current, focus on first
            self.focus_first()
        } else {
            self.focus_next()
                .expect("zipper invariant: focus_next called while right is non-empty")
        }
    }

    /// Move focus to the previous element, wrapping around to the last if at the beginning.
    ///
    /// # Example
    ///
    /// ```
    /// use ordofp_core::zipper::Zipper;
    ///
    /// let z = Zipper::from_vec(vec![1, 2, 3]).unwrap();
    /// let z = z.focus_prev_wrap();
    /// assert_eq!(z.focus(), &3); // wrapped to last
    /// ```
    /// # Panics
    ///
    /// Panics only if the internal zipper invariant is violated (the
    /// `expect` sits in the branch where `left` was just checked
    /// non-empty, so `focus_prev` always succeeds); reaching it would
    /// indicate a bug in this crate.
    #[inline]
    pub fn focus_prev_wrap(self) -> Self {
        if self.left.is_empty() {
            self.focus_last()
        } else {
            self.focus_prev()
                .expect("zipper invariant: focus_prev called while left is non-empty")
        }
    }

    /// Move focus to the first element.
    ///
    /// # Example
    ///
    /// ```
    /// use ordofp_core::zipper::Zipper;
    ///
    /// let z = Zipper::new(3, vec![1, 2], vec![4, 5]);
    /// let z = z.focus_first();
    /// assert_eq!(z.focus(), &1);
    /// ```
    pub fn focus_first(mut self) -> Self {
        if self.left.is_empty() {
            self
        } else {
            // Left is [1, 2] where first element is at index 0
            let first = self.left.remove(0);
            // New right = remaining left + focus + old right
            let mut new_right = self.left;
            new_right.push(self.focus);
            new_right.extend(self.right);
            Zipper {
                focus: first,
                left: Vec::new(),
                right: new_right,
            }
        }
    }

    /// Move focus to the last element.
    ///
    /// # Example
    ///
    /// ```
    /// use ordofp_core::zipper::Zipper;
    ///
    /// let z = Zipper::new(3, vec![1, 2], vec![4, 5]);
    /// let z = z.focus_last();
    /// assert_eq!(z.focus(), &5);
    /// ```
    /// # Panics
    ///
    /// Panics only if the internal zipper invariant is violated (the
    /// `expect` guards a `pop` in the branch where `right` was just
    /// checked non-empty); reaching it would indicate a bug in this
    /// crate.
    pub fn focus_last(mut self) -> Self {
        if self.right.is_empty() {
            self
        } else {
            let last = self
                .right
                .pop()
                .expect("zipper invariant: right is non-empty in this branch (checked above)");
            // New left = old left + focus + remaining right
            self.left.push(self.focus);
            self.left.extend(self.right);
            Zipper {
                focus: last,
                left: self.left,
                right: Vec::new(),
            }
        }
    }

    /// Convert the Zipper back to a Vec, preserving order.
    ///
    /// # Example
    ///
    /// ```
    /// use ordofp_core::zipper::Zipper;
    ///
    /// let z = Zipper::new(2, vec![1], vec![3, 4]);
    /// assert_eq!(z.to_vec(), vec![1, 2, 3, 4]);
    /// ```
    #[inline]
    pub fn to_vec(self) -> Vec<A> {
        // left stores elements in natural order [1, 2] where 2 is nearest focus
        let total = self.left.len() + 1 + self.right.len();
        let mut result = self.left;
        // Reserve exactly the remaining capacity so the two extend calls below
        // never reallocate.
        result.reserve(total - result.len());
        result.push(self.focus);
        result.extend(self.right);
        result
    }

    /// Get the total length of the Zipper.
    ///
    /// # Example
    ///
    /// ```
    /// use ordofp_core::zipper::Zipper;
    ///
    /// let z = Zipper::new(2, vec![1], vec![3, 4]);
    /// assert_eq!(z.len(), 4);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.left.len() + 1 + self.right.len()
    }

    /// A Zipper is never empty since it always has a focus.
    #[inline]
    pub fn is_empty(&self) -> bool {
        false
    }

    /// Check if this is a singleton Zipper.
    #[inline]
    pub fn is_singleton(&self) -> bool {
        self.left.is_empty() && self.right.is_empty()
    }

    /// Insert an element to the left of focus.
    ///
    /// # Example
    ///
    /// ```
    /// use ordofp_core::zipper::Zipper;
    ///
    /// let z = Zipper::from_vec(vec![1, 3]).unwrap();
    /// let z = z.focus_next().unwrap();
    /// let z = z.insert_left(2);
    /// assert_eq!(z.to_vec(), vec![1, 2, 3]);
    /// ```
    #[inline]
    pub fn insert_left(mut self, a: A) -> Self {
        self.left.push(a);
        self
    }

    /// Insert an element to the right of focus.
    ///
    /// # Example
    ///
    /// ```
    /// use ordofp_core::zipper::Zipper;
    ///
    /// let z = Zipper::from_vec(vec![1, 3]).unwrap();
    /// let z = z.insert_right(2);
    /// assert_eq!(z.to_vec(), vec![1, 2, 3]);
    /// ```
    #[inline]
    pub fn insert_right(mut self, a: A) -> Self {
        self.right.insert(0, a);
        self
    }

    /// Remove the focused element, moving focus to the right if possible,
    /// otherwise to the left. Returns `None` if this was the only element.
    ///
    /// # Example
    ///
    /// ```
    /// use ordofp_core::zipper::Zipper;
    ///
    /// let z = Zipper::from_vec(vec![1, 2, 3]).unwrap();
    /// let z = z.focus_next().unwrap(); // focus on 2
    /// let (removed, z) = z.delete().unwrap();
    /// assert_eq!(removed, 2);
    /// assert_eq!(z.focus(), &3); // focus moved right
    /// ```
    pub fn delete(mut self) -> Option<(A, Self)> {
        let old_focus = self.focus;
        if !self.right.is_empty() {
            // Move focus right
            let new_focus = self.right.remove(0);
            self.focus = new_focus;
            Some((old_focus, self))
        } else if let Some(new_focus) = self.left.pop() {
            // Move focus left
            self.focus = new_focus;
            Some((old_focus, self))
        } else {
            // Singleton - can't delete
            None
        }
    }

    /// Replace the focused element, returning the old value.
    ///
    /// # Example
    ///
    /// ```
    /// use ordofp_core::zipper::Zipper;
    ///
    /// let z = Zipper::singleton(1);
    /// let (old, z) = z.replace(42);
    /// assert_eq!(old, 1);
    /// assert_eq!(z.focus(), &42);
    /// ```
    #[inline]
    pub fn replace(mut self, a: A) -> (A, Self) {
        let old = core::mem::replace(&mut self.focus, a);
        (old, self)
    }

    /// Update the focused element with a function.
    ///
    /// # Example
    ///
    /// ```
    /// use ordofp_core::zipper::Zipper;
    ///
    /// let z = Zipper::singleton(1);
    /// let z = z.update(|x| x + 10);
    /// assert_eq!(z.focus(), &11);
    /// ```
    #[inline]
    pub fn update<F>(mut self, f: F) -> Self
    where
        F: FnOnce(A) -> A,
    {
        self.focus = f(self.focus);
        self
    }

    /// Map a function over all elements.
    ///
    /// # Example
    ///
    /// ```
    /// use ordofp_core::zipper::Zipper;
    ///
    /// let z = Zipper::from_vec(vec![1, 2, 3]).unwrap();
    /// let z = z.map(|x| x * 2);
    /// assert_eq!(z.to_vec(), vec![2, 4, 6]);
    /// ```
    pub fn map<B, F>(self, f: F) -> Zipper<B>
    where
        F: Fn(A) -> B,
    {
        Zipper {
            focus: f(self.focus),
            left: self.left.into_iter().map(&f).collect(),
            right: self.right.into_iter().map(&f).collect(),
        }
    }

    /// Swap the focused element with its left neighbor.
    /// Returns `None` if there's no left neighbor.
    ///
    /// Note: Focus moves to the position where the left neighbor was.
    ///
    /// # Example
    ///
    /// ```
    /// use ordofp_core::zipper::Zipper;
    ///
    /// let z = Zipper::new(2, vec![1], vec![3]);
    /// let z = z.swap_left().unwrap();
    /// assert_eq!(z.clone().to_vec(), vec![2, 1, 3]);
    /// assert_eq!(z.focus(), &1); // focus is now on what was the left element
    /// ```
    pub fn swap_left(mut self) -> Option<Self> {
        self.left.pop().map(|left_val| {
            self.left.push(self.focus);
            self.focus = left_val;
            self
        })
    }

    /// Swap the focused element with its right neighbor.
    /// Returns `None` if there's no right neighbor.
    ///
    /// Note: Focus moves to the position where the right neighbor was.
    ///
    /// # Example
    ///
    /// ```
    /// use ordofp_core::zipper::Zipper;
    ///
    /// let z = Zipper::new(2, vec![1], vec![3]);
    /// let z = z.swap_right().unwrap();
    /// assert_eq!(z.clone().to_vec(), vec![1, 3, 2]);
    /// assert_eq!(z.focus(), &3); // focus is now on what was the right element
    /// ```
    pub fn swap_right(mut self) -> Option<Self> {
        if self.right.is_empty() {
            None
        } else {
            let right_val = self.right.remove(0);
            self.right.insert(0, self.focus);
            self.focus = right_val;
            // Result: [1, <2>, 3] -> [1, <3>, 2]. The two values swap; the
            // cursor stays at the same index, now focusing the former right
            // neighbor (matching the doc example above).
            Some(self)
        }
    }

    /// Filter elements, keeping only those satisfying the predicate.
    /// The focus is preserved if it satisfies the predicate; otherwise,
    /// focus moves right then left.
    ///
    /// Returns `None` if no elements satisfy the predicate.
    ///
    /// # Example
    ///
    /// ```
    /// use ordofp_core::zipper::Zipper;
    ///
    /// let z = Zipper::from_vec(vec![1, 2, 3, 4, 5]).unwrap();
    /// let z = z.focus_next().unwrap().focus_next().unwrap(); // focus on 3
    /// let z = z.filter(|&x| x % 2 == 1).unwrap(); // keep odd numbers
    /// assert_eq!(z.focus(), &3); // focus preserved (3 is odd)
    /// assert_eq!(z.to_vec(), vec![1, 3, 5]);
    /// ```
    ///
    /// # Panics
    ///
    /// Panics only if the internal zipper invariant is violated (the
    /// `expect` guards a `pop` in the branch where `left` was just
    /// checked non-empty; the genuinely empty case already returns
    /// `None`); reaching it would indicate a bug in this crate.
    pub fn filter<F>(self, mut pred: F) -> Option<Self>
    where
        F: FnMut(&A) -> bool,
    {
        let focus_ok = pred(&self.focus);
        // Use extract_if (stable 1.87) for in-place removal without intermediate allocation.
        // This is more efficient than filter().collect() as it modifies in place.
        let mut left = self.left;
        let mut right = self.right;
        // extract_if removes elements where the predicate returns true,
        // so we negate to keep elements where pred returns true
        let _removed_left: Vec<A> = left.extract_if(.., |x| !pred(x)).collect();
        let _removed_right: Vec<A> = right.extract_if(.., |x| !pred(x)).collect();

        if focus_ok {
            Some(Zipper {
                focus: self.focus,
                left,
                right,
            })
        } else if !right.is_empty() {
            let focus = right.remove(0);
            Some(Zipper { focus, left, right })
        } else if !left.is_empty() {
            // Invariant note: this `expect` is unreachable — `pop` runs in
            // the branch where `left` was just checked non-empty, and the
            // function already returns Option for the genuinely empty case.
            let focus = left
                .pop()
                .expect("zipper invariant: left is non-empty in this branch (checked above)");
            Some(Zipper {
                focus,
                left,
                right: Vec::new(),
            })
        } else {
            None
        }
    }

    /// Get the index of the currently focused element (0-based from left).
    ///
    /// # Example
    ///
    /// ```
    /// use ordofp_core::zipper::Zipper;
    ///
    /// let z = Zipper::new(3, vec![1, 2], vec![4, 5]);
    /// assert_eq!(z.focus_index(), 2);
    /// ```
    #[inline]
    pub fn focus_index(&self) -> usize {
        self.left.len()
    }

    /// Move focus to a specific index.
    /// Returns `None` if the index is out of bounds.
    ///
    /// # Example
    ///
    /// ```
    /// use ordofp_core::zipper::Zipper;
    ///
    /// let z = Zipper::from_vec(vec![0, 1, 2, 3, 4]).unwrap();
    /// let z = z.focus_at(3).unwrap();
    /// assert_eq!(z.focus(), &3);
    /// ```
    pub fn focus_at(self, index: usize) -> Option<Self> {
        let current_index = self.focus_index();
        match index.cmp(&current_index) {
            core::cmp::Ordering::Equal => Some(self),
            core::cmp::Ordering::Greater => {
                let steps = index - current_index;
                let mut z = self;
                for _ in 0..steps {
                    z = z.focus_next()?;
                }
                Some(z)
            }
            core::cmp::Ordering::Less => {
                let steps = current_index - index;
                let mut z = self;
                for _ in 0..steps {
                    z = z.focus_prev()?;
                }
                Some(z)
            }
        }
    }

    /// Find and focus the first element matching the predicate.
    /// Returns `None` if no element matches.
    /// Requires `A: Clone` to search through the zipper.
    ///
    /// # Example
    ///
    /// ```
    /// use ordofp_core::zipper::Zipper;
    ///
    /// let z = Zipper::from_vec(vec![1, 2, 3, 4, 5]).unwrap();
    /// let z = z.find_focus(|&x| x > 3).unwrap();
    /// assert_eq!(z.focus(), &4);
    /// ```
    pub fn find_focus<F>(self, pred: F) -> Option<Self>
    where
        F: Fn(&A) -> bool,
        A: Clone,
    {
        // Check current focus first
        if pred(&self.focus) {
            return Some(self);
        }

        // Start from beginning and search
        let z = self.focus_first();
        let mut current = z;

        loop {
            if pred(current.focus()) {
                return Some(current);
            }
            {
                let next = current.focus_next()?;
                current = next;
            }
        }
    }

    /// Reverse the zipper while keeping focus on the same element.
    ///
    /// # Example
    ///
    /// ```
    /// use ordofp_core::zipper::Zipper;
    ///
    /// let z = Zipper::new(2, vec![1], vec![3, 4]);
    /// let z = z.reverse();
    /// assert_eq!(z.focus(), &2);
    /// assert_eq!(z.to_vec(), vec![4, 3, 2, 1]);
    /// ```
    pub fn reverse(self) -> Self {
        // Original: left=[1], focus=2, right=[3,4] → order: 1,2,3,4
        // Reversed: order: 4,3,2,1 → left=[4,3], focus=2, right=[1]
        let mut new_left = self.right;
        new_left.reverse();
        let mut new_right = self.left;
        new_right.reverse();
        Zipper {
            focus: self.focus,
            left: new_left,
            right: new_right,
        }
    }
}

#[cfg(feature = "alloc")]
impl<A> Zipper<A>
where
    A: Clone,
{
    /// Duplicate the focused element to the left.
    #[inline]
    pub fn duplicate_left(mut self) -> Self {
        self.left.push(self.focus.clone());
        self
    }

    /// Duplicate the focused element to the right.
    #[inline]
    pub fn duplicate_right(mut self) -> Self {
        self.right.insert(0, self.focus.clone());
        self
    }
}

#[cfg(feature = "alloc")]
impl<A> IntoIterator for Zipper<A> {
    type Item = A;
    type IntoIter = alloc::vec::IntoIter<A>;

    fn into_iter(self) -> Self::IntoIter {
        self.to_vec().into_iter()
    }
}

#[cfg(all(feature = "alloc", test))]
mod tests {
    use super::*;
    use alloc::vec;
    use alloc::vec::Vec;

    #[test]
    fn test_singleton() {
        let z = Zipper::singleton(42);
        assert_eq!(z.focus(), &42);
        assert_eq!(z.len(), 1);
        assert!(z.is_singleton());
    }

    #[test]
    fn test_from_vec() {
        let z = Zipper::from_vec(vec![1, 2, 3])
            .expect("non-empty vec [1, 2, 3] should produce a valid Zipper");
        assert_eq!(z.focus(), &1);
        assert_eq!(z.to_vec(), vec![1, 2, 3]);
    }

    #[test]
    fn test_from_empty_vec() {
        let z: Option<Zipper<i32>> = Zipper::from_vec(vec![]);
        assert!(z.is_none());
    }

    /// `Zipper::from_slice` on an empty slice must return `None`, matching the
    /// behaviour of `from_vec` with an empty input.
    #[test]
    fn test_from_slice_empty_returns_none() {
        let z: Option<Zipper<i32>> = Zipper::from_slice(&[]);
        assert!(z.is_none(), "from_slice on an empty slice must return None");
    }

    /// `Zipper::from_slice` on a single-element slice must focus that element
    /// with both left and right neighbourhoods empty, and `to_vec` must round-trip.
    #[test]
    fn test_from_slice_singleton_has_empty_neighbours() {
        let z = Zipper::from_slice(&[99_i32])
            .expect("single-element slice must produce a valid Zipper");
        assert_eq!(z.focus(), &99, "focus must be the sole element");
        assert!(
            z.left.is_empty(),
            "left must be empty for a single-element slice"
        );
        assert!(
            z.right.is_empty(),
            "right must be empty for a single-element slice"
        );
        assert_eq!(
            z.to_vec(),
            vec![99_i32],
            "to_vec must round-trip the single element"
        );
    }

    #[test]
    fn test_focus_next() {
        let z = Zipper::from_vec(vec![1, 2, 3]).expect("non-empty vec should construct a Zipper");
        let z = z
            .focus_next()
            .expect("zipper has a right element to move focus to");
        assert_eq!(z.focus(), &2);
        let z = z
            .focus_next()
            .expect("zipper still has a right element to move focus to");
        assert_eq!(z.focus(), &3);
        assert!(z.focus_next().is_none());
    }

    #[test]
    fn test_focus_prev() {
        let z = Zipper::new(3, vec![1, 2], vec![4, 5]);
        let z = z
            .focus_prev()
            .expect("zipper has two left neighbors, first focus_prev must succeed");
        assert_eq!(z.focus(), &2);
        let z = z
            .focus_prev()
            .expect("zipper still has one left neighbor, second focus_prev must succeed");
        assert_eq!(z.focus(), &1);
        assert!(z.focus_prev().is_none());
    }

    #[test]
    fn test_focus_wrap() {
        let z = Zipper::from_vec(vec![1, 2, 3]).expect("non-empty vec must produce a Zipper");

        // Wrap forward
        let z = z.focus_next_wrap().focus_next_wrap().focus_next_wrap();
        assert_eq!(z.focus(), &1);

        // Wrap backward
        let z = Zipper::from_vec(vec![1, 2, 3]).expect("non-empty vec must produce a Zipper");
        let z = z.focus_prev_wrap();
        assert_eq!(z.focus(), &3);
    }

    #[test]
    fn test_focus_first_last() {
        let z = Zipper::new(3, vec![1, 2], vec![4, 5]);

        let z_first = z.clone().focus_first();
        assert_eq!(z_first.focus(), &1);

        let z_last = z.focus_last();
        assert_eq!(z_last.focus(), &5);
    }

    #[test]
    fn test_insert() {
        let z = Zipper::from_vec(vec![1, 3])
            .expect("non-empty vec [1, 3] should construct a valid Zipper");
        let z = z.insert_right(2);
        assert_eq!(z.to_vec(), vec![1, 2, 3]);

        let z = Zipper::from_vec(vec![2, 3])
            .expect("non-empty vec [2, 3] should construct a valid Zipper");
        let z = z.insert_left(1);
        // focus is on 2, insert 1 to left
        assert_eq!(z.to_vec(), vec![1, 2, 3]);
    }

    #[test]
    fn test_delete() {
        let z = Zipper::from_vec(vec![1, 2, 3])
            .expect("non-empty vec [1, 2, 3] should always construct a valid Zipper");
        let z = z
            .focus_next()
            .expect("non-empty vec should allow moving focus to next element");
        let (removed, z) = z
            .delete()
            .expect("delete should succeed when zipper has more than one element");
        assert_eq!(removed, 2);
        assert_eq!(z.focus(), &3);
        assert_eq!(z.to_vec(), vec![1, 3]);
    }

    #[test]
    fn test_delete_singleton() {
        let z = Zipper::singleton(42);
        assert!(z.delete().is_none());
    }

    #[test]
    fn test_map() {
        let z = Zipper::from_vec(vec![1, 2, 3]).expect("non-empty vec should construct a Zipper");
        let z = z.map(|x| x * 10);
        assert_eq!(z.to_vec(), vec![10, 20, 30]);
    }

    #[test]
    fn test_update() {
        let z = Zipper::singleton(5);
        let z = z.update(|x| x * 2);
        assert_eq!(z.focus(), &10);
    }

    #[test]
    fn test_filter() {
        let z = Zipper::from_vec(vec![1, 2, 3, 4, 5])
            .expect("non-empty vec [1,2,3,4,5] must produce a valid Zipper");
        let z = z
            .focus_next()
            .expect("zipper has elements to the right, first focus_next must succeed")
            .focus_next()
            .expect("zipper still has elements to the right, second focus_next must succeed"); // focus on 3
        let z = z
            .filter(|&x| x % 2 == 1)
            .expect("odd-only filter on [1,2,3,4,5] keeps at least one element");
        assert_eq!(z.focus(), &3);
        assert_eq!(z.to_vec(), vec![1, 3, 5]);
    }

    #[test]
    fn test_filter_removes_focus() {
        let z = Zipper::from_vec(vec![1, 2, 3, 4, 5]).expect("non-empty vec must produce a zipper");
        let z = z
            .focus_next()
            .expect("zipper has a next element after index 0");
        let z = z
            .filter(|&x| x % 2 == 1)
            .expect("filtering [1,2,3,4,5] by odd keeps elements, so result is non-empty"); // 2 is even, removed
        assert_eq!(z.focus(), &3); // focus moved to next odd
        assert_eq!(z.to_vec(), vec![1, 3, 5]);
    }

    #[test]
    fn test_focus_index() {
        let z = Zipper::new(3, vec![1, 2], vec![4, 5]);
        assert_eq!(z.focus_index(), 2);
    }

    #[test]
    fn test_focus_at() {
        let z = Zipper::from_vec(vec![0, 1, 2, 3, 4])
            .expect("from_vec with a non-empty vec should return Some");
        let z = z
            .focus_at(3)
            .expect("focus_at(3) on a 5-element zipper is in-bounds");
        assert_eq!(z.focus(), &3);
        assert_eq!(z.focus_index(), 3);
    }

    #[test]
    fn test_focus_at_out_of_bounds_returns_none() {
        // An index beyond the last element must yield None, not panic.
        let z = Zipper::from_vec(vec![0, 1, 2, 3, 4])
            .expect("non-empty vec with 5 elements should produce a valid Zipper");
        assert!(
            z.focus_at(5).is_none(),
            "index 5 is out of bounds for a 5-element zipper"
        );
    }

    #[test]
    fn test_focus_at_same_index_is_noop() {
        // Focusing on the current index should return the zipper unchanged.
        let z = Zipper::from_vec(vec![10, 20, 30])
            .expect("non-empty vec should produce a valid Zipper");
        let z2 = z
            .focus_at(0)
            .expect("focus_at current index 0 should succeed as a no-op");
        assert_eq!(z2.focus(), &10);
        assert_eq!(z2.focus_index(), 0);
    }

    #[test]
    fn test_focus_at_backward() {
        // Moving the focus to an earlier index must work symmetrically with forward movement.
        let z = Zipper::from_vec(vec![10, 20, 30, 40, 50])
            .expect("non-empty vec must produce a valid Zipper");
        let z = z
            .focus_at(4)
            .expect("index 4 is the last valid index of a 5-element Zipper");
        let z = z
            .focus_at(1)
            .expect("index 1 is within bounds of a 5-element Zipper");
        assert_eq!(z.focus(), &20);
        assert_eq!(z.focus_index(), 1);
    }

    #[test]
    fn test_reverse() {
        let z = Zipper::new(2, vec![1], vec![3, 4]);
        let z = z.reverse();
        assert_eq!(z.focus(), &2);
        assert_eq!(z.to_vec(), vec![4, 3, 2, 1]);
    }

    #[test]
    fn test_into_iter() {
        let z = Zipper::from_vec(vec![1, 2, 3])
            .expect("non-empty vec should always produce a valid Zipper");
        let v: Vec<_> = z.into_iter().collect();
        assert_eq!(v, vec![1, 2, 3]);
    }

    #[test]
    fn test_find_focus() {
        let z = Zipper::from_vec(vec![1, 2, 3, 4, 5])
            .expect("non-empty vec should always produce a Zipper");
        let z = z
            .find_focus(|&x| x > 3)
            .expect("vec [1,2,3,4,5] contains values > 3, so find_focus must succeed");
        assert_eq!(z.focus(), &4);
    }

    /// `swap_left` on a zipper with no left neighbor must return `None`.
    /// When a left neighbor exists the two elements exchange positions and
    /// the focus moves one step left (to where the neighbour was).
    #[test]
    fn test_swap_left_edge_cases() {
        // Edge case: focus is already at the leftmost position – no swap possible.
        let singleton = Zipper::singleton(42);
        assert!(
            singleton.swap_left().is_none(),
            "swap_left on singleton must return None"
        );

        let leftmost =
            Zipper::from_vec(vec![1, 2, 3]).expect("non-empty vec must produce a Zipper"); // focus on 1
        assert!(
            leftmost.swap_left().is_none(),
            "swap_left with empty left must return None"
        );

        // Normal case: [1, <2>, 3] → swap → [<1>, 2, 3]  (focus moves to old left position)
        let z = Zipper::new(2, vec![1], vec![3]);
        let swapped = z
            .swap_left()
            .expect("swap_left should succeed with a left neighbour");
        assert_eq!(
            swapped.focus(),
            &1,
            "focus must move to the element that was the left neighbour"
        );
        assert_eq!(
            swapped.to_vec(),
            vec![2, 1, 3],
            "swap_left must exchange the focused element with its left neighbour"
        );
    }

    /// `swap_right` on a zipper with no right neighbor must return `None`.
    /// When a right neighbor exists the two elements exchange positions and
    /// the focus moves one step right (to where the neighbour was).
    #[test]
    fn test_swap_right_edge_cases() {
        // Edge case: focus is already at the rightmost position – no swap possible.
        let singleton = Zipper::singleton(42);
        assert!(
            singleton.swap_right().is_none(),
            "swap_right on singleton must return None"
        );

        let rightmost = Zipper::new(3, vec![1, 2], vec![]);
        assert!(
            rightmost.swap_right().is_none(),
            "swap_right with empty right must return None"
        );

        // Normal case: [1, <2>, 3] → swap → [1, 3, <2>]  (focus moves to old right position)
        let z = Zipper::new(2, vec![1], vec![3]);
        let swapped = z
            .swap_right()
            .expect("swap_right should succeed with a right neighbour");
        assert_eq!(
            swapped.focus(),
            &3,
            "focus must move to the element that was the right neighbour"
        );
        assert_eq!(
            swapped.to_vec(),
            vec![1, 3, 2],
            "swap_right must exchange the focused element with its right neighbour"
        );
    }

    /// `duplicate_left` inserts a copy of the focused element immediately to
    /// its left without changing the focus.  On a singleton the result must be
    /// a two-element zipper `[focus, focus]` still focused on the original value.
    #[test]
    fn test_duplicate_left() {
        // Singleton edge case: left is empty, so the copy becomes the only left neighbour.
        let z = Zipper::singleton(7);
        let z = z.duplicate_left();
        assert_eq!(
            z.focus(),
            &7,
            "focus must remain unchanged after duplicate_left"
        );
        assert_eq!(
            z.to_vec(),
            vec![7, 7],
            "duplicate_left on singleton must produce [focus, focus]"
        );

        // Multi-element case: [1, <2>, 3] → duplicate_left → [1, 2, <2>, 3]
        let z = Zipper::new(2, vec![1], vec![3]);
        let z = z.duplicate_left();
        assert_eq!(z.focus(), &2);
        assert_eq!(z.to_vec(), vec![1, 2, 2, 3]);
    }

    /// `filter` returns `None` when the predicate rejects every element,
    /// including the focus and every element to the left and right.
    #[test]
    fn test_filter_all_removed() {
        // All elements fail the predicate → the zipper collapses to None.
        let z = Zipper::from_vec(vec![2, 4, 6]).expect("non-empty vec must produce a Zipper");
        let result: Option<Zipper<i32>> = z.filter(|&x| x % 2 == 1); // no odd numbers
        assert!(
            result.is_none(),
            "filter must return None when no elements satisfy the predicate"
        );

        // Same for a singleton whose focus is also rejected.
        let z = Zipper::singleton(0);
        let result: Option<Zipper<i32>> = z.filter(|&x| x > 0);
        assert!(
            result.is_none(),
            "filter on a singleton must return None when the focus is rejected"
        );
    }

    /// `duplicate_right` inserts a copy of the focused element immediately to
    /// its right without changing the focus.  On a singleton the result must be
    /// a two-element zipper `[focus, focus]` still focused on the original value.
    #[test]
    fn test_duplicate_right() {
        // Singleton edge case: right is empty, so the copy becomes the only right neighbour.
        let z = Zipper::singleton(9);
        let z = z.duplicate_right();
        assert_eq!(
            z.focus(),
            &9,
            "focus must remain unchanged after duplicate_right"
        );
        assert_eq!(
            z.to_vec(),
            vec![9, 9],
            "duplicate_right on singleton must produce [focus, focus]"
        );

        // Multi-element case: [1, <2>, 3] → duplicate_right → [1, <2>, 2, 3]
        let z = Zipper::new(2, vec![1], vec![3]);
        let z = z.duplicate_right();
        assert_eq!(z.focus(), &2);
        assert_eq!(z.to_vec(), vec![1, 2, 2, 3]);
    }
}
