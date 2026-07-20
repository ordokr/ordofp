//! `ParFlumen` backends - execution strategies for parallel pipelines.
//!
//! > *"Machina Computandi"*
//! > — Computing machine. (Neo-Latin)
//!
//! This module provides different execution backends for `FlumenParallelum`:
//!
//! - `CpuScalar` - Single-threaded scalar execution
//! - `CpuRayon` - Multi-threaded execution using Rayon (requires `rayon` feature)
//! - `wgpu::GpuWgpu` - GPU execution via wgpu/WGSL (requires `gpu-wgpu` feature)
//!
//! # Example
//!
//! ```rust
//! use ordofp_core::par::{ParFlumen, backend::CpuScalar};
//!
//! let stream = ParFlumen::from_vec(vec![1, 2, 3, 4, 5]);
//!
//! // Scalar execution
//! let result_scalar = stream.collect_vec(&CpuScalar);
//! assert_eq!(result_scalar, vec![1, 2, 3, 4, 5]);
//!
//! // `backend::CpuRayon` (requires the `rayon` feature) gives multi-threaded
//! // execution over the same pipeline -- see `CpuRayon`'s own doc example.
//! ```

#![cfg(feature = "par")]

use alloc::vec::Vec;

use super::Nodus;

#[cfg(feature = "gpu-wgpu")]
pub mod wgpu;

/// Backend trait for executing parallel pipelines.
///
/// Backends provide different execution strategies for the same pipeline.
/// This allows choosing between scalar, parallel, or even GPU execution
/// without changing the pipeline definition.
pub trait Backend {
    /// Collect all elements into a vector.
    fn collect<T>(&self, node: &dyn Nodus<Item = T>) -> Vec<T>
    where
        T: Clone + Send + Sync + 'static;

    /// Reduce elements using a binary associative operation.
    fn reduce<T, F>(&self, node: &dyn Nodus<Item = T>, f: F) -> Option<T>
    where
        T: Clone + Send + Sync + 'static,
        F: Fn(T, T) -> T + Send + Sync;

    /// Reduce elements using a GPU-accelerated operation if available.
    ///
    /// # Arguments
    ///
    /// * `node` - The data node to reduce
    /// * `wgsl_op` - WGSL binary operation (e.g. "+", "*", "min", "max")
    /// * `fallback` - Fallback closure for CPU execution
    fn reduce_gpu<T, F>(&self, node: &dyn Nodus<Item = T>, wgsl_op: &str, fallback: F) -> Option<T>
    where
        T: Clone + Send + Sync + 'static,
        F: Fn(T, T) -> T + Send + Sync,
    {
        // Default implementation uses the fallback
        let _ = wgsl_op; // Unused in default impl
        self.reduce(node, fallback)
    }

    /// Fold elements with an initial value.
    fn fold<T, B, F>(&self, node: &dyn Nodus<Item = T>, init: B, f: F) -> B
    where
        T: Clone + Send + Sync + 'static,
        B: Clone + Send + Sync + 'static,
        F: Fn(B, T) -> B + Send + Sync;

    /// Execute a side-effect for each element.
    fn for_each<T, F>(&self, node: &dyn Nodus<Item = T>, f: F)
    where
        T: Clone + Send + Sync + 'static,
        F: Fn(T) + Send + Sync;

    /// Check if any element satisfies the predicate.
    fn any<T, F>(&self, node: &dyn Nodus<Item = T>, predicate: F) -> bool
    where
        T: Clone + Send + Sync + 'static,
        F: Fn(&T) -> bool + Send + Sync;

    /// Check if all elements satisfy the predicate.
    fn all<T, F>(&self, node: &dyn Nodus<Item = T>, predicate: F) -> bool
    where
        T: Clone + Send + Sync + 'static,
        F: Fn(&T) -> bool + Send + Sync;

    /// Find the first element satisfying the predicate.
    fn find<T, F>(&self, node: &dyn Nodus<Item = T>, predicate: F) -> Option<T>
    where
        T: Clone + Send + Sync + 'static,
        F: Fn(&T) -> bool + Send + Sync;

    /// Count the number of elements.
    fn count<T>(&self, node: &dyn Nodus<Item = T>) -> usize
    where
        T: Clone + Send + Sync + 'static;
}

// =============================================================================
// Helper functions for scalar execution
// =============================================================================

