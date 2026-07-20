//! `ParFlumen` — bulk, data-parallel collection semantics.
//!
//! > *"Flumen Parallelum"*
//! > — Parallel stream. (Neo-Latin)
//!
//! Phase 2 (CPU backend) provides a minimal execution model for `map/reduce/scan`.
//! This module implements lazy parallel pipelines that can be executed on different
//! backends (scalar CPU, Rayon parallel, future GPU).
//!
//! # Architecture
//!
//! `ParFlumen` uses a lazy IR (Intermediate Representation) based on `Nodus` nodes:
//! - `NodusInit` - Source data
//! - `NodusMap` - Map transformation
//! - `NodusFilter` - Filter predicate
//! - `NodusScan` - Prefix scan
//! - `NodusTake` / `NodusSkip` - Slicing operations
//! - `NodusEnumerate` - Index pairing
//!
//! # Provenance
//!
//! Architecture inspiration:
//! - Rayon iterator plumbing: <https://github.com/rayon-rs/rayon>
//! - Haskell vector fusion: <https://github.com/haskell/vector>
//!
//! # Example
//!
//! ```rust
//! use ordofp_core::par::{ParFlumen, backend::CpuScalar};
//!
//! let data = vec![1, 2, 3, 4, 5];
//! let result = ParFlumen::from_vec(data)
//!     .map(|x| x * 2)
//!     .filter(|x| *x > 4)
//!     .collect_vec(&CpuScalar);
//! assert_eq!(result, vec![6, 8, 10]);
//! ```

#![cfg(feature = "par")]

use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;

pub mod backend;

#[cfg(feature = "gpu-wgpu")]
pub mod codegen;

#[cfg(feature = "gpu-wgpu")]
pub mod opt;

pub mod simd;

mod nodes;
use nodes::{
    NodusChain, NodusEnumerate, NodusFilter, NodusFilterMap, NodusInit, NodusInspect, NodusMap,
    NodusScan, NodusSkip, NodusTake, NodusZip,
};

pub mod fast;
pub use fast::FlumenParallelumFast;

#[cfg(feature = "gpu-wgpu")]
mod gpu;
#[cfg(feature = "gpu-wgpu")]
pub use gpu::GpuMapChain;

/// Type alias for the parallel stream type.
pub type ParFlumen<T> = FlumenParallelum<T>;

/// A lazy, parallel-capable collection pipeline.
///
/// `FlumenParallelum` represents a computation graph that can be executed
/// on various backends. Operations like `map`, `filter`, and `scan` build
/// up the graph without executing it. Execution happens when `collect_vec`,
/// `reduce`, or `for_each` is called with a backend.
///
/// # Latin Etymology
///
/// *Flumen Parallelum* = Parallel stream
#[derive(Clone)]
pub struct FlumenParallelum<T> {
    node: Arc<dyn Nodus<Item = T>>,
}

