//! SIMD Kernel Layer - Portable vectorized operations for `ParFlumen`
//!
//! > *"Simul Instructio Multiplicis Dati"*
//! > — Single Instruction Multiple Data. (Neo-Latin)
//!
//! This module provides portable SIMD types and operations that accelerate
//! bulk data operations in `ParFlumen`.
//!
//! # Toolchain support
//!
//! With the `nightly` cargo feature (requires a nightly toolchain), this
//! module is backed by the `portable_simd` feature (`core::simd`), giving:
//! - Hardware-accelerated SIMD on x86 (SSE/AVX), ARM (NEON), and other platforms
//! - Type-safe SIMD abstractions without target-specific intrinsics
//!
//! On stable Rust (the default) the same API is backed by scalar loops with
//! identical semantics — LLVM still auto-vectorizes many of them, but codegen
//! is not guaranteed.
//!
//! # Architecture
//!
//! The SIMD layer provides:
//! - `Simd4<T>` - 4-wide SIMD type (128-bit on SSE/NEON)
//! - `Simd8<T>` - 8-wide SIMD type (256-bit on AVX)
//! - Vectorized map, reduce, scan operations
//!
//! # Reference
//!
//! Design inspired by the `wide` crate (Zlib license) and `std::simd`.
//!
//! # Example
//!
//! ```rust
//! use ordofp_core::par::simd::{Simd4f32, simd_map_f32};
//!
//! let data = vec![1.0f32, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
//! let result = simd_map_f32(&data, |x| x * 2.0);
//! assert_eq!(result, vec![2.0, 4.0, 6.0, 8.0, 10.0, 12.0, 14.0, 16.0]);
//! ```

#![cfg(feature = "par")]

use alloc::vec::Vec;
use core::ops::{Add, Div, Mul, Sub};

// Backing vector types: portable SIMD from core under the `nightly` feature,
// otherwise scalar stand-ins with the same method surface (identical results).
#[cfg(feature = "nightly")]
use core::simd::{f32x4, f32x8, num::SimdFloat};
// Import StdFloat for sqrt, mul_add, etc. (only available with std)
#[cfg(all(feature = "nightly", feature = "std"))]
use std::simd::StdFloat;

#[cfg(feature = "nightly")]
type Backing4 = f32x4;
#[cfg(feature = "nightly")]
type Backing8 = f32x8;
#[cfg(not(feature = "nightly"))]
type Backing4 = scalar::F32x4;
#[cfg(not(feature = "nightly"))]
type Backing8 = scalar::F32x8;

/// Scalar stand-ins for `core::simd::{f32x4, f32x8}`, used when the `nightly`
/// feature is off. Same method surface and semantics as the portable-SIMD
/// types; plain loops the optimizer is free to vectorize.
#[cfg(not(feature = "nightly"))]
mod scalar {
    use core::ops::{Add, Div, Mul, Sub};

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct F32x4([f32; 4]);

    impl F32x4 {
        pub const fn from_array(arr: [f32; 4]) -> Self {
            Self(arr)
        }

        pub const fn splat(value: f32) -> Self {
            Self([value; 4])
        }

        /// Panics if the slice has fewer than 4 elements (matches `core::simd`).
        pub fn from_slice(slice: &[f32]) -> Self {
            let mut arr = [0.0f32; 4];
            arr.copy_from_slice(&slice[..4]);
            Self(arr)
        }

        pub const fn to_array(self) -> [f32; 4] {
            self.0
        }

        pub fn reduce_sum(self) -> f32 {
            self.0[0] + self.0[1] + self.0[2] + self.0[3]
        }

        pub fn reduce_product(self) -> f32 {
            self.0[0] * self.0[1] * self.0[2] * self.0[3]
        }

        pub fn reduce_min(self) -> f32 {
            self.0[0].min(self.0[1]).min(self.0[2]).min(self.0[3])
        }

        pub fn reduce_max(self) -> f32 {
            self.0[0].max(self.0[1]).max(self.0[2]).max(self.0[3])
        }

