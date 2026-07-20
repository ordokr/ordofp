//! Branch Prediction and Optimization Hints
//!
//! Provides hints to guide compiler optimizations.

// =============================================================================
// Branch Prediction Hints
// =============================================================================

/// Hint that a condition is likely to be true.
///
/// This helps the compiler optimize the hot path.
///
/// # Example
///
/// ```rust
/// use ordofp_core::specialization::likely;
///
/// fn handle_negative(x: i32) -> i32 {
///     -x
/// }
///
/// fn process(x: i32) -> i32 {
///     if likely(x > 0) {
///         x * 2  // This branch is optimized
///     } else {
///         handle_negative(x)
///     }
/// }
///
/// assert_eq!(process(5), 10);
/// assert_eq!(process(-5), 5);
/// ```
#[inline(always)]
pub fn likely(b: bool) -> bool {
    // Thin alias for the intrinsic-backed version; prefer crate::hints::likely.
    crate::hints::likely(b)
}

/// Hint that a condition is unlikely to be true.
///
/// This helps the compiler optimize the common case.
///
/// # Example
///
/// ```rust
/// use ordofp_core::specialization::unlikely;
///
/// fn divide(x: i32, y: i32) -> Option<i32> {
///     if unlikely(y == 0) {
///         None  // Rare case
///     } else {
///         Some(x / y)  // Common case - optimized
///     }
/// }
///
/// assert_eq!(divide(10, 2), Some(5));
/// assert_eq!(divide(10, 0), None);
/// ```
#[inline(always)]
pub fn unlikely(b: bool) -> bool {
    // Thin alias for the intrinsic-backed version; prefer crate::hints::unlikely.
    crate::hints::unlikely(b)
}

// =============================================================================
// Function Temperature Hints
// =============================================================================

/// Mark a function as "cold" (rarely called).
///
/// Cold functions are optimized for size rather than speed,
/// and are placed in a separate section to improve cache locality.
///
/// # Example
///
/// ```rust
/// use ordofp_core::specialization::cold_path;
///
/// #[cold]
/// fn handle_error(code: i32) -> i32 {
///     // Error handling code - rarely executed
///     code
/// }
///
/// let result = cold_path(|| handle_error(-1));
/// assert_eq!(result, -1);
/// ```
///
/// Note: Use `#[cold]` attribute on functions directly.
/// This function is for runtime cold path marking; see also the
/// [`crate::cold_path!`] macro for the statement form.
#[inline(never)]
#[cold]
pub fn cold_path<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    f()
}

/// Mark a code path as "hot" (frequently executed).
///
/// Hot paths are aggressively inlined and optimized for speed.
/// Thin alias for [`crate::hints::hot_path`].
#[inline(always)]
pub fn hot_path<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    crate::hints::hot_path(f)
}

// =============================================================================
// Prefetch Hints
// =============================================================================

/// Hint to prefetch data for reading.
///
/// This can improve performance by loading data into cache
/// before it's needed.
///
/// Note: This is a no-op on platforms without prefetch support
/// or when the intrinsics are not available.
#[inline(always)]
pub fn prefetch_read<T>(_data: &T) {
    // Prefetch hints require platform-specific intrinsics.
    // On stable Rust without std, we provide a no-op implementation.
    // The compiler may still optimize based on access patterns.
}

/// Hint to prefetch data for writing.
#[inline(always)]
pub fn prefetch_write<T>(_data: &mut T) {
    // No-op on stable Rust without platform-specific features
}

// =============================================================================
// Assume Hints
// =============================================================================

/// Assert a condition that the optimizer can assume is true.
///
/// # Safety
///
/// The condition MUST be true. If false, behavior is undefined.
///
/// # Example
///
/// ```rust
/// use ordofp_core::specialization::assume;
///
/// fn get_positive(x: i32) -> i32 {
///     // SAFETY: caller passes a positive value below, upholding the
///     // precondition of `assume`.
///     unsafe { assume(x > 0) };
///     // Optimizer knows x > 0 here
///     x
/// }
///
/// assert_eq!(get_positive(5), 5);
/// ```
#[inline(always)]
pub unsafe fn assume(cond: bool) {
    if !cond {
        // SAFETY: Caller guarantees cond is true
        unsafe { core::hint::unreachable_unchecked() }
    }
}

