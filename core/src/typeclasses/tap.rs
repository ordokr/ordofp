//! # Tap — Side Effect Inspection
//!
//! > *"Inspicere sine mutare."*
//! > — To inspect without changing.
//!
//! The `Tap` trait provides methods for inspecting values without consuming them,
//! particularly useful for debugging and logging in functional pipelines.
//!
//! # Example
//!
//! ```rust
//! use ordofp_core::typeclasses::Tap;
//!
//! let result = Some(42)
//!     .tap(|x| println!("Value is: {:?}", x))
//!     .map(|x| x * 2)
//!     .tap(|x| println!("After doubling: {:?}", x));
//!
//! assert_eq!(result, Some(84));
//! ```
//!
//! The `tap` method is especially useful in method chains where you want to
//! observe intermediate values without breaking the fluent API.

use core::fmt::Debug;

/// A trait for inspecting values without consuming them.
///
/// `Tap` provides methods for observing values in a pipeline without affecting
/// the computation. This is particularly useful for:
///
/// - Debugging intermediate values in a chain
/// - Adding logging or metrics without restructuring code
/// - Performing side effects while maintaining functional style
///
/// # Scholastic Naming
///
/// - `tap` (*pulsare*, to touch) — inspect without modifying
/// - `tap_debug` — debug-print the value
/// - `tap_some` — tap into Some variants
/// - `tap_ok` — tap into Ok variants
pub trait Tap: Sized {
    /// Inspects the value with a closure, then returns the value unchanged.
    ///
    /// This is useful for debugging or logging without affecting the value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::typeclasses::Tap;
    ///
    /// let value = 42.tap(|x| println!("Value: {}", x));
    /// assert_eq!(value, 42);
    /// ```
    #[inline]
    fn tap<F>(self, f: F) -> Self
    where
        F: FnOnce(&Self),
    {
        f(&self);
        self
    }

    /// Mutably inspects the value with a closure, then returns the value.
    ///
    /// This allows modifying the value while still returning it.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::typeclasses::Tap;
    ///
    /// let mut v = vec![1, 2, 3];
    /// let result = v.tap_mut(|vec| vec.push(4));
    /// assert_eq!(result, vec![1, 2, 3, 4]);
    /// ```
    #[inline]
    fn tap_mut<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut Self),
    {
        f(&mut self);
        self
    }

    /// Debug-prints the value, then returns it unchanged.
    ///
    /// This is a convenience method for quick debugging.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::typeclasses::Tap;
    ///
    /// let value = 42.tap_debug(); // Prints: 42
    /// assert_eq!(value, 42);
    /// ```
    #[inline]
    fn tap_debug(self) -> Self
    where
        Self: Debug,
    {
        #[cfg(feature = "std")]
        {
            std::eprintln!("{self:?}");
        }
        self
    }

    /// Debug-prints the value with a label, then returns it unchanged.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::typeclasses::Tap;
    ///
    /// let value = 42.tap_debug_label("my_value"); // Prints: my_value: 42
    /// assert_eq!(value, 42);
    /// ```
    #[inline]
    fn tap_debug_label(self, label: &str) -> Self
    where
        Self: Debug,
    {
        #[cfg(feature = "std")]
        {
            std::eprintln!("{label}: {self:?}");
        }
        let _ = label; // Suppress unused warning in no_std
        self
    }

    /// Applies a predicate and runs a closure only if the predicate is true.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::typeclasses::Tap;
    ///
    /// let value = 42.tap_if(|x| *x > 40, |x| println!("Large value: {}", x));
    /// assert_eq!(value, 42);
    /// ```
    #[inline]
    fn tap_if<P, F>(self, pred: P, f: F) -> Self
    where
        P: FnOnce(&Self) -> bool,
        F: FnOnce(&Self),
    {
        if pred(&self) {
            f(&self);
        }
        self
    }
}

// Blanket implementation for all types
impl<T> Tap for T {}

/// Extension trait for tapping into Option values.
///
/// Provides methods for inspecting Option values without unwrapping them.
pub trait TapOption<T>: Sized {
    /// Taps into Some values, running the closure only if present.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::typeclasses::TapOption;
    ///
    /// let opt = Some(42).tap_some(|x| println!("Got: {}", x));
    /// assert_eq!(opt, Some(42));
    ///
    /// let none: Option<i32> = None;
    /// let result = none.tap_some(|x| println!("Got: {}", x)); // Closure not called
    /// assert_eq!(result, None);
    /// ```
    fn tap_some<F>(self, f: F) -> Self
    where
        F: FnOnce(&T);

    /// Taps into None values, running the closure only if absent.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::typeclasses::TapOption;
    ///
    /// let none: Option<i32> = None;
    /// let result = none.tap_none(|| println!("Value is None"));
    /// assert_eq!(result, None);
    /// ```
    fn tap_none<F>(self, f: F) -> Self
    where
        F: FnOnce();
}

impl<T> TapOption<T> for Option<T> {
    #[inline]
    fn tap_some<F>(self, f: F) -> Self
    where
        F: FnOnce(&T),
    {
        if let Some(ref x) = self {
            f(x);
        }
        self
    }