        pub fn simd_min(self, other: Self) -> Self {
            let mut r = [0.0f32; 4];
            for (i, v) in r.iter_mut().enumerate() {
                *v = self.0[i].min(other.0[i]);
            }
            Self(r)
        }

        pub fn simd_max(self, other: Self) -> Self {
            let mut r = [0.0f32; 4];
            for (i, v) in r.iter_mut().enumerate() {
                *v = self.0[i].max(other.0[i]);
            }
            Self(r)
        }

        pub fn abs(self) -> Self {
            let mut r = self.0;
            for v in &mut r {
                *v = v.abs();
            }
            Self(r)
        }

        /// Per-lane square root (requires std for libm).
        #[cfg(feature = "std")]
        pub fn sqrt(self) -> Self {
            let mut r = self.0;
            for v in &mut r {
                *v = v.sqrt();
            }
            Self(r)
        }

        /// Per-lane fused multiply-add (requires std for libm).
        #[cfg(feature = "std")]
        pub fn mul_add(self, a: Self, b: Self) -> Self {
            let mut r = [0.0f32; 4];
            for (i, v) in r.iter_mut().enumerate() {
                *v = self.0[i].mul_add(a.0[i], b.0[i]);
            }
            Self(r)
        }
    }

    macro_rules! lanewise_op {
        ($ty:ident, $lanes:literal, $trait:ident, $method:ident, $op:tt) => {
            impl $trait for $ty {
                type Output = Self;

                #[inline]
                fn $method(self, rhs: Self) -> Self {
                    let mut r = [0.0f32; $lanes];
                    for (i, v) in r.iter_mut().enumerate() {
                        *v = self.0[i] $op rhs.0[i];
                    }
                    Self(r)
                }
            }
        };
    }

    lanewise_op!(F32x4, 4, Add, add, +);
    lanewise_op!(F32x4, 4, Sub, sub, -);
    lanewise_op!(F32x4, 4, Mul, mul, *);
    lanewise_op!(F32x4, 4, Div, div, /);

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct F32x8([f32; 8]);

    impl F32x8 {
        pub const fn from_array(arr: [f32; 8]) -> Self {
            Self(arr)
        }

        pub const fn splat(value: f32) -> Self {
            Self([value; 8])
        }

        /// Panics if the slice has fewer than 8 elements (matches `core::simd`).
        pub fn from_slice(slice: &[f32]) -> Self {
            let mut arr = [0.0f32; 8];
            arr.copy_from_slice(&slice[..8]);
            Self(arr)
        }

        pub const fn to_array(self) -> [f32; 8] {
            self.0
        }

        pub fn reduce_sum(self) -> f32 {
            let mut acc = 0.0f32;
            for v in self.0 {
                acc += v;
            }
            acc
        }
    }

    lanewise_op!(F32x8, 8, Add, add, +);
    lanewise_op!(F32x8, 8, Mul, mul, *);
}

// =============================================================================
// Simd4f32 - 4-wide f32 SIMD type (backed by portable_simd)
// =============================================================================

/// 4-wide f32 SIMD vector.
///
/// Backed by `core::simd::f32x4` (hardware SIMD on x86 SSE, ARM NEON, …) when
/// the `nightly` feature is enabled; backed by scalar loops with identical
/// semantics on stable Rust.
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(transparent)]
pub struct Simd4f32 {
    /// The underlying vector (portable SIMD under `nightly`, scalar otherwise).
    inner: Backing4,
}

impl Simd4f32 {
    /// Number of lanes in this SIMD type.
    pub const LANES: usize = 4;

    /// Create from an array.
    #[inline]
    pub const fn new(arr: [f32; 4]) -> Self {
        Self {
            inner: Backing4::from_array(arr),
        }
    }

    /// Create a SIMD vector with all lanes set to the same value.
    #[inline]
    pub const fn splat(value: f32) -> Self {
        Self {
            inner: Backing4::splat(value),
        }
    }

