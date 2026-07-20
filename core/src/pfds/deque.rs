//! Persistent Deque (Double-Ended Queue) implementation.
//!
//! A fully persistent double-ended queue supporting efficient operations
//! at both ends.

use core::iter::FromIterator;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[cfg(feature = "serde")]
use serde::Serialize;

/// A persistent (immutable) double-ended queue.
///
/// Supports efficient push/pop at both front and back using a pair of
/// vectors. Based on the real-time deque from Okasaki's PFDS.
///
/// # Time Complexity
///
/// | Operation    | Time (amortized) |
/// |--------------|------------------|
/// | `push_front` | O(1)             |
/// | `push_back`  | O(1)             |
/// | `pop_front`  | O(1)             |
/// | `pop_back`   | O(1)             |
/// | `peek_front` | O(1)             |
/// | `peek_back`  | O(1)             |
/// | `len`        | O(1)             |
///
/// # Example
///
/// ```rust
/// use ordofp_core::pfds::Deque;
///
/// let d = Deque::new()
///     .push_back(1)
///     .push_back(2)
///     .push_front(0);
///
/// assert_eq!(d.peek_front(), Some(&0));
/// assert_eq!(d.peek_back(), Some(&2));
///
/// let (front, d) = d.pop_front().unwrap();
/// assert_eq!(front, 0);
/// ```
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg(feature = "alloc")]
pub struct Deque<A> {
    /// Front elements (top of vec is front of deque).
    front: Vec<A>,
    /// Back elements (top of vec is back of deque).
    back: Vec<A>,
    /// Cached length for O(1) access.
    len: usize,
}

#[cfg(all(feature = "serde", feature = "alloc"))]
impl<'de, A: serde::Deserialize<'de>> serde::Deserialize<'de> for Deque<A> {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        #[serde(rename = "Deque")]
        struct Wire<A> {
            front: Vec<A>,
            back: Vec<A>,
            // Accepted for round-trip compatibility, never trusted: the whole
            // type relies on len == front.len() + back.len().
            #[serde(default)]
            #[allow(dead_code)]
            len: usize,
        }
        let w = Wire::deserialize(d)?;
        Ok(Deque {
            len: w.front.len() + w.back.len(),
            front: w.front,
            back: w.back,
        })
    }
}

#[cfg(feature = "alloc")]
impl<A> Default for Deque<A> {
    fn default() -> Self {
        Deque::new()
    }
}

#[cfg(feature = "alloc")]
impl<A: PartialEq> PartialEq for Deque<A> {
    fn eq(&self, other: &Self) -> bool {
        if self.len != other.len {
            return false;
        }
        // Compare element by element
        let self_iter = self.front.iter().rev().chain(self.back.iter());
        let other_iter = other.front.iter().rev().chain(other.back.iter());
        self_iter.eq(other_iter)
    }
}

#[cfg(feature = "alloc")]
impl<A: Eq> Eq for Deque<A> {}

#[cfg(feature = "alloc")]
impl<A> Deque<A> {
    /// Create a new empty deque.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::pfds::Deque;
    ///
    /// let d: Deque<i32> = Deque::new();
    /// assert!(d.is_empty());
    /// ```
    #[inline]
    pub fn new() -> Self {
        Deque {
            front: Vec::new(),
            back: Vec::new(),
            len: 0,
        }
    }

    /// Check if the deque is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Get the number of elements in the deque.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Push a value to the front of the deque.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::pfds::Deque;
    ///
    /// let d = Deque::new().push_front(1).push_front(2);
    /// assert_eq!(d.peek_front(), Some(&2));
    /// ```
    #[inline]
    pub fn push_front(mut self, value: A) -> Self {
        self.front.push(value);
        Deque {
            front: self.front,
            back: self.back,
            len: self.len + 1,
        }
    }

    /// Push a value to the back of the deque.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::pfds::Deque;
    ///
    /// let d = Deque::new().push_back(1).push_back(2);
    /// assert_eq!(d.peek_back(), Some(&2));
    /// ```
    #[inline]
    pub fn push_back(mut self, value: A) -> Self {
        self.back.push(value);
        Deque {
            front: self.front,
            back: self.back,
            len: self.len + 1,
        }
    }

