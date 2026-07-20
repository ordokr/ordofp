//! Traversable type class for effectful iteration.
//!
//! `Traversable` allows mapping effectful operations over a structure
//! while collecting the effects.
//!
//! # Example
//!
//! ```rust
//! use ordofp_core::traversable::sequence_option;
//!
//! // Sequence: turn Vec<Option<T>> into Option<Vec<T>>
//! let sequenced = sequence_option(
//!     vec![Some(1), Some(2), Some(3)]
//! );
//! assert_eq!(sequenced, Some(vec![1, 2, 3]));
//!
//! let sequenced = sequence_option(
//!     vec![Some(1), None, Some(3)]
//! );
//! assert_eq!(sequenced, None);
//! ```

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// Traversable functor - for effectful iteration.
///
/// A `Traversable` is a structure that can be traversed while
/// accumulating effects. The key operations are:
///
/// - `traverse`: Map a function producing effects, then sequence the effects
/// - `sequence`: Turn `F<G<A>>` into `G<F<A>>` by sequencing the inner effects
///
/// # Laws
///
/// 1. **Identity**: `traverse(Identity) == Identity`
/// 2. **Naturality**: `t . traverse(f) == traverse(t . f)` for applicative morphism `t`
/// 3. **Composition**: `traverse(Compose . fmap(g) . f) == Compose . fmap(traverse(g)) . traverse(f)`
pub trait Traversable<A> {
    /// The output container type.
    type Output<B>;

    /// Map a function producing Option over the structure,
    /// then sequence the Options.
    ///
    /// Returns `None` if any element produces `None`.
    fn traverse_option<F, B>(&self, f: F) -> Option<Self::Output<B>>
    where
        F: Fn(&A) -> Option<B>;

    /// Map a function producing Result over the structure,
    /// then sequence the Results.
    ///
    /// Returns `Err` on the first error encountered.
    ///
    /// # Errors
    ///
    /// Returns the first `Err` that `f` produces, in traversal order;
    /// elements after it are not visited and earlier successes are
    /// discarded.
    fn traverse_result<F, B, E>(&self, f: F) -> Result<Self::Output<B>, E>
    where
        F: Fn(&A) -> Result<B, E>;

    /// Sequence a structure of Options into an Option of structure.
    ///
    /// `sequence([Some(1), Some(2), Some(3)]) == Some([1, 2, 3])`
    /// `sequence([Some(1), None, Some(3)]) == None`
    #[inline]
    fn sequence_option(structure: Self) -> Option<Self::Output<A>>
    where
        Self: Sized,
    {
        Self::traverse_option_owned(structure, Some)
    }

    /// Sequence a structure of Results into a Result of structure.
    ///
    /// # Errors
    ///
    /// Returns the first `Err` found in the structure, in traversal
    /// order; the remaining elements' successes are discarded.
    #[inline]
    fn sequence_result<E>(structure: Self) -> Result<Self::Output<A>, E>
    where
        Self: Sized,
    {
        Self::traverse_result_owned(structure, Ok)
    }

    /// Traverse with ownership.
    fn traverse_option_owned<F, B>(this: Self, f: F) -> Option<Self::Output<B>>
    where
        F: FnMut(A) -> Option<B>,
        Self: Sized;

    /// Traverse Result with ownership.
    ///
    /// # Errors
    ///
    /// Returns the first `Err` that `f` produces, in traversal order;
    /// elements after it are not visited.
    fn traverse_result_owned<F, B, E>(this: Self, f: F) -> Result<Self::Output<B>, E>
    where
        F: FnMut(A) -> Result<B, E>,
        Self: Sized;
}

// Implementation for Vec
#[cfg(feature = "alloc")]
impl<A> Traversable<A> for Vec<A> {
    type Output<B> = Vec<B>;

    #[inline]
    fn traverse_option<F, B>(&self, f: F) -> Option<Vec<B>>
    where
        F: Fn(&A) -> Option<B>,
    {
        let mut result = Vec::with_capacity(self.len());
        for item in self {
            {
                let b = f(item)?;
                result.push(b);
            }
        }
        Some(result)
    }

    #[inline]
    fn traverse_result<F, B, E>(&self, f: F) -> Result<Vec<B>, E>
    where
        F: Fn(&A) -> Result<B, E>,
    {
        let mut result = Vec::with_capacity(self.len());
        for item in self {
            {
                let b = f(item)?;
                result.push(b);
            }
        }
        Ok(result)
    }

    #[inline]
    fn traverse_option_owned<F, B>(this: Self, mut f: F) -> Option<Vec<B>>
    where
        F: FnMut(A) -> Option<B>,
    {
        let mut result = Vec::with_capacity(this.len());
        for item in this {
            {
                let b = f(item)?;
                result.push(b);
            }
        }
        Some(result)
    }

