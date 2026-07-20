//! # Pigritia — Lazy Evaluation
//!
//! > *"Festina lente."*
//! > — Make haste slowly. (Augustus)
//!
//! The `Pigritia` (Latin for "laziness") type provides deferred computation
//! with optional memoization. This is useful for expensive computations that
//! may not always be needed.
//!
//! # Example
//!
//! ```rust
//! use ordofp_core::datatypes::Pigritia;
//!
//! // Create a lazy computation
//! let lazy = Pigritia::new(|| {
//!     println!("Computing...");
//!     42
//! });
//!
//! // Computation is not performed until force() is called
//! let value = lazy.force(); // Prints "Computing..." and returns 42
//! assert_eq!(*value, 42);
//! ```

use core::cell::OnceCell;

/// A lazy computation that evaluates its thunk at most once.
///
/// `Pigritia` (Latin for "laziness") wraps a computation that is deferred
/// until explicitly requested via `force()`. The result is then memoized
/// for subsequent accesses.
///
/// # Thread Safety
///
/// This implementation uses `OnceCell` which is not thread-safe.
/// For multi-threaded use, consider using `std::sync::OnceLock`.
///
/// # Examples
///
/// ```rust
/// use ordofp_core::datatypes::Pigritia;
///
/// // Expensive computation is deferred
/// let lazy = Pigritia::new(|| (1..=1000).sum::<i32>());
///
/// // Nothing computed yet
/// assert!(!lazy.is_evaluated());
///
/// // Force evaluation
/// let value = lazy.force();
/// assert_eq!(*value, 500500);
///
/// // Now it's evaluated and memoized
/// assert!(lazy.is_evaluated());
///
/// // Subsequent calls return the cached value
/// let same_value = lazy.force();
/// assert_eq!(*same_value, 500500);
/// ```
pub struct Pigritia<T, F = fn() -> T>
where
    F: FnOnce() -> T,
{
    cell: OnceCell<T>,
    thunk: core::cell::Cell<Option<F>>,
}

