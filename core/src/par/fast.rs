//! Monomorphic fast path for `ParFlumen` pipelines.
//!
//! `FlumenParallelumFast<P>` is an additive, opt-in alternative to the
//! default `FlumenParallelum<T>` pipeline. It keeps the pipeline node as a
//! **concrete generic type** instead of `Arc<dyn Nodus<Item = T>>`, so every
//! internal `visit_scalar` call is a direct (inlineable) call rather than a
//! vtable indirect call.
//!
//! # When to use the fast path
//!
//! - Hot pipelines that are built once and executed per element over large inputs.
//! - Pipelines that don't need to be stored behind a uniform `Arc<dyn>` type
//!   (e.g. in a heterogeneous collection).
//!
//! # Trade-offs
//!
//! - **Pros:** zero-vtable per-element cost; LLVM can inline through the whole
//!   chain and vectorize/unroll.
//! - **Cons:** each `.map` / `.filter` / `.scan` changes the return type, so
//!   the fast pipeline is harder to store in a field. Monomorphization
//!   multiplies concrete types across call sites, so deep pipelines cost
//!   compile time / code size.
//!
//! # Example
//!
//! ```rust
//! use ordofp_core::par::{FlumenParallelumFast, backend::CpuScalar};
//!
//! let data = vec![1, 2, 3, 4, 5];
//! let result = FlumenParallelumFast::from_vec(data)
//!     .map(|x| x * 2)
//!     .filter(|x| *x > 4)
//!     .collect_vec(&CpuScalar);
//! assert_eq!(result, vec![6, 8, 10]);
//! ```
//!
//! # Bridge to the dynamic path
//!
//! Call [`FlumenParallelumFast::into_dyn`] to convert a fast pipeline into a
//! regular [`super::FlumenParallelum`] (shareable, uniform type) once the
//! shape is fixed.

#![cfg(feature = "par")]

use alloc::vec;
use alloc::vec::Vec;
use core::marker::PhantomData;
use core::ops::ControlFlow;

use super::Nodus;
use super::backend::Backend;

// =============================================================================
// Fast-path node types (no Arc<dyn>, no Arc<dyn Fn>)
// =============================================================================

/// Source node holding owned data (fast path).
pub struct FastInit<T> {
    data: Vec<T>,
}

impl<T> Nodus for FastInit<T>
where
    T: Clone + Send + Sync + 'static,
{
    type Item = T;

    #[inline(always)]
    fn len(&self) -> usize {
        self.data.len()
    }

    #[inline(always)]
    fn visit_scalar(&self, sink: &mut dyn FnMut(Self::Item)) {
        for item in &self.data {
            sink(item.clone());
        }
    }

    #[inline(always)]
    fn visit_scalar_ref(&self, sink: &mut dyn FnMut(&Self::Item)) {
        // Own the storage — iterate by reference, zero clones. Without this
        // override FastInit falls back to the default (visit_scalar + clone),
        // cloning every source element and making the fast path SLOWER than
        // the dyn path (which has this override on NodusInit).
        for item in &self.data {
            sink(item);
        }
    }

    #[inline(always)]
    fn is_indexed(&self) -> bool {
        true
    }

    #[inline(always)]
    fn get(&self, index: usize) -> Self::Item {
        self.data[index].clone()
    }

    #[inline(always)]
    fn collect_scalar(&self) -> Vec<Self::Item> {
        self.data.clone()
    }

    #[inline(always)]
    fn try_visit_scalar_ref(&self, f: &mut dyn FnMut(&Self::Item) -> ControlFlow<()>) {
        for item in &self.data {
            if f(item).is_break() {
                return;
            }
        }
    }

    #[cfg(feature = "rayon")]
    #[inline]
    fn collect_rayon(&self) -> Vec<Self::Item> {
        use rayon::prelude::*;
        self.data.par_iter().cloned().collect()
    }

    #[inline]
    fn try_drain_vec(&mut self) -> Option<Vec<Self::Item>> {
        Some(core::mem::take(&mut self.data))
    }
}