    /// Pop a value from the front of the deque.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::pfds::Deque;
    ///
    /// let d = Deque::new().push_back(1).push_back(2);
    /// let (first, rest) = d.pop_front().unwrap();
    /// assert_eq!(first, 1);
    /// ```
    ///
    /// # Panics
    ///
    /// Panics only if the internal front/back balance invariant is violated
    /// (a non-empty deque must have a non-empty front after rebalancing),
    /// which indicates a bug in this crate.
    #[inline]
    pub fn pop_front(mut self) -> Option<(A, Self)>
    where
        A: Clone,
    {
        if self.is_empty() {
            return None;
        }

        // Rebalance if front is empty
        if self.front.is_empty() {
            self.rebalance_front();
        }

        let value = self.front.pop().unwrap();
        Some((
            value,
            Deque {
                front: self.front,
                back: self.back,
                len: self.len - 1,
            },
        ))
    }

    /// Pop a value from the back of the deque.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::pfds::Deque;
    ///
    /// let d = Deque::new().push_front(1).push_front(2);
    /// let (last, rest) = d.pop_back().unwrap();
    /// assert_eq!(last, 1);
    /// ```
    ///
    /// # Panics
    ///
    /// Panics only if the internal front/back balance invariant is violated
    /// (a non-empty deque must have a non-empty back after rebalancing),
    /// which indicates a bug in this crate.
    #[inline]
    pub fn pop_back(mut self) -> Option<(A, Self)>
    where
        A: Clone,
    {
        if self.is_empty() {
            return None;
        }

        // Rebalance if back is empty
        if self.back.is_empty() {
            self.rebalance_back();
        }

        let value = self.back.pop().unwrap();
        Some((
            value,
            Deque {
                front: self.front,
                back: self.back,
                len: self.len - 1,
            },
        ))
    }

    /// Peek at the front value without removing it.
    #[inline]
    pub fn peek_front(&self) -> Option<&A> {
        if self.is_empty() {
            None
        } else if !self.front.is_empty() {
            self.front.last()
        } else {
            self.back.first()
        }
    }

    /// Peek at the back value without removing it.
    #[inline]
    pub fn peek_back(&self) -> Option<&A> {
        if self.is_empty() {
            None
        } else if !self.back.is_empty() {
            self.back.last()
        } else {
            self.front.first()
        }
    }

    /// Rebalance when front is empty: move half of back to front.
    fn rebalance_front(&mut self)
    where
        A: Clone,
    {
        if self.back.is_empty() {
            return;
        }

        let mid = self.back.len() / 2;
        let mut new_front: Vec<A> = self.back.drain(..=mid).collect();
        new_front.reverse();
        self.front = new_front;
    }

    /// Rebalance when back is empty: move half of front to back.
    fn rebalance_back(&mut self)
    where
        A: Clone,
    {
        if self.front.is_empty() {
            return;
        }

        let mid = self.front.len() / 2;
        let mut new_back: Vec<A> = self.front.drain(..=mid).collect();
        new_back.reverse();
        self.back = new_back;
    }

    /// Map a function over the deque.
    #[inline]
    pub fn map<B, F>(&self, f: F) -> Deque<B>
    where
        A: Clone,
        F: Fn(&A) -> B,
    {
        Deque {
            front: self.front.iter().map(&f).collect(),
            back: self.back.iter().map(&f).collect(),
            len: self.len,
        }
    }

    /// Fold the deque from front to back.
    #[inline]
    pub fn fold<B, F>(&self, init: B, f: F) -> B
    where
        A: Clone,
        F: Fn(B, &A) -> B,
    {
        let acc = self.front.iter().rev().fold(init, &f);
        self.back.iter().fold(acc, f)
    }

    /// Filter elements that satisfy the predicate.
    #[inline]
    pub fn filter<F>(&self, pred: F) -> Self
    where
        A: Clone,
        F: Fn(&A) -> bool,
    {
        let front: Vec<A> = self.front.iter().filter(|x| pred(x)).cloned().collect();
        let back: Vec<A> = self.back.iter().filter(|x| pred(x)).cloned().collect();
        let len = front.len() + back.len();
        Deque { front, back, len }
    }

    /// Convert to a Vec (front to back order).
    #[inline]
    pub fn to_vec(&self) -> Vec<A>
    where
        A: Clone,
    {
        let mut result: Vec<A> = self.front.iter().rev().cloned().collect();
        result.extend(self.back.iter().cloned());
        result
    }

    /// Reverse the deque.
    #[inline]
    pub fn reverse(self) -> Self {
        Deque {
            front: self.back,
            back: self.front,
            len: self.len,
        }
    }

    /// Concatenate two deques.
    #[inline]
    pub fn concat(&self, other: &Self) -> Self
    where
        A: Clone,
    {
        let mut result = self.clone();
        // Add front of other (in order)
        for item in other.front.iter().rev() {
            result = result.push_back(item.clone());
        }
        // Add back of other
        for item in &other.back {
            result = result.push_back(item.clone());
        }
        result
    }
}

