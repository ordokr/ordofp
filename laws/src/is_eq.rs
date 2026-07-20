//! Equality verification for law testing
//!
//! The [`IsEq`] type represents an equality assertion that can be verified.
//! It's used throughout the laws modules to express that two values should be equal
//! according to a particular law.

/// Represents an equality assertion between two values.
///
/// This is typically returned by law-checking functions and then verified
/// using the `holds()` method.
///
/// # Example
///
/// ```
/// use ordofp_laws::is_eq::IsEq;
///
/// let eq = IsEq::new(1 + 2, 3);
/// assert!(eq.holds());
/// ```
#[derive(Debug, Clone)]
pub struct IsEq<T> {
    /// The left-hand side of the equality
    pub lhs: T,
    /// The right-hand side of the equality
    pub rhs: T,
}

impl<T> IsEq<T> {
    /// Create a new equality assertion.
    ///
    /// The two values are expected to be equal according to some law.
    pub fn new(lhs: T, rhs: T) -> Self {
        IsEq { lhs, rhs }
    }

    /// Alias for `new` - creates an equality check from two values
    /// that should be equal under a particular law.
    pub fn equal_under_law(lhs: T, rhs: T) -> Self {
        Self::new(lhs, rhs)
    }
}

impl<T: Eq> IsEq<T> {
    /// Check if the equality holds.
    ///
    /// Returns `true` if `lhs == rhs`, `false` otherwise.
    pub fn holds(self) -> bool {
        self.lhs == self.rhs
    }
}

impl<T: PartialEq> IsEq<T> {
    /// Check if the equality holds using partial equality.
    ///
    /// This is useful for types that implement `PartialEq` but not `Eq`,
    /// such as floating-point numbers (with caveats).
    pub fn holds_partial(self) -> bool {
        self.lhs == self.rhs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_eq_holds() {
        assert!(IsEq::new(42, 42).holds());
        assert!(!IsEq::new(42, 43).holds());
    }

    #[test]
    fn test_is_eq_strings() {
        assert!(IsEq::new(String::from("hello"), String::from("hello")).holds());
        assert!(!IsEq::new(String::from("hello"), String::from("world")).holds());
    }

    #[test]
    fn test_equal_under_law() {
        let eq = IsEq::equal_under_law(vec![1, 2, 3], vec![1, 2, 3]);
        assert!(eq.holds());
    }
}
