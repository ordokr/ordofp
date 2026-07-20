//! Layout Optimization Hints
//!
//! Provides compile-time information about type layouts for optimization.

use core::mem;

// =============================================================================
// Layout Information
// =============================================================================

/// Compile-time layout information for a type.
pub struct LayoutInfo<T> {
    _marker: core::marker::PhantomData<T>,
}

impl<T> LayoutInfo<T> {
    /// Create layout info for a type.
    pub const fn new() -> Self {
        LayoutInfo {
            _marker: core::marker::PhantomData,
        }
    }

    /// Get the size of the type in bytes.
    pub const fn size() -> usize {
        mem::size_of::<T>()
    }

    /// Get the alignment of the type in bytes.
    pub const fn align() -> usize {
        mem::align_of::<T>()
    }

    /// Check if the type is zero-sized.
    pub const fn is_zst() -> bool {
        mem::size_of::<T>() == 0
    }

    /// Check if the type fits in a register (typically 8 bytes or less).
    pub const fn fits_in_register() -> bool {
        mem::size_of::<T>() <= mem::size_of::<usize>()
    }

    /// Check if the type fits in two registers.
    pub const fn fits_in_two_registers() -> bool {
        mem::size_of::<T>() <= 2 * mem::size_of::<usize>()
    }

    /// Check if the type is pointer-sized.
    pub const fn is_pointer_sized() -> bool {
        mem::size_of::<T>() == mem::size_of::<usize>()
    }

    /// Get the number of bytes needed for proper alignment.
    pub const fn padding_needed(current_offset: usize) -> usize {
        let align = mem::align_of::<T>();
        let misalign = current_offset % align;
        if misalign == 0 { 0 } else { align - misalign }
    }
}

impl<T> Default for LayoutInfo<T> {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Size Categories
// =============================================================================

/// Categorize types by size for optimization decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SizeCategory {
    /// Zero-sized type (0 bytes)
    Zero,
    /// Tiny type (1-8 bytes, fits in register)
    Tiny,
    /// Small type (9-32 bytes)
    Small,
    /// Medium type (33-256 bytes)
    Medium,
    /// Large type (257+ bytes)
    Large,
}

impl SizeCategory {
    /// Categorize a size.
    pub const fn from_size(size: usize) -> Self {
        if size == 0 {
            SizeCategory::Zero
        } else if size <= 8 {
            SizeCategory::Tiny
        } else if size <= 32 {
            SizeCategory::Small
        } else if size <= 256 {
            SizeCategory::Medium
        } else {
            SizeCategory::Large
        }
    }

    /// Categorize a type.
    pub const fn of<T>() -> Self {
        Self::from_size(mem::size_of::<T>())
    }

    /// Whether this category should be passed by value.
    #[inline]
    pub const fn pass_by_value(self) -> bool {
        matches!(
            self,
            SizeCategory::Zero | SizeCategory::Tiny | SizeCategory::Small
        )
    }

    /// Whether this category benefits from inlining.
    #[inline]
    pub const fn inline_beneficial(self) -> bool {
        matches!(self, SizeCategory::Zero | SizeCategory::Tiny)
    }
}

// =============================================================================
// Optimization Decisions
// =============================================================================

/// Compile-time optimization decisions based on type layout.
pub struct OptimizationHints<T> {
    _marker: core::marker::PhantomData<T>,
}

impl<T> OptimizationHints<T> {
    /// Create optimization hints for a type.
    pub const fn new() -> Self {
        OptimizationHints {
            _marker: core::marker::PhantomData,
        }
    }

    /// Whether to pass this type by value.
    #[inline]
    pub const fn pass_by_value() -> bool {
        SizeCategory::of::<T>().pass_by_value()
    }

    /// Whether to inline functions operating on this type.
    #[inline]
    pub const fn should_inline() -> bool {
        SizeCategory::of::<T>().inline_beneficial()
    }

    /// Whether to box this type for storage.
    #[inline]
    pub const fn should_box() -> bool {
        matches!(SizeCategory::of::<T>(), SizeCategory::Large)
    }

    /// Whether to use arena allocation.
    #[inline]
    pub const fn use_arena() -> bool {
        matches!(
            SizeCategory::of::<T>(),
            SizeCategory::Small | SizeCategory::Medium
        )
    }
}

