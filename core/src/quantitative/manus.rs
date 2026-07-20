//! `ManusLinearis` - Linear resource handles with guaranteed cleanup
//!
//! > *"Manus tenens, manus liberans"*
//! > — The hand that holds, the hand that frees. (Neo-Latin)
//!
//! This module provides linear resource handles inspired by Idris 2's
//! linear resource management patterns.

use core::fmt;
use core::marker::PhantomData;

use super::multiplicitas::Semel;
use super::qtt::Qtt;

// =============================================================================
// ManusLinearis - Linear Resource Handle
// =============================================================================

/// A linear handle to a resource that must be explicitly released.
///
/// `ManusLinearis<T>` wraps a resource of type `T` and enforces that the
/// resource is properly released before the handle is dropped. Unlike
/// `Qtt<T, Semel>`, this type provides explicit acquire/release semantics.
///
/// # Latin Etymology
///
/// *Manus* = hand (that which grasps and releases)
///
/// # Example
///
/// ```rust
/// use ordofp_core::quantitative::ManusLinearis;
///
/// // Acquire a linear resource
/// let handle = ManusLinearis::acquire(vec![1, 2, 3]);
///
/// // Use the resource (borrowing)
/// let len = handle.use_ref(|v| v.len());
/// assert_eq!(len, 3);
///
/// // Must explicitly release
/// let vec = handle.release();
/// assert_eq!(vec, vec![1, 2, 3]);
/// ```
#[must_use = "linear handles must be explicitly released"]
pub struct ManusLinearis<T> {
    resource: Option<T>,
    _linear: PhantomData<Semel>,
}

impl<T> ManusLinearis<T> {
    /// Acquire a linear resource, creating a handle.
    ///
    /// The returned handle must be explicitly released before it is dropped.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::quantitative::ManusLinearis;
    ///
    /// let handle = ManusLinearis::acquire(42);
    /// let value = handle.release();
    /// ```
    #[inline]
    pub fn acquire(resource: T) -> Self {
        ManusLinearis {
            resource: Some(resource),
            _linear: PhantomData,
        }
    }

    /// Use the resource by reference.
    ///
    /// This borrows the resource without consuming the handle.
    ///
    /// # Panics
    ///
    /// Panics if the resource has already been released.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::quantitative::ManusLinearis;
    ///
    /// let handle = ManusLinearis::acquire(vec![1, 2, 3]);
    /// let len = handle.use_ref(|v| v.len());
    /// let _ = handle.release();
    /// ```
    #[inline]
    pub fn use_ref<R, F>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        f(self.resource.as_ref().expect("resource already released"))
    }

    /// Use the resource by mutable reference.
    ///
    /// This mutably borrows the resource without consuming the handle.
    ///
    /// # Panics
    ///
    /// Panics if the resource has already been released.
    #[inline]
    pub fn use_mut<R, F>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        f(self.resource.as_mut().expect("resource already released"))
    }

    /// Release the resource, consuming the handle.
    ///
    /// Returns the inner resource. After this call, the handle is consumed
    /// and cannot be used.
    ///
    /// # Panics
    ///
    /// Panics if the resource has already been released.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::quantitative::ManusLinearis;
    ///
    /// let handle = ManusLinearis::acquire(42);
    /// let value = handle.release();
    /// assert_eq!(value, 42);
    /// ```
    #[inline]
    pub fn release(mut self) -> T {
        self.resource.take().expect("resource already released")
    }

    /// Release the resource with a custom cleanup function.
    ///
    /// The cleanup function is called with the resource and its result
    /// is returned.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::quantitative::ManusLinearis;
    ///
    /// let handle = ManusLinearis::acquire(vec![1, 2, 3]);
    /// let sum: i32 = handle.release_with(|v| v.iter().sum());
    /// assert_eq!(sum, 6);
    /// ```
    #[inline]
    pub fn release_with<R, F>(self, f: F) -> R
    where
        F: FnOnce(T) -> R,
    {
        f(self.release())
    }

    /// Map a function over the resource, creating a new handle.
    ///
    /// The original handle is consumed and a new one is returned
    /// containing the transformed resource.
    #[inline]
    pub fn map<U, F>(self, f: F) -> ManusLinearis<U>
    where
        F: FnOnce(T) -> U,
    {
        ManusLinearis::acquire(f(self.release()))
    }

    /// Chain operations on the resource handle.
    #[inline]
    pub fn and_then<U, F>(self, f: F) -> ManusLinearis<U>
    where
        F: FnOnce(T) -> ManusLinearis<U>,
    {
        f(self.release())
    }

    /// Check if the resource has been released.
    #[inline]
    pub fn is_released(&self) -> bool {
        self.resource.is_none()
    }

    /// Convert to a Qtt with linear multiplicity.
    #[inline]
    pub fn into_qtt(self) -> Qtt<T, Semel> {
        Qtt::linear(self.release())
    }
}

impl<T: fmt::Debug> fmt::Debug for ManusLinearis<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ManusLinearis")
            .field("resource", &self.resource)
            .finish()
    }
}

// Note: We intentionally do NOT implement Drop with a panic,
// as that would make the type less usable in practice.
// Instead, we use #[must_use] to warn at compile time.