/// Map node storing the previous node and closure by concrete type.
pub struct FastMap<P, F, A, B> {
    prev: P,
    f: F,
    _phantom: PhantomData<fn(A) -> B>,
}

impl<P, F, A, B> Nodus for FastMap<P, F, A, B>
where
    P: Nodus<Item = A> + Send + Sync,
    F: Fn(A) -> B + Send + Sync,
    A: Clone + Send + Sync + 'static,
    B: Clone + Send + Sync + 'static,
{
    type Item = B;

    #[inline(always)]
    fn len(&self) -> usize {
        self.prev.len()
    }

    #[inline(always)]
    fn visit_scalar(&self, sink: &mut dyn FnMut(Self::Item)) {
        let f = &self.f;
        self.prev.visit_scalar(&mut |a| sink(f(a)));
    }

    #[inline(always)]
    fn is_indexed(&self) -> bool {
        self.prev.is_indexed()
    }

    #[inline(always)]
    fn get(&self, index: usize) -> Self::Item {
        (self.f)(self.prev.get(index))
    }

    #[inline(always)]
    fn try_visit_scalar_ref(&self, f: &mut dyn FnMut(&Self::Item) -> ControlFlow<()>) {
        let func = &self.f;
        self.prev.try_visit_scalar_ref(&mut |a| {
            let b = func(a.clone());
            f(&b)
        });
    }
}

/// Filter node (fast path).
pub struct FastFilter<P, F, T> {
    prev: P,
    predicate: F,
    _phantom: PhantomData<fn(&T) -> bool>,
}

impl<P, F, T> Nodus for FastFilter<P, F, T>
where
    P: Nodus<Item = T> + Send + Sync,
    F: Fn(&T) -> bool + Send + Sync,
    T: Clone + Send + Sync + 'static,
{
    type Item = T;

    #[inline(always)]
    fn len(&self) -> usize {
        self.prev.len()
    }

    #[inline(always)]
    fn visit_scalar(&self, sink: &mut dyn FnMut(Self::Item)) {
        let predicate = &self.predicate;
        self.prev.visit_scalar(&mut |a| {
            if predicate(&a) {
                sink(a);
            }
        });
    }

    #[inline(always)]
    fn visit_scalar_ref(&self, sink: &mut dyn FnMut(&Self::Item)) {
        let predicate = &self.predicate;
        self.prev.visit_scalar_ref(&mut |a| {
            if predicate(a) {
                sink(a);
            }
        });
    }

    #[inline(always)]
    fn try_visit_scalar_ref(&self, f: &mut dyn FnMut(&Self::Item) -> ControlFlow<()>) {
        let predicate = &self.predicate;
        self.prev.try_visit_scalar_ref(&mut |a| {
            if predicate(a) {
                f(a)
            } else {
                ControlFlow::Continue(())
            }
        });
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.prev.len()))
    }

    #[inline]
    fn collect_scalar(&self) -> Vec<Self::Item> {
        // Start with at most 1/4 of upstream length, capped at 1024, with a
        // floor of 16. This avoids worst-case over-allocation while keeping
        // realloc count low for typical filter rates.
        let cap = (self.prev.len() / 4).clamp(16, 1024);
        let mut out = Vec::with_capacity(cap);
        self.visit_scalar_ref(&mut |x| out.push(x.clone()));
        out
    }
}

/// `FilterMap` node (fast path).
pub struct FastFilterMap<P, F, A, B> {
    prev: P,
    f: F,
    _phantom: PhantomData<fn(A) -> Option<B>>,
}