    /// Create a SIMD vector with all zeros.
    #[inline]
    pub const fn zero() -> Self {
        Self::splat(0.0)
    }

    /// Create a SIMD vector with all ones.
    #[inline]
    pub const fn one() -> Self {
        Self::splat(1.0)
    }

    /// Load from a slice. Panics if slice has fewer than 4 elements.
    #[inline]
    pub fn load(slice: &[f32]) -> Self {
        Self {
            inner: Backing4::from_slice(slice),
        }
    }

    /// Load from a slice, using zeros for missing elements.
    #[inline]
    pub fn load_partial(slice: &[f32]) -> Self {
        let mut arr = [0.0f32; 4];
        let len = slice.len().min(4);
        arr[..len].copy_from_slice(&slice[..len]);
        Self::new(arr)
    }

    /// Store to a slice.
    #[inline]
    pub fn store(self, slice: &mut [f32]) {
        slice[..4].copy_from_slice(&self.inner.to_array());
    }

    /// Store partial (only elements that fit).
    #[inline]
    pub fn store_partial(self, slice: &mut [f32]) {
        let len = slice.len().min(4);
        let arr = self.inner.to_array();
        slice[..len].copy_from_slice(&arr[..len]);
    }

    /// Convert to array.
    #[inline]
    pub const fn to_array(self) -> [f32; 4] {
        self.inner.to_array()
    }

    /// Access the underlying array (for compatibility).
    #[inline]
    pub const fn arr(&self) -> [f32; 4] {
        self.inner.to_array()
    }

    /// Horizontal sum of all lanes using SIMD reduce.
    #[inline]
    pub fn sum(self) -> f32 {
        self.inner.reduce_sum()
    }

    /// Horizontal product of all lanes using SIMD reduce.
    #[inline]
    pub fn product(self) -> f32 {
        self.inner.reduce_product()
    }

    /// Element-wise minimum using SIMD.
    #[inline]
    pub fn min(self, other: Self) -> Self {
        Self {
            inner: self.inner.simd_min(other.inner),
        }
    }

    /// Element-wise maximum using SIMD.
    #[inline]
    pub fn max(self, other: Self) -> Self {
        Self {
            inner: self.inner.simd_max(other.inner),
        }
    }

    /// Horizontal minimum using SIMD reduce.
    #[inline]
    pub fn min_element(self) -> f32 {
        self.inner.reduce_min()
    }

    /// Horizontal maximum using SIMD reduce.
    #[inline]
    pub fn max_element(self) -> f32 {
        self.inner.reduce_max()
    }

    /// Element-wise absolute value using SIMD.
    #[inline]
    pub fn abs(self) -> Self {
        Self {
            inner: self.inner.abs(),
        }
    }

    /// Element-wise square root using SIMD.
    #[inline]
    #[cfg(feature = "std")]
    pub fn sqrt(self) -> Self {
        Self {
            inner: self.inner.sqrt(),
        }
    }

    /// Fused multiply-add: self * a + b (uses FMA instructions when available)
    #[inline]
    #[cfg(feature = "std")]
    pub fn mul_add(self, a: Self, b: Self) -> Self {
        Self {
            inner: self.inner.mul_add(a.inner, b.inner),
        }
    }

    /// Apply a function to each element.
    /// Note: This is a scalar fallback; prefer SIMD operations when possible.
    #[inline]
    pub fn map<F: Fn(f32) -> f32>(self, f: F) -> Self {
        let arr = self.inner.to_array();
        Self::new([f(arr[0]), f(arr[1]), f(arr[2]), f(arr[3])])
    }
}

impl Default for Simd4f32 {
    #[inline]
    fn default() -> Self {
        Self::zero()
    }
}

impl From<[f32; 4]> for Simd4f32 {
    #[inline]
    fn from(arr: [f32; 4]) -> Self {
        Self::new(arr)
    }
}