#[inline]
fn reduce_scalar<T>(node: &dyn Nodus<Item = T>, f: &dyn Fn(T, T) -> T) -> Option<T>
where
    T: Clone + Send + Sync + 'static,
{
    // visit_scalar gives T by value — no extra clone needed for the accumulator.
    // For map pipelines this saves 1 clone per element vs. visit_scalar_ref.
    let mut acc: Option<T> = None;
    node.visit_scalar(&mut |x| {
        acc = Some(match acc.take() {
            None => x,
            Some(a) => f(a, x),
        });
    });
    acc
}

#[inline]
fn fold_scalar<T, B>(node: &dyn Nodus<Item = T>, init: B, f: &dyn Fn(B, T) -> B) -> B
where
    T: Clone + Send + Sync + 'static,
    B: Clone,
{
    // Option<B> moves acc out to feed f without cloning B.
    // visit_scalar gives T by value — no extra clone needed.
    let mut acc: Option<B> = Some(init);
    node.visit_scalar(&mut |x| {
        let old = acc.take().unwrap();
        acc = Some(f(old, x));
    });
    acc.unwrap()
}

#[inline]
fn for_each_scalar<T>(node: &dyn Nodus<Item = T>, f: &dyn Fn(T))
where
    T: Clone + Send + Sync + 'static,
{
    node.visit_scalar(&mut |x| f(x));
}

