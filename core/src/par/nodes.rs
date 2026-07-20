//! Nodus node types for the `ParFlumen` pipeline IR.
//!
//! Each node represents a lazy operation in the computation graph.

use alloc::sync::Arc;
use alloc::vec::Vec;
use core::ops::ControlFlow;

use super::Nodus;

// =============================================================================
// NodusInit - Source data
// =============================================================================

pub(crate) struct NodusInit<T> {
    pub(crate) data: Vec<T>,
}

impl<T> Nodus for NodusInit<T>
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
        // Own the storage — iterate by reference, zero clones.
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

// =============================================================================
// NodusMap - Map transformation
// =============================================================================

pub(crate) struct NodusMap<A, B> {
    pub(crate) prev: Arc<dyn Nodus<Item = A>>,
    pub(crate) f: Arc<dyn Fn(A) -> B + Send + Sync>,
}

impl<A, B> Nodus for NodusMap<A, B>
where
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
        // Pull references from prev; clone only to feed f (mandatory because
        // f: Fn(A) -> B takes A by value). Sink gets the owned b directly.
        let f = &self.f;
        self.prev.visit_scalar_ref(&mut |a| sink(f(a.clone())));
    }

    #[inline(always)]
    fn visit_scalar_ref(&self, sink: &mut dyn FnMut(&Self::Item)) {
        // Same as above, but forward a reference to b so downstream nodes
        // don't clone again.
        let f = &self.f;
        self.prev.visit_scalar_ref(&mut |a| {
            let b = f(a.clone());
            sink(&b);
        });
    }

    #[inline(always)]
    fn is_indexed(&self) -> bool {
        self.prev.is_indexed()
    }

    #[inline(always)]
    fn get(&self, index: usize) -> Self::Item {
        let a = self.prev.get(index);
        (self.f)(a)
    }

    #[inline]
    fn collect_scalar(&self) -> Vec<Self::Item> {
        if self.is_indexed() {
            let len = self.len();
            let mut out = Vec::with_capacity(len);
            for i in 0..len {
                let a = self.prev.get(i);
                out.push((self.f)(a));
            }
            out
        } else {
            let mut out = Vec::with_capacity(self.len());
            self.visit_scalar_ref(&mut |x| out.push(x.clone()));
            out
        }
    }

    #[inline(always)]
    fn try_visit_scalar_ref(&self, f: &mut dyn FnMut(&Self::Item) -> ControlFlow<()>) {
        let func = &self.f;
        self.prev.try_visit_scalar_ref(&mut |a| {
            let b = func(a.clone());
            f(&b)
        });
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
            // Non-indexed upstream (e.g. after a filter): collect upstream in
            // parallel — NodusFilter::collect_rayon is itself parallel when its
            // upstream is indexed — then apply the map in parallel.
            let f = &self.f;
            self.prev
                .collect_rayon()
                .into_par_iter()
                .map(|a| f(a))
                .collect()
        }
    }

    #[cfg(feature = "rayon")]
    #[inline]
    fn reduce_rayon(
        &self,
        g: &(dyn Fn(Self::Item, Self::Item) -> Self::Item + Send + Sync),
    ) -> Option<Self::Item> {
        use rayon::prelude::*;
        if self.is_indexed() {
            (0..self.len())
                .into_par_iter()
                .map(|i| self.get(i))
                .reduce_with(g)
        } else {
            // Fuse map into the reduce: collect only the (non-indexed) upstream
            // in parallel, then map+reduce in a single parallel pass. Avoids the
            // throwaway intermediate Vec<B> that the default
            // `collect_rayon().into_par_iter().reduce_with()` would materialize.
            let f = &self.f;
            self.prev
                .collect_rayon()
                .into_par_iter()
                .map(|a| f(a))
                .reduce_with(g)
        }
    }

    #[cfg(feature = "rayon")]
    #[inline]
    fn for_each_rayon(&self, g: &(dyn Fn(Self::Item) + Send + Sync)) {
        use rayon::prelude::*;
        if self.is_indexed() {
            (0..self.len()).into_par_iter().for_each(|i| g(self.get(i)));
        } else {
            // Fuse map into the parallel for_each: collect only the non-indexed
            // upstream in parallel, then map+for_each in a single pass. Avoids
            // the throwaway intermediate Vec<B> that the default
            // `collect_rayon().into_par_iter().for_each()` would materialize.
            let f = &self.f;
            self.prev
                .collect_rayon()
                .into_par_iter()
                .for_each(|a| g(f(a)));
        }
    }
}