#[cfg(feature = "alloc")]
impl<A: Clone> From<Vec<A>> for Deque<A> {
    fn from(vec: Vec<A>) -> Self {
        let len = vec.len();
        Deque {
            front: Vec::new(),
            back: vec,
            len,
        }
    }
}

#[cfg(feature = "alloc")]
impl<A: Clone> FromIterator<A> for Deque<A> {
    fn from_iter<I: IntoIterator<Item = A>>(iter: I) -> Self {
        let back: Vec<A> = iter.into_iter().collect();
        let len = back.len();
        Deque {
            front: Vec::new(),
            back,
            len,
        }
    }
}

/// Iterator over a Deque (front to back).
#[cfg(feature = "alloc")]
pub struct DequeIter<A> {
    front: Vec<A>,
    back: Vec<A>,
}

#[cfg(feature = "alloc")]
impl<A> Iterator for DequeIter<A> {
    type Item = A;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.front.is_empty() {
            Some(self.front.pop().unwrap())
        } else if !self.back.is_empty() {
            // Reverse back so we can pop() from the logical front — O(n) once,
            // then each subsequent call is O(1) instead of O(n) via remove(0).
            self.back.reverse();
            core::mem::swap(&mut self.front, &mut self.back);
            self.front.pop()
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.front.len() + self.back.len();
        (len, Some(len))
    }
}

#[cfg(feature = "alloc")]
impl<A: Clone> IntoIterator for Deque<A> {
    type Item = A;
    type IntoIter = DequeIter<A>;