impl From<Simd4f32> for [f32; 4] {
    #[inline]
    fn from(simd: Simd4f32) -> Self {
        simd.inner.to_array()
    }
}

impl From<f32> for Simd4f32 {
    #[inline]
    fn from(value: f32) -> Self {
        Self::splat(value)
    }
}

#[cfg(feature = "nightly")]
impl From<f32x4> for Simd4f32 {
    #[inline]
    fn from(inner: f32x4) -> Self {
        Self { inner }
    }
}

impl Add for Simd4f32 {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            inner: self.inner + rhs.inner,
        }
    }
}

impl Sub for Simd4f32 {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            inner: self.inner - rhs.inner,
        }
    }
}

impl Mul for Simd4f32 {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            inner: self.inner * rhs.inner,
        }
    }
}

impl Div for Simd4f32 {
    type Output = Self;

    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        Self {
            inner: self.inner / rhs.inner,
        }
    }
}

// =============================================================================
// Simd8f32 - 8-wide f32 SIMD type (backed by portable_simd for AVX)
// =============================================================================

/// 8-wide f32 SIMD vector.
///
/// Backed by `core::simd::f32x8` (256-bit AVX on `x86_64`, …) when the
/// `nightly` feature is enabled; backed by scalar loops with identical
/// semantics on stable Rust.
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(transparent)]
pub struct Simd8f32 {
    /// The underlying vector (portable SIMD under `nightly`, scalar otherwise).
    inner: Backing8,
}

impl Simd8f32 {
    /// Number of lanes in this SIMD type.
    pub const LANES: usize = 8;

    /// Create from an array.
    #[inline]
    pub const fn new(arr: [f32; 8]) -> Self {
        Self {
            inner: Backing8::from_array(arr),
        }
    }

    /// Create a SIMD vector with all lanes set to the same value.
    #[inline]
    pub const fn splat(value: f32) -> Self {
        Self {
            inner: Backing8::splat(value),
        }
    }

    /// Create a SIMD vector with all zeros.
    #[inline]
    pub const fn zero() -> Self {
        Self::splat(0.0)
    }

    /// Load from a slice.
    #[inline]
    pub fn load(slice: &[f32]) -> Self {
        Self {
            inner: Backing8::from_slice(slice),
        }
    }

    /// Load from a slice, using zeros for missing elements.
    #[inline]
    pub fn load_partial(slice: &[f32]) -> Self {
        let mut arr = [0.0f32; 8];
        let len = slice.len().min(8);
        arr[..len].copy_from_slice(&slice[..len]);
        Self::new(arr)
    }

    /// Store to a slice.
    #[inline]
    pub fn store(self, slice: &mut [f32]) {
        slice[..8].copy_from_slice(&self.inner.to_array());
    }

    /// Store partial.
    #[inline]
    pub fn store_partial(self, slice: &mut [f32]) {
        let len = slice.len().min(8);
        let arr = self.inner.to_array();
        slice[..len].copy_from_slice(&arr[..len]);
    }

    /// Access the underlying array (for compatibility).
    #[inline]
    pub const fn arr(&self) -> [f32; 8] {
        self.inner.to_array()
    }

    /// Horizontal sum using SIMD reduce.
    #[inline]
    pub fn sum(self) -> f32 {
        self.inner.reduce_sum()
    }

    /// Apply a function to each element.
    /// Note: This is a scalar fallback; prefer SIMD operations when possible.
    #[inline]
    pub fn map<F: Fn(f32) -> f32>(self, f: F) -> Self {
        let arr = self.inner.to_array();
        Self::new([
            f(arr[0]),
            f(arr[1]),
            f(arr[2]),
            f(arr[3]),
            f(arr[4]),
            f(arr[5]),
            f(arr[6]),
            f(arr[7]),
        ])
    }
}

impl Default for Simd8f32 {
    #[inline]
    fn default() -> Self {
        Self::zero()
    }
}

impl Add for Simd8f32 {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            inner: self.inner + rhs.inner,
        }
    }
}