impl<P, F, A, B> Nodus for FastFilterMap<P, F, A, B>
where
    P: Nodus<Item = A> + Send + Sync,
    F: Fn(A) -> Option<B> + Send + Sync,
    A: Clone + Send + Sync + 'static,
    B: Clone + Send + Sync + 'static,
{
    type Item = B;

    #[inline(always)]
    fn len(&self) -> usize {
        self.prev.len()
    }

    #[inline(always)]
    fn visit_scalar(&self, sink: &mut dyn FnMut(Self::Item)) {
        let f = &self.f;
        self.prev.visit_scalar(&mut |a| {
            if let Some(b) = f(a) {
                sink(b);
            }
        });
    }

    #[inline(always)]
    fn try_visit_scalar_ref(&self, f: &mut dyn FnMut(&Self::Item) -> ControlFlow<()>) {
        let func = &self.f;
        self.prev.try_visit_scalar_ref(&mut |a| {
            if let Some(b) = func(a.clone()) {
                f(&b)
            } else {
                ControlFlow::Continue(())
            }
        });
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.prev.len()))
    }

    #[inline]
    fn collect_scalar(&self) -> Vec<Self::Item> {
        // Start with at most 1/4 of upstream length, capped at 1024, with a
        // floor of 16. This avoids worst-case over-allocation while keeping
        // realloc count low for typical filter rates.
        let cap = (self.prev.len() / 4).clamp(16, 1024);
        let mut out = Vec::with_capacity(cap);
        // Use visit_scalar (owned) rather than visit_scalar_ref + clone: each
        // matching element is produced once as an owned B and pushed directly,
        // saving one clone per surviving element vs. the ref path.
        self.visit_scalar(&mut |x| out.push(x));
        out
    }
}

/// Scan node (fast path).
pub struct FastScan<P, F, A, B> {
    prev: P,
    init: B,
    f: F,
    _phantom: PhantomData<fn(B, A) -> B>,
}

impl<P, F, A, B> Nodus for FastScan<P, F, A, B>
where
    P: Nodus<Item = A> + Send + Sync,
    F: Fn(B, A) -> B + Send + Sync,
    A: Clone + Send + Sync + 'static,
    B: Clone + Send + Sync + 'static,
{
    type Item = B;

    #[inline(always)]
    fn len(&self) -> usize {
        self.prev.len()
    }

    #[inline(always)]
    fn visit_scalar(&self, sink: &mut dyn FnMut(Self::Item)) {
        let f = &self.f;
        // Use Option<B> to move acc out to feed f, eliminating the extra
        // clone the naive `f(acc.clone(), a); sink(acc.clone())` approach
        // required. Cost: 1 clone per element (for sink) instead of 2.
        let mut acc: Option<B> = Some(self.init.clone());
        self.prev.visit_scalar(&mut |a| {
            let old = acc.take().unwrap();
            let next = f(old, a);
            sink(next.clone());
            acc = Some(next);
        });
    }

    #[inline(always)]
    fn visit_scalar_ref(&self, sink: &mut dyn FnMut(&Self::Item)) {
        let f = &self.f;
        // Same Option<B> pattern; sink receives &B (no extra clone of B).
        let mut acc: Option<B> = Some(self.init.clone());
        self.prev.visit_scalar(&mut |a| {
            let old = acc.take().unwrap();
            let next = f(old, a);
            acc = Some(next);
            sink(acc.as_ref().unwrap());
        });
    }
}

/// Take first N elements (fast path).
pub struct FastTake<P> {
    prev: P,
    count: usize,
}

