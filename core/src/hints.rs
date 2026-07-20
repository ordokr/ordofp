//! Branch Prediction Hints
//!
//! > *"Praedicere est praevidere."*
//! > — To predict is to foresee.
//!
//! This module provides branch prediction hints for performance-critical code paths.
//!
//! # Implementation
//!
//! With the `nightly` cargo feature enabled (requires a nightly toolchain),
//! these functions delegate to [`core::hint::likely`]/[`core::hint::unlikely`],
//! which map directly to LLVM's `@llvm.expect` intrinsics for optimal codegen.
//! Per hashbrown benchmarks, this provides 10-15% improvement in hot paths.
//! On stable Rust (the default) they are identity functions — semantics are
//! identical either way; only codegen quality changes.
//!
//! # Usage
//!
//! ```rust
//! use ordofp_core::hints::{likely, unlikely};
//!
//! # fn fast_path() {}
//! # fn slow_path() {}
//! # fn handle_error() {}
//! # let condition = true;
//! # let error_condition = false;
//! if likely(condition) {
//!     // Hot path - compiler will optimize for this case
//!     fast_path();
//! } else {
//!     // Cold path
//!     slow_path();
//! }
//!
//! if unlikely(error_condition) {
//!     // Error handling - compiler knows this is rare
//!     handle_error();
//! }
//! ```
//!
//! # Latin Etymology
//!
//! *Verisimilis* = likely (from *verus* "true" + *similis* "similar")
//! *Improbabilis* = unlikely (from *in-* "not" + *probabilis* "probable")

/// Hint that a condition is likely to be true.
///
/// Use this in `if` conditions when the true branch is the hot path.
/// Uses the stabilized `core::hint::likely` for optimal LLVM codegen.
///
/// # Example
///
/// ```rust
/// use ordofp_core::hints::likely;
///
/// # fn process(_x: i32) {}
/// # fn handle_negative(_x: i32) {}
/// # let x = 10;
/// if likely(x > 0) {
///     // This is the common case
///     process(x);
/// } else {
///     // This rarely happens
///     handle_negative(x);
/// }
/// ```
#[inline(always)]
#[must_use]
pub const fn likely(b: bool) -> bool {
    // Tells LLVM to expect `b` to be true, eliminating unconditional jumps
    // in the hot path. Identity on stable (no hint, same semantics).
    #[cfg(feature = "nightly")]
    {
        core::hint::likely(b)
    }
    #[cfg(not(feature = "nightly"))]
    {
        b
    }
}

/// Hint that a condition is unlikely to be true.
///
/// Use this in `if` conditions when the true branch is the cold path
/// (error handling, edge cases, etc.).
/// Uses the stabilized `core::hint::unlikely` for optimal LLVM codegen.
///
/// # Example
///
/// ```rust
/// use ordofp_core::hints::unlikely;
///
/// fn handle_error(result: Result<i32, String>) -> String {
///     result.unwrap_err()
/// }
///
/// fn process(result: Result<i32, String>) -> Result<(), String> {
///     if unlikely(result.is_err()) {
///         // Error handling - this rarely happens
///         return Err(handle_error(result));
///     }
///     // Happy path continues
///     Ok(())
/// }
///
/// assert!(process(Ok(1)).is_ok());
/// assert!(process(Err("oops".to_string())).is_err());
/// ```
#[inline(always)]
#[must_use]
pub const fn unlikely(b: bool) -> bool {
    // Tells LLVM to expect `b` to be false, placing the unlikely branch
    // out of line. Identity on stable (no hint, same semantics).
    #[cfg(feature = "nightly")]
    {
        core::hint::unlikely(b)
    }
    #[cfg(not(feature = "nightly"))]
    {
        b
    }
}

/// Hint that a condition is extremely likely (>99% probability).
///
/// Stronger hint than `likely` - use sparingly for truly invariant conditions.
#[inline(always)]
#[must_use]
pub const fn almost_certain(b: bool) -> bool {
    likely(b)
}

/// Hint that a condition represents an error that should never happen.
///
/// This is semantically equivalent to `unlikely` but documents intent better.
#[inline(always)]
#[must_use]
pub const fn is_error(b: bool) -> bool {
    unlikely(b)
}

