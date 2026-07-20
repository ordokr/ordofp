//! # `MonadPlus` — Monad with Choice Operations
//!
//! > *"In pluribus unum, ex pluribus electio."*
//! > — In many, one; from many, choice.
//!
//! The `MonadPlus` trait extends monads with choice operations, providing:
//! - `vacuus` (mzero): A monad representing failure or empty computation
//! - `electio` (mplus): An operation to combine two monads as alternatives
//!
//! This is similar to `Alternative` in Haskell but specifically for monads.
//!
//! # Mathematical Laws
//!
//! For a valid `MonadPlus` implementation:
//!
//! 1. **Left Identity**: `vacuus().electio(&x) == x`
//! 2. **Right Identity**: `x.electio(&vacuus()) == x`
//! 3. **Associativity**: `a.electio(&b).electio(&c) == a.electio(&b.electio(&c))`
//! 4. **Left Zero**: `vacuus().flat_map(f) == vacuus()`
//! 5. **Right Zero**: `x.flat_map(|_| vacuus()) == vacuus()`
//!
//! # Example
//!
//! ```rust
//! use ordofp_core::typeclasses::MonadPlus;
//!
//! // Option is a MonadPlus where None is vacuus
//! let opt1: Option<i32> = Some(42);
//! let opt2: Option<i32> = None;
//! let opt3: Option<i32> = Some(7);
//!
//! // vacuus is the empty/failure case
//! assert_eq!(Option::<i32>::vacuus::<i32>(), None);
//!
//! // electio chooses the first successful value
//! assert_eq!(opt1.electio(&opt2), Some(42));
//! assert_eq!(opt2.electio(&opt3), Some(7));
//! assert_eq!(opt2.clone().electio(&opt2), None);
//! ```

/// A trait for monads that support choice operations.
///
/// `MonadPlus` extends the basic Monad trait with operations for alternative
/// computations, providing a "zero" element (`vacuus`) and a way to combine
/// alternatives (`electio`).
///
/// # Scholastic Naming
///
/// - `vacuus` (Latin: *empty, void*) — the empty computation
/// - `electio` (Latin: *choice, selection*) — combining alternatives
///
/// # Laws
///
/// 1. **Identity**: `vacuus().electio(&x) == x` and `x.electio(&vacuus()) == x`
/// 2. **Associativity**: `a.electio(&b).electio(&c) == a.electio(&b.electio(&c))`
/// 3. **Left Zero**: `vacuus().flat_map(f) == vacuus()`
pub trait MonadPlus: Sized + Clone {
    /// The inner type contained by this monad.
    type Inner;

    /// Creates an empty or failed computation.
    ///
    /// This is the identity element for the `electio` operation.
    ///
    /// # Returns
    ///
    /// A new monadic value representing failure or empty computation.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::typeclasses::MonadPlus;
    ///
    /// let empty: Option<i32> = Option::vacuus();
    /// assert_eq!(empty, None);
    /// ```
    fn vacuus<T>() -> Self
    where
        Self: MonadPlus<Inner = T>;

    /// Combines two monads, representing a choice between them.
    ///
    /// In most implementations, this takes the first non-empty/non-failure value.
    ///
    /// # Parameters
    ///
    /// * `other`: Another monadic value to combine with this one
    ///
    /// # Returns
    ///
    /// A combined monadic value representing a choice between the two.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::typeclasses::MonadPlus;
    ///
    /// let a: Option<i32> = Some(10);
    /// let b: Option<i32> = Some(20);
    /// let c: Option<i32> = None;
    ///
    /// assert_eq!(a.electio(&b), Some(10)); // first wins
    /// assert_eq!(c.electio(&b), Some(20)); // fallback to second
    /// ```
    fn electio(&self, other: &Self) -> Self;

    /// Combines two monads, consuming both.
    ///
    /// This is an owned variant of `electio` that can avoid cloning.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::typeclasses::MonadPlus;
    ///
    /// let a: Option<i32> = None;
    /// let b: Option<i32> = Some(42);
    ///
    /// assert_eq!(a.electio_owned(b), Some(42));
    /// ```
    fn electio_owned(self, other: Self) -> Self;

    /// Filters values, returning `vacuus` if the predicate fails.
    ///
    /// This is the monadic filter operation.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::typeclasses::MonadPlus;
    ///
    /// let a: Option<i32> = Some(42);
    /// assert_eq!(a.mfilter(|x| *x > 40), Some(42));
    /// assert_eq!(a.mfilter(|x| *x > 50), None);
    /// ```
    fn mfilter<F>(self, pred: F) -> Self
    where
        F: FnOnce(&Self::Inner) -> bool;