impl<P, T> Nodus for FastTake<P>
where
    P: Nodus<Item = T> + Send + Sync,
    T: Clone + Send + Sync + 'static,
{
    type Item = T;

    #[inline(always)]
    fn len(&self) -> usize {
        self.prev.len().min(self.count)
    }

    #[inline(always)]
    fn visit_scalar(&self, sink: &mut dyn FnMut(Self::Item)) {
        if self.prev.is_indexed() {
            for i in 0..self.len() {
                sink(self.prev.get(i));
            }
            return;
        }
        let mut remaining = self.count;
        self.prev.visit_scalar(&mut |a| {
            if remaining > 0 {
                remaining -= 1;
                sink(a);
            }
        });
    }

    #[inline(always)]
    fn visit_scalar_ref(&self, sink: &mut dyn FnMut(&Self::Item)) {
        if self.prev.is_indexed() {
            for i in 0..self.len() {
                let item = self.prev.get(i);
                sink(&item);
            }
            return;
        }
        let mut remaining = self.count;
        self.prev.visit_scalar_ref(&mut |a| {
            if remaining > 0 {
                remaining -= 1;
                sink(a);
            }
        });
    }

    #[inline(always)]
    fn is_indexed(&self) -> bool {
        self.prev.is_indexed()
    }

    #[inline(always)]
    fn get(&self, index: usize) -> Self::Item {
        crate::unlikely_panic!(index >= self.count, "FastTake: index out of bounds");
        self.prev.get(index)
    }

    #[inline(always)]
    fn try_visit_scalar_ref(&self, f: &mut dyn FnMut(&Self::Item) -> ControlFlow<()>) {
        let mut remaining = self.count;
        self.prev.try_visit_scalar_ref(&mut |a| {
            if remaining == 0 {
                return ControlFlow::Break(());
            }
            remaining -= 1;
            f(a)
        });
    }
}

/// Skip first N elements (fast path).
pub struct FastSkip<P> {
    prev: P,
    count: usize,
}

impl<P, T> Nodus for FastSkip<P>
where
    P: Nodus<Item = T> + Send + Sync,
    T: Clone + Send + Sync + 'static,
{
    type Item = T;

    #[inline(always)]
    fn len(&self) -> usize {
        self.prev.len().saturating_sub(self.count)
    }

    #[inline(always)]
    fn visit_scalar(&self, sink: &mut dyn FnMut(Self::Item)) {
        if self.prev.is_indexed() {
            let len = self.len();
            for i in 0..len {
                sink(self.prev.get(i + self.count));
            }
            return;
        }
        let mut skipped = 0;
        let count = self.count;
        self.prev.visit_scalar(&mut |a| {
            if skipped >= count {
                sink(a);
            } else {
                skipped += 1;
            }
        });
    }

    #[inline(always)]
    fn visit_scalar_ref(&self, sink: &mut dyn FnMut(&Self::Item)) {
        if self.prev.is_indexed() {
            let len = self.len();
            for i in 0..len {
                let item = self.prev.get(i + self.count);
                sink(&item);
            }
            return;
        }
        let mut skipped = 0;
        let count = self.count;
        self.prev.visit_scalar_ref(&mut |a| {
            if skipped >= count {
                sink(a);
            } else {
                skipped += 1;
            }
        });
    }

    #[inline(always)]
    fn is_indexed(&self) -> bool {
        self.prev.is_indexed()
    }

    #[inline(always)]
    fn get(&self, index: usize) -> Self::Item {
        self.prev.get(index + self.count)
    }

    #[inline(always)]
    fn try_visit_scalar_ref(&self, f: &mut dyn FnMut(&Self::Item) -> ControlFlow<()>) {
        let mut skipped = 0;
        let count = self.count;
        self.prev.try_visit_scalar_ref(&mut |a| {
            if skipped >= count {
                f(a)
            } else {
                skipped += 1;
                ControlFlow::Continue(())
            }
        });
    }
}

/// Enumerate each element with its index (fast path).
pub struct FastEnumerate<P> {
    prev: P,
}