    #[inline]
    fn traverse_result_owned<F, B, E>(this: Self, mut f: F) -> Result<Vec<B>, E>
    where
        F: FnMut(A) -> Result<B, E>,
    {
        let mut result = Vec::with_capacity(this.len());
        for item in this {
            {
                let b = f(item)?;
                result.push(b);
            }
        }
        Ok(result)
    }
}

// Implementation for Option
impl<A> Traversable<A> for Option<A> {
    type Output<B> = Option<B>;

    #[inline]
    fn traverse_option<F, B>(&self, f: F) -> Option<Option<B>>
    where
        F: Fn(&A) -> Option<B>,
    {
        match self {
            Some(a) => f(a).map(Some),
            None => Some(None),
        }
    }

    #[inline]
    fn traverse_result<F, B, E>(&self, f: F) -> Result<Option<B>, E>
    where
        F: Fn(&A) -> Result<B, E>,
    {
        match self {
            Some(a) => f(a).map(Some),
            None => Ok(None),
        }
    }

    #[inline]
    fn traverse_option_owned<F, B>(this: Self, mut f: F) -> Option<Option<B>>
    where
        F: FnMut(A) -> Option<B>,
    {
        match this {
            Some(a) => f(a).map(Some),
            None => Some(None),
        }
    }

    #[inline]
    fn traverse_result_owned<F, B, E>(this: Self, mut f: F) -> Result<Option<B>, E>
    where
        F: FnMut(A) -> Result<B, E>,
    {
        match this {
            Some(a) => f(a).map(Some),
            None => Ok(None),
        }
    }
}

// Implementation for Result (E2: Clone because the borrowed traversals must
// produce an owned Err from &self)
impl<A, E2: Clone> Traversable<A> for Result<A, E2> {
    type Output<B> = Result<B, E2>;

    #[inline]
    fn traverse_option<F, B>(&self, f: F) -> Option<Result<B, E2>>
    where
        F: Fn(&A) -> Option<B>,
    {
        match self {
            Ok(a) => f(a).map(Ok),
            Err(e) => Some(Err(e.clone())),
        }
    }

    #[inline]
    fn traverse_result<F, B, E>(&self, f: F) -> Result<Result<B, E2>, E>
    where
        F: Fn(&A) -> Result<B, E>,
    {
        match self {
            Ok(a) => f(a).map(Ok),
            Err(e) => Ok(Err(e.clone())),
        }
    }

    #[inline]
    fn traverse_option_owned<F, B>(this: Self, mut f: F) -> Option<Result<B, E2>>
    where
        F: FnMut(A) -> Option<B>,
    {
        match this {
            Ok(a) => f(a).map(Ok),
            Err(e) => Some(Err(e)),
        }
    }

    #[inline]
    fn traverse_result_owned<F, B, E>(this: Self, mut f: F) -> Result<Result<B, E2>, E>
    where
        F: FnMut(A) -> Result<B, E>,
    {
        match this {
            Ok(a) => f(a).map(Ok),
            Err(e) => Ok(Err(e)),
        }
    }
}

/// Helper for sequencing Vec of Options
#[cfg(feature = "alloc")]
#[inline]
pub fn sequence_option<A>(v: Vec<Option<A>>) -> Option<Vec<A>> {
    // No `A: Clone` needed: traverse_*_owned consumes `v` and moves each
    // element out, so this works for non-Clone element types too.
    Traversable::traverse_option_owned(v, core::convert::identity)
}

/// Helper for sequencing Vec of Results
///
/// # Errors
///
/// Returns the first `Err` in `v`, in order; later elements and earlier
/// successes are discarded.
#[cfg(feature = "alloc")]
#[inline]
pub fn sequence_result<A, E>(v: Vec<Result<A, E>>) -> Result<Vec<A>, E> {
    // No `A: Clone` needed: traverse_*_owned consumes `v` and moves each
    // element out, so this works for non-Clone element types too.
    Traversable::traverse_result_owned(v, core::convert::identity)
}

/// Helper for traversing with Option-producing function
#[cfg(feature = "alloc")]
#[inline]
pub fn traverse_option<A, B, F>(v: Vec<A>, f: F) -> Option<Vec<B>>
where
    F: FnMut(A) -> Option<B>,
{
    Traversable::traverse_option_owned(v, f)
}