impl Mul for Simd8f32 {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            inner: self.inner * rhs.inner,
        }
    }
}

// =============================================================================
// Vectorized Operations
// =============================================================================

/// Map a function over an f32 slice using SIMD.
///
/// Processes 4 elements at a time when possible, with scalar fallback
/// for remaining elements.
#[inline]
pub fn simd_map_f32<F>(data: &[f32], f: F) -> Vec<f32>
where
    F: Fn(f32) -> f32,
{
    let mut result = Vec::with_capacity(data.len());

    // Process in chunks of 4
    let (chunks, remainder) = data.as_chunks::<4>();

    for chunk in chunks {
        let simd = Simd4f32::load(chunk);
        let mapped = simd.map(&f);
        result.extend_from_slice(&mapped.arr());
    }

    // Handle remaining elements
    for &x in remainder {
        result.push(f(x));
    }

    result
}

/// Map a function over an f32 slice in place using SIMD.
#[inline]
pub fn simd_map_f32_inplace<F>(data: &mut [f32], f: F)
where
    F: Fn(f32) -> f32,
{
    // Process in chunks of 4
    let (chunks, remainder) = data.split_at_mut(data.len() - data.len() % 4);

    for chunk in chunks.as_chunks_mut::<4>().0 {
        let simd = Simd4f32::load(chunk);
        let mapped = simd.map(&f);
        mapped.store(chunk);
    }

    // Handle remaining elements
    for x in remainder {
        *x = f(*x);
    }
}

/// Sum an f32 slice using SIMD.
#[inline]
pub fn simd_sum_f32(data: &[f32]) -> f32 {
    let mut acc = Simd4f32::zero();

    // Process in chunks of 4
    let (chunks, remainder) = data.as_chunks::<4>();

    for chunk in chunks {
        let simd = Simd4f32::load(chunk);
        acc = acc + simd;
    }

    // Sum the accumulator lanes + remainder
    acc.sum() + remainder.iter().sum::<f32>()
}

/// Dot product of two f32 slices using SIMD.
///
/// # Panics
///
/// Panics if `a` and `b` have different lengths.
#[inline]
pub fn simd_dot_f32(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len(), "Vectors must have same length");

    let mut acc = Simd4f32::zero();
    let simd_len = a.len() / 4 * 4;

    // as_chunks gives the compiler tighter bounds info than step_by;
    // FMA (when std is available) fuses multiply-add into a single instruction.
    for (chunk_a, chunk_b) in a[..simd_len]
        .as_chunks::<4>()
        .0
        .iter()
        .zip(b[..simd_len].as_chunks::<4>().0)
    {
        let va = Simd4f32::load(chunk_a);
        let vb = Simd4f32::load(chunk_b);
        #[cfg(feature = "std")]
        {
            acc = va.mul_add(vb, acc);
        }
        #[cfg(not(feature = "std"))]
        {
            acc = acc + va * vb;
        }
    }

    let mut scalar_sum = 0.0f32;
    for i in simd_len..a.len() {
        scalar_sum += a[i] * b[i];
    }

    acc.sum() + scalar_sum
}

/// Find minimum value in an f32 slice using SIMD.
#[inline]
pub fn simd_min_f32(data: &[f32]) -> Option<f32> {
    if data.is_empty() {
        return None;
    }

    let mut acc = Simd4f32::splat(f32::INFINITY);

    let (chunks, remainder) = data.as_chunks::<4>();

    for chunk in chunks {
        let simd = Simd4f32::load(chunk);
        acc = acc.min(simd);
    }

    let mut min_val = acc.min_element();
    for &x in remainder {
        min_val = min_val.min(x);
    }

    Some(min_val)
}