    fn into_iter(self) -> Self::IntoIter {
        DequeIter {
            front: self.front,
            back: self.back,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_new_is_empty() {
        let d: Deque<i32> = Deque::new();
        assert!(d.is_empty());
        assert_eq!(d.len(), 0);
    }

    #[test]
    fn test_push_front() {
        let d = Deque::new().push_front(1).push_front(2).push_front(3);
        assert_eq!(d.len(), 3);
        assert_eq!(d.peek_front(), Some(&3));
        assert_eq!(d.peek_back(), Some(&1));
    }

    #[test]
    fn test_push_back() {
        let d = Deque::new().push_back(1).push_back(2).push_back(3);
        assert_eq!(d.len(), 3);
        assert_eq!(d.peek_front(), Some(&1));
        assert_eq!(d.peek_back(), Some(&3));
    }

    #[test]
    fn test_pop_front() {
        let d = Deque::new().push_back(1).push_back(2).push_back(3);

        let (first, d) = d
            .pop_front()
            .expect("deque with 3 elements should have a front element");
        assert_eq!(first, 1);

        let (second, d) = d
            .pop_front()
            .expect("deque with 2 remaining elements should have a front element");
        assert_eq!(second, 2);

        let (third, d) = d
            .pop_front()
            .expect("deque with 1 remaining element should have a front element");
        assert_eq!(third, 3);

        assert!(d.is_empty());
    }

    #[test]
    fn test_pop_back() {
        let d = Deque::new().push_front(1).push_front(2).push_front(3);

        let (last, d) = d
            .pop_back()
            .expect("deque with 3 elements should have a back element");
        assert_eq!(last, 1);

        let (second_last, d) = d
            .pop_back()
            .expect("deque with 2 remaining elements should have a back element");
        assert_eq!(second_last, 2);

        let (first, d) = d
            .pop_back()
            .expect("deque with 1 remaining element should have a back element");
        assert_eq!(first, 3);

        assert!(d.is_empty());
    }

    #[test]
    fn test_persistence() {
        let d1 = Deque::new().push_back(1).push_back(2);
        let d2 = d1.clone().push_back(3);

        // d1 unchanged
        assert_eq!(d1.len(), 2);

        // d2 has new element
        assert_eq!(d2.len(), 3);
    }

    #[test]
    fn test_mixed_operations() {
        let d = Deque::new()
            .push_back(2)
            .push_front(1)
            .push_back(3)
            .push_front(0);

        assert_eq!(d.peek_front(), Some(&0));
        assert_eq!(d.peek_back(), Some(&3));

        let items: Vec<_> = d.into_iter().collect();
        assert_eq!(items, vec![0, 1, 2, 3]);
    }

    #[test]
    fn test_reverse() {
        let d = Deque::new().push_back(1).push_back(2).push_back(3);
        let r = d.reverse();

        assert_eq!(r.peek_front(), Some(&3));
        assert_eq!(r.peek_back(), Some(&1));
    }

    #[test]
    fn test_map() {
        let d = Deque::new().push_back(1).push_back(2).push_back(3);
        let doubled = d.map(|x| x * 2);

        let items: Vec<_> = doubled.into_iter().collect();
        assert_eq!(items, vec![2, 4, 6]);
    }

    #[test]
    fn test_fold() {
        let d = Deque::new().push_back(1).push_back(2).push_back(3);
        let sum = d.fold(0, |acc, x| acc + x);
        assert_eq!(sum, 6);
    }

    #[test]
    fn test_filter() {
        let d = Deque::new()
            .push_back(1)
            .push_back(2)
            .push_back(3)
            .push_back(4);
        let evens = d.filter(|x| x % 2 == 0);

        assert_eq!(evens.len(), 2);
    }

    #[test]
    fn test_filter_empty_deque_returns_empty() {
        let d: Deque<i32> = Deque::new();
        let result = d.filter(|_| true);
        assert_eq!(result.len(), 0);
        assert_eq!(result.to_vec(), Vec::<i32>::new());
    }

    #[test]
    fn test_filter_no_match_returns_empty() {
        let d = Deque::new().push_back(1).push_back(3).push_back(5);
        let evens = d.filter(|x| x % 2 == 0);
        assert_eq!(evens.len(), 0);
        assert_eq!(evens.to_vec(), Vec::<i32>::new());
    }

    #[test]
    fn test_filter_preserves_element_values() {
        // Elements pushed to front land in the front half; verify filter
        // correctly retains values from both internal halves of the deque.
        let d = Deque::new()
            .push_front(4) // front: [4]
            .push_front(3) // front: [3, 4]
            .push_back(5) // back:  [5]
            .push_back(6); // back:  [5, 6]  → to_vec: [3, 4, 5, 6]
        let evens = d.filter(|x| x % 2 == 0);
        assert_eq!(evens.len(), 2);
        let mut v = evens.to_vec();
        v.sort_unstable();
        assert_eq!(v, vec![4, 6]);
    }

    #[test]
    fn test_to_vec() {
        let d = Deque::new().push_back(1).push_back(2).push_back(3);
        assert_eq!(d.to_vec(), vec![1, 2, 3]);
    }

    #[test]
    fn test_concat() {
        let d1 = Deque::new().push_back(1).push_back(2);
        let d2 = Deque::new().push_back(3).push_back(4);
        let combined = d1.concat(&d2);

        assert_eq!(combined.len(), 4);
        assert_eq!(combined.to_vec(), vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_from_vec() {
        let v = vec![1, 2, 3];
        let d = Deque::from(v);

        assert_eq!(d.len(), 3);
        assert_eq!(d.to_vec(), vec![1, 2, 3]);
    }

    #[test]
    fn test_from_iter() {
        let d: Deque<i32> = vec![1, 2, 3].into_iter().collect();
        assert_eq!(d.to_vec(), vec![1, 2, 3]);
    }

    #[test]
    fn test_rebalance() {
        // Create deque with all elements in back
        let d = Deque::new()
            .push_back(1)
            .push_back(2)
            .push_back(3)
            .push_back(4);

        // Pop from front - should trigger rebalance
        let (first, d) = d
            .pop_front()
            .expect("deque should be non-empty before first front pop");
        assert_eq!(first, 1);

        let (second, _d) = d
            .pop_front()
            .expect("deque should be non-empty before second front pop");
        assert_eq!(second, 2);

        // Create deque with all elements in front
        let d = Deque::new()
            .push_front(1)
            .push_front(2)
            .push_front(3)
            .push_front(4);

        // Pop from back - should trigger rebalance
        let (last, d) = d
            .pop_back()
            .expect("deque should be non-empty before first back pop");
        assert_eq!(last, 1);

        let (second_last, _) = d
            .pop_back()
            .expect("deque should be non-empty before second back pop");
        assert_eq!(second_last, 2);
    }
}

#[cfg(all(test, feature = "serde"))]
mod serde_invariant_tests {
    use super::*;

    /// S2 regression: forged `len` made pop_front hit `.unwrap()` on an
    /// empty Vec (deque.rs:180). len is now recomputed, never trusted.
    #[test]
    fn forged_len_is_recomputed() {
        let d: Deque<i32> = serde_json::from_str(r#"{"front":[],"back":[],"len":5}"#).unwrap();
        assert_eq!(d.len(), 0);
        assert!(d.pop_front().is_none()); // pre-fix: panic
        let d: Deque<i32> =
            serde_json::from_str(r#"{"front":[1],"back":[2,3],"len":999}"#).unwrap();
        assert_eq!(d.len(), 3);
    }
}