    /// Repeats the choice operation, taking any successful value.
    ///
    /// Combines multiple monadic values, taking the first success.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::typeclasses::MonadPlus;
    ///
    /// let vals: [Option<i32>; 4] = [None, None, Some(42), Some(100)];
    /// let result = Option::<i32>::asum(vals.iter().cloned());
    /// assert_eq!(result, Some(42));
    /// ```
    fn asum<I>(iter: I) -> Self
    where
        I: Iterator<Item = Self>;
}

// ============================================================================
// Implementation for Option<A>
// ============================================================================

impl<A: Clone> MonadPlus for Option<A> {
    type Inner = A;

    #[inline]
    fn vacuus<T>() -> Self
    where
        Self: MonadPlus<Inner = T>,
    {
        None
    }

    #[inline]
    fn electio(&self, other: &Self) -> Self {
        match self {
            Some(_) => self.clone(),
            None => other.clone(),
        }
    }

    #[inline]
    fn electio_owned(self, other: Self) -> Self {
        match self {
            Some(_) => self,
            None => other,
        }
    }

    #[inline]
    fn mfilter<F>(self, pred: F) -> Self
    where
        F: FnOnce(&Self::Inner) -> bool,
    {
        match self {
            Some(ref a) if pred(a) => self,
            _ => None,
        }
    }

    #[inline]
    fn asum<I>(iter: I) -> Self
    where
        I: Iterator<Item = Self>,
    {
        let mut result = None;
        for item in iter {
            result = result.electio_owned(item);
            if result.is_some() {
                break;
            }
        }
        result
    }
}

// ============================================================================
// Implementation for Result<A, E>
// ============================================================================

impl<A: Clone, E: Clone + Default> MonadPlus for Result<A, E> {
    type Inner = A;

    #[inline]
    fn vacuus<T>() -> Self
    where
        Self: MonadPlus<Inner = T>,
    {
        Err(E::default())
    }

    #[inline]
    fn electio(&self, other: &Self) -> Self {
        match self {
            Ok(_) => self.clone(),
            Err(_) => other.clone(),
        }
    }

    #[inline]
    fn electio_owned(self, other: Self) -> Self {
        match self {
            Ok(_) => self,
            Err(_) => other,
        }
    }

    #[inline]
    fn mfilter<F>(self, pred: F) -> Self
    where
        F: FnOnce(&Self::Inner) -> bool,
    {
        match self {
            Ok(ref a) if pred(a) => self,
            Ok(_) => Err(E::default()),
            Err(e) => Err(e),
        }
    }

    #[inline]
    fn asum<I>(iter: I) -> Self
    where
        I: Iterator<Item = Self>,
    {
        let mut result: Self = Err(E::default());
        for item in iter {
            result = result.electio_owned(item);
            if result.is_ok() {
                break;
            }
        }
        result
    }
}

// ============================================================================
// Helper functions
// ============================================================================

/// Creates an empty/failure computation.
///
/// # Example
///
/// ```rust
/// use ordofp_core::typeclasses::monad_plus::vacuus;
///
/// let empty: Option<i32> = vacuus();
/// assert_eq!(empty, None);
/// ```
#[inline]
pub fn vacuus<A, M>() -> M
where
    M: MonadPlus<Inner = A>,
{
    M::vacuus()
}

/// Chooses between two monadic values.
///
/// # Example
///
/// ```rust
/// use ordofp_core::typeclasses::monad_plus::electio;
///
/// let a: Option<i32> = None;
/// let b: Option<i32> = Some(42);
/// assert_eq!(electio(&a, &b), Some(42));
/// ```
#[inline]
pub fn electio<M: MonadPlus>(a: &M, b: &M) -> M {
    a.electio(b)
}

/// Combines an iterator of monadic values, taking the first success.
///
/// # Example
///
/// ```rust
/// use ordofp_core::typeclasses::monad_plus::asum;
///
/// let vals = vec![None, None, Some(42), Some(100)];
/// let result: Option<i32> = asum(vals.into_iter());
/// assert_eq!(result, Some(42));
/// ```
#[inline]
pub fn asum<A, M, I>(iter: I) -> M
where
    M: MonadPlus<Inner = A>,
    I: Iterator<Item = M>,
{
    M::asum(iter)
}