#[inline]
fn any_scalar<T>(node: &dyn Nodus<Item = T>, predicate: &dyn Fn(&T) -> bool) -> bool
where
    T: Clone + Send + Sync + 'static,
{
    use core::ops::ControlFlow;
    let mut found = false;
    node.try_visit_scalar_ref(&mut |x| {
        if predicate(x) {
            found = true;
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    });
    found
}

#[inline]
fn all_scalar<T>(node: &dyn Nodus<Item = T>, predicate: &dyn Fn(&T) -> bool) -> bool
where
    T: Clone + Send + Sync + 'static,
{
    use core::ops::ControlFlow;
    let mut all_match = true;
    node.try_visit_scalar_ref(&mut |x| {
        if predicate(x) {
            ControlFlow::Continue(())
        } else {
            all_match = false;
            ControlFlow::Break(())
        }
    });
    all_match
}

#[inline]
fn find_scalar<T>(node: &dyn Nodus<Item = T>, predicate: &dyn Fn(&T) -> bool) -> Option<T>
where
    T: Clone + Send + Sync + 'static,
{
    use core::ops::ControlFlow;
    let mut result: Option<T> = None;
    node.try_visit_scalar_ref(&mut |x| {
        if predicate(x) {
            // Record only the FIRST match. We still return Break to request
            // early termination from nodes that honor it, but the is_none()
            // guard keeps `find` correct even for nodes whose
            // try_visit_scalar_ref uses the non-short-circuiting default impl
            // (otherwise a later match would overwrite the first one).
            if result.is_none() {
                result = Some(x.clone());
            }
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    });
    result
}

#[inline]
fn count_scalar<T>(node: &dyn Nodus<Item = T>) -> usize
where
    T: Clone + Send + Sync + 'static,
{
    // Zero clones: visit_scalar_ref yields &T so no clone is performed.
    // Implementors that back a dense buffer (e.g. NodusInit) override
    // visit_scalar_ref to iterate by raw reference, making this a cheap
    // pointer-chase loop with no element movement at all.
    let mut count = 0;
    node.visit_scalar_ref(&mut |_| count += 1);
    count
}

// =============================================================================
// CpuScalar - Single-threaded execution
// =============================================================================

/// Single-threaded scalar backend.
///
/// This is the simplest backend that executes operations sequentially.
/// Suitable for small datasets or when parallel overhead would dominate.
#[derive(Clone, Copy, Debug, Default)]
pub struct CpuScalar;

impl Backend for CpuScalar {
    #[inline]
    fn collect<T>(&self, node: &dyn Nodus<Item = T>) -> Vec<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        node.collect_scalar()
    }

    #[inline]
    fn reduce<T, F>(&self, node: &dyn Nodus<Item = T>, f: F) -> Option<T>
    where
        T: Clone + Send + Sync + 'static,
        F: Fn(T, T) -> T + Send + Sync,
    {
        reduce_scalar(node, &f)
    }

    #[inline]
    fn fold<T, B, F>(&self, node: &dyn Nodus<Item = T>, init: B, f: F) -> B
    where
        T: Clone + Send + Sync + 'static,
        B: Clone + Send + Sync + 'static,
        F: Fn(B, T) -> B + Send + Sync,
    {
        fold_scalar(node, init, &f)
    }

    #[inline]
    fn for_each<T, F>(&self, node: &dyn Nodus<Item = T>, f: F)
    where
        T: Clone + Send + Sync + 'static,
        F: Fn(T) + Send + Sync,
    {
        for_each_scalar(node, &f);
    }

    #[inline]
    fn any<T, F>(&self, node: &dyn Nodus<Item = T>, predicate: F) -> bool
    where
        T: Clone + Send + Sync + 'static,
        F: Fn(&T) -> bool + Send + Sync,
    {
        any_scalar(node, &predicate)
    }

    #[inline]
    fn all<T, F>(&self, node: &dyn Nodus<Item = T>, predicate: F) -> bool
    where
        T: Clone + Send + Sync + 'static,
        F: Fn(&T) -> bool + Send + Sync,
    {
        all_scalar(node, &predicate)
    }

    #[inline]
    fn find<T, F>(&self, node: &dyn Nodus<Item = T>, predicate: F) -> Option<T>
    where
        T: Clone + Send + Sync + 'static,
        F: Fn(&T) -> bool + Send + Sync,
    {
        find_scalar(node, &predicate)
    }

    #[inline]
    fn count<T>(&self, node: &dyn Nodus<Item = T>) -> usize
    where
        T: Clone + Send + Sync + 'static,
    {
        // For indexed nodes (no filters in chain), len() is O(1) and exact.
        if node.is_indexed() {
            node.len()
        } else {
            count_scalar(node)
        }
    }
}

// =============================================================================
// CpuRayon - Multi-threaded execution using Rayon
// =============================================================================

/// Rayon parallel-dispatch overhead, in nanoseconds (order of magnitude).
///
/// Measured on this workload: a parallel reduce over ~10 trivially-cheap
/// elements costs ~10 µs more than the scalar path — that is the fixed cost of
/// spawning/joining a rayon job plus its work-stealing/`Arc` bookkeeping.
/// Parallelism only pays once the *total* estimated work clears this overhead
/// with margin; empirically the trivial-work (~2 ns/elem) scalar↔parallel
/// crossover sits near `5·10^4` elements (`5·10^4 × 2 ns = 10^5 ns`). We
/// therefore treat `10^5` ns (~10× the raw dispatch cost) of estimated work as
/// the break-even point below which the scalar terminal is used.
#[cfg(feature = "rayon")]
const PARALLEL_OVERHEAD_NS: u64 = 100_000;

/// Default per-element work estimate (ns) used by [`CpuRayon::default`]: a
/// single cheap arithmetic op (~2 ns, measured). Conservative on purpose — it
/// makes the default backend fall back to scalar for cheap work until the
/// input is large enough to amortize dispatch, so the parallel path is never a
/// net regression at small sizes. Pass a larger hint via
/// [`CpuRayon::for_cost_ns`] for heavier per-element closures.
#[cfg(feature = "rayon")]
const DEFAULT_COST_NS_PER_ELEM: u32 = 2;

/// Multi-threaded parallel backend using Rayon.
///
/// Rayon's work-stealing scheduler has a fixed per-job dispatch overhead, so
/// parallelizing *cheap* work over a *small* input is a net loss. This backend
/// guards against that with a single `min_len` threshold: inputs shorter than
/// `min_len` run on the scalar terminal.
///
/// # Choosing `min_len`: work-aware construction
///
/// Rather than guess an element count, prefer [`CpuRayon::for_cost_ns`], which
/// derives `min_len` from an estimate of per-element work cost so the backend
/// only parallelizes once total work (`len × cost`) clears rayon's dispatch
/// overhead. [`CpuRayon::default`] is exactly `for_cost_ns(2)` (assume cheap,
/// ~arithmetic work) — chosen so the default never regresses cheap pipelines.
///
/// # Example
///
/// ```rust
/// use ordofp_core::par::backend::CpuRayon;
///
/// // Work-aware: heavy per-element closure (~500 ns) → parallelize from ~200 elems.
/// let heavy = CpuRayon::for_cost_ns(500);
/// // Never-regress default (assumes cheap work → high threshold).
/// let default = CpuRayon::default();
/// // Explicit element threshold, if you really want one.
/// let manual = CpuRayon { min_len: 512 };
/// assert!(heavy.min_len < default.min_len);
/// assert_eq!(manual.min_len, 512);
/// ```
#[cfg(feature = "rayon")]
#[derive(Clone, Copy, Debug)]
pub struct CpuRayon {
    /// Minimum input length to trigger parallel execution. Inputs shorter than
    /// this run on the scalar terminal. See [`CpuRayon::for_cost_ns`] to derive
    /// this from a per-element work estimate.
    pub min_len: usize,
}

#[cfg(feature = "rayon")]
impl CpuRayon {
    /// Hard floor: never attempt parallelism below this many elements,
    /// regardless of the cost-derived threshold (guards pathological tiny `N`
    /// even when a caller claims very expensive per-element work).
    const MIN_PARALLEL_FLOOR: usize = 16;

    /// Build a backend whose parallel threshold is *derived* from an estimate
    /// of per-element work cost, in nanoseconds.
    ///
    /// The backend parallelizes only once `len × cost_ns_per_elem ≥
    /// PARALLEL_OVERHEAD_NS` — i.e. once there is enough total work to amortize
    /// rayon's dispatch overhead. Equivalently it sets
    /// `min_len = max(MIN_PARALLEL_FLOOR, PARALLEL_OVERHEAD_NS / cost_ns_per_elem)`,
    /// so cheap work stays on the scalar path (no net lag) until the input is
    /// large enough to benefit, while heavier work parallelizes sooner.
    ///
    /// `cost_ns_per_elem == 0` is treated as "free" → never parallelize.
    #[must_use]
    pub fn for_cost_ns(cost_ns_per_elem: u32) -> Self {
        let min_len = if cost_ns_per_elem == 0 {
            usize::MAX
        } else {
            let break_even = PARALLEL_OVERHEAD_NS / u64::from(cost_ns_per_elem);
            (break_even as usize).max(Self::MIN_PARALLEL_FLOOR)
        };
        Self { min_len }
    }
}

#[cfg(feature = "rayon")]
impl Default for CpuRayon {
    /// Work-aware default: assumes cheap (`DEFAULT_COST_NS_PER_ELEM` ns/elem)
    /// per-element work, yielding a threshold (~`5·10^4`) that keeps cheap
    /// pipelines on the scalar path until parallelism actually pays — so the
    /// default `CpuRayon` is never a net regression versus `CpuScalar`.
    fn default() -> Self {
        Self::for_cost_ns(DEFAULT_COST_NS_PER_ELEM)
    }
}

#[cfg(feature = "rayon")]
impl Backend for CpuRayon {
    #[inline]
    fn collect<T>(&self, node: &dyn Nodus<Item = T>) -> Vec<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        if node.len() < self.min_len {
            return node.collect_scalar();
        }
        node.collect_rayon()
    }

    #[inline]
    fn reduce<T, F>(&self, node: &dyn Nodus<Item = T>, f: F) -> Option<T>
    where
        T: Clone + Send + Sync + 'static,
        F: Fn(T, T) -> T + Send + Sync,
    {
        if node.len() < self.min_len {
            return reduce_scalar(node, &f);
        }

        // Delegate to the node's `reduce_rayon`: indexed nodes map-by-index and
        // tree-reduce; non-indexed nodes collect then reduce, except map-like
        // nodes (NodusMap) which fuse the map into the reduce, avoiding a
        // throwaway intermediate Vec for `filter().map().reduce()` pipelines.
        node.reduce_rayon(&f)
    }

    #[inline]
    fn fold<T, B, F>(&self, node: &dyn Nodus<Item = T>, init: B, f: F) -> B
    where
        T: Clone + Send + Sync + 'static,
        B: Clone + Send + Sync + 'static,
        F: Fn(B, T) -> B + Send + Sync,
    {
        // Fold is inherently sequential due to accumulator dependency
        // For parallel fold, use reduce with associative operations
        fold_scalar(node, init, &f)
    }

    #[inline]
    fn for_each<T, F>(&self, node: &dyn Nodus<Item = T>, f: F)
    where
        T: Clone + Send + Sync + 'static,
        F: Fn(T) + Send + Sync,
    {
        if node.len() < self.min_len {
            return for_each_scalar(node, &f);
        }

        // Delegate to the node's `for_each_rayon`: indexed nodes apply by index;
        // non-indexed nodes collect then for_each, except map-like nodes
        // (NodusMap) which fuse the map into the parallel for_each, avoiding a
        // throwaway intermediate Vec for `filter().map().for_each()` pipelines.
        node.for_each_rayon(&f);
    }

    #[inline]
    fn any<T, F>(&self, node: &dyn Nodus<Item = T>, predicate: F) -> bool
    where
        T: Clone + Send + Sync + 'static,
        F: Fn(&T) -> bool + Send + Sync,
    {
        use rayon::prelude::*;

        if node.len() < self.min_len || !node.is_indexed() {
            return any_scalar(node, &predicate);
        }

        let pred_ref = &predicate;
        (0..node.len())
            .into_par_iter()
            .any(|i| pred_ref(&node.get(i)))
    }

    #[inline]
    fn all<T, F>(&self, node: &dyn Nodus<Item = T>, predicate: F) -> bool
    where
        T: Clone + Send + Sync + 'static,
        F: Fn(&T) -> bool + Send + Sync,
    {
        use rayon::prelude::*;

        if node.len() < self.min_len || !node.is_indexed() {
            return all_scalar(node, &predicate);
        }

        let pred_ref = &predicate;
        (0..node.len())
            .into_par_iter()
            .all(|i| pred_ref(&node.get(i)))
    }

    #[inline]
    fn find<T, F>(&self, node: &dyn Nodus<Item = T>, predicate: F) -> Option<T>
    where
        T: Clone + Send + Sync + 'static,
        F: Fn(&T) -> bool + Send + Sync,
    {
        use rayon::prelude::*;

        if node.len() < self.min_len || !node.is_indexed() {
            return find_scalar(node, &predicate);
        }

        let pred_ref = &predicate;
        // find_map_first fetches and returns the item in one shot (avoiding the
        // double node.get(i) of the old find_any + .map approach) while honoring
        // `find`'s documented "first element" contract — find_map_any could
        // return any matching element, which is wrong when several match.
        (0..node.len()).into_par_iter().find_map_first(|i| {
            let item = node.get(i);
            if pred_ref(&item) { Some(item) } else { None }
        })
    }

    #[inline]
    fn count<T>(&self, node: &dyn Nodus<Item = T>) -> usize
    where
        T: Clone + Send + Sync + 'static,
    {
        // For indexed nodes, length is known without iteration
        if node.is_indexed() {
            return node.len();
        }
        count_scalar(node)
    }
}