impl<T, F> Pigritia<T, F>
where
    F: FnOnce() -> T,
{
    /// Creates a new lazy computation from a thunk.
    ///
    /// The thunk will not be evaluated until `force()` is called.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::datatypes::Pigritia;
    ///
    /// let lazy = Pigritia::new(|| 42);
    /// assert_eq!(*lazy.force(), 42);
    /// ```
    #[inline]
    pub fn new(thunk: F) -> Self {
        Pigritia {
            cell: OnceCell::new(),
            thunk: core::cell::Cell::new(Some(thunk)),
        }
    }

    /// Forces evaluation of the lazy computation.
    ///
    /// If the computation has already been evaluated, returns the cached result.
    /// Otherwise, evaluates the thunk and caches the result.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::datatypes::Pigritia;
    ///
    /// let lazy = Pigritia::new(|| "hello".to_string());
    /// let value = lazy.force();
    /// assert_eq!(value.as_str(), "hello");
    /// ```
    ///
    /// # Panics
    ///
    /// Propagates any panic raised by the thunk itself. Additionally, if a
    /// previous `force` panicked mid-evaluation (consuming the thunk without
    /// caching a value), a subsequent call panics with "Pigritia thunk
    /// already consumed".
    #[inline]
    pub fn force(&self) -> &T {
        self.cell.get_or_init(|| {
            let thunk = self.thunk.take().expect("Pigritia thunk already consumed");
            thunk()
        })
    }

    /// Checks if the computation has been evaluated.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::datatypes::Pigritia;
    ///
    /// let lazy = Pigritia::new(|| 42);
    /// assert!(!lazy.is_evaluated());
    ///
    /// let _ = lazy.force();
    /// assert!(lazy.is_evaluated());
    /// ```
    #[inline]
    pub fn is_evaluated(&self) -> bool {
        self.cell.get().is_some()
    }

    /// Returns the cached value if already evaluated, without forcing.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::datatypes::Pigritia;
    ///
    /// let lazy = Pigritia::new(|| 42);
    /// assert_eq!(lazy.get(), None);
    ///
    /// let _ = lazy.force();
    /// assert_eq!(lazy.get(), Some(&42));
    /// ```
    #[inline]
    pub fn get(&self) -> Option<&T> {
        self.cell.get()
    }

    /// Consumes the lazy value and returns the computed result.
    ///
    /// If not yet evaluated, evaluates the thunk first.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::datatypes::Pigritia;
    ///
    /// let lazy = Pigritia::new(|| vec![1, 2, 3]);
    /// let value = lazy.into_inner();
    /// assert_eq!(value, vec![1, 2, 3]);
    /// ```
    ///
    /// # Panics
    ///
    /// Propagates any panic raised by the thunk itself. Additionally, if an
    /// earlier `force` panicked mid-evaluation (consuming the thunk without
    /// caching a value), this call panics with "Pigritia thunk already
    /// consumed".
    #[inline]
    pub fn into_inner(self) -> T {
        if let Some(value) = self.cell.into_inner() {
            value
        } else {
            let thunk = self.thunk.take().expect("Pigritia thunk already consumed");
            thunk()
        }
    }

    /// Maps a function over the lazy value, creating a new lazy computation.
    ///
    /// The mapping is also lazy - it won't be applied until the result is forced.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::datatypes::Pigritia;
    ///
    /// let lazy = Pigritia::new(|| 21);
    /// let doubled = lazy.map(|x| x * 2);
    /// assert_eq!(*doubled.force(), 42);
    /// ```
    #[inline]
    pub fn map<U, G>(self, f: G) -> Pigritia<U, impl FnOnce() -> U>
    where
        G: FnOnce(T) -> U,
    {
        Pigritia::new(move || f(self.into_inner()))
    }

    /// Flat maps a function over the lazy value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::datatypes::Pigritia;
    ///
    /// let lazy = Pigritia::new(|| 21);
    /// let result = lazy.flat_map(|x| Pigritia::new(move || x * 2));
    /// assert_eq!(*result.force(), 42);
    /// ```
    #[inline]
    pub fn flat_map<U, G, H>(self, f: G) -> Pigritia<U, impl FnOnce() -> U>
    where
        G: FnOnce(T) -> Pigritia<U, H>,
        H: FnOnce() -> U,
    {
        Pigritia::new(move || f(self.into_inner()).into_inner())
    }
}

impl<T> Pigritia<T, fn() -> T> {
    /// Creates a lazy value that is already evaluated.
    ///
    /// This is useful for lifting pure values into the lazy context.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::datatypes::Pigritia;
    ///
    /// let lazy = Pigritia::<i32, fn() -> i32>::pure(42);
    /// assert!(lazy.is_evaluated());
    /// assert_eq!(*lazy.force(), 42);
    /// ```
    #[inline]
    pub fn pure(value: T) -> Self {
        let lazy = Pigritia {
            cell: OnceCell::new(),
            thunk: core::cell::Cell::new(None),
        };
        let _ = lazy.cell.set(value);
        lazy
    }
}

impl<T: Clone> Clone for Pigritia<T, fn() -> T> {
    /// # Panics
    ///
    /// Panics if the value has not been evaluated yet (`force` not called):
    /// the thunk cannot be duplicated, so only evaluated `Pigritia` clones.
    fn clone(&self) -> Self {
        if let Some(value) = self.cell.get() {
            Pigritia::<T, fn() -> T>::pure(value.clone())
        } else {
            panic!("Cannot clone unevaluated Pigritia")
        }
    }
}

impl<T: core::fmt::Debug, F: FnOnce() -> T> core::fmt::Debug for Pigritia<T, F> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.cell.get() {
            Some(value) => f.debug_tuple("Pigritia").field(value).finish(),
            None => f.debug_tuple("Pigritia").field(&"<unevaluated>").finish(),
        }
    }
}