impl<T> FlumenParallelum<T>
where
    T: Clone + Send + Sync + 'static,
{
    /// Crate-internal constructor used by the fast-path bridge.
    ///
    /// Takes a pre-boxed `Arc<dyn Nodus>` and wraps it as a dyn pipeline.
    #[doc(hidden)]
    #[inline(always)]
    pub fn __from_node(node: Arc<dyn Nodus<Item = T>>) -> Self {
        Self { node }
    }

    /// Create a parallel stream from a vector.
    #[inline(always)]
    pub fn from_vec(vec: Vec<T>) -> Self {
        Self {
            node: Arc::new(NodusInit { data: vec }),
        }
    }

    /// Create a parallel stream from a slice (clones the data).
    #[inline(always)]
    pub fn from_slice(slice: &[T]) -> Self {
        Self::from_vec(slice.to_vec())
    }

    /// Create an empty parallel stream.
    #[inline(always)]
    pub fn empty() -> Self {
        Self::from_vec(Vec::new())
    }

    /// Create a parallel stream with a single element.
    #[inline(always)]
    pub fn singleton(value: T) -> Self {
        Self::from_vec(vec![value])
    }

    /// Get the length of the stream.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.node.len()
    }

    /// Check if the stream is empty.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Map a function over the stream elements.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::par::{ParFlumen, backend::CpuScalar};
    ///
    /// let stream = ParFlumen::from_vec(vec![1, 2, 3]);
    /// let doubled = stream.map(|x| x * 2);
    /// assert_eq!(doubled.collect_vec(&CpuScalar), vec![2, 4, 6]);
    /// ```
    #[inline(always)]
    pub fn map<U, F>(self, f: F) -> FlumenParallelum<U>
    where
        U: Clone + Send + Sync + 'static,
        F: Fn(T) -> U + Send + Sync + 'static,
    {
        FlumenParallelum {
            node: Arc::new(NodusMap {
                prev: self.node,
                f: Arc::new(f),
            }),
        }
    }

    /// Filter elements that satisfy a predicate.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::par::{ParFlumen, backend::CpuScalar};
    ///
    /// let stream = ParFlumen::from_vec(vec![1, 2, 3, 4, 5]);
    /// let evens = stream.filter(|x| x % 2 == 0);
    /// assert_eq!(evens.collect_vec(&CpuScalar), vec![2, 4]);
    /// ```
    #[inline(always)]
    pub fn filter<F>(self, predicate: F) -> FlumenParallelum<T>
    where
        F: Fn(&T) -> bool + Send + Sync + 'static,
    {
        FlumenParallelum {
            node: Arc::new(NodusFilter {
                prev: self.node,
                predicate: Arc::new(predicate),
            }),
        }
    }

    /// Filter and map in one pass.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::par::{ParFlumen, backend::CpuScalar};
    ///
    /// let stream = ParFlumen::from_vec(vec!["1", "x", "3"]);
    /// let parsed = stream.filter_map(|s| s.parse::<i32>().ok());
    /// assert_eq!(parsed.collect_vec(&CpuScalar), vec![1, 3]);
    /// ```
    #[inline(always)]
    pub fn filter_map<U, F>(self, f: F) -> FlumenParallelum<U>
    where
        U: Clone + Send + Sync + 'static,
        F: Fn(T) -> Option<U> + Send + Sync + 'static,
    {
        FlumenParallelum {
            node: Arc::new(NodusFilterMap {
                prev: self.node,
                f: Arc::new(f),
            }),
        }
    }

    /// Prefix scan (cumulative fold).
    ///
    /// Each element becomes the accumulator after folding all previous elements.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::par::{ParFlumen, backend::CpuScalar};
    ///
    /// let stream = ParFlumen::from_vec(vec![1, 2, 3]);
    /// let running_sum = stream.scan(0, |acc, x| acc + x);
    /// assert_eq!(running_sum.collect_vec(&CpuScalar), vec![1, 3, 6]);
    /// ```
    #[inline(always)]
    pub fn scan<B, F>(self, init: B, f: F) -> FlumenParallelum<B>
    where
        B: Clone + Send + Sync + 'static,
        F: Fn(B, T) -> B + Send + Sync + 'static,
    {
        FlumenParallelum {
            node: Arc::new(NodusScan {
                prev: self.node,
                init,
                f: Arc::new(f),
            }),
        }
    }

    /// Take the first `n` elements.
    #[inline(always)]
    pub fn take(self, n: usize) -> FlumenParallelum<T> {
        FlumenParallelum {
            node: Arc::new(NodusTake {
                prev: self.node,
                count: n,
            }),
        }
    }

    /// Skip the first `n` elements.
    #[inline(always)]
    pub fn skip(self, n: usize) -> FlumenParallelum<T> {
        FlumenParallelum {
            node: Arc::new(NodusSkip {
                prev: self.node,
                count: n,
            }),
        }
    }

    /// Pair each element with its index.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::par::{ParFlumen, backend::CpuScalar};
    ///
    /// let stream = ParFlumen::from_vec(vec!["a", "b", "c"]);
    /// let indexed = stream.enumerate();
    /// assert_eq!(
    ///     indexed.collect_vec(&CpuScalar),
    ///     vec![(0, "a"), (1, "b"), (2, "c")]
    /// );
    /// ```
    #[inline(always)]
    pub fn enumerate(self) -> FlumenParallelum<(usize, T)> {
        FlumenParallelum {
            node: Arc::new(NodusEnumerate { prev: self.node }),
        }
    }

    /// Inspect each element without modifying it.
    ///
    /// Useful for debugging.
    #[inline(always)]
    pub fn inspect<F>(self, f: F) -> FlumenParallelum<T>
    where
        F: Fn(&T) + Send + Sync + 'static,
    {
        FlumenParallelum {
            node: Arc::new(NodusInspect {
                prev: self.node,
                f: Arc::new(f),
            }),
        }
    }

    /// Chain two streams together.
    #[inline(always)]
    pub fn chain(self, other: FlumenParallelum<T>) -> FlumenParallelum<T> {
        FlumenParallelum {
            node: Arc::new(NodusChain {
                first: self.node,
                second: other.node,
            }),
        }
    }

    /// Zip two streams together.
    #[inline(always)]
    pub fn zip<U>(self, other: FlumenParallelum<U>) -> FlumenParallelum<(T, U)>
    where
        U: Clone + Send + Sync + 'static,
    {
        FlumenParallelum {
            node: Arc::new(NodusZip {
                first: self.node,
                second: other.node,
            }),
        }
    }

    /// Collect the stream into a vector using the given backend.
    #[inline(always)]
    pub fn collect_vec<Bk>(&self, backend: &Bk) -> Vec<T>
    where
        Bk: backend::Backend,
    {
        backend.collect(&*self.node)
    }

    /// Reduce the stream to a single value using the given backend.
    #[inline(always)]
    pub fn reduce<Bk, F>(&self, backend: &Bk, f: F) -> Option<T>
    where
        Bk: backend::Backend,
        F: Fn(T, T) -> T + Send + Sync,
    {
        backend.reduce(&*self.node, f)
    }

    /// Reduce the stream using a GPU-accelerated operation (if available).
    ///
    /// # Arguments
    ///
    /// * `backend` - The execution backend
    /// * `wgsl_op` - The WGSL binary operation (e.g. "+", "*", "min", "max")
    /// * `fallback` - Rust fallback function for CPU execution
    #[inline(always)]
    pub fn reduce_gpu<Bk, F>(&self, backend: &Bk, wgsl_op: &str, fallback: F) -> Option<T>
    where
        Bk: backend::Backend,
        F: Fn(T, T) -> T + Send + Sync,
    {
        backend.reduce_gpu(&*self.node, wgsl_op, fallback)
    }

    /// Fold the stream with an initial value using the given backend.
    #[inline(always)]
    pub fn fold<Bk, B, F>(&self, backend: &Bk, init: B, f: F) -> B
    where
        Bk: backend::Backend,
        B: Clone + Send + Sync + 'static,
        F: Fn(B, T) -> B + Send + Sync,
    {
        backend.fold(&*self.node, init, f)
    }

    /// Execute a side-effect for each element using the given backend.
    #[inline(always)]
    pub fn for_each<Bk, F>(&self, backend: &Bk, f: F)
    where
        Bk: backend::Backend,
        F: Fn(T) + Send + Sync,
    {
        backend.for_each(&*self.node, f);
    }

    /// Check if any element satisfies the predicate.
    #[inline(always)]
    pub fn any<Bk, F>(&self, backend: &Bk, predicate: F) -> bool
    where
        Bk: backend::Backend,
        F: Fn(&T) -> bool + Send + Sync,
    {
        backend.any(&*self.node, predicate)
    }

    /// Check if all elements satisfy the predicate.
    #[inline(always)]
    pub fn all<Bk, F>(&self, backend: &Bk, predicate: F) -> bool
    where
        Bk: backend::Backend,
        F: Fn(&T) -> bool + Send + Sync,
    {
        backend.all(&*self.node, predicate)
    }

    /// Find the first element satisfying the predicate.
    #[inline(always)]
    pub fn find<Bk, F>(&self, backend: &Bk, predicate: F) -> Option<T>
    where
        Bk: backend::Backend,
        F: Fn(&T) -> bool + Send + Sync,
    {
        backend.find(&*self.node, predicate)
    }

    /// Count the number of elements.
    #[inline(always)]
    pub fn count<Bk>(&self, backend: &Bk) -> usize
    where
        Bk: backend::Backend,
    {
        backend.count(&*self.node)
    }

    /// Sum the elements (requires the element type to support addition).
    #[inline(always)]
    pub fn sum<Bk>(&self, backend: &Bk) -> T
    where
        Bk: backend::Backend,
        T: core::ops::Add<Output = T> + Default,
    {
        self.fold(backend, T::default(), |acc, x| acc + x)
    }

    /// Product of the elements (requires the element type to support multiplication).
    #[inline(always)]
    pub fn product<Bk>(&self, backend: &Bk) -> T
    where
        Bk: backend::Backend,
        T: core::ops::Mul<Output = T> + From<u8>,
    {
        self.fold(backend, T::from(1u8), |acc, x| acc * x)
    }

    /// Consume the pipeline and collect into a `Vec<T>` without a backend.
    ///
    /// When the internal `Arc<dyn Nodus>` is uniquely owned (strong count == 1)
    /// **and** the root node is a `NodusInit`, this moves the underlying `Vec`
    /// out of the node without any cloning. Otherwise it falls back to a normal
    /// `collect_scalar`.
    #[inline]
    pub fn into_vec_scalar(mut self) -> Vec<T> {
        if let Some(inner) = Arc::get_mut(&mut self.node)
            && let Some(v) = inner.try_drain_vec()
        {
            return v;
        }
        self.node.collect_scalar()
    }
}