/// Find maximum value in an f32 slice using SIMD.
#[inline]
pub fn simd_max_f32(data: &[f32]) -> Option<f32> {
    if data.is_empty() {
        return None;
    }

    let mut acc = Simd4f32::splat(f32::NEG_INFINITY);

    let (chunks, remainder) = data.as_chunks::<4>();

    for chunk in chunks {
        let simd = Simd4f32::load(chunk);
        acc = acc.max(simd);
    }

    let mut max_val = acc.max_element();
    for &x in remainder {
        max_val = max_val.max(x);
    }

    Some(max_val)
}

/// Compute element-wise sum of two f32 slices using SIMD.
///
/// # Panics
///
/// Panics if `a` and `b` have different lengths.
#[inline]
pub fn simd_add_f32(a: &[f32], b: &[f32]) -> Vec<f32> {
    assert_eq!(a.len(), b.len(), "Vectors must have same length");

    let mut result = Vec::with_capacity(a.len());
    let simd_len = a.len() / 4 * 4;

    for (chunk_a, chunk_b) in a[..simd_len]
        .as_chunks::<4>()
        .0
        .iter()
        .zip(b[..simd_len].as_chunks::<4>().0)
    {
        let va = Simd4f32::load(chunk_a);
        let vb = Simd4f32::load(chunk_b);
        result.extend_from_slice(&(va + vb).arr());
    }

    for i in simd_len..a.len() {
        result.push(a[i] + b[i]);
    }

    result
}

/// Compute element-wise product of two f32 slices using SIMD.
///
/// # Panics
///
/// Panics if `a` and `b` have different lengths.
#[inline]
pub fn simd_mul_f32(a: &[f32], b: &[f32]) -> Vec<f32> {
    assert_eq!(a.len(), b.len(), "Vectors must have same length");

    let mut result = Vec::with_capacity(a.len());
    let simd_len = a.len() / 4 * 4;

    for (chunk_a, chunk_b) in a[..simd_len]
        .as_chunks::<4>()
        .0
        .iter()
        .zip(b[..simd_len].as_chunks::<4>().0)
    {
        let va = Simd4f32::load(chunk_a);
        let vb = Simd4f32::load(chunk_b);
        result.extend_from_slice(&(va * vb).arr());
    }

    for i in simd_len..a.len() {
        result.push(a[i] * b[i]);
    }

    result
}

/// Scale an f32 slice by a constant using SIMD.
#[inline]
pub fn simd_scale_f32(data: &[f32], scale: f32) -> Vec<f32> {
    let scale_vec = Simd4f32::splat(scale);
    let mut result = Vec::with_capacity(data.len());

    let (chunks, remainder) = data.as_chunks::<4>();

    for chunk in chunks {
        let simd = Simd4f32::load(chunk);
        let scaled = simd * scale_vec;
        result.extend_from_slice(&scaled.arr());
    }

    for &x in remainder {
        result.push(x * scale);
    }

    result
}

// =============================================================================
// SIMD Backend for ParFlumen
// =============================================================================