/// Assert that a value is non-null.
///
/// # Safety
///
/// The pointer MUST be non-null.
#[inline(always)]
pub unsafe fn assume_non_null<T>(ptr: *const T) -> *const T {
    // SAFETY: Caller guarantees ptr is non-null
    unsafe { assume(!ptr.is_null()) };
    ptr
}

/// Assert that a slice is non-empty.
///
/// # Safety
///
/// The slice MUST be non-empty.
#[inline(always)]
pub unsafe fn assume_non_empty<T>(slice: &[T]) -> &[T] {
    // SAFETY: Caller guarantees slice is non-empty
    unsafe { assume(!slice.is_empty()) };
    slice
}

// =============================================================================
// Black Box (Optimization Barrier)
// =============================================================================

/// Prevent the compiler from optimizing away a value.
///
/// Useful in benchmarks to ensure computations aren't eliminated.
///
/// # Example
///
/// ```rust
/// use ordofp_core::specialization::black_box;
///
/// fn expensive_computation() -> i32 {
///     41 + 1
/// }
///
/// // In benchmark
/// let result = expensive_computation();
/// black_box(result);  // Prevent optimization
/// assert_eq!(result, 42);
/// ```
#[inline(always)]
pub fn black_box<T>(x: T) -> T {
    core::hint::black_box(x)
}

// =============================================================================
// Spin Loop Hint
// =============================================================================

/// Hint that we're in a spin loop.
///
/// Reduces power consumption and improves performance of spin locks.
#[inline(always)]
pub fn spin_loop_hint() {
    core::hint::spin_loop();
}

// =============================================================================
// Assert Unchecked
// =============================================================================

/// Assert a condition, with undefined behavior if false.
///
/// This is similar to `debug_assert!` but the condition
/// is assumed true in release builds for optimization.
///
/// # Safety
///
/// The condition MUST be true. If false, behavior is undefined.
#[inline(always)]
pub unsafe fn assert_unchecked(cond: bool) {
    debug_assert!(cond, "assert_unchecked condition was false");
    if !cond {
        // SAFETY: Caller guarantees cond is true
        unsafe { core::hint::unreachable_unchecked() }
    }
}

/// Indicate that a code path is unreachable.
///
/// # Safety
///
/// This code path must never be executed.
#[inline(always)]
pub unsafe fn unreachable() -> ! {
    // SAFETY: Caller guarantees this is unreachable
    unsafe { core::hint::unreachable_unchecked() }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_likely_unlikely() {
        // These should compile and run without issues
        assert!(likely(true));
        assert!(!likely(false));
        assert!(unlikely(true));
        assert!(!unlikely(false));
    }

    #[test]
    fn test_cold_hot_paths() {
        let result = cold_path(|| 42);
        assert_eq!(result, 42);

        let result = hot_path(|| 100);
        assert_eq!(result, 100);
    }

    #[test]
    fn test_black_box() {
        let x = black_box(42);
        assert_eq!(x, 42);
    }

    #[test]
    fn test_prefetch() {
        let data = 42i32;
        prefetch_read(&data);

        let mut mutable = 42i32;
        prefetch_write(&mut mutable);
    }

    #[test]
    fn test_assume() {
        let x = 42;
        // SAFETY: x is 42, which is strictly greater than 0, so the condition
        // is guaranteed to be true and `assume` will not invoke undefined behaviour.
        unsafe { assume(x > 0) };
        assert_eq!(x, 42);
    }

    #[test]
    fn test_assume_non_null() {
        let x = 42;
        let ptr = &raw const x;
        // SAFETY: `ptr` is derived from a shared reference to a live stack
        // variable, so it is guaranteed to be non-null.
        let result = unsafe { assume_non_null(ptr) };
        assert_eq!(result, ptr);
    }

    #[test]
    fn test_assume_non_empty() {
        let slice = &[1, 2, 3];
        // SAFETY: The slice literal `&[1, 2, 3]` has length 3, so it is
        // guaranteed to be non-empty.
        let result = unsafe { assume_non_empty(slice) };
        assert_eq!(result, slice);
    }
}