// =============================================================================
// CpuSimd-specific f32 convenience wrappers
// =============================================================================

impl FlumenParallelum<f32> {
    /// SIMD-accelerated horizontal sum. Equivalent to
    /// `.fold(&CpuSimd, 0.0, |a, x| a + x)` but uses vectorised
    /// reduction instead of scalar iteration.
    #[inline]
    pub fn simd_sum(&self, backend: &backend::CpuSimd) -> f32 {
        backend.sum_stream_f32(&*self.node)
    }

    /// SIMD-accelerated element-wise minimum.
    #[inline]
    pub fn simd_min(&self, backend: &backend::CpuSimd) -> Option<f32> {
        backend.min_stream_f32(&*self.node)
    }

    /// SIMD-accelerated element-wise maximum.
    #[inline]
    pub fn simd_max(&self, backend: &backend::CpuSimd) -> Option<f32> {
        backend.max_stream_f32(&*self.node)
    }

    /// Scale every element by `factor` using SIMD.
    #[inline]
    pub fn simd_scale(&self, backend: &backend::CpuSimd, factor: f32) -> alloc::vec::Vec<f32> {
        backend.scale_stream_f32(&*self.node, factor)
    }
}

// =============================================================================
// GPU-specific implementations
// =============================================================================

#[cfg(feature = "gpu-wgpu")]
impl<T: backend::wgpu::GpuScalar> FlumenParallelum<T> {
    /// Create a GPU-capable parallel stream from data.
    ///
    /// This uses a specialized source node that supports GPU execution.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::par::ParFlumen;
    ///
    /// let stream = ParFlumen::from_vec_gpu(vec![1.0f32, 2.0, 3.0]);
    /// assert_eq!(stream.len(), 3);
    /// ```
    #[inline(always)]
    pub fn from_vec_gpu(vec: Vec<T>) -> Self {
        Self {
            node: Arc::new(gpu::NodusInitGpu { data: vec }),
        }
    }

    /// Map with a WGSL expression for GPU execution.
    ///
    /// The WGSL expression should use `x` as the input variable.
    /// A Rust fallback function is required for CPU execution.
    ///
    /// # Arguments
    ///
    /// * `wgsl_expr` - WGSL expression (e.g., "x * 2.0", "sin(x)", "x * x + 1.0")
    /// * `fallback` - Rust function for CPU fallback
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::par::{ParFlumen, backend::CpuScalar};
    ///
    /// // The Rust fallback runs on CPU backends like `CpuScalar`; the WGSL
    /// // expression is only used when actually executing on a GPU backend.
    /// let stream = ParFlumen::from_vec_gpu(vec![1.0f32, 2.0, 3.0]);
    /// let doubled = stream.map_gpu("x * 2.0", |x| x * 2.0);
    /// assert_eq!(doubled.collect_vec(&CpuScalar), vec![2.0, 4.0, 6.0]);
    /// ```
    #[inline(always)]
    pub fn map_gpu<F>(self, wgsl_expr: &str, fallback: F) -> FlumenParallelum<T>
    where
        F: Fn(T) -> T + Send + Sync + 'static,
    {
        FlumenParallelum {
            node: Arc::new(gpu::NodusGpuMap {
                prev: self.node,
                wgsl_expr: alloc::string::String::from(wgsl_expr),
                fallback: Arc::new(fallback),
            }),
        }
    }