// =============================================================================
// NodusScan - Prefix scan (cumulative fold)
// =============================================================================

pub(crate) struct NodusScan<A, B> {
    pub(crate) prev: Arc<dyn Nodus<Item = A>>,
    pub(crate) init: B,
    pub(crate) f: Arc<dyn Fn(B, A) -> B + Send + Sync>,
}

impl<A, B> Nodus for NodusScan<A, B>
where
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
        // Owned sink wants B by value. Use Option<B> to move the accumulator
        // out to feed f without cloning, then move the result back in. Sink
        // still receives an owned value (one clone of the final acc). Down
        // from 3 clones per element (old impl) to 1.
        let f = &self.f;
        let mut acc: Option<B> = Some(self.init.clone());
        self.prev.visit_scalar_ref(&mut |a| {
            // SAFETY-ish: `acc` is Some on entry (initialized above, and
            // restored to Some on every iteration). unwrap is the invariant.
            let old = acc.take().unwrap();
            let next = f(old, a.clone());
            sink(next.clone());
            acc = Some(next);
        });
    }

    #[inline(always)]
    fn visit_scalar_ref(&self, sink: &mut dyn FnMut(&Self::Item)) {
        let f = &self.f;
        let mut acc: Option<B> = Some(self.init.clone());
        self.prev.visit_scalar_ref(&mut |a| {
            let old = acc.take().unwrap();
            let next = f(old, a.clone());
            acc = Some(next);
            sink(acc.as_ref().unwrap());
        });
    }

    #[inline(always)]
    fn try_visit_scalar_ref(&self, f: &mut dyn FnMut(&Self::Item) -> ControlFlow<()>) {
        let func = &self.f;
        let mut acc: Option<B> = Some(self.init.clone());
        self.prev.try_visit_scalar_ref(&mut |a| {
            let old = acc.take().unwrap();
            let next = func(old, a.clone());
            acc = Some(next);
            f(acc.as_ref().unwrap())
        });
    }
}

// =============================================================================
// NodusFilter - Filter elements by predicate
// =============================================================================

pub(crate) struct NodusFilter<T> {
    pub(crate) prev: Arc<dyn Nodus<Item = T>>,
    pub(crate) predicate: Arc<dyn Fn(&T) -> bool + Send + Sync>,
}