/// Guards a computation, returning the unit value if successful.
///
/// This is a standalone function for `Option<()>` type.
///
/// # Example
///
/// ```rust
/// use ordofp_core::typeclasses::monad_plus::guard;
///
/// let result: Option<()> = guard(true);
/// assert_eq!(result, Some(()));
///
/// let result: Option<()> = guard(false);
/// assert_eq!(result, None);
/// ```
#[inline]
pub fn guard(cond: bool) -> Option<()> {
    if cond { Some(()) } else { None }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_option_vacuus() {
        let empty: Option<i32> = Option::vacuus();
        assert_eq!(empty, None);
    }

    #[test]
    fn test_option_electio() {
        let a: Option<i32> = Some(10);
        let b: Option<i32> = Some(20);
        let c: Option<i32> = None;

        assert_eq!(a.electio(&b), Some(10));
        assert_eq!(a.electio(&c), Some(10));
        assert_eq!(c.electio(&b), Some(20));
        assert_eq!(c.electio(&c), None);
    }

    #[test]
    fn test_option_electio_owned() {
        let a: Option<i32> = Some(10);
        let b: Option<i32> = Some(20);

        assert_eq!(a.electio_owned(b), Some(10));

        let a: Option<i32> = None;
        let b: Option<i32> = Some(20);
        assert_eq!(a.electio_owned(b), Some(20));
    }

    #[test]
    fn test_option_mfilter() {
        let a: Option<i32> = Some(42);
        assert_eq!(a.mfilter(|x| *x > 40), Some(42));
        assert_eq!(a.mfilter(|x| *x > 50), None);

        let b: Option<i32> = None;
        assert_eq!(b.mfilter(|x| *x > 0), None);
    }

    #[test]
    fn test_option_asum() {
        let vals: [Option<i32>; 4] = [None, None, Some(42), Some(100)];
        let result = Option::<i32>::asum(vals.into_iter());
        assert_eq!(result, Some(42));

        let empty: [Option<i32>; 3] = [None, None, None];
        let result = Option::<i32>::asum(empty.into_iter());
        assert_eq!(result, None);
    }

    #[test]
    fn test_guard() {
        let result = guard(true);
        assert_eq!(result, Some(()));

        let result = guard(false);
        assert_eq!(result, None);
    }

    #[test]
    fn test_result_vacuus() {
        let empty: Result<i32, &str> = Result::vacuus();
        assert_eq!(empty, Err(""));
    }

    #[test]
    fn test_result_electio() {
        let a: Result<i32, &str> = Ok(10);
        let b: Result<i32, &str> = Ok(20);
        let c: Result<i32, &str> = Err("error");

        assert_eq!(a.electio(&b), Ok(10));
        assert_eq!(a.electio(&c), Ok(10));
        assert_eq!(c.electio(&b), Ok(20));
        assert_eq!(c.electio(&c), Err("error"));
    }

    #[test]
    fn test_result_mfilter() {
        let a: Result<i32, &str> = Ok(42);
        assert_eq!(a.mfilter(|x| *x > 40), Ok(42));
        assert_eq!(a.mfilter(|x| *x > 50), Err(""));
    }

    // ========================================================================
    // Laws Tests
    // ========================================================================

    #[test]
    fn test_left_identity_law() {
        // vacuus().electio(&x) == x
        let x: Option<i32> = Some(42);
        let empty: Option<i32> = Option::vacuus();
        assert_eq!(empty.electio(&x), x);
    }

    #[test]
    fn test_right_identity_law() {
        // x.electio(&vacuus()) == x
        let x: Option<i32> = Some(42);
        let empty: Option<i32> = Option::vacuus();
        assert_eq!(x.electio(&empty), x);
    }

    #[test]
    fn test_associativity_law() {
        // a.electio(&b).electio(&c) == a.electio(&b.electio(&c))
        let a: Option<i32> = Some(1);
        let b: Option<i32> = Some(2);
        let c: Option<i32> = Some(3);

        let left = a.electio(&b).electio(&c);
        let right = a.electio(&b.electio(&c));
        assert_eq!(left, right);

        // Also with None values
        let a: Option<i32> = None;
        let b: Option<i32> = Some(2);
        let c: Option<i32> = None;

        let left = a.electio(&b).electio(&c);
        let right = a.electio(&b.electio(&c));
        assert_eq!(left, right);
    }

    #[test]
    fn test_helper_functions() {
        let empty: Option<i32> = vacuus();
        assert_eq!(empty, None);

        let a: Option<i32> = None;
        let b: Option<i32> = Some(42);
        assert_eq!(electio(&a, &b), Some(42));

        let vals = [None, None, Some(42), Some(100)];
        let result: Option<i32> = asum(vals.into_iter());
        assert_eq!(result, Some(42));

        let g: Option<()> = guard(true);
        assert_eq!(g, Some(()));
    }
}