// =============================================================================
// CpuSimd - SIMD-accelerated scalar backend
// =============================================================================

/// SIMD-accelerated CPU backend.
///
/// This backend uses SIMD (Single Instruction Multiple Data) instructions
/// to accelerate bulk operations on f32 data. Falls back to scalar for
/// non-f32 types or when SIMD is unavailable.
///
/// # Acceleration
///
/// The following operations are SIMD-accelerated for f32:
/// - `sum_f32`: Vectorized horizontal sum
/// - `dot_f32`: Vectorized dot product  
/// - `scale_f32`: Vectorized scalar multiplication
/// - `map_f32`: Vectorized element-wise mapping
///
/// # Example
///
/// ```rust
/// use ordofp_core::par::backend::CpuSimd;
///
/// let backend = CpuSimd;
/// let sum = backend.sum_f32(&[1.0, 2.0, 3.0, 4.0]);
/// assert_eq!(sum, 10.0);
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct CpuSimd;

impl CpuSimd {
    /// SIMD-accelerated sum of f32 values.
    #[inline]
    pub fn sum_f32(&self, data: &[f32]) -> f32 {
        super::simd::simd_sum_f32(data)
    }

    /// SIMD-accelerated dot product of f32 vectors.
    #[inline]
    pub fn dot_f32(&self, a: &[f32], b: &[f32]) -> f32 {
        super::simd::simd_dot_f32(a, b)
    }