    #[inline]
    fn tap_none<F>(self, f: F) -> Self
    where
        F: FnOnce(),
    {
        if self.is_none() {
            f();
        }
        self
    }
}

/// Extension trait for tapping into Result values.
///
/// Provides methods for inspecting Result values without unwrapping them.
pub trait TapResult<T, E>: Sized {
    /// Taps into Ok values, running the closure only if successful.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::typeclasses::TapResult;
    ///
    /// let ok: Result<i32, &str> = Ok(42);
    /// let result = ok.tap_ok(|x| println!("Success: {}", x));
    /// assert_eq!(result, Ok(42));
    /// ```
    fn tap_ok<F>(self, f: F) -> Self
    where
        F: FnOnce(&T);

    /// Taps into Err values, running the closure only if error.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::typeclasses::TapResult;
    ///
    /// let err: Result<i32, &str> = Err("error");
    /// let result = err.tap_err(|e| println!("Error: {}", e));
    /// assert_eq!(result, Err("error"));
    /// ```
    fn tap_err<F>(self, f: F) -> Self
    where
        F: FnOnce(&E);
}

impl<T, E> TapResult<T, E> for Result<T, E> {
    #[inline]
    fn tap_ok<F>(self, f: F) -> Self
    where
        F: FnOnce(&T),
    {
        if let Ok(ref x) = self {
            f(x);
        }
        self
    }

    #[inline]
    fn tap_err<F>(self, f: F) -> Self
    where
        F: FnOnce(&E),
    {
        if let Err(ref e) = self {
            f(e);
        }
        self
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use core::cell::Cell;

    #[test]
    fn test_tap() {
        let called = Cell::new(false);
        let value = 42.tap(|x| {
            assert_eq!(*x, 42);
            called.set(true);
        });
        assert_eq!(value, 42);
        assert!(called.get());
    }

    #[test]
    fn test_tap_mut() {
        let v = alloc::vec![1, 2, 3];
        let result = v.tap_mut(|vec| vec.push(4));
        assert_eq!(result, alloc::vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_tap_if_true() {
        let called = Cell::new(false);
        let value = 42.tap_if(|x| *x > 40, |_| called.set(true));
        assert_eq!(value, 42);
        assert!(called.get());
    }

    #[test]
    fn test_tap_if_false() {
        let called = Cell::new(false);
        let value = 42.tap_if(|x| *x > 50, |_| called.set(true));
        assert_eq!(value, 42);
        assert!(!called.get());
    }

    #[test]
    fn test_tap_some() {
        let called = Cell::new(false);
        let opt = Some(42).tap_some(|x| {
            assert_eq!(*x, 42);
            called.set(true);
        });
        assert_eq!(opt, Some(42));
        assert!(called.get());
    }

    #[test]
    fn test_tap_some_none() {
        let called = Cell::new(false);
        let none: Option<i32> = None;
        let result = none.tap_some(|_| called.set(true));
        assert_eq!(result, None);
        assert!(!called.get());
    }

    #[test]
    fn test_tap_none() {
        let called = Cell::new(false);
        let none: Option<i32> = None;
        let result = none.tap_none(|| called.set(true));
        assert_eq!(result, None);
        assert!(called.get());
    }

    #[test]
    fn test_tap_none_some() {
        let called = Cell::new(false);
        let opt = Some(42).tap_none(|| called.set(true));
        assert_eq!(opt, Some(42));
        assert!(!called.get());
    }

    #[test]
    fn test_tap_ok() {
        let called = Cell::new(false);
        let ok: Result<i32, &str> = Ok(42);
        let result = ok.tap_ok(|x| {
            assert_eq!(*x, 42);
            called.set(true);
        });
        assert_eq!(result, Ok(42));
        assert!(called.get());
    }

    #[test]
    fn test_tap_ok_err() {
        let called = Cell::new(false);
        let err: Result<i32, &str> = Err("error");
        let result = err.tap_ok(|_| called.set(true));
        assert_eq!(result, Err("error"));
        assert!(!called.get());
    }

    #[test]
    fn test_tap_err() {
        let called = Cell::new(false);
        let err: Result<i32, &str> = Err("error");
        let result = err.tap_err(|e| {
            assert_eq!(*e, "error");
            called.set(true);
        });
        assert_eq!(result, Err("error"));
        assert!(called.get());
    }

    #[test]
    fn test_tap_err_ok() {
        let called = Cell::new(false);
        let ok: Result<i32, &str> = Ok(42);
        let result = ok.tap_err(|_| called.set(true));
        assert_eq!(result, Ok(42));
        assert!(!called.get());
    }

    #[test]
    fn test_method_chain() {
        let called1 = Cell::new(false);
        let called2 = Cell::new(false);

        let result = Some(42)
            .tap_some(|_| called1.set(true))
            .map(|x| x * 2)
            .tap_some(|x| {
                assert_eq!(*x, 84);
                called2.set(true);
            });

        assert_eq!(result, Some(84));
        assert!(called1.get());
        assert!(called2.get());
    }
}