impl<P, T> Nodus for FastEnumerate<P>
where
    P: Nodus<Item = T> + Send + Sync,
    T: Clone + Send + Sync + 'static,
{
    type Item = (usize, T);

    #[inline(always)]
    fn len(&self) -> usize {
        self.prev.len()
    }

    #[inline(always)]
    fn visit_scalar(&self, sink: &mut dyn FnMut(Self::Item)) {
        let mut index = 0usize;
        self.prev.visit_scalar(&mut |a| {
            sink((index, a));
            index += 1;
        });
    }

    #[inline(always)]
    fn is_indexed(&self) -> bool {
        self.prev.is_indexed()
    }

    #[inline(always)]
    fn get(&self, index: usize) -> Self::Item {
        (index, self.prev.get(index))
    }

    #[inline(always)]
    fn try_visit_scalar_ref(&self, f: &mut dyn FnMut(&Self::Item) -> ControlFlow<()>) {
        let mut index = 0usize;
        self.prev.try_visit_scalar_ref(&mut |a| {
            let pair = (index, a.clone());
            index += 1;
            f(&pair)
        });
    }
}

/// Inspect each element (fast path).
pub struct FastInspect<P, F, T> {
    prev: P,
    f: F,
    _phantom: PhantomData<fn(&T)>,
}

impl<P, F, T> Nodus for FastInspect<P, F, T>
where
    P: Nodus<Item = T> + Send + Sync,
    F: Fn(&T) + Send + Sync,
    T: Clone + Send + Sync + 'static,
{
    type Item = T;

    #[inline(always)]
    fn len(&self) -> usize {
        self.prev.len()
    }

    #[inline(always)]
    fn visit_scalar(&self, sink: &mut dyn FnMut(Self::Item)) {
        let f = &self.f;
        self.prev.visit_scalar(&mut |a| {
            f(&a);
            sink(a);
        });
    }

    #[inline(always)]
    fn visit_scalar_ref(&self, sink: &mut dyn FnMut(&Self::Item)) {
        let f = &self.f;
        self.prev.visit_scalar_ref(&mut |a| {
            f(a);
            sink(a);
        });
    }

    #[inline(always)]
    fn is_indexed(&self) -> bool {
        self.prev.is_indexed()
    }

    #[inline(always)]
    fn get(&self, index: usize) -> Self::Item {
        let item = self.prev.get(index);
        (self.f)(&item);
        item
    }

    #[inline(always)]
    fn try_visit_scalar_ref(&self, f: &mut dyn FnMut(&Self::Item) -> ControlFlow<()>) {
        let inspect_f = &self.f;
        self.prev.try_visit_scalar_ref(&mut |a| {
            inspect_f(a);
            f(a)
        });
    }
}

// =============================================================================
// FlumenParallelumFast - monomorphic wrapper
// =============================================================================

/// Monomorphic, zero-vtable parallel pipeline.
///
/// Generic over the concrete node type `P` so every chained operation builds
/// up a larger concrete type and `visit_scalar` calls stay as static dispatch.
///
/// See module-level docs for guidance on when to prefer this over
/// [`super::FlumenParallelum`].
pub struct FlumenParallelumFast<P> {
    node: P,
}

impl<T> FlumenParallelumFast<FastInit<T>>
where
    T: Clone + Send + Sync + 'static,
{
    /// Build a fast-path pipeline from an owned vector.
    #[inline(always)]
    pub fn from_vec(vec: Vec<T>) -> Self {
        Self {
            node: FastInit { data: vec },
        }
    }

    /// Build a fast-path pipeline from a slice (clones the data).
    #[inline(always)]
    pub fn from_slice(slice: &[T]) -> Self {
        Self::from_vec(slice.to_vec())
    }

    /// Empty fast-path pipeline.
    #[inline(always)]
    pub fn empty() -> Self {
        Self::from_vec(Vec::new())
    }

    /// Singleton fast-path pipeline.
    #[inline(always)]
    pub fn singleton(value: T) -> Self {
        Self::from_vec(vec![value])
    }
}