    /// SIMD-accelerated element-wise addition.
    #[inline]
    pub fn add_f32(&self, a: &[f32], b: &[f32]) -> Vec<f32> {
        super::simd::simd_add_f32(a, b)
    }

    /// SIMD-accelerated element-wise multiplication.
    #[inline]
    pub fn mul_f32(&self, a: &[f32], b: &[f32]) -> Vec<f32> {
        super::simd::simd_mul_f32(a, b)
    }

    /// SIMD-accelerated scalar multiplication.
    #[inline]
    pub fn scale_f32(&self, data: &[f32], scale: f32) -> Vec<f32> {
        super::simd::simd_scale_f32(data, scale)
    }

    /// SIMD-accelerated element-wise map.
    #[inline]
    pub fn map_f32<F: Fn(f32) -> f32>(&self, data: &[f32], f: F) -> Vec<f32> {
        super::simd::simd_map_f32(data, f)
    }

    /// SIMD-accelerated min/max.
    #[inline]
    pub fn min_f32(&self, data: &[f32]) -> Option<f32> {
        super::simd::simd_min_f32(data)
    }

    /// SIMD-accelerated maximum.
    #[inline]
    pub fn max_f32(&self, data: &[f32]) -> Option<f32> {
        super::simd::simd_max_f32(data)
    }