impl<T> Default for OptimizationHints<T> {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Cache Optimization
// =============================================================================

/// Common cache line size (64 bytes on most architectures).
pub const CACHE_LINE_SIZE: usize = 64;

/// Check if a type fits within a cache line.
pub const fn fits_in_cache_line<T>() -> bool {
    mem::size_of::<T>() <= CACHE_LINE_SIZE
}

/// Calculate how many instances of T fit in a cache line.
pub const fn instances_per_cache_line<T>() -> usize {
    if mem::size_of::<T>() == 0 {
        usize::MAX
    } else {
        CACHE_LINE_SIZE / mem::size_of::<T>()
    }
}

/// Padding to align to cache line boundary.
#[repr(align(64))]
#[derive(Debug, Clone, Copy)]
pub struct CacheAligned<T>(pub T);

impl<T> CacheAligned<T> {
    /// Create a cache-aligned wrapper.
    pub const fn new(value: T) -> Self {
        CacheAligned(value)
    }
}

impl<T: Default> Default for CacheAligned<T> {
    fn default() -> Self {
        CacheAligned(T::default())
    }
}

// =============================================================================
// Memory Layout Assertions
// =============================================================================

/// Assert that a type has a specific size at compile time.
///
/// # Example
///
/// ```rust
/// use ordofp_core::assert_size;
///
/// struct MyStruct {
///     a: u32,
///     b: u32,
/// }
///
/// assert_size!(MyStruct, 8);
/// ```
#[macro_export]
macro_rules! assert_size {
    ($t:ty, $size:expr) => {
        const _: () = {
            if core::mem::size_of::<$t>() != $size {
                panic!("Size assertion failed");
            }
        };
    };
}

/// Assert that a type has a specific alignment at compile time.
#[macro_export]
macro_rules! assert_align {
    ($t:ty, $align:expr) => {
        const _: () = {
            if core::mem::align_of::<$t>() != $align {
                panic!("Alignment assertion failed");
            }
        };
    };
}

/// Assert that a type is zero-sized at compile time.
#[macro_export]
macro_rules! assert_zst {
    ($t:ty) => {
        const _: () = {
            if core::mem::size_of::<$t>() != 0 {
                panic!("ZST assertion failed");
            }
        };
    };
}

// =============================================================================
// Repr Hints
// =============================================================================

/// Marker for types that should use C representation.
pub trait CRepr {}

/// Marker for types that should be packed.
pub trait Packed {}

/// Marker for types that should be transparent.
pub trait Transparent {}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_info() {
        assert_eq!(LayoutInfo::<u32>::size(), 4);
        assert_eq!(LayoutInfo::<u64>::size(), 8);
        assert!(LayoutInfo::<()>::is_zst());
        assert!(LayoutInfo::<u64>::fits_in_register());
    }

    #[test]
    fn test_size_category() {
        assert_eq!(SizeCategory::of::<()>(), SizeCategory::Zero);
        assert_eq!(SizeCategory::of::<u8>(), SizeCategory::Tiny);
        assert_eq!(SizeCategory::of::<u64>(), SizeCategory::Tiny);
        assert_eq!(SizeCategory::of::<[u64; 3]>(), SizeCategory::Small);
        assert_eq!(SizeCategory::of::<[u64; 10]>(), SizeCategory::Medium);
        assert_eq!(SizeCategory::of::<[u64; 100]>(), SizeCategory::Large);
    }

    #[test]
    fn test_optimization_hints() {
        assert!(OptimizationHints::<u32>::pass_by_value());
        assert!(OptimizationHints::<u8>::should_inline());
        assert!(OptimizationHints::<[u8; 1000]>::should_box());
    }

    #[test]
    fn test_cache_optimization() {
        assert!(fits_in_cache_line::<u64>());
        assert!(fits_in_cache_line::<[u8; 64]>());
        assert!(!fits_in_cache_line::<[u8; 65]>());

        assert_eq!(instances_per_cache_line::<u64>(), 8);
        assert_eq!(instances_per_cache_line::<u8>(), 64);
    }

    #[test]
    fn test_cache_aligned() {
        let aligned = CacheAligned::new(42u32);
        assert_eq!(aligned.0, 42);
        // The wrapper should be cache-line aligned
        assert!(core::mem::align_of::<CacheAligned<u32>>() >= 64);
    }

    // Compile-time assertions
    assert_size!(u32, 4);
    assert_size!(u64, 8);
    assert_align!(u64, 8);
    assert_zst!(());
}