/// A CPU backend that uses SIMD acceleration for f32 operations.
///
/// This backend processes f32 data using SIMD instructions when possible,
/// falling back to scalar operations for other types and edge cases.
#[derive(Clone, Copy, Debug, Default)]
pub struct CpuSimd;

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_simd4f32_basic() {
        let a = Simd4f32::new([1.0, 2.0, 3.0, 4.0]);
        let b = Simd4f32::new([5.0, 6.0, 7.0, 8.0]);

        let sum = a + b;
        assert_eq!(sum.arr(), [6.0, 8.0, 10.0, 12.0]);

        let prod = a * b;
        assert_eq!(prod.arr(), [5.0, 12.0, 21.0, 32.0]);

        let diff = b - a;
        assert_eq!(diff.arr(), [4.0, 4.0, 4.0, 4.0]);
    }

    #[test]
    fn test_simd4f32_splat() {
        let v = Simd4f32::splat(2.5);
        assert_eq!(v.arr(), [2.5, 2.5, 2.5, 2.5]);
    }

    #[test]
    fn test_simd4f32_sum() {
        let v = Simd4f32::new([1.0, 2.0, 3.0, 4.0]);
        assert_eq!(v.sum(), 10.0);
    }

    #[test]
    fn test_simd4f32_min_max() {
        let a = Simd4f32::new([1.0, 5.0, 3.0, 7.0]);
        let b = Simd4f32::new([2.0, 4.0, 6.0, 1.0]);

        let min = a.min(b);
        assert_eq!(min.arr(), [1.0, 4.0, 3.0, 1.0]);

        let max = a.max(b);
        assert_eq!(max.arr(), [2.0, 5.0, 6.0, 7.0]);
    }

    #[test]
    fn test_simd4f32_map() {
        let v = Simd4f32::new([1.0, 2.0, 3.0, 4.0]);
        let doubled = v.map(|x| x * 2.0);
        assert_eq!(doubled.arr(), [2.0, 4.0, 6.0, 8.0]);
    }

    #[test]
    fn test_simd_map_f32() {
        let data = vec![1.0f32, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let result = simd_map_f32(&data, |x| x * 2.0);
        assert_eq!(result, vec![2.0, 4.0, 6.0, 8.0, 10.0, 12.0, 14.0, 16.0]);
    }

    #[test]
    fn test_simd_map_f32_odd_length() {
        let data = vec![1.0f32, 2.0, 3.0, 4.0, 5.0];
        let result = simd_map_f32(&data, |x| x + 1.0);
        assert_eq!(result, vec![2.0, 3.0, 4.0, 5.0, 6.0]);
    }

    #[test]
    fn test_simd_sum_f32() {
        let data = vec![1.0f32, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        assert_eq!(simd_sum_f32(&data), 36.0);
    }

    #[test]
    fn test_simd_sum_f32_odd() {
        let data = vec![1.0f32, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(simd_sum_f32(&data), 15.0);
    }

    #[test]
    fn test_simd_dot_f32() {
        let a = vec![1.0f32, 2.0, 3.0, 4.0];
        let b = vec![5.0f32, 6.0, 7.0, 8.0];
        // 1*5 + 2*6 + 3*7 + 4*8 = 5 + 12 + 21 + 32 = 70
        assert_eq!(simd_dot_f32(&a, &b), 70.0);
    }

    #[test]
    fn test_simd_min_max_f32() {
        let data = vec![3.0f32, 1.0, 4.0, 1.0, 5.0, 9.0, 2.0, 6.0];
        assert_eq!(simd_min_f32(&data), Some(1.0));
        assert_eq!(simd_max_f32(&data), Some(9.0));
    }

    #[test]
    fn test_simd_add_f32() {
        let a = vec![1.0f32, 2.0, 3.0, 4.0, 5.0];
        let b = vec![10.0f32, 20.0, 30.0, 40.0, 50.0];
        let result = simd_add_f32(&a, &b);
        assert_eq!(result, vec![11.0, 22.0, 33.0, 44.0, 55.0]);
    }

    #[test]
    fn test_simd_scale_f32() {
        let data = vec![1.0f32, 2.0, 3.0, 4.0, 5.0];
        let result = simd_scale_f32(&data, 3.0);
        assert_eq!(result, vec![3.0, 6.0, 9.0, 12.0, 15.0]);
    }

    #[test]
    fn test_simd_map_inplace() {
        let mut data = vec![1.0f32, 2.0, 3.0, 4.0, 5.0, 6.0];
        simd_map_f32_inplace(&mut data, |x| x * x);
        assert_eq!(data, vec![1.0, 4.0, 9.0, 16.0, 25.0, 36.0]);
    }

    #[test]
    fn test_simd8f32_basic() {
        let a = Simd8f32::new([1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]);
        let b = Simd8f32::splat(2.0);

        let sum = a + b;
        assert_eq!(sum.arr(), [3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0]);

        let prod = a * b;
        assert_eq!(prod.arr(), [2.0, 4.0, 6.0, 8.0, 10.0, 12.0, 14.0, 16.0]);
    }
}