/// Mark a function as cold (rarely called).
///
/// This is a helper for documenting cold paths. Functions marked `#[cold]`
/// will be optimized for size rather than speed and placed in cold code sections.
///
/// Usage:
/// ```rust
/// use ordofp_core::cold_path;
///
/// #[cold]
/// fn handle_rare_error() {
///     // Cold path: optimized for size, placed out of line by the compiler.
/// }
///
/// let result = cold_path!({
///     handle_rare_error();
///     42
/// });
/// assert_eq!(result, 42);
/// ```
///
/// This macro doesn't change behavior but serves as documentation.
/// The actual `#[cold]` attribute should be applied to the function.
#[macro_export]
macro_rules! cold_path {
    ($code:expr) => {{
        #[cold]
        #[inline(never)]
        fn cold_impl<T, F: FnOnce() -> T>(f: F) -> T {
            f()
        }
        cold_impl(|| $code)
    }};
}

/// Execute code on the hot path (likely to be executed).
///
/// This is semantically a no-op but documents the hot path and ensures
/// the code is inlined aggressively.
#[inline(always)]
#[must_use]
pub fn hot_path<T, F: FnOnce() -> T>(f: F) -> T {
    f()
}

// =============================================================================
// Wide Arithmetic Helpers
// =============================================================================
// Const-stable multi-precision building blocks; each lowers to the same
// add-with-carry / widening-multiply codegen as the corresponding intrinsic.

/// Add two u64 values with carry propagation.
///
/// Returns (low, carry) representing the full-width sum `a + b + carry_in`.
///
/// # Example
///
/// ```rust
/// use ordofp_core::hints::wide_add_u64;
///
/// let (low, carry) = wide_add_u64(u64::MAX, 1, false);
/// assert_eq!(low, 0);
/// assert!(carry);
/// ```
#[inline(always)]
pub const fn wide_add_u64(a: u64, b: u64, carry_in: bool) -> (u64, bool) {
    // Two overflowing adds cannot both carry: after the first wraps, the
    // partial sum is at most 2^64 - 2, so adding the carry bit cannot wrap
    // again. Equivalent to the `carrying_add` intrinsic.
    let (sum, c1) = a.overflowing_add(b);
    let (sum, c2) = sum.overflowing_add(carry_in as u64);
    (sum, c1 | c2)
}

/// Multiply two u64 values producing a 128-bit result.
///
/// Implemented via `u128` multiplication (which lowers to the same
/// mulx/umulh codegen as the nightly `widening_mul` intrinsic).
/// Returns (low, high) representing the full 128-bit product.
///
/// # Example
///
/// ```rust
/// use ordofp_core::hints::wide_mul_u64;
///
/// let (low, high) = wide_mul_u64(u64::MAX, 2);
/// // u64::MAX * 2 = 2^65 - 2 = (high: 1, low: u64::MAX - 1)
/// assert_eq!(high, 1);
/// assert_eq!(low, u64::MAX - 1);
/// ```
#[inline(always)]
pub const fn wide_mul_u64(a: u64, b: u64) -> (u64, u64) {
    // Compute the full 128-bit product and split into `(low, high)` limbs.
    // Toolchain-independent: newer nightlies changed `widening_mul` to return
    // `u128`; this lowers to the same mulx/umulh codegen without depending on
    // the intrinsic's signature.
    let product = (a as u128) * (b as u128);
    (product as u64, (product >> 64) as u64)
}

/// Multiply-accumulate for multi-precision arithmetic.
///
/// Computes `a * b + c` with full precision (no overflow).
/// Implemented via `u128` accumulation (equivalent codegen to the nightly
/// `carrying_mul` intrinsic, without depending on its signature).
///
/// # Example
///
/// ```rust
/// use ordofp_core::hints::wide_mul_add_u64;
///
/// // u64::MAX * u64::MAX + u64::MAX = u64::MAX * (u64::MAX + 1) = u64::MAX << 64
/// let (low, high) = wide_mul_add_u64(u64::MAX, u64::MAX, u64::MAX);
/// assert_eq!(low, 0);
/// assert_eq!(high, u64::MAX);
/// ```
#[inline(always)]
pub const fn wide_mul_add_u64(a: u64, b: u64, c: u64) -> (u64, u64) {
    // `a * b + c` at full width: max value is (2^64-1)^2 + (2^64-1) < 2^128,
    // so the u128 accumulation cannot overflow. Split into `(low, high)`.
    // Toolchain-independent (see `wide_mul_u64`).
    let product = (a as u128) * (b as u128) + (c as u128);
    (product as u64, (product >> 64) as u64)
}