    /// Check if this pipeline can be executed on GPU.
    pub fn is_gpu_capable(&self) -> bool {
        self.node.try_gpu_map_chain().is_some()
    }

    /// Get the GPU map chain for this pipeline (if available).
    pub fn gpu_chain(&self) -> Option<GpuMapChain> {
        self.node.try_gpu_map_chain()
    }

    /// Collect results using the GPU backend.
    ///
    /// This method executes the pipeline on the GPU if possible,
    /// falling back to CPU if GPU execution fails or is unavailable.
    ///
    /// # Example
    ///
    /// This requires a physical GPU adapter, so it cannot run in a headless
    /// doctest environment.
    ///
    /// ```rust,no_run
    /// use ordofp_core::par::{ParFlumen, backend::wgpu::GpuWgpu};
    ///
    /// let backend = GpuWgpu::new().unwrap();
    /// let data = vec![1.0f32, 2.0, 3.0, 4.0, 5.0];
    /// let result = ParFlumen::from_vec_gpu(data)
    ///     .map_gpu("x * 2.0", |x| x * 2.0)
    ///     .collect_gpu(&backend);
    /// ```
    pub fn collect_gpu(&self, backend: &backend::wgpu::GpuWgpu) -> Vec<T> {
        // `Backend::collect` must be in scope for method resolution. Import it
        // *locally* so the trait `use` is compiled only with this `gpu-wgpu`-gated
        // method — a file-level `use` was previously removed as "unused" by a
        // GPU-less `clippy -D warnings` run, silently breaking this path (E0599).
        use crate::par::backend::Backend;
        backend.collect(&*self.node)
    }
}

#[doc(hidden)]
pub trait Nodus: Send + Sync {
    type Item: Clone + Send + Sync + 'static;

    fn len(&self) -> usize;

    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn visit_scalar(&self, sink: &mut dyn FnMut(Self::Item));

    /// Borrow-based visitor for the hot path. Default impl delegates to
    /// `visit_scalar` by cloning each item into a local, then passing a
    /// reference to `sink`. Implementors that already own their items (or
    /// can chain through a prev node's `visit_scalar_ref`) should override
    /// this to avoid per-element clones in the middle of the pipeline.
    #[inline]
    fn visit_scalar_ref(&self, sink: &mut dyn FnMut(&Self::Item)) {
        self.visit_scalar(&mut |x| sink(&x));
    }

    /// Short-circuit borrow visitor.
    ///
    /// Calls `f` for each element; stops immediately when `f` returns
    /// `ControlFlow::Break(())`. The default implementation calls
    /// `visit_scalar_ref` and **ignores** the `ControlFlow` return value,
    /// meaning it does NOT short-circuit — implementors that want early
    /// termination (e.g. `NodusTake`, source nodes) must override this.
    #[inline]
    fn try_visit_scalar_ref(&self, f: &mut dyn FnMut(&Self::Item) -> core::ops::ControlFlow<()>) {
        self.visit_scalar_ref(&mut |x| {
            let _ = f(x);
        });
    }