impl<P> FlumenParallelumFast<P>
where
    P: Nodus + Send + Sync,
    P::Item: Clone + Send + Sync + 'static,
{
    /// Stream length.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.node.len()
    }

    /// Whether the stream is empty.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Borrow the underlying node.
    #[inline(always)]
    pub fn as_node(&self) -> &P {
        &self.node
    }

    /// Consume and return the underlying node.
    #[inline(always)]
    pub fn into_node(self) -> P {
        self.node
    }

    /// Map each element with `f`.
    #[inline(always)]
    pub fn map<B, F>(self, f: F) -> FlumenParallelumFast<FastMap<P, F, P::Item, B>>
    where
        F: Fn(P::Item) -> B + Send + Sync,
        B: Clone + Send + Sync + 'static,
    {
        FlumenParallelumFast {
            node: FastMap {
                prev: self.node,
                f,
                _phantom: PhantomData,
            },
        }
    }

    /// Filter elements matching `predicate`.
    #[inline(always)]
    pub fn filter<F>(self, predicate: F) -> FlumenParallelumFast<FastFilter<P, F, P::Item>>
    where
        F: Fn(&P::Item) -> bool + Send + Sync,
    {
        FlumenParallelumFast {
            node: FastFilter {
                prev: self.node,
                predicate,
                _phantom: PhantomData,
            },
        }
    }

    /// Map + filter in one pass.
    #[inline(always)]
    pub fn filter_map<B, F>(self, f: F) -> FlumenParallelumFast<FastFilterMap<P, F, P::Item, B>>
    where
        F: Fn(P::Item) -> Option<B> + Send + Sync,
        B: Clone + Send + Sync + 'static,
    {
        FlumenParallelumFast {
            node: FastFilterMap {
                prev: self.node,
                f,
                _phantom: PhantomData,
            },
        }
    }

    /// Prefix scan (cumulative fold).
    #[inline(always)]
    pub fn scan<B, F>(self, init: B, f: F) -> FlumenParallelumFast<FastScan<P, F, P::Item, B>>
    where
        F: Fn(B, P::Item) -> B + Send + Sync,
        B: Clone + Send + Sync + 'static,
    {
        FlumenParallelumFast {
            node: FastScan {
                prev: self.node,
                init,
                f,
                _phantom: PhantomData,
            },
        }
    }

    /// Take the first `n` elements.
    #[inline(always)]
    pub fn take(self, n: usize) -> FlumenParallelumFast<FastTake<P>> {
        FlumenParallelumFast {
            node: FastTake {
                prev: self.node,
                count: n,
            },
        }
    }

    /// Skip the first `n` elements.
    #[inline(always)]
    pub fn skip(self, n: usize) -> FlumenParallelumFast<FastSkip<P>> {
        FlumenParallelumFast {
            node: FastSkip {
                prev: self.node,
                count: n,
            },
        }
    }

    /// Pair each element with its index.
    #[inline(always)]
    pub fn enumerate(self) -> FlumenParallelumFast<FastEnumerate<P>> {
        FlumenParallelumFast {
            node: FastEnumerate { prev: self.node },
        }
    }

    /// Inspect each element without modifying it.
    #[inline(always)]
    pub fn inspect<F>(self, f: F) -> FlumenParallelumFast<FastInspect<P, F, P::Item>>
    where
        F: Fn(&P::Item) + Send + Sync,
    {
        FlumenParallelumFast {
            node: FastInspect {
                prev: self.node,
                f,
                _phantom: PhantomData,
            },
        }
    }

    /// Collect elements into a `Vec` using the given backend.
    #[inline(always)]
    pub fn collect_vec<Bk>(&self, backend: &Bk) -> Vec<P::Item>
    where
        Bk: Backend,
    {
        backend.collect(&self.node)
    }

    /// Reduce using an associative operation.
    #[inline(always)]
    pub fn reduce<Bk, F>(&self, backend: &Bk, f: F) -> Option<P::Item>
    where
        Bk: Backend,
        F: Fn(P::Item, P::Item) -> P::Item + Send + Sync,
    {
        backend.reduce(&self.node, f)
    }

    /// Fold with an initial value.
    #[inline(always)]
    pub fn fold<Bk, B, F>(&self, backend: &Bk, init: B, f: F) -> B
    where
        Bk: Backend,
        B: Clone + Send + Sync + 'static,
        F: Fn(B, P::Item) -> B + Send + Sync,
    {
        backend.fold(&self.node, init, f)
    }

    /// Side-effect for each element.
    #[inline(always)]
    pub fn for_each<Bk, F>(&self, backend: &Bk, f: F)
    where
        Bk: Backend,
        F: Fn(P::Item) + Send + Sync,
    {
        backend.for_each(&self.node, f);
    }

    /// Any element matching predicate?
    #[inline(always)]
    pub fn any<Bk, F>(&self, backend: &Bk, predicate: F) -> bool
    where
        Bk: Backend,
        F: Fn(&P::Item) -> bool + Send + Sync,
    {
        backend.any(&self.node, predicate)
    }

    /// All elements matching predicate?
    #[inline(always)]
    pub fn all<Bk, F>(&self, backend: &Bk, predicate: F) -> bool
    where
        Bk: Backend,
        F: Fn(&P::Item) -> bool + Send + Sync,
    {
        backend.all(&self.node, predicate)
    }

    /// First element matching predicate.
    #[inline(always)]
    pub fn find<Bk, F>(&self, backend: &Bk, predicate: F) -> Option<P::Item>
    where
        Bk: Backend,
        F: Fn(&P::Item) -> bool + Send + Sync,
    {
        backend.find(&self.node, predicate)
    }

    /// Count elements.
    #[inline(always)]
    pub fn count<Bk>(&self, backend: &Bk) -> usize
    where
        Bk: Backend,
    {
        backend.count(&self.node)
    }
}