/// Helper for traversing with Result-producing function
///
/// # Errors
///
/// Returns the first `Err` that `f` produces while walking `v` in
/// order; elements after it are not visited.
#[cfg(feature = "alloc")]
#[inline]
pub fn traverse_result<A, B, E, F>(v: Vec<A>, f: F) -> Result<Vec<B>, E>
where
    F: FnMut(A) -> Result<B, E>,
{
    Traversable::traverse_result_owned(v, f)
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate alloc;
    use alloc::vec;
    use alloc::vec::Vec;

    #[test]
    fn test_vec_traverse_option_all_some() {
        let v = vec![1, 2, 3, 4];
        let result = v.traverse_option(|x| Some(x * 2));
        assert_eq!(result, Some(vec![2, 4, 6, 8]));
    }

    #[test]
    fn sequence_helpers_do_not_require_clone() {
        // Regression: sequence_option/sequence_result consume their Vec and
        // move elements out, so they must compile for non-Clone element types.
        // This test only builds if the spurious `A: Clone` bound stays removed.
        struct NoClone(i32);
        let some = vec![Some(NoClone(1)), Some(NoClone(2))];
        // Read the payload too: proves the moved-out elements are intact.
        assert_eq!(
            sequence_option(some).map(|v| (v.len(), v[0].0)),
            Some((2, 1))
        );
        let has_none = vec![Some(NoClone(1)), None];
        assert!(sequence_option(has_none).is_none());
        let oks: Vec<Result<NoClone, ()>> = vec![Ok(NoClone(1)), Ok(NoClone(2))];
        assert_eq!(sequence_result(oks).map(|v| v.len()), Ok(2));
    }

    #[test]
    fn test_vec_traverse_option_with_none() {
        let v = vec![1, 2, 3, 4];
        let result = v.traverse_option(|&x| if x == 3 { None } else { Some(x * 2) });
        assert_eq!(result, None);
    }

    #[test]
    fn test_vec_traverse_result_all_ok() {
        let v = vec![1, 2, 3, 4];
        let result: Result<Vec<i32>, &str> = v.traverse_result(|x| Ok(x * 2));
        assert_eq!(result, Ok(vec![2, 4, 6, 8]));
    }

    #[test]
    fn test_vec_traverse_result_with_err() {
        let v = vec![1, 2, 3, 4];
        let result: Result<Vec<i32>, &str> =
            v.traverse_result(|&x| if x == 3 { Err("three") } else { Ok(x * 2) });
        assert_eq!(result, Err("three"));
    }

    #[test]
    fn test_sequence_option_all_some() {
        let v = vec![Some(1), Some(2), Some(3)];
        assert_eq!(sequence_option(v), Some(vec![1, 2, 3]));
    }

    #[test]
    fn test_sequence_option_with_none() {
        let v = vec![Some(1), None, Some(3)];
        assert_eq!(sequence_option(v), None);
    }

    #[test]
    fn test_sequence_result_all_ok() {
        let v: Vec<Result<i32, &str>> = vec![Ok(1), Ok(2), Ok(3)];
        assert_eq!(sequence_result(v), Ok(vec![1, 2, 3]));
    }

    #[test]
    fn test_sequence_result_with_err() {
        let v: Vec<Result<i32, &str>> = vec![Ok(1), Err("error"), Ok(3)];
        assert_eq!(sequence_result(v), Err("error"));
    }

    #[test]
    fn test_option_traverse_option_some() {
        let opt = Some(5);
        let result = opt.traverse_option(|x| Some(x * 2));
        assert_eq!(result, Some(Some(10)));
    }

    #[test]
    fn test_option_traverse_option_none_input() {
        let opt: Option<i32> = None;
        let result = opt.traverse_option(|x| Some(x * 2));
        assert_eq!(result, Some(None));
    }

    #[test]
    fn test_option_traverse_option_none_result() {
        let opt = Some(5);
        let result: Option<Option<i32>> = opt.traverse_option(|_| None);
        assert_eq!(result, None);
    }

    #[test]
    fn test_traverse_helper_option() {
        let v = vec![1, 2, 3];
        let result = traverse_option(v, |x| Some(x * 2));
        assert_eq!(result, Some(vec![2, 4, 6]));
    }

    #[test]
    fn test_traverse_helper_result() {
        let v = vec![1, 2, 3];
        let result: Result<Vec<i32>, &str> = traverse_result(v, |x| Ok(x * 2));
        assert_eq!(result, Ok(vec![2, 4, 6]));
    }

    #[test]
    fn test_empty_vec_traverse_option() {
        let v: Vec<i32> = vec![];
        let result = v.traverse_option(|x| Some(x * 2));
        assert_eq!(result, Some(vec![]));
    }

    #[test]
    fn test_empty_vec_traverse_result() {
        let v: Vec<i32> = vec![];
        let result: Result<Vec<i32>, &str> = v.traverse_result(|x| Ok(x * 2));
        assert_eq!(result, Ok(vec![]));
    }

    #[test]
    fn test_result_traverse_option_ok() {
        let r: Result<i32, &str> = Ok(5);
        let result = r.traverse_option(|x| Some(x * 2));
        assert_eq!(result, Some(Ok(10)));
    }

    #[test]
    fn test_result_traverse_option_err() {
        let r: Result<i32, &str> = Err("error");
        let result = r.traverse_option(|x| Some(x * 2));
        assert_eq!(result, Some(Err("error")));
    }
}