impl<T: PartialEq, F: FnOnce() -> T> PartialEq for Pigritia<T, F> {
    /// Note: comparing **forces evaluation** of both sides — `eq` calls
    /// `force()` on `self` and `other`, running any pending thunks.
    fn eq(&self, other: &Self) -> bool {
        self.force() == other.force()
    }
}

impl<T: Eq, F: FnOnce() -> T> Eq for Pigritia<T, F> {}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use core::cell::Cell;

    #[test]
    fn test_lazy_creation() {
        let lazy = Pigritia::new(|| 42);
        assert!(!lazy.is_evaluated());
    }

    #[test]
    fn test_lazy_force() {
        let lazy = Pigritia::new(|| 42);
        assert_eq!(*lazy.force(), 42);
        assert!(lazy.is_evaluated());
    }

    #[test]
    fn test_lazy_memoization() {
        let counter = Cell::new(0);
        let lazy = Pigritia::new(|| {
            counter.set(counter.get() + 1);
            42
        });

        assert_eq!(counter.get(), 0);
        let _ = lazy.force();
        assert_eq!(counter.get(), 1);
        let _ = lazy.force();
        assert_eq!(counter.get(), 1); // Not called again
    }

    #[test]
    fn test_lazy_get() {
        let lazy = Pigritia::new(|| 42);
        assert_eq!(lazy.get(), None);
        let _ = lazy.force();
        assert_eq!(lazy.get(), Some(&42));
    }

    #[test]
    fn test_lazy_into_inner() {
        let lazy = Pigritia::new(|| alloc::vec![1, 2, 3]);
        let value = lazy.into_inner();
        assert_eq!(value, alloc::vec![1, 2, 3]);
    }

    #[test]
    fn test_lazy_map() {
        let lazy = Pigritia::new(|| 21);
        let doubled = lazy.map(|x| x * 2);
        assert_eq!(*doubled.force(), 42);
    }

    #[test]
    fn test_lazy_flat_map() {
        let lazy = Pigritia::new(|| 21);
        let result = lazy.flat_map(|x| Pigritia::new(move || x * 2));
        assert_eq!(*result.force(), 42);
    }

    #[test]
    fn test_lazy_pure() {
        let lazy = Pigritia::<i32, fn() -> i32>::pure(42);
        assert!(lazy.is_evaluated());
        assert_eq!(*lazy.force(), 42);
    }

    #[test]
    fn test_lazy_into_inner_already_evaluated() {
        // When `force()` has been called first, `into_inner` must return the
        // memoised value via the `cell.into_inner()` branch, not the thunk.
        let call_count = Cell::new(0usize);
        let lazy = Pigritia::new(|| {
            call_count.set(call_count.get() + 1);
            99
        });
        let _ = lazy.force(); // evaluate and memoise
        assert_eq!(
            call_count.get(),
            1,
            "thunk should have been called exactly once"
        );
        let value = lazy.into_inner(); // must use the cached branch
        assert_eq!(value, 99, "into_inner must return the memoised value");
        // call_count is still 1 because the thunk was NOT re-invoked.
        assert_eq!(
            call_count.get(),
            1,
            "thunk must not be called again by into_inner"
        );
    }

    #[test]
    fn test_lazy_eq() {
        let a = Pigritia::<i32, fn() -> i32>::pure(42);
        let b = Pigritia::<i32, fn() -> i32>::pure(42);
        assert_eq!(a, b);
    }

    #[test]
    fn test_lazy_debug() {
        let lazy = Pigritia::new(|| 42);
        let debug_str = alloc::format!("{lazy:?}");
        assert!(debug_str.contains("unevaluated"));

        let _ = lazy.force();
        let debug_str = alloc::format!("{lazy:?}");
        assert!(debug_str.contains("42"));
    }
}