// Bridge: fast -> dyn
impl<P> FlumenParallelumFast<P>
where
    P: Nodus + Send + Sync + 'static,
    P::Item: Clone + Send + Sync + 'static,
{
    /// Erase the concrete node type and return a shareable
    /// [`super::FlumenParallelum`] pipeline.
    ///
    /// Use this when you need uniform storage (e.g. a `Vec<FlumenParallelum<T>>`)
    /// or `Clone` semantics — the fast pipeline is not `Clone`.
    #[inline]
    pub fn into_dyn(self) -> super::FlumenParallelum<P::Item> {
        super::FlumenParallelum::__from_node(alloc::sync::Arc::new(self.node))
    }
}

// =============================================================================
// Monomorphization evidence hook
//
// This function is a public, non-inlined entry point that forces LLVM to emit
// a concrete specialisation of a known fast-path pipeline. Inspecting the
// generated code for this symbol shows that no vtable dispatch appears along
// the per-element visit path — the `visit_scalar` calls on `FastInit`,
// `FastMap`, `FastFilter` are statically resolved.
// =============================================================================

/// Evidence-hook: sum of `(x * 2)` for even `x` across a slice using the fast path.
///
/// Not part of the normal public API surface — exposed to allow inspection of
/// the monomorphised pipeline without a dev-dep on `criterion`. The
/// `#[inline(never)]` ensures the symbol survives codegen so tools (cargo-asm,
/// llvm-objdump, etc.) can find it.
#[doc(hidden)]
#[inline(never)]
pub fn __fastpath_evidence_sum_even_doubled_i64(xs: &[i64]) -> i64 {
    use super::backend::CpuScalar;
    FlumenParallelumFast::from_slice(xs)
        .filter(|x| x % 2 == 0)
        .map(|x| x * 2)
        .fold(&CpuScalar, 0i64, |acc, x| acc + x)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::super::backend::CpuScalar;
    use super::*;

    #[test]
    fn fast_from_vec_collect() {
        let v = vec![1i32, 2, 3, 4, 5];
        let got = FlumenParallelumFast::from_vec(v).collect_vec(&CpuScalar);
        assert_eq!(got, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn fast_from_slice_collect() {
        let v = [1i32, 2, 3];
        let got = FlumenParallelumFast::from_slice(&v).collect_vec(&CpuScalar);
        assert_eq!(got, vec![1, 2, 3]);
    }

    #[test]
    fn fast_map() {
        let got = FlumenParallelumFast::from_vec(vec![1, 2, 3])
            .map(|x| x * 2)
            .collect_vec(&CpuScalar);
        assert_eq!(got, vec![2, 4, 6]);
    }

    #[test]
    fn fast_filter() {
        let got = FlumenParallelumFast::from_vec(vec![1, 2, 3, 4, 5])
            .filter(|x| x % 2 == 0)
            .collect_vec(&CpuScalar);
        assert_eq!(got, vec![2, 4]);
    }

    #[test]
    fn fast_filter_map() {
        let got = FlumenParallelumFast::from_vec(vec![1, 2, 3, 4])
            .filter_map(|x| if x % 2 == 0 { Some(x * 10) } else { None })
            .collect_vec(&CpuScalar);
        assert_eq!(got, vec![20, 40]);
    }

    #[test]
    fn fast_scan() {
        let got = FlumenParallelumFast::from_vec(vec![1, 2, 3, 4])
            .scan(0, |acc, x| acc + x)
            .collect_vec(&CpuScalar);
        assert_eq!(got, vec![1, 3, 6, 10]);
    }

    #[test]
    fn fast_take_skip() {
        let got = FlumenParallelumFast::from_vec(vec![1, 2, 3, 4, 5])
            .skip(1)
            .take(3)
            .collect_vec(&CpuScalar);
        assert_eq!(got, vec![2, 3, 4]);
    }

    #[test]
    fn fast_enumerate() {
        let got = FlumenParallelumFast::from_vec(vec!["a", "b", "c"])
            .enumerate()
            .collect_vec(&CpuScalar);
        assert_eq!(got, vec![(0, "a"), (1, "b"), (2, "c")]);
    }

    #[test]
    fn fast_pipeline() {
        // map -> filter -> take -> enumerate, mirroring the dyn pipeline test
        let got = FlumenParallelumFast::from_vec(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10])
            .map(|x| x * 2)
            .filter(|x| *x > 6)
            .take(4)
            .enumerate()
            .collect_vec(&CpuScalar);
        assert_eq!(got, vec![(0, 8), (1, 10), (2, 12), (3, 14)]);
    }

    #[test]
    fn fast_reduce_fold_count() {
        let pipe = FlumenParallelumFast::from_vec(vec![1, 2, 3, 4, 5]);
        assert_eq!(pipe.count(&CpuScalar), 5);
        assert_eq!(pipe.reduce(&CpuScalar, |a, b| a + b), Some(15));
        assert_eq!(pipe.fold(&CpuScalar, 0, |a, x| a + x), 15);
    }

    #[test]
    fn fast_any_all_find() {
        let pipe = FlumenParallelumFast::from_vec(vec![1, 2, 3, 4, 5]);
        assert!(pipe.any(&CpuScalar, |x| *x > 3));
        assert!(!pipe.all(&CpuScalar, |x| *x > 3));
        assert_eq!(pipe.find(&CpuScalar, |x| *x > 3), Some(4));
    }

    #[test]
    fn fast_into_dyn_bridge() {
        let fast = FlumenParallelumFast::from_vec(vec![1, 2, 3])
            .map(|x| x + 10)
            .filter(|x| *x > 10);
        let dynamic = fast.into_dyn();
        assert_eq!(dynamic.collect_vec(&CpuScalar), vec![11, 12, 13]);
    }

    #[test]
    fn fast_empty_singleton() {
        let e: FlumenParallelumFast<FastInit<i32>> = FlumenParallelumFast::empty();
        assert!(e.is_empty());
        let s = FlumenParallelumFast::singleton(42);
        assert_eq!(s.len(), 1);
        assert_eq!(s.collect_vec(&CpuScalar), vec![42]);
    }
}