impl<T> Nodus for NodusFilter<T>
where
    T: Clone + Send + Sync + 'static,
{
    type Item = T;

    #[inline(always)]
    fn len(&self) -> usize {
        // Filter length is not known statically; use worst case
        self.prev.len()
    }

    #[inline(always)]
    fn visit_scalar(&self, sink: &mut dyn FnMut(Self::Item)) {
        // Owned sink: must clone on match.
        let predicate = &self.predicate;
        self.prev.visit_scalar_ref(&mut |a| {
            if predicate(a) {
                sink(a.clone());
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

    #[cfg(feature = "rayon")]
    #[inline]
    fn collect_rayon(&self) -> Vec<Self::Item> {
        use rayon::prelude::*;
        let predicate = &self.predicate;
        if self.prev.is_indexed() {
            (0..self.prev.len())
                .into_par_iter()
                .filter_map(|i| {
                    let item = self.prev.get(i);
                    if predicate(&item) { Some(item) } else { None }
                })
                .collect()
        } else {
            self.collect_scalar()
        }
    }

    #[cfg(feature = "rayon")]
    #[inline]
    fn reduce_rayon(
        &self,
        f: &(dyn Fn(Self::Item, Self::Item) -> Self::Item + Send + Sync),
    ) -> Option<Self::Item> {
        use rayon::prelude::*;
        if self.prev.is_indexed() {
            // Fuse the predicate into the parallel reduce: filter by index and
            // tree-reduce in one pass, skipping the intermediate Vec that
            // collect_rayon would materialize.
            let predicate = &self.predicate;
            (0..self.prev.len())
                .into_par_iter()
                .filter_map(|i| {
                    let item = self.prev.get(i);
                    if predicate(&item) { Some(item) } else { None }
                })
                .reduce_with(f)
        } else {
            self.collect_rayon().into_par_iter().reduce_with(f)
        }
    }

    #[cfg(feature = "rayon")]
    #[inline]
    fn for_each_rayon(&self, g: &(dyn Fn(Self::Item) + Send + Sync)) {
        use rayon::prelude::*;
        if self.prev.is_indexed() {
            // Fuse the predicate into the parallel for_each (one pass, no Vec).
            let predicate = &self.predicate;
            (0..self.prev.len()).into_par_iter().for_each(|i| {
                let item = self.prev.get(i);
                if predicate(&item) {
                    g(item);
                }
            });
        } else {
            self.collect_rayon().into_par_iter().for_each(g);
        }
    }
}

// =============================================================================
// NodusFilterMap - Filter and map in one pass
// =============================================================================

pub(crate) struct NodusFilterMap<A, B> {
    pub(crate) prev: Arc<dyn Nodus<Item = A>>,
    pub(crate) f: Arc<dyn Fn(A) -> Option<B> + Send + Sync>,
}

impl<A, B> Nodus for NodusFilterMap<A, B>
where
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
        // f takes A by value — unavoidable clone on input.
        let f = &self.f;
        self.prev.visit_scalar_ref(&mut |a| {
            if let Some(b) = f(a.clone()) {
                sink(b);
            }
        });
    }

    #[inline(always)]
    fn visit_scalar_ref(&self, sink: &mut dyn FnMut(&Self::Item)) {
        let f = &self.f;
        self.prev.visit_scalar_ref(&mut |a| {
            if let Some(b) = f(a.clone()) {
                sink(&b);
            }
        });
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.prev.len()))
    }

    #[inline]
    fn collect_scalar(&self) -> Vec<Self::Item> {
        if self.prev.is_indexed() {
            let len = self.prev.len();
            // Start with at most 1/4 of upstream length, capped at 1024, with a
            // floor of 16. This avoids worst-case over-allocation while keeping
            // realloc count low for typical filter rates.
            let cap = (len / 4).clamp(16, 1024);
            let mut out = Vec::with_capacity(cap);
            let f = &self.f;
            for i in 0..len {
                if let Some(b) = f(self.prev.get(i)) {
                    out.push(b);
                }
            }
            out
        } else {
            let cap = (self.prev.len() / 4).clamp(16, 1024);
            let mut out = Vec::with_capacity(cap);
            self.visit_scalar_ref(&mut |x| out.push(x.clone()));
            out
        }
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

    #[cfg(feature = "rayon")]
    #[inline]
    fn collect_rayon(&self) -> Vec<Self::Item> {
        use rayon::prelude::*;
        let f = &self.f;
        if self.prev.is_indexed() {
            (0..self.prev.len())
                .into_par_iter()
                .filter_map(|i| f(self.prev.get(i)))
                .collect()
        } else {
            self.collect_scalar()
        }
    }

    #[cfg(feature = "rayon")]
    #[inline]
    fn reduce_rayon(
        &self,
        g: &(dyn Fn(Self::Item, Self::Item) -> Self::Item + Send + Sync),
    ) -> Option<Self::Item> {
        use rayon::prelude::*;
        if self.prev.is_indexed() {
            // Fuse the filter-map into the parallel reduce (one pass, no Vec).
            let f = &self.f;
            (0..self.prev.len())
                .into_par_iter()
                .filter_map(|i| f(self.prev.get(i)))
                .reduce_with(g)
        } else {
            self.collect_rayon().into_par_iter().reduce_with(g)
        }
    }

    #[cfg(feature = "rayon")]
    #[inline]
    fn for_each_rayon(&self, g: &(dyn Fn(Self::Item) + Send + Sync)) {
        use rayon::prelude::*;
        if self.prev.is_indexed() {
            // Fuse the filter-map into the parallel for_each (one pass, no Vec).
            let f = &self.f;
            (0..self.prev.len()).into_par_iter().for_each(|i| {
                if let Some(b) = f(self.prev.get(i)) {
                    g(b);
                }
            });
        } else {
            self.collect_rayon().into_par_iter().for_each(g);
        }
    }
}

// =============================================================================
// NodusTake - Take first n elements
// =============================================================================

pub(crate) struct NodusTake<T> {
    pub(crate) prev: Arc<dyn Nodus<Item = T>>,
    pub(crate) count: usize,
}

impl<T> Nodus for NodusTake<T>
where
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
        self.prev.visit_scalar_ref(&mut |a| {
            if remaining > 0 {
                remaining -= 1;
                sink(a.clone());
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
        crate::unlikely_panic!(index >= self.count, "NodusTake: index out of bounds");
        self.prev.get(index)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.len(); // already computes min(prev.len(), count)
        (n, Some(n))
    }

    #[inline]
    fn collect_scalar(&self) -> Vec<Self::Item> {
        if self.is_indexed() {
            let len = self.len();
            let mut out = Vec::with_capacity(len);
            for i in 0..len {
                out.push(self.prev.get(i));
            }
            out
        } else {
            let mut out = Vec::with_capacity(self.len());
            self.visit_scalar_ref(&mut |x| out.push(x.clone()));
            out
        }
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

// =============================================================================
// NodusSkip - Skip first n elements
// =============================================================================

pub(crate) struct NodusSkip<T> {
    pub(crate) prev: Arc<dyn Nodus<Item = T>>,
    pub(crate) count: usize,
}

impl<T> Nodus for NodusSkip<T>
where
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
        self.prev.visit_scalar_ref(&mut |a| {
            if skipped >= count {
                sink(a.clone());
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

    #[inline]
    fn collect_scalar(&self) -> Vec<Self::Item> {
        if self.is_indexed() {
            let len = self.len();
            let mut out = Vec::with_capacity(len);
            for i in 0..len {
                out.push(self.prev.get(i + self.count));
            }
            out
        } else {
            let mut out = Vec::with_capacity(self.len());
            self.visit_scalar_ref(&mut |x| out.push(x.clone()));
            out
        }
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

// =============================================================================
// NodusEnumerate - Pair elements with their indices
// =============================================================================

pub(crate) struct NodusEnumerate<T> {
    pub(crate) prev: Arc<dyn Nodus<Item = T>>,
}

impl<T> Nodus for NodusEnumerate<T>
where
    T: Clone + Send + Sync + 'static,
{
    type Item = (usize, T);

    #[inline(always)]
    fn len(&self) -> usize {
        self.prev.len()
    }

    #[inline(always)]
    fn visit_scalar(&self, sink: &mut dyn FnMut(Self::Item)) {
        // Owned sink wants (usize, T); clone once to build the pair.
        let mut index = 0usize;
        self.prev.visit_scalar_ref(&mut |a| {
            sink((index, a.clone()));
            index += 1;
        });
    }

    #[inline(always)]
    fn visit_scalar_ref(&self, sink: &mut dyn FnMut(&Self::Item)) {
        // The pair itself must be owned to take its address; still only
        // one clone of T per element.
        let mut index = 0usize;
        self.prev.visit_scalar_ref(&mut |a| {
            let pair = (index, a.clone());
            sink(&pair);
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

    #[inline]
    fn collect_scalar(&self) -> Vec<Self::Item> {
        if self.is_indexed() {
            let len = self.len();
            let mut out = Vec::with_capacity(len);
            for i in 0..len {
                out.push((i, self.prev.get(i)));
            }
            out
        } else {
            let mut out = Vec::with_capacity(self.len());
            self.visit_scalar_ref(&mut |x| out.push(x.clone()));
            out
        }
    }

    #[cfg(feature = "rayon")]
    #[inline]
    fn collect_rayon(&self) -> Vec<Self::Item> {
        use rayon::prelude::*;
        if self.prev.is_indexed() {
            let len = self.prev.len();
            (0..len)
                .into_par_iter()
                .map(|i| (i, self.prev.get(i)))
                .collect()
        } else {
            self.prev.collect_rayon().into_iter().enumerate().collect()
        }
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

// =============================================================================
// NodusInspect - Inspect elements without modifying
// =============================================================================

pub(crate) struct NodusInspect<T> {
    pub(crate) prev: Arc<dyn Nodus<Item = T>>,
    pub(crate) f: Arc<dyn Fn(&T) + Send + Sync>,
}

impl<T> Nodus for NodusInspect<T>
where
    T: Clone + Send + Sync + 'static,
{
    type Item = T;

    #[inline(always)]
    fn len(&self) -> usize {
        self.prev.len()
    }

    #[inline(always)]
    fn visit_scalar(&self, sink: &mut dyn FnMut(Self::Item)) {
        // Owned sink: clone after inspection.
        let f = &self.f;
        self.prev.visit_scalar_ref(&mut |a| {
            f(a);
            sink(a.clone());
        });
    }

    #[inline(always)]
    fn visit_scalar_ref(&self, sink: &mut dyn FnMut(&Self::Item)) {
        // Zero clones: inspect then forward the reference.
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

    #[inline]
    fn collect_scalar(&self) -> Vec<Self::Item> {
        if self.is_indexed() {
            let len = self.len();
            let mut out = Vec::with_capacity(len);
            for i in 0..len {
                let item = self.prev.get(i);
                (self.f)(&item);
                out.push(item);
            }
            out
        } else {
            let mut out = Vec::with_capacity(self.len());
            self.visit_scalar_ref(&mut |x| out.push(x.clone()));
            out
        }
    }

    #[cfg(feature = "rayon")]
    #[inline]
    fn collect_rayon(&self) -> Vec<Self::Item> {
        // Parallelise the upstream collection; then call f sequentially in
        // order so that inspection side-effects are deterministically ordered.
        let items = self.prev.collect_rayon();
        for x in &items {
            (self.f)(x);
        }
        items
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
// NodusChain - Chain two streams together
// =============================================================================

pub(crate) struct NodusChain<T> {
    pub(crate) first: Arc<dyn Nodus<Item = T>>,
    pub(crate) second: Arc<dyn Nodus<Item = T>>,
}

impl<T> Nodus for NodusChain<T>
where
    T: Clone + Send + Sync + 'static,
{
    type Item = T;

    #[inline(always)]
    fn len(&self) -> usize {
        self.first.len() + self.second.len()
    }

    #[inline(always)]
    fn visit_scalar(&self, sink: &mut dyn FnMut(Self::Item)) {
        self.first.visit_scalar(sink);
        self.second.visit_scalar(sink);
    }

    #[inline(always)]
    fn visit_scalar_ref(&self, sink: &mut dyn FnMut(&Self::Item)) {
        self.first.visit_scalar_ref(sink);
        self.second.visit_scalar_ref(sink);
    }

    #[inline(always)]
    fn is_indexed(&self) -> bool {
        self.first.is_indexed() && self.second.is_indexed()
    }

    #[inline(always)]
    fn get(&self, index: usize) -> Self::Item {
        let first_len = self.first.len();
        if index < first_len {
            self.first.get(index)
        } else {
            self.second.get(index - first_len)
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lo1, hi1) = self.first.size_hint();
        let (lo2, hi2) = self.second.size_hint();
        let hi = hi1.zip(hi2).map(|(h1, h2)| h1.saturating_add(h2));
        (lo1.saturating_add(lo2), hi)
    }

    #[inline(always)]
    fn try_visit_scalar_ref(&self, f: &mut dyn FnMut(&Self::Item) -> ControlFlow<()>) {
        let mut broken = false;
        self.first.try_visit_scalar_ref(&mut |a| {
            let cf = f(a);
            if cf.is_break() {
                broken = true;
            }
            cf
        });
        if !broken {
            self.second.try_visit_scalar_ref(f);
        }
    }

    #[cfg(feature = "rayon")]
    #[inline]
    fn collect_rayon(&self) -> Vec<Self::Item> {
        // Collect both halves in parallel, then concatenate. Each half may
        // itself use a parallel strategy (e.g. NodusFilter::collect_rayon).
        let (mut first, second) = rayon::join(
            || self.first.collect_rayon(),
            || self.second.collect_rayon(),
        );
        first.extend(second);
        first
    }

    #[cfg(feature = "rayon")]
    #[inline]
    fn reduce_rayon(
        &self,
        f: &(dyn Fn(Self::Item, Self::Item) -> Self::Item + Send + Sync),
    ) -> Option<Self::Item> {
        // Reduce each half in parallel (each via its own fused terminal), then
        // combine the two partial results. `reduce` is associative and
        // `reduce_with` is order-preserving, so this equals reducing the
        // concatenation — without materializing/concatenating it.
        let (a, b) = rayon::join(
            || self.first.reduce_rayon(f),
            || self.second.reduce_rayon(f),
        );
        match (a, b) {
            (Some(a), Some(b)) => Some(f(a, b)),
            (Some(a), None) => Some(a),
            (None, b) => b,
        }
    }

    #[cfg(feature = "rayon")]
    #[inline]
    fn for_each_rayon(&self, g: &(dyn Fn(Self::Item) + Send + Sync)) {
        // Run both halves' fused for_each in parallel; no concatenated Vec.
        rayon::join(
            || self.first.for_each_rayon(g),
            || self.second.for_each_rayon(g),
        );
    }
}

// =============================================================================
// NodusZip - Zip two streams together
// =============================================================================

pub(crate) struct NodusZip<A, B> {
    pub(crate) first: Arc<dyn Nodus<Item = A>>,
    pub(crate) second: Arc<dyn Nodus<Item = B>>,
}

impl<A, B> Nodus for NodusZip<A, B>
where
    A: Clone + Send + Sync + 'static,
    B: Clone + Send + Sync + 'static,
{
    type Item = (A, B);

    #[inline(always)]
    fn len(&self) -> usize {
        self.first.len().min(self.second.len())
    }

    #[inline(always)]
    fn visit_scalar(&self, sink: &mut dyn FnMut(Self::Item)) {
        // Optimization: Avoid materializing both streams.
        // We materialize the indexed stream (which usually has fast `collect_scalar`, e.g., memcpy)
        // and stream the unindexed one (which would be slow to collect due to frequent allocations).

        // Note: If both were indexed, `collect_scalar` would use the manual loop path,
        // so we wouldn't be here. Thus, at least one is not indexed.

        if self.second.is_indexed() {
            // Second is indexed, first is not. Materialize second.
            let second_items = self.second.collect_scalar();
            let mut second_iter = second_items.into_iter();
            self.first.visit_scalar_ref(&mut |a| {
                if let Some(b) = second_iter.next() {
                    sink((a.clone(), b));
                }
            });
        } else {
            // First is indexed (and second is not), or neither is indexed.
            // Materialize first.
            let first_items = self.first.collect_scalar();
            let mut first_iter = first_items.into_iter();
            self.second.visit_scalar_ref(&mut |b| {
                if let Some(a) = first_iter.next() {
                    sink((a, b.clone()));
                }
            });
        }
    }

    #[inline(always)]
    fn visit_scalar_ref(&self, sink: &mut dyn FnMut(&Self::Item)) {
        // Same strategy as visit_scalar but pass the constructed pair by ref.
        if self.second.is_indexed() {
            let second_items = self.second.collect_scalar();
            let mut second_iter = second_items.into_iter();
            self.first.visit_scalar_ref(&mut |a| {
                if let Some(b) = second_iter.next() {
                    let pair = (a.clone(), b);
                    sink(&pair);
                }
            });
        } else {
            let first_items = self.first.collect_scalar();
            let mut first_iter = first_items.into_iter();
            self.second.visit_scalar_ref(&mut |b| {
                if let Some(a) = first_iter.next() {
                    let pair = (a, b.clone());
                    sink(&pair);
                }
            });
        }
    }

    #[inline(always)]
    fn is_indexed(&self) -> bool {
        self.first.is_indexed() && self.second.is_indexed()
    }

    #[inline(always)]
    fn get(&self, index: usize) -> Self::Item {
        (self.first.get(index), self.second.get(index))
    }

    #[inline]
    fn collect_scalar(&self) -> Vec<Self::Item> {
        if self.is_indexed() {
            let len = self.len();
            let mut out = Vec::with_capacity(len);
            for i in 0..len {
                // Optimization: bypass dynamic dispatch on NodusZip::get
                out.push((self.first.get(i), self.second.get(i)));
            }
            out
        } else {
            let mut out = Vec::with_capacity(self.len());
            self.visit_scalar_ref(&mut |x| out.push(x.clone()));
            out
        }
    }

    #[cfg(feature = "rayon")]
    #[inline]
    fn collect_rayon(&self) -> Vec<Self::Item> {
        use rayon::prelude::*;
        if self.is_indexed() {
            let len = self.len();
            (0..len)
                .into_par_iter()
                .map(|i| (self.first.get(i), self.second.get(i)))
                .collect()
        } else {
            // Collect both sides in parallel (each may use its own parallel
            // strategy), then zip sequentially to preserve pairing order.
            let (first_items, second_items) = rayon::join(
                || self.first.collect_rayon(),
                || self.second.collect_rayon(),
            );
            first_items.into_iter().zip(second_items).collect()
        }
    }

    #[inline]
    fn try_visit_scalar_ref(&self, f: &mut dyn FnMut(&Self::Item) -> ControlFlow<()>) {
        if self.is_indexed() {
            // Both sides support get(i) — iterate with direct index access and
            // break as soon as f signals Break.
            for i in 0..self.len() {
                let pair = (self.first.get(i), self.second.get(i));
                if f(&pair).is_break() {
                    return;
                }
            }
            return;
        }
        // Non-indexed: materialize the indexed side (or first if neither),
        // then iterate the other side with early-exit support.
        if self.second.is_indexed() {
            let second_items = self.second.collect_scalar();
            let mut second_iter = second_items.into_iter();
            self.first
                .try_visit_scalar_ref(&mut |a| match second_iter.next() {
                    Some(b) => {
                        let pair = (a.clone(), b);
                        f(&pair)
                    }
                    None => ControlFlow::Break(()),
                });
        } else {
            let first_items = self.first.collect_scalar();
            let mut first_iter = first_items.into_iter();
            self.second
                .try_visit_scalar_ref(&mut |b| match first_iter.next() {
                    Some(a) => {
                        let pair = (a, b.clone());
                        f(&pair)
                    }
                    None => ControlFlow::Break(()),
                });
        }
    }
}