    /// SIMD-accelerated sum of a `ParFlumen<f32>` or any `dyn Nodus<Item = f32>`.
    ///
    /// Collects the stream into a `Vec<f32>` first, then runs the vectorized
    /// `simd_sum_f32`. For pipelines backed by a dense `NodusInit<f32>` this
    /// effectively reduces to two passes (collect + sum), which is still faster
    /// than the one-pass scalar fold because the SIMD sum is throughput-bound
    /// rather than latency-bound.
    #[inline]
    pub fn sum_stream_f32(&self, node: &dyn super::Nodus<Item = f32>) -> f32 {
        let data = node.collect_scalar();
        super::simd::simd_sum_f32(&data)
    }

    /// SIMD-accelerated minimum of a `dyn Nodus<Item = f32>`.
    #[inline]
    pub fn min_stream_f32(&self, node: &dyn super::Nodus<Item = f32>) -> Option<f32> {
        let data = node.collect_scalar();
        super::simd::simd_min_f32(&data)
    }

    /// SIMD-accelerated maximum of a `dyn Nodus<Item = f32>`.
    #[inline]
    pub fn max_stream_f32(&self, node: &dyn super::Nodus<Item = f32>) -> Option<f32> {
        let data = node.collect_scalar();
        super::simd::simd_max_f32(&data)
    }

    /// SIMD-accelerated dot product of two `dyn Nodus<Item = f32>` streams.
    #[inline]
    pub fn dot_stream_f32(
        &self,
        a: &dyn super::Nodus<Item = f32>,
        b: &dyn super::Nodus<Item = f32>,
    ) -> f32 {
        let va = a.collect_scalar();
        let vb = b.collect_scalar();
        super::simd::simd_dot_f32(&va, &vb)
    }

    /// SIMD-accelerated fold over a `dyn Nodus<Item = f32>` for associative,
    /// commutative operations (e.g. sum). For arbitrary folds that are not
    /// just horizontal sum/min/max, falls back to scalar.
    ///
    /// This is provided as an opt-in method rather than overriding the generic
    /// `Backend::fold` because we cannot specialize at compile time without
    /// `#![feature(specialization)]`.
    #[inline]
    pub fn fold_f32<F>(&self, node: &dyn super::Nodus<Item = f32>, init: f32, f: F) -> f32
    where
        F: Fn(f32, f32) -> f32 + Send + Sync,
    {
        // Collect then scalar fold; the collect itself benefits from SIMD when
        // the underlying node is NodusInit and LLVM vectorises the clone loop.
        let data = node.collect_scalar();
        data.into_iter().fold(init, f)
    }

    /// Multiply every element of a `dyn Nodus<Item = f32>` stream by `scale`
    /// using SIMD, returning a new `Vec<f32>`.
    #[inline]
    pub fn scale_stream_f32(
        &self,
        node: &dyn super::Nodus<Item = f32>,
        scale: f32,
    ) -> alloc::vec::Vec<f32> {
        let data = node.collect_scalar();
        super::simd::simd_scale_f32(&data, scale)
    }

    /// Element-wise SIMD sum of two `dyn Nodus<Item = f32>` streams.
    #[inline]
    pub fn add_streams_f32(
        &self,
        a: &dyn super::Nodus<Item = f32>,
        b: &dyn super::Nodus<Item = f32>,
    ) -> alloc::vec::Vec<f32> {
        let va = a.collect_scalar();
        let vb = b.collect_scalar();
        super::simd::simd_add_f32(&va, &vb)
    }
}