    /// Size hint, mirroring `Iterator::size_hint`.
    ///
    /// Returns `(lower, upper)` where `lower` is a guaranteed minimum number
    /// of elements and `upper` is an optional guaranteed maximum. The default
    /// implementation returns `(self.len(), Some(self.len()))` — exact for
    /// indexed nodes. Filtering nodes must override this to return accurate
    /// bounds so that `collect_scalar` can choose a sensible pre-allocation.
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.len();
        (n, Some(n))
    }

    #[inline]
    fn collect_scalar(&self) -> Vec<Self::Item> {
        // Use the borrow-based visitor and clone only at the leaf (push).
        // For implementors that override `visit_scalar_ref`, this eliminates
        // every middle-of-pipeline clone and leaves just the single clone
        // required to push an owned value into the output Vec.
        // Use the lower bound from size_hint to avoid worst-case over-allocation
        // in filter pipelines.
        let (lo, _) = self.size_hint();
        let mut out = Vec::with_capacity(lo);
        self.visit_scalar_ref(&mut |x| out.push(x.clone()));
        out
    }

    #[inline(always)]
    fn is_indexed(&self) -> bool {
        false
    }

    #[inline(always)]
    fn get(&self, _index: usize) -> Self::Item {
        crate::cold_panic!("Nodus::get called on a non-indexed node")
    }

    /// Try to get a GPU-executable map chain.
    ///
    /// Returns `Some(chain)` if this node and all its predecessors can be executed on GPU.
    /// Returns `None` if GPU execution is not possible for this pipeline.
    #[cfg(feature = "gpu-wgpu")]
    fn try_gpu_map_chain(&self) -> Option<GpuMapChain> {
        None // Default: not GPU-capable
    }

    /// Try to get the source data as byte slice and WGSL type name for GPU execution.
    #[cfg(feature = "gpu-wgpu")]
    fn try_as_gpu_source(&self) -> Option<(&[u8], &'static str)> {
        None
    }

    #[cfg(feature = "rayon")]
    #[inline]
    fn collect_rayon(&self) -> Vec<Self::Item> {
        use rayon::prelude::*;

        if self.is_indexed() {
            (0..self.len())
                .into_par_iter()
                .map(|i| self.get(i))
                .collect()
        } else {
            self.collect_scalar()
        }
    }

    /// Parallel tree-reduce over the node's items (rayon).
    ///
    /// Default: indexed nodes map-by-index then `reduce_with`; non-indexed
    /// nodes materialize via `collect_rayon` then `reduce_with`. Map-like
    /// nodes override this to fuse the map into the reduce, eliminating the
    /// throwaway intermediate `Vec` that `collect_rayon` would build for a
    /// non-indexed pipeline (e.g. `filter().map().reduce()`).
    #[cfg(feature = "rayon")]
    #[inline]
    fn reduce_rayon(
        &self,
        f: &(dyn Fn(Self::Item, Self::Item) -> Self::Item + Send + Sync),
    ) -> Option<Self::Item> {
        use rayon::prelude::*;

        if self.is_indexed() {
            (0..self.len())
                .into_par_iter()
                .map(|i| self.get(i))
                .reduce_with(f)
        } else {
            self.collect_rayon().into_par_iter().reduce_with(f)
        }
    }

    /// Parallel `for_each` over the node's items (rayon).
    ///
    /// Default: indexed nodes apply `f` by index in parallel; non-indexed
    /// nodes materialize via `collect_rayon` then `for_each`. Map-like nodes
    /// override this to fuse the map into the parallel `for_each`, eliminating
    /// the throwaway intermediate `Vec` that `collect_rayon` would build for a
    /// non-indexed pipeline (e.g. `filter().map().for_each()`).
    #[cfg(feature = "rayon")]
    #[inline]
    fn for_each_rayon(&self, f: &(dyn Fn(Self::Item) + Send + Sync)) {
        use rayon::prelude::*;

        if self.is_indexed() {
            (0..self.len()).into_par_iter().for_each(|i| f(self.get(i)));
        } else {
            self.collect_rayon().into_par_iter().for_each(f);
        }
    }

    /// Try to drain the underlying storage without cloning.
    ///
    /// Returns `Some(vec)` only if this node owns a `Vec<Self::Item>` that can
    /// be moved out (i.e. `NodusInit`). All other nodes return `None`. Used by
    /// `FlumenParallelum::into_vec_scalar` when the `Arc` is uniquely owned, so
    /// the data can be moved rather than cloned.
    #[inline]
    fn try_drain_vec(&mut self) -> Option<Vec<Self::Item>> {
        None
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use backend::CpuScalar;

    #[test]
    fn test_par_flumen_map() {
        let data = vec![1, 2, 3, 4, 5];
        let result = ParFlumen::from_vec(data)
            .map(|x| x * 2)
            .collect_vec(&CpuScalar);
        assert_eq!(result, vec![2, 4, 6, 8, 10]);
    }

    #[test]
    fn test_par_flumen_filter() {
        let data = vec![1, 2, 3, 4, 5, 6];
        let result = ParFlumen::from_vec(data)
            .filter(|x| x % 2 == 0)
            .collect_vec(&CpuScalar);
        assert_eq!(result, vec![2, 4, 6]);
    }

    /// Regression test for the fused parallel reduce (`Nodus::reduce_rayon`
    /// override on `NodusMap`). `filter().map()` is a non-indexed pipeline, so
    /// `CpuRayon { min_len: 1 }` forces the parallel path that fuses the map
    /// into the tree-reduce (skipping the throwaway intermediate `Vec`). The
    /// fused result must equal the scalar reduce and the expected arithmetic.
    #[cfg(feature = "rayon")]
    #[test]
    fn test_par_reduce_rayon_fused_matches_scalar() {
        use backend::CpuRayon;
        let n = 10_000i64;
        let build = || {
            ParFlumen::from_vec((0..n).collect::<Vec<_>>())
                .filter(|x| x % 2 == 0)
                .map(|x| x * 3)
        };
        let expected: i64 = (0..n).filter(|x| x % 2 == 0).map(|x| x * 3).sum();
        let scalar = build().reduce(&CpuScalar, |a, b| a + b);
        let parallel = build().reduce(&CpuRayon { min_len: 1 }, |a, b| a + b);
        assert_eq!(scalar, Some(expected));
        assert_eq!(parallel, scalar, "fused parallel reduce must match scalar");
    }

    /// Regression test for the fused parallel `for_each` (`Nodus::for_each_rayon`
    /// override on `NodusMap`). `filter().map()` is non-indexed, so
    /// `CpuRayon { min_len: 1 }` forces the parallel path that fuses the map
    /// into the `for_each` (skipping the throwaway intermediate `Vec`). The sum
    /// accumulated over every visited element must match the scalar backend.
    #[cfg(feature = "rayon")]
    #[test]
    fn test_par_for_each_rayon_fused_matches_scalar() {
        use backend::CpuRayon;
        use core::sync::atomic::{AtomicI64, Ordering};
        let n = 10_000i64;
        let build = || {
            ParFlumen::from_vec((0..n).collect::<Vec<_>>())
                .filter(|x| x % 2 == 0)
                .map(|x| x * 3)
        };
        let expected: i64 = (0..n).filter(|x| x % 2 == 0).map(|x| x * 3).sum();
        let scalar = AtomicI64::new(0);
        build().for_each(&CpuScalar, |x| {
            scalar.fetch_add(x, Ordering::Relaxed);
        });
        let parallel = AtomicI64::new(0);
        build().for_each(&CpuRayon { min_len: 1 }, |x| {
            parallel.fetch_add(x, Ordering::Relaxed);
        });
        assert_eq!(scalar.load(Ordering::Relaxed), expected);
        assert_eq!(
            parallel.load(Ordering::Relaxed),
            expected,
            "fused parallel for_each must visit every element exactly once"
        );
    }

    /// Fused parallel reduce + for_each on `NodusFilter` (`filter().reduce()` /
    /// `filter().for_each()`, no map). `CpuRayon { min_len: 1 }` forces the
    /// fused path (`NodusFilter::reduce_rayon`/`for_each_rayon`); results must
    /// match the scalar backend.
    #[cfg(feature = "rayon")]
    #[test]
    fn test_par_filter_reduce_for_each_rayon_fused_matches_scalar() {
        use backend::CpuRayon;
        use core::sync::atomic::{AtomicI64, Ordering};
        let n = 10_000i64;
        let build = || ParFlumen::from_vec((0..n).collect::<Vec<_>>()).filter(|x| x % 3 == 0);
        let expected: i64 = (0..n).filter(|x| x % 3 == 0).sum();
        assert_eq!(build().reduce(&CpuScalar, |a, b| a + b), Some(expected));
        assert_eq!(
            build().reduce(&CpuRayon { min_len: 1 }, |a, b| a + b),
            Some(expected),
            "fused filter reduce must match scalar"
        );
        let acc = AtomicI64::new(0);
        build().for_each(&CpuRayon { min_len: 1 }, |x| {
            acc.fetch_add(x, Ordering::Relaxed);
        });
        assert_eq!(acc.load(Ordering::Relaxed), expected);
    }

    /// Fused parallel reduce + for_each on `NodusFilterMap`
    /// (`filter_map().reduce()` / `filter_map().for_each()`). Forced parallel
    /// path must match the scalar backend.
    #[cfg(feature = "rayon")]
    #[test]
    fn test_par_filter_map_reduce_for_each_rayon_fused_matches_scalar() {
        use backend::CpuRayon;
        use core::sync::atomic::{AtomicI64, Ordering};
        let n = 10_000i64;
        let build = || {
            ParFlumen::from_vec((0..n).collect::<Vec<_>>())
                .filter_map(|x| if x % 2 == 0 { Some(x * 2) } else { None })
        };
        let expected: i64 = (0..n).filter(|x| x % 2 == 0).map(|x| x * 2).sum();
        assert_eq!(build().reduce(&CpuScalar, |a, b| a + b), Some(expected));
        assert_eq!(
            build().reduce(&CpuRayon { min_len: 1 }, |a, b| a + b),
            Some(expected),
            "fused filter_map reduce must match scalar"
        );
        let acc = AtomicI64::new(0);
        build().for_each(&CpuRayon { min_len: 1 }, |x| {
            acc.fetch_add(x, Ordering::Relaxed);
        });
        assert_eq!(acc.load(Ordering::Relaxed), expected);
    }

    /// Fused parallel reduce + for_each on `NodusChain` (`a.chain(b)`).
    /// Each half is reduced in parallel via `rayon::join` and combined (reduce
    /// is associative), with no concatenated Vec. Uses non-indexed (filtered)
    /// halves to exercise the fused path, plus an empty-half edge case.
    #[cfg(feature = "rayon")]
    #[test]
    fn test_par_chain_reduce_for_each_rayon_fused_matches_scalar() {
        use backend::CpuRayon;
        use core::sync::atomic::{AtomicI64, Ordering};
        let force = CpuRayon { min_len: 1 };
        let build = || {
            ParFlumen::from_vec((0..5_000i64).collect::<Vec<_>>())
                .filter(|x| x % 2 == 0)
                .chain(
                    ParFlumen::from_vec((5_000..10_000i64).collect::<Vec<_>>())
                        .filter(|x| x % 3 == 0),
                )
        };
        let expected: i64 = (0..5_000).filter(|x| x % 2 == 0).sum::<i64>()
            + (5_000..10_000).filter(|x| x % 3 == 0).sum::<i64>();
        assert_eq!(build().reduce(&CpuScalar, |a, b| a + b), Some(expected));
        assert_eq!(
            build().reduce(&force, |a, b| a + b),
            Some(expected),
            "fused chain reduce must match scalar"
        );
        let acc = AtomicI64::new(0);
        build().for_each(&force, |x| {
            acc.fetch_add(x, Ordering::Relaxed);
        });
        assert_eq!(acc.load(Ordering::Relaxed), expected);

        // Edge case: first half filters to empty — combine must yield the
        // second half's result (None + Some(b) => Some(b)).
        let empty_first = ParFlumen::from_vec((0..100i64).collect::<Vec<_>>())
            .filter(|_| false)
            .chain(ParFlumen::from_vec(alloc::vec![1i64, 2, 3]));
        assert_eq!(empty_first.reduce(&force, |a, b| a + b), Some(6));
    }

    /// Assert the forced-parallel fused terminals (`reduce`, `for_each`,
    /// `collect`) agree with the scalar backend for one pipeline shape.
    /// `CpuRayon { min_len: 1 }` forces the parallel terminal regardless of
    /// size, so callers control which branch (indexed vs non-indexed-upstream
    /// fallback) runs purely via pipeline structure.
    #[cfg(feature = "rayon")]
    fn assert_fused_par_matches_scalar<F>(label: &str, build: F)
    where
        F: Fn() -> ParFlumen<i64>,
    {
        use backend::CpuRayon;
        use core::sync::atomic::{AtomicI64, Ordering};
        let force = CpuRayon { min_len: 1 };

        // reduce: parallel tree-reduce must equal the scalar left-fold (add is
        // associative), including `None` for an empty result.
        let scalar_reduce = build().reduce(&CpuScalar, |a, b| a + b);
        let par_reduce = build().reduce(&force, |a, b| a + b);
        assert_eq!(
            par_reduce, scalar_reduce,
            "{label}: reduce parallel != scalar"
        );

        // for_each: sum + count via atomics prove every element is visited
        // exactly once, matching scalar.
        let s_sum = AtomicI64::new(0);
        let s_cnt = AtomicI64::new(0);
        build().for_each(&CpuScalar, |x| {
            s_sum.fetch_add(x, Ordering::Relaxed);
            s_cnt.fetch_add(1, Ordering::Relaxed);
        });
        let p_sum = AtomicI64::new(0);
        let p_cnt = AtomicI64::new(0);
        build().for_each(&force, |x| {
            p_sum.fetch_add(x, Ordering::Relaxed);
            p_cnt.fetch_add(1, Ordering::Relaxed);
        });
        assert_eq!(
            p_sum.load(Ordering::Relaxed),
            s_sum.load(Ordering::Relaxed),
            "{label}: for_each sum parallel != scalar"
        );
        assert_eq!(
            p_cnt.load(Ordering::Relaxed),
            s_cnt.load(Ordering::Relaxed),
            "{label}: for_each count parallel != scalar"
        );

        // collect: parallel must preserve element order identically to scalar.
        assert_eq!(
            build().collect_vec(&force),
            build().collect_vec(&CpuScalar),
            "{label}: collect parallel != scalar"
        );
    }

    /// Coverage for the fused `reduce_rayon`/`for_each_rayon` across every node
    /// that overrides them, on BOTH the indexed branch and the non-indexed-
    /// upstream fallback `else` branch (e.g. `filter().filter()` makes the
    /// outer node's `prev` non-indexed, forcing the `collect_rayon`-then-
    /// reduce/for_each fallback). All forced-parallel results must equal scalar.
    #[cfg(feature = "rayon")]
    #[test]
    fn test_par_fused_terminals_all_paths_match_scalar() {
        let n = 2_000i64;
        // NodusMap: indexed prev, then non-indexed-prev fallback.
        assert_fused_par_matches_scalar("map[indexed]", || {
            ParFlumen::from_vec((0..n).collect::<Vec<_>>()).map(|x| x * 2)
        });
        assert_fused_par_matches_scalar("filter.map[fallback]", || {
            ParFlumen::from_vec((0..n).collect::<Vec<_>>())
                .filter(|x| x % 2 == 0)
                .map(|x| x * 2)
        });
        // NodusFilter: indexed prev, then non-indexed-prev fallback.
        assert_fused_par_matches_scalar("filter[indexed]", || {
            ParFlumen::from_vec((0..n).collect::<Vec<_>>()).filter(|x| x % 2 == 0)
        });
        assert_fused_par_matches_scalar("filter.filter[fallback]", || {
            ParFlumen::from_vec((0..n).collect::<Vec<_>>())
                .filter(|x| x % 2 == 0)
                .filter(|x| x % 4 == 0)
        });
        // NodusFilterMap: indexed prev, then non-indexed-prev fallback.
        assert_fused_par_matches_scalar("filter_map[indexed]", || {
            ParFlumen::from_vec((0..n).collect::<Vec<_>>())
                .filter_map(|x| if x % 2 == 0 { Some(x * 3) } else { None })
        });
        assert_fused_par_matches_scalar("filter.filter_map[fallback]", || {
            ParFlumen::from_vec((0..n).collect::<Vec<_>>())
                .filter(|x| x % 2 == 0)
                .filter_map(|x| if x % 4 == 0 { Some(x * 3) } else { None })
        });
        // NodusChain: indexed halves, then non-indexed (filtered) halves.
        assert_fused_par_matches_scalar("chain[indexed,indexed]", || {
            ParFlumen::from_vec((0..n).collect::<Vec<_>>())
                .chain(ParFlumen::from_vec((n..2 * n).collect::<Vec<_>>()))
        });
        assert_fused_par_matches_scalar("filter.chain(filter)[fallback]", || {
            ParFlumen::from_vec((0..n).collect::<Vec<_>>())
                .filter(|x| x % 2 == 0)
                .chain(ParFlumen::from_vec((n..2 * n).collect::<Vec<_>>()).filter(|x| x % 3 == 0))
        });
    }

    /// Edge cases — empty input, single element, all-filtered-out — across the
    /// fused fallback shapes. Empty/all-filtered must yield `None` from reduce
    /// and zero `for_each` visits on both backends.
    #[cfg(feature = "rayon")]
    #[test]
    fn test_par_fused_terminals_edge_cases_match_scalar() {
        for n in [0i64, 1, 2, 3] {
            assert_fused_par_matches_scalar("filter.map[edge]", move || {
                ParFlumen::from_vec((0..n).collect::<Vec<_>>())
                    .filter(|x| x % 2 == 0)
                    .map(|x| x + 1)
            });
            assert_fused_par_matches_scalar("filter.filter[edge]", move || {
                ParFlumen::from_vec((0..n).collect::<Vec<_>>())
                    .filter(|x| x % 2 == 0)
                    .filter(|x| *x >= 0)
            });
            assert_fused_par_matches_scalar("filter.filter_map[edge]", move || {
                ParFlumen::from_vec((0..n).collect::<Vec<_>>())
                    .filter(|x| x % 2 == 0)
                    .filter_map(Some)
            });
            assert_fused_par_matches_scalar("filter.chain(empty)[edge]", move || {
                ParFlumen::from_vec((0..n).collect::<Vec<_>>())
                    .filter(|x| x % 2 == 0)
                    .chain(ParFlumen::from_vec((0..n).collect::<Vec<_>>()).filter(|_| false))
            });
        }
        // All elements filtered out (non-trivial input): reduce => None.
        assert_fused_par_matches_scalar("all-filtered", || {
            ParFlumen::from_vec((0..1000i64).collect::<Vec<_>>()).filter(|_| false)
        });
        // Exactly one survivor through a non-indexed-upstream fallback chain.
        assert_fused_par_matches_scalar("single-survivor[fallback]", || {
            ParFlumen::from_vec((0..1000i64).collect::<Vec<_>>())
                .filter(|x| x % 2 == 0)
                .filter(|x| *x == 500)
        });
    }

    #[test]
    fn test_par_flumen_filter_map() {
        let data = vec![1, 2, 3, 4, 5];
        let result = ParFlumen::from_vec(data)
            .filter_map(|x| if x % 2 == 0 { Some(x * 10) } else { None })
            .collect_vec(&CpuScalar);
        assert_eq!(result, vec![20, 40]);
    }

    #[test]
    fn test_par_flumen_take() {
        let data = vec![1, 2, 3, 4, 5];
        let result = ParFlumen::from_vec(data).take(3).collect_vec(&CpuScalar);
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[test]
    fn test_par_flumen_skip() {
        let data = vec![1, 2, 3, 4, 5];
        let result = ParFlumen::from_vec(data).skip(2).collect_vec(&CpuScalar);
        assert_eq!(result, vec![3, 4, 5]);
    }

    #[test]
    fn test_par_flumen_enumerate() {
        let data = vec!["a", "b", "c"];
        let result = ParFlumen::from_vec(data)
            .enumerate()
            .collect_vec(&CpuScalar);
        assert_eq!(result, vec![(0, "a"), (1, "b"), (2, "c")]);
    }

    #[test]
    fn test_par_flumen_chain() {
        let a = ParFlumen::from_vec(vec![1, 2]);
        let b = ParFlumen::from_vec(vec![3, 4, 5]);
        let result = a.chain(b).collect_vec(&CpuScalar);
        assert_eq!(result, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_par_flumen_zip() {
        let a = ParFlumen::from_vec(vec![1, 2, 3]);
        let b = ParFlumen::from_vec(vec!["a", "b", "c"]);
        let result = a.zip(b).collect_vec(&CpuScalar);
        assert_eq!(result, vec![(1, "a"), (2, "b"), (3, "c")]);
    }

    #[test]
    fn test_par_flumen_reduce() {
        let data = vec![1, 2, 3, 4, 5];
        let result = ParFlumen::from_vec(data).reduce(&CpuScalar, |a, b| a + b);
        assert_eq!(result, Some(15));
    }

    #[test]
    fn test_par_flumen_fold() {
        let data = vec![1, 2, 3, 4, 5];
        let result = ParFlumen::from_vec(data).fold(&CpuScalar, 0, |acc, x| acc + x);
        assert_eq!(result, 15);
    }

    #[test]
    fn test_par_flumen_scan() {
        let data = vec![1, 2, 3, 4];
        let result = ParFlumen::from_vec(data)
            .scan(0, |acc, x| acc + x)
            .collect_vec(&CpuScalar);
        assert_eq!(result, vec![1, 3, 6, 10]);
    }

    #[test]
    fn test_par_flumen_any() {
        let data = vec![1, 2, 3, 4, 5];
        assert!(ParFlumen::from_vec(data.clone()).any(&CpuScalar, |x| *x > 3));
        assert!(!ParFlumen::from_vec(data).any(&CpuScalar, |x| *x > 10));
    }

    #[test]
    fn test_par_flumen_all() {
        let data = vec![2, 4, 6, 8];
        assert!(ParFlumen::from_vec(data.clone()).all(&CpuScalar, |x| x % 2 == 0));
        assert!(!ParFlumen::from_vec(data).all(&CpuScalar, |x| *x > 5));
    }

    #[test]
    fn test_par_flumen_find() {
        let data = vec![1, 2, 3, 4, 5];
        let result = ParFlumen::from_vec(data).find(&CpuScalar, |x| *x > 3);
        assert_eq!(result, Some(4));
    }

    #[test]
    fn test_par_flumen_count() {
        let data = vec![1, 2, 3, 4, 5];
        assert_eq!(ParFlumen::from_vec(data).count(&CpuScalar), 5);
    }

    #[test]
    fn test_par_flumen_sum() {
        let data = vec![1, 2, 3, 4, 5];
        assert_eq!(ParFlumen::from_vec(data).sum(&CpuScalar), 15);
    }

    #[test]
    fn test_par_flumen_empty() {
        let empty: ParFlumen<i32> = ParFlumen::empty();
        assert!(empty.is_empty());
        assert_eq!(empty.len(), 0);
    }

    #[test]
    fn test_par_flumen_singleton() {
        let single = ParFlumen::singleton(42);
        assert_eq!(single.len(), 1);
        assert_eq!(single.collect_vec(&CpuScalar), vec![42]);
    }

    #[test]
    fn test_par_flumen_pipeline() {
        // Complex pipeline: map -> filter -> take -> enumerate
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let result = ParFlumen::from_vec(data)
            .map(|x| x * 2) // [2, 4, 6, 8, 10, 12, 14, 16, 18, 20]
            .filter(|x| *x > 6) // [8, 10, 12, 14, 16, 18, 20]
            .take(4) // [8, 10, 12, 14]
            .enumerate() // [(0,8), (1,10), (2,12), (3,14)]
            .collect_vec(&CpuScalar);
        assert_eq!(result, vec![(0, 8), (1, 10), (2, 12), (3, 14)]);
    }
}