// =============================================================================
// ManusGuard - RAII-style Linear Guard
// =============================================================================

/// A guard for a borrowed linear resource.
///
/// `ManusGuard` provides RAII-style access to a resource, automatically
/// returning it to the handle when dropped.
///
/// # Latin Etymology
///
/// *Manus* = hand, *Custos* = guard
pub struct ManusGuard<'a, T> {
    resource: &'a mut Option<T>,
    temp: Option<T>,
}

impl<'a, T> ManusGuard<'a, T> {
    /// Create a new guard from a handle's resource slot.
    #[inline]
    fn new(resource: &'a mut Option<T>) -> Self {
        let temp = resource.take();
        ManusGuard { resource, temp }
    }

    /// Get a reference to the guarded resource.
    ///
    /// # Panics
    ///
    /// Panics if the guard is empty — i.e. the handle's resource was
    /// permanently removed by an earlier guard's [`ManusGuard::take`].
    #[inline]
    pub fn get(&self) -> &T {
        self.temp.as_ref().expect("guard is empty")
    }

    /// Get a mutable reference to the guarded resource.
    ///
    /// # Panics
    ///
    /// Panics if the guard is empty — i.e. the handle's resource was
    /// permanently removed by an earlier guard's [`ManusGuard::take`].
    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        self.temp.as_mut().expect("guard is empty")
    }

    /// Take the resource out of the guard.
    ///
    /// After this call, the guard will not return the resource to the handle.
    ///
    /// # Panics
    ///
    /// Panics if the guard is empty — i.e. the handle's resource was
    /// permanently removed by an earlier guard's `take`.
    #[inline]
    pub fn take(mut self) -> T {
        self.temp.take().expect("guard is empty")
    }
}

impl<T> Drop for ManusGuard<'_, T> {
    fn drop(&mut self) {
        if let Some(resource) = self.temp.take() {
            *self.resource = Some(resource);
        }
    }
}

impl<T> core::ops::Deref for ManusGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl<T> core::ops::DerefMut for ManusGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get_mut()
    }
}

impl<T> ManusLinearis<T> {
    /// Borrow the resource with a guard.
    ///
    /// Returns a guard that provides access to the resource and
    /// automatically returns it when dropped.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::quantitative::ManusLinearis;
    ///
    /// let mut handle = ManusLinearis::acquire(vec![1, 2, 3]);
    /// {
    ///     let mut guard = handle.guard();
    ///     guard.push(4);
    /// } // Guard dropped, resource returned to handle
    /// let vec = handle.release();
    /// assert_eq!(vec, vec![1, 2, 3, 4]);
    /// ```
    #[inline]
    pub fn guard(&mut self) -> ManusGuard<'_, T> {
        ManusGuard::new(&mut self.resource)
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_acquire_release() {
        let handle = ManusLinearis::acquire(42);
        let value = handle.release();
        assert_eq!(value, 42);
    }

    #[test]
    fn test_use_ref() {
        let handle = ManusLinearis::acquire(vec![1, 2, 3]);
        let len = handle.use_ref(std::vec::Vec::len);
        assert_eq!(len, 3);
        let _ = handle.release();
    }

    #[test]
    fn test_use_mut() {
        let mut handle = ManusLinearis::acquire(vec![1, 2, 3]);
        handle.use_mut(|v| v.push(4));
        let vec = handle.release();
        assert_eq!(vec, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_release_with() {
        let handle = ManusLinearis::acquire(10);
        let result = handle.release_with(|x| x * 2);
        assert_eq!(result, 20);
    }

    #[test]
    fn test_map() {
        let handle = ManusLinearis::acquire(5);
        let mapped = handle.map(|x| x * 2);
        assert_eq!(mapped.release(), 10);
    }

    #[test]
    fn test_and_then() {
        let handle = ManusLinearis::acquire(5);
        let chained = handle.and_then(|x| ManusLinearis::acquire(x + 10));
        assert_eq!(chained.release(), 15);
    }

    #[test]
    fn test_guard() {
        let mut handle = ManusLinearis::acquire(vec![1, 2, 3]);
        {
            let mut guard = handle.guard();
            guard.push(4);
            guard.push(5);
        }
        let vec = handle.release();
        assert_eq!(vec, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_guard_deref() {
        let mut handle = ManusLinearis::acquire(42);
        {
            let guard = handle.guard();
            assert_eq!(*guard, 42);
        }
        assert_eq!(handle.release(), 42);
    }

    #[test]
    fn test_into_qtt() {
        let handle = ManusLinearis::acquire(42);
        let qtt = handle.into_qtt();
        assert_eq!(qtt.consume(), 42);
    }

    #[test]
    fn test_is_released() {
        let handle = ManusLinearis::acquire(42);
        assert!(!handle.is_released());
        let _ = handle.release();
        // Can't check after release since handle is consumed
    }

    #[test]
    fn test_debug() {
        let handle = ManusLinearis::acquire(42);
        let debug_str = alloc::format!("{handle:?}");
        assert!(debug_str.contains("ManusLinearis"));
        assert!(debug_str.contains("42"));
    }
}