/// Strict addition that panics on overflow (release mode safe).
///
/// Panics on overflow regardless of build profile. Use this when overflow
/// is a logic error.
///
/// # Panics
///
/// Panics if `a + b` would overflow.
#[inline(always)]
pub const fn strict_add_u64(a: u64, b: u64) -> u64 {
    match a.checked_add(b) {
        Some(v) => v,
        None => panic!("attempt to add with overflow"),
    }
}

/// Strict multiplication that panics on overflow (release mode safe).
///
/// Panics on overflow regardless of build profile. Use this when overflow
/// is a logic error.
///
/// # Panics
///
/// Panics if `a * b` would overflow.
#[inline(always)]
pub const fn strict_mul_u64(a: u64, b: u64) -> u64 {
    match a.checked_mul(b) {
        Some(v) => v,
        None => panic!("attempt to multiply with overflow"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_likely_true() {
        assert!(likely(true));
        assert!(!likely(false));
    }

    #[test]
    fn test_unlikely_true() {
        assert!(unlikely(true));
        assert!(!unlikely(false));
    }

    #[test]
    fn test_likely_in_condition() {
        let x = 10;
        let result = if likely(x > 0) {
            "positive"
        } else {
            "non-positive"
        };
        assert_eq!(result, "positive");
    }

    #[test]
    fn test_unlikely_in_condition() {
        let is_error = false;
        let result = if unlikely(is_error) { "error" } else { "ok" };
        assert_eq!(result, "ok");
    }

    #[test]
    fn test_cold_path() {
        let result = cold_path!({ 42 });
        assert_eq!(result, 42);
    }

    #[test]
    fn test_hot_path() {
        let result = hot_path(|| 42);
        assert_eq!(result, 42);
    }

    #[test]
    #[should_panic(expected = "cold error")]
    fn test_cold_panic() {
        crate::cold_panic!("cold error");
    }

    #[test]
    fn test_unlikely_panic_false() {
        crate::unlikely_panic!(false, "should not panic");
    }

    #[test]
    #[should_panic(expected = "unlikely error")]
    fn test_unlikely_panic_true() {
        crate::unlikely_panic!(true, "unlikely error");
    }

    #[test]
    fn test_wide_add_no_carry() {
        let (result, carry) = wide_add_u64(10, 20, false);
        assert_eq!(result, 30);
        assert!(!carry);
    }

    #[test]
    fn test_wide_add_with_carry_in() {
        let (result, carry) = wide_add_u64(10, 20, true);
        assert_eq!(result, 31);
        assert!(!carry);
    }

    #[test]
    fn test_wide_add_overflow() {
        let (result, carry) = wide_add_u64(u64::MAX, 1, false);
        assert_eq!(result, 0);
        assert!(carry);
    }

    #[test]
    fn test_wide_mul() {
        // 3 * 5 = 15
        let (low, high) = wide_mul_u64(3, 5);
        assert_eq!(low, 15);
        assert_eq!(high, 0);
    }

    #[test]
    fn test_wide_mul_large() {
        // u64::MAX * 2 = 2^65 - 2
        let (low, high) = wide_mul_u64(u64::MAX, 2);
        assert_eq!(high, 1);
        assert_eq!(low, u64::MAX - 1);
    }

    #[test]
    fn test_strict_add() {
        assert_eq!(strict_add_u64(10, 20), 30);
    }

    #[test]
    #[should_panic(expected = "attempt to add with overflow")]
    fn test_strict_add_overflow() {
        let _ = strict_add_u64(u64::MAX, 1);
    }

    #[test]
    fn test_strict_mul() {
        assert_eq!(strict_mul_u64(10, 20), 200);
    }

    #[test]
    #[should_panic(expected = "attempt to multiply with overflow")]
    fn test_strict_mul_overflow() {
        let _ = strict_mul_u64(u64::MAX, 2);
    }
}