// CpuSimd also implements Backend by delegating to CpuScalar
// The SIMD acceleration is primarily through the specialized f32 methods above
impl Backend for CpuSimd {
    #[inline]
    fn collect<T>(&self, node: &dyn Nodus<Item = T>) -> Vec<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        node.collect_scalar()
    }

    #[inline]
    fn reduce<T, F>(&self, node: &dyn Nodus<Item = T>, f: F) -> Option<T>
    where
        T: Clone + Send + Sync + 'static,
        F: Fn(T, T) -> T + Send + Sync,
    {
        reduce_scalar(node, &f)
    }

    #[inline]
    fn fold<T, B, F>(&self, node: &dyn Nodus<Item = T>, init: B, f: F) -> B
    where
        T: Clone + Send + Sync + 'static,
        B: Clone + Send + Sync + 'static,
        F: Fn(B, T) -> B + Send + Sync,
    {
        fold_scalar(node, init, &f)
    }

    #[inline]
    fn for_each<T, F>(&self, node: &dyn Nodus<Item = T>, f: F)
    where
        T: Clone + Send + Sync + 'static,
        F: Fn(T) + Send + Sync,
    {
        for_each_scalar(node, &f);
    }

    #[inline]
    fn any<T, F>(&self, node: &dyn Nodus<Item = T>, predicate: F) -> bool
    where
        T: Clone + Send + Sync + 'static,
        F: Fn(&T) -> bool + Send + Sync,
    {
        any_scalar(node, &predicate)
    }

    #[inline]
    fn all<T, F>(&self, node: &dyn Nodus<Item = T>, predicate: F) -> bool
    where
        T: Clone + Send + Sync + 'static,
        F: Fn(&T) -> bool + Send + Sync,
    {
        all_scalar(node, &predicate)
    }

    #[inline]
    fn find<T, F>(&self, node: &dyn Nodus<Item = T>, predicate: F) -> Option<T>
    where
        T: Clone + Send + Sync + 'static,
        F: Fn(&T) -> bool + Send + Sync,
    {
        find_scalar(node, &predicate)
    }

    #[inline]
    fn count<T>(&self, node: &dyn Nodus<Item = T>) -> usize
    where
        T: Clone + Send + Sync + 'static,
    {
        if node.is_indexed() {
            node.len()
        } else {
            count_scalar(node)
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    struct SimpleNode {
        data: Vec<i32>,
    }

    impl Nodus for SimpleNode {
        type Item = i32;

        fn len(&self) -> usize {
            self.data.len()
        }

        fn visit_scalar(&self, sink: &mut dyn FnMut(Self::Item)) {
            for &x in &self.data {
                sink(x);
            }
        }

        fn is_indexed(&self) -> bool {
            true
        }

        fn get(&self, index: usize) -> Self::Item {
            self.data[index]
        }

        #[cfg(feature = "rayon")]
        fn collect_rayon(&self) -> Vec<Self::Item> {
            use rayon::prelude::*;
            self.data.par_iter().cloned().collect()
        }
    }

    #[test]
    fn test_cpu_scalar_collect() {
        let node = SimpleNode {
            data: vec![1, 2, 3],
        };
        let result = CpuScalar.collect(&node);
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[test]
    fn test_cpu_scalar_reduce() {
        let node = SimpleNode {
            data: vec![1, 2, 3, 4],
        };
        let result = CpuScalar.reduce(&node, |a, b| a + b);
        assert_eq!(result, Some(10));
    }

    #[test]
    fn test_cpu_scalar_fold() {
        let node = SimpleNode {
            data: vec![1, 2, 3],
        };
        let result = CpuScalar.fold(&node, 0, |acc, x| acc + x);
        assert_eq!(result, 6);
    }

    #[test]
    fn test_cpu_scalar_any() {
        let node = SimpleNode {
            data: vec![1, 2, 3, 4, 5],
        };
        assert!(CpuScalar.any(&node, |x| *x > 3));
        assert!(!CpuScalar.any(&node, |x| *x > 10));
    }

    #[test]
    fn test_cpu_scalar_all() {
        let node = SimpleNode {
            data: vec![2, 4, 6],
        };
        assert!(CpuScalar.all(&node, |x| x % 2 == 0));
        assert!(!CpuScalar.all(&node, |x| *x > 3));
    }

    #[test]
    fn test_cpu_scalar_find() {
        let node = SimpleNode {
            data: vec![1, 2, 3, 4, 5],
        };
        assert_eq!(CpuScalar.find(&node, |x| *x > 3), Some(4));
        assert_eq!(CpuScalar.find(&node, |x| *x > 10), None);
    }

    #[test]
    fn test_cpu_scalar_count() {
        let node = SimpleNode {
            data: vec![1, 2, 3, 4, 5],
        };
        assert_eq!(CpuScalar.count(&node), 5);
    }

    #[cfg(feature = "rayon")]
    mod rayon_tests {
        use super::*;

        #[test]
        fn test_cpu_rayon_collect_small() {
            let node = SimpleNode {
                data: vec![1, 2, 3],
            };
            let backend = CpuRayon { min_len: 10 };
            let result = backend.collect(&node);
            assert_eq!(result, vec![1, 2, 3]);
        }

        /// The work-aware constructor derives the parallel threshold from a
        /// per-element cost estimate: `min_len = max(16, 100_000 / cost_ns)`.
        /// Cheap work => high threshold (stay scalar); expensive => low.
        #[test]
        fn test_cpu_rayon_work_aware_threshold() {
            // Cheap (~2 ns/elem): only parallelize at very large inputs.
            assert_eq!(CpuRayon::for_cost_ns(2).min_len, 50_000);
            // Moderate (~100 ns/elem): parallelize from ~1k.
            assert_eq!(CpuRayon::for_cost_ns(100).min_len, 1_000);
            // Heavy (~500 ns/elem): parallelize from ~200.
            assert_eq!(CpuRayon::for_cost_ns(500).min_len, 200);
            // Extremely heavy: clamped to the hard floor, never below it.
            assert_eq!(CpuRayon::for_cost_ns(u32::MAX).min_len, 16);
            // "Free" work is never worth parallelizing.
            assert_eq!(CpuRayon::for_cost_ns(0).min_len, usize::MAX);
            // The default is the cheap-work (never-regress) threshold.
            assert_eq!(CpuRayon::default().min_len, 50_000);
        }

        /// Regression guard for the "never a net lag" property: with the
        /// work-aware default, a cheap pipeline over an input below the derived
        /// threshold must take the *scalar* terminal (not the lagging parallel
        /// one), while still producing the correct result.
        #[test]
        fn test_cpu_rayon_default_stays_scalar_for_cheap_small_input() {
            let backend = CpuRayon::default();
            // 1000 elements of cheap work is far below the ~50k threshold, so
            // the backend must decline to parallelize (1000 < min_len).
            assert!(1000 < backend.min_len);
            let node = SimpleNode {
                data: (0..1000i32).collect(),
            };
            // Result still correct regardless of which terminal ran.
            assert_eq!(
                backend.reduce(&node, |a, b| a + b),
                Some((0..1000i32).sum())
            );
        }

        #[test]
        fn test_cpu_rayon_reduce() {
            let data: Vec<i32> = (1..=100).collect();
            let node = SimpleNode { data };
            let backend = CpuRayon { min_len: 10 };
            let result = backend.reduce(&node, |a, b| a + b);
            assert_eq!(result, Some(5050));
        }

        #[test]
        fn test_cpu_rayon_any() {
            let data: Vec<i32> = (1..=100).collect();
            let node = SimpleNode { data };
            let backend = CpuRayon { min_len: 10 };
            assert!(backend.any(&node, |x| *x > 50));
            assert!(!backend.any(&node, |x| *x > 200));
        }

        #[test]
        fn test_cpu_rayon_all() {
            let data: Vec<i32> = (1..=100).collect();
            let node = SimpleNode { data };
            let backend = CpuRayon { min_len: 10 };
            assert!(backend.all(&node, |x| *x <= 100));
            assert!(!backend.all(&node, |x| *x > 50));
        }
    }

    mod simd_backend_tests {
        use super::*;

        #[test]
        fn test_cpu_simd_sum() {
            let data = vec![1.0f32, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
            let backend = CpuSimd;
            assert_eq!(backend.sum_f32(&data), 36.0);
        }

        #[test]
        fn test_cpu_simd_dot() {
            let a = vec![1.0f32, 2.0, 3.0, 4.0];
            let b = vec![5.0f32, 6.0, 7.0, 8.0];
            let backend = CpuSimd;
            // 1*5 + 2*6 + 3*7 + 4*8 = 5 + 12 + 21 + 32 = 70
            assert_eq!(backend.dot_f32(&a, &b), 70.0);
        }

        #[test]
        fn test_cpu_simd_scale() {
            let data = vec![1.0f32, 2.0, 3.0, 4.0];
            let backend = CpuSimd;
            assert_eq!(backend.scale_f32(&data, 2.0), vec![2.0, 4.0, 6.0, 8.0]);
        }

        #[test]
        fn test_cpu_simd_map() {
            let data = vec![1.0f32, 4.0, 9.0, 16.0];
            let backend = CpuSimd;
            let result = backend.map_f32(&data, f32::sqrt);
            assert_eq!(result, vec![1.0, 2.0, 3.0, 4.0]);
        }

        #[test]
        fn test_cpu_simd_min_max() {
            let data = vec![3.0f32, 1.0, 4.0, 1.0, 5.0, 9.0, 2.0, 6.0];
            let backend = CpuSimd;
            assert_eq!(backend.min_f32(&data), Some(1.0));
            assert_eq!(backend.max_f32(&data), Some(9.0));
        }

        #[test]
        fn test_cpu_simd_add_mul() {
            let a = vec![1.0f32, 2.0, 3.0, 4.0];
            let b = vec![5.0f32, 6.0, 7.0, 8.0];
            let backend = CpuSimd;

            assert_eq!(backend.add_f32(&a, &b), vec![6.0, 8.0, 10.0, 12.0]);
            assert_eq!(backend.mul_f32(&a, &b), vec![5.0, 12.0, 21.0, 32.0]);
        }
    }
}
