//! `FlumenFusus` - Fused async stream state machine (Step/Stream style)
//!
//! This module ports the classic `Step`/`Stream` fusion representation
//! (`Step s a = Yield a s | Skip s | Done`; `Stream m a = ∃s. (s -> m (Step s a), s)`).
//! Adapted third-party patterns are inventoried in the repo-root
//! `THIRD_PARTY_NOTICES.md`.
//!
//! Rust adaptation:
//! - `Stream m a` becomes `FlumenFusus<S, StepFn, A>` where `StepFn` is a `FnMut`
//!   from `(state, Context)` to `Poll<Gradus<A>>`.
//! - `m` is the async poll monad (`Poll`), with `Skip` modeled explicitly to enable fusion.

#![cfg(feature = "fusion")]

use alloc::vec::Vec;
use core::future::poll_fn;
use core::pin::Pin;
use core::task::{Context, Poll};

use futures_core::Stream;

/// Result of taking a single step in a fused stream.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Gradus<A> {
    /// Produce an item.
    Yield(A),
    /// Produce no item, but keep going (fusion “Skip”).
    Skip,
    /// End of stream.
    Done,
}

/// Trait alias for the step function of a fused stream: advances state `S`
/// and produces `Poll<Gradus<A>>`.
///
/// Blanket-implemented for every matching closure; exists so combinators can
/// name their opaque return types without spelling out the nested `Fn` sugar.
pub trait GradusStep<S, A>:
    for<'a> FnMut(&mut S, &mut Context<'a>) -> Poll<Gradus<A>> + Unpin
{
}
impl<S, A, T> GradusStep<S, A> for T where
    T: for<'a> FnMut(&mut S, &mut Context<'a>) -> Poll<Gradus<A>> + Unpin
{
}

/// State carried by the [`FlumenFusus::chunks`] adapter: the inner stream
/// state, the chunk size, the in-progress buffer, and a "source done" flag.
pub type ChunksState<S, A> = (S, usize, Vec<A>, bool);

/// Step shape of [`FlumenFusus::scan`]: drives state `(S, Option<B>)`
/// (inner state plus accumulator slot) and yields accumulator values `B`.
pub trait ScanStep<S, B>: GradusStep<(S, Option<B>), B> {}
impl<S, B, T: GradusStep<(S, Option<B>), B>> ScanStep<S, B> for T {}

/// Step shape of [`FlumenFusus::scan_with`]: drives state `(S, St, bool)`
/// (inner state, user state, done flag) and yields values `B`.
pub trait ScanWithStep<S, St, B>: GradusStep<(S, St, bool), B> {}
impl<S, St, B, T: GradusStep<(S, St, bool), B>> ScanWithStep<S, St, B> for T {}

/// Step shape of [`FlumenFusus::enumerate`]: drives state `(S, usize)`
/// (inner state plus counter) and yields indexed items `(usize, A)`.
pub trait EnumerateStep<S, A>: GradusStep<(S, usize), (usize, A)> {}
impl<S, A, T: GradusStep<(S, usize), (usize, A)>> EnumerateStep<S, A> for T {}

/// Step shape of [`FlumenFusus::chunks`]: drives a [`ChunksState`] and
/// yields completed batches `Vec<A>`.
pub trait ChunksStep<S, A>: GradusStep<ChunksState<S, A>, Vec<A>> {}
impl<S, A, T: GradusStep<ChunksState<S, A>, Vec<A>>> ChunksStep<S, A> for T {}

/// Step shape of [`FlumenFusus::chain`]: drives state `(S, S2, bool)`
/// (both inner states plus a "first exhausted" flag) and yields items `A`.
pub trait ChainStep<S, S2, A>: GradusStep<(S, S2, bool), A> {}
impl<S, S2, A, T: GradusStep<(S, S2, bool), A>> ChainStep<S, S2, A> for T {}

/// Step shape of [`FlumenFusus::zip`]: drives paired states plus an `Option<A>`
/// stash (an item from the first stream awaiting its pair) and yields paired
/// items `(A, B)`.
pub trait ZipStep<S, S2, A, B>: GradusStep<(S, S2, Option<A>), (A, B)> {}
impl<S, S2, A, B, T: GradusStep<(S, S2, Option<A>), (A, B)>> ZipStep<S, S2, A, B> for T {}

/// The fused stream returned by [`FlumenFusus::zip`]: paired states `(S, S2)`
/// plus an `Option<A>` stash, yielding paired items `(A, B)`.
pub type ZipFusus<S, S2, Step, A, B> = FlumenFusus<(S, S2, Option<A>), Step, (A, B)>;

/// A fused stream represented as an explicit step-function and state.
///
/// This mirrors `Stream m a = forall s. Stream (s -> m (Step s a)) s` by storing
/// the state `S` and a `step` function that advances that state.
pub struct FlumenFusus<S, StepFn, A> {
    state: S,
    step: StepFn,
    _phantom: core::marker::PhantomData<fn() -> A>,
}

impl<S, StepFn, A> FlumenFusus<S, StepFn, A> {
    /// Construct a fused stream from state and a step function.
    #[inline(always)]
    pub fn new(state: S, step: StepFn) -> Self {
        Self {
            state,
            step,
            _phantom: core::marker::PhantomData,
        }
    }

    /// Map over yielded items (fused).
    #[inline(always)]
    pub fn map<B, F>(
        self,
        mut f: F,
    ) -> FlumenFusus<S, impl for<'a> FnMut(&mut S, &mut Context<'a>) -> Poll<Gradus<B>> + Unpin, B>
    where
        S: Unpin,
        StepFn: for<'a> FnMut(&mut S, &mut Context<'a>) -> Poll<Gradus<A>> + Unpin,
        F: FnMut(A) -> B + Unpin,
    {
        let mut step0 = self.step;
        FlumenFusus::new(
            self.state,
            move |s: &mut S, cx: &mut Context<'_>| match step0(s, cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(Gradus::Yield(a)) => Poll::Ready(Gradus::Yield(f(a))),
                Poll::Ready(Gradus::Skip) => Poll::Ready(Gradus::Skip),
                Poll::Ready(Gradus::Done) => Poll::Ready(Gradus::Done),
            },
        )
    }

    /// Filter yielded items (fused; uses `Skip`).
    #[inline(always)]
    pub fn filter<F>(
        self,
        mut predicate: F,
    ) -> FlumenFusus<S, impl for<'a> FnMut(&mut S, &mut Context<'a>) -> Poll<Gradus<A>> + Unpin, A>
    where
        S: Unpin,
        StepFn: for<'a> FnMut(&mut S, &mut Context<'a>) -> Poll<Gradus<A>> + Unpin,
        F: FnMut(&A) -> bool + Unpin,
    {
        let mut step0 = self.step;
        FlumenFusus::new(
            self.state,
            move |s: &mut S, cx: &mut Context<'_>| match step0(s, cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(Gradus::Yield(a)) => {
                    if predicate(&a) {
                        Poll::Ready(Gradus::Yield(a))
                    } else {
                        Poll::Ready(Gradus::Skip)
                    }
                }
                Poll::Ready(Gradus::Skip) => Poll::Ready(Gradus::Skip),
                Poll::Ready(Gradus::Done) => Poll::Ready(Gradus::Done),
            },
        )
    }

    /// Filter-map yielded items (fused; uses `Skip`).
    #[inline(always)]
    pub fn filter_map<B, F>(
        self,
        mut f: F,
    ) -> FlumenFusus<S, impl for<'a> FnMut(&mut S, &mut Context<'a>) -> Poll<Gradus<B>> + Unpin, B>
    where
        S: Unpin,
        StepFn: for<'a> FnMut(&mut S, &mut Context<'a>) -> Poll<Gradus<A>> + Unpin,
        F: FnMut(A) -> Option<B> + Unpin,
    {
        let mut step0 = self.step;
        FlumenFusus::new(
            self.state,
            move |s: &mut S, cx: &mut Context<'_>| match step0(s, cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(Gradus::Yield(a)) => match f(a) {
                    Some(b) => Poll::Ready(Gradus::Yield(b)),
                    None => Poll::Ready(Gradus::Skip),
                },
                Poll::Ready(Gradus::Skip) => Poll::Ready(Gradus::Skip),
                Poll::Ready(Gradus::Done) => Poll::Ready(Gradus::Done),
            },
        )
    }

    /// Take at most `n` items (fused).
    #[inline(always)]
    pub fn take(self, n: usize) -> FlumenFusus<(S, usize), impl GradusStep<(S, usize), A>, A>
    where
        S: Unpin,
        StepFn: for<'a> FnMut(&mut S, &mut Context<'a>) -> Poll<Gradus<A>> + Unpin,
    {
        let mut step0 = self.step;
        FlumenFusus::new(
            (self.state, n),
            move |st: &mut (S, usize), cx: &mut Context<'_>| {
                if st.1 == 0 {
                    return Poll::Ready(Gradus::Done);
                }

                match step0(&mut st.0, cx) {
                    Poll::Pending => Poll::Pending,
                    Poll::Ready(Gradus::Yield(a)) => {
                        st.1 -= 1;
                        Poll::Ready(Gradus::Yield(a))
                    }
                    Poll::Ready(Gradus::Skip) => Poll::Ready(Gradus::Skip),
                    Poll::Ready(Gradus::Done) => Poll::Ready(Gradus::Done),
                }
            },
        )
    }

    /// Skip the first `n` items (fused).
    #[inline(always)]
    pub fn skip(self, n: usize) -> FlumenFusus<(S, usize), impl GradusStep<(S, usize), A>, A>
    where
        S: Unpin,
        StepFn: for<'a> FnMut(&mut S, &mut Context<'a>) -> Poll<Gradus<A>> + Unpin,
    {
        let mut step0 = self.step;
        FlumenFusus::new(
            (self.state, n),
            move |st: &mut (S, usize), cx: &mut Context<'_>| match step0(&mut st.0, cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(Gradus::Yield(a)) => {
                    if st.1 == 0 {
                        Poll::Ready(Gradus::Yield(a))
                    } else {
                        st.1 -= 1;
                        Poll::Ready(Gradus::Skip)
                    }
                }
                Poll::Ready(Gradus::Skip) => Poll::Ready(Gradus::Skip),
                Poll::Ready(Gradus::Done) => Poll::Ready(Gradus::Done),
            },
        )
    }

    /// Scan - a lazy fold that yields intermediate accumulator values (fused).
    ///
    /// # Panics
    ///
    /// Panics only if the internal accumulator-slot invariant (the slot is
    /// `Some` between steps) is violated, which would indicate a bug in
    /// this crate. A panic inside `f` can strand the slot empty, so a
    /// stream whose closure panicked must not be polled again.
    #[inline(always)]
    pub fn scan<B, F>(
        self,
        init: B,
        mut f: F,
    ) -> FlumenFusus<(S, Option<B>), impl ScanStep<S, B>, B>
    where
        S: Unpin,
        StepFn: for<'a> FnMut(&mut S, &mut Context<'a>) -> Poll<Gradus<A>> + Unpin,
        B: Clone + Unpin,
        F: FnMut(B, A) -> B + Unpin,
    {
        // State carries `Option<B>` so the fold can receive the previous
        // accumulator by move. The slot is Some between steps and only
        // transiently None while `f` is running.
        let mut step0 = self.step;
        FlumenFusus::new(
            (self.state, Some(init)),
            move |st: &mut (S, Option<B>), cx: &mut Context<'_>| match step0(&mut st.0, cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(Gradus::Yield(a)) => {
                    let prev = st.1.take().expect("scan acc is Some between steps");
                    let new_acc = f(prev, a);
                    st.1 = Some(new_acc.clone());
                    Poll::Ready(Gradus::Yield(new_acc))
                }
                Poll::Ready(Gradus::Skip) => Poll::Ready(Gradus::Skip),
                Poll::Ready(Gradus::Done) => Poll::Ready(Gradus::Done),
            },
        )
    }

    /// Scan with state that differs from the output type (fused).
    ///
    /// Returns `None` to terminate the stream early.
    #[inline(always)]
    pub fn scan_with<St, B, F>(
        self,
        init: St,
        mut f: F,
    ) -> FlumenFusus<(S, St, bool), impl ScanWithStep<S, St, B>, B>
    where
        S: Unpin,
        StepFn: for<'a> FnMut(&mut S, &mut Context<'a>) -> Poll<Gradus<A>> + Unpin,
        St: Unpin,
        F: FnMut(&mut St, A) -> Option<B> + Unpin,
    {
        let mut step0 = self.step;
        FlumenFusus::new(
            (self.state, init, false),
            move |st: &mut (S, St, bool), cx: &mut Context<'_>| {
                if st.2 {
                    return Poll::Ready(Gradus::Done);
                }

                match step0(&mut st.0, cx) {
                    Poll::Pending => Poll::Pending,
                    Poll::Ready(Gradus::Yield(a)) => {
                        if let Some(b) = f(&mut st.1, a) {
                            Poll::Ready(Gradus::Yield(b))
                        } else {
                            st.2 = true;
                            Poll::Ready(Gradus::Done)
                        }
                    }
                    Poll::Ready(Gradus::Skip) => Poll::Ready(Gradus::Skip),
                    Poll::Ready(Gradus::Done) => Poll::Ready(Gradus::Done),
                }
            },
        )
    }

    /// Inspect each yielded item without modifying it (fused).
    #[inline(always)]
    pub fn inspect<F>(
        self,
        mut f: F,
    ) -> FlumenFusus<S, impl for<'a> FnMut(&mut S, &mut Context<'a>) -> Poll<Gradus<A>> + Unpin, A>
    where
        S: Unpin,
        StepFn: for<'a> FnMut(&mut S, &mut Context<'a>) -> Poll<Gradus<A>> + Unpin,
        F: FnMut(&A) + Unpin,
    {
        let mut step0 = self.step;
        FlumenFusus::new(
            self.state,
            move |s: &mut S, cx: &mut Context<'_>| match step0(s, cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(Gradus::Yield(a)) => {
                    f(&a);
                    Poll::Ready(Gradus::Yield(a))
                }
                Poll::Ready(Gradus::Skip) => Poll::Ready(Gradus::Skip),
                Poll::Ready(Gradus::Done) => Poll::Ready(Gradus::Done),
            },
        )
    }

    /// Enumerate yielded items with their index (fused).
    #[inline(always)]
    pub fn enumerate(self) -> FlumenFusus<(S, usize), impl EnumerateStep<S, A>, (usize, A)>
    where
        S: Unpin,
        StepFn: for<'a> FnMut(&mut S, &mut Context<'a>) -> Poll<Gradus<A>> + Unpin,
    {
        let mut step0 = self.step;
        FlumenFusus::new(
            (self.state, 0),
            move |st: &mut (S, usize), cx: &mut Context<'_>| match step0(&mut st.0, cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(Gradus::Yield(a)) => {
                    let idx = st.1;
                    st.1 += 1;
                    Poll::Ready(Gradus::Yield((idx, a)))
                }
                Poll::Ready(Gradus::Skip) => Poll::Ready(Gradus::Skip),
                Poll::Ready(Gradus::Done) => Poll::Ready(Gradus::Done),
            },
        )
    }

    /// Take items while a predicate holds (fused).
    #[inline(always)]
    pub fn take_while<F>(
        self,
        mut predicate: F,
    ) -> FlumenFusus<(S, bool), impl GradusStep<(S, bool), A>, A>
    where
        S: Unpin,
        StepFn: for<'a> FnMut(&mut S, &mut Context<'a>) -> Poll<Gradus<A>> + Unpin,
        F: FnMut(&A) -> bool + Unpin,
    {
        let mut step0 = self.step;
        FlumenFusus::new(
            (self.state, false),
            move |st: &mut (S, bool), cx: &mut Context<'_>| {
                if st.1 {
                    return Poll::Ready(Gradus::Done);
                }

                match step0(&mut st.0, cx) {
                    Poll::Pending => Poll::Pending,
                    Poll::Ready(Gradus::Yield(a)) => {
                        if predicate(&a) {
                            Poll::Ready(Gradus::Yield(a))
                        } else {
                            st.1 = true;
                            Poll::Ready(Gradus::Done)
                        }
                    }
                    Poll::Ready(Gradus::Skip) => Poll::Ready(Gradus::Skip),
                    Poll::Ready(Gradus::Done) => Poll::Ready(Gradus::Done),
                }
            },
        )
    }

    /// Skip items while a predicate holds (fused).
    #[inline(always)]
    pub fn skip_while<F>(
        self,
        mut predicate: F,
    ) -> FlumenFusus<(S, bool), impl GradusStep<(S, bool), A>, A>
    where
        S: Unpin,
        StepFn: for<'a> FnMut(&mut S, &mut Context<'a>) -> Poll<Gradus<A>> + Unpin,
        F: FnMut(&A) -> bool + Unpin,
    {
        let mut step0 = self.step;
        FlumenFusus::new(
            (self.state, true),
            move |st: &mut (S, bool), cx: &mut Context<'_>| match step0(&mut st.0, cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(Gradus::Yield(a)) => {
                    if st.1 && predicate(&a) {
                        Poll::Ready(Gradus::Skip)
                    } else {
                        st.1 = false;
                        Poll::Ready(Gradus::Yield(a))
                    }
                }
                Poll::Ready(Gradus::Skip) => Poll::Ready(Gradus::Skip),
                Poll::Ready(Gradus::Done) => Poll::Ready(Gradus::Done),
            },
        )
    }

    /// Collect items into fixed-size batches (fused).
    ///
    /// Yields `Vec`s of exactly `size` items, except possibly a shorter
    /// final batch when the source ends mid-chunk.
    ///
    /// # Panics
    ///
    /// Panics if `size` is zero — a zero-sized chunk could never fill and
    /// the stream would spin forever, so it is rejected eagerly at
    /// construction rather than at first poll.
    #[inline]
    pub fn chunks(
        self,
        size: usize,
    ) -> FlumenFusus<ChunksState<S, A>, impl ChunksStep<S, A>, Vec<A>>
    where
        S: Unpin,
        StepFn: for<'a> FnMut(&mut S, &mut Context<'a>) -> Poll<Gradus<A>> + Unpin,
    {
        assert!(size > 0, "chunk size must be greater than 0");
        let mut step0 = self.step;

        FlumenFusus::new(
            (self.state, size, Vec::with_capacity(size), false),
            move |st: &mut (S, usize, Vec<A>, bool), cx: &mut Context<'_>| {
                if st.3 && st.2.is_empty() {
                    return Poll::Ready(Gradus::Done);
                }

                loop {
                    if st.2.len() >= st.1 {
                        let chunk = core::mem::replace(&mut st.2, Vec::with_capacity(st.1));
                        return Poll::Ready(Gradus::Yield(chunk));
                    }

                    if st.3 {
                        let final_chunk = core::mem::take(&mut st.2);
                        return Poll::Ready(Gradus::Yield(final_chunk));
                    }

                    match step0(&mut st.0, cx) {
                        Poll::Pending => return Poll::Pending,
                        Poll::Ready(Gradus::Yield(a)) => {
                            st.2.push(a);
                        }
                        Poll::Ready(Gradus::Skip) => {
                            return Poll::Ready(Gradus::Skip);
                        }
                        Poll::Ready(Gradus::Done) => {
                            st.3 = true;
                            if st.2.is_empty() {
                                return Poll::Ready(Gradus::Done);
                            }
                        }
                    }
                }
            },
        )
    }

    /// Fold the stream to a single value (async).
    ///
    /// Drains the whole stream in a single future (no per-item `poll_fn`
    /// allocation), threading the accumulator through `f` by move.
    ///
    /// # Panics
    ///
    /// Panics only if the internal accumulator-slot invariant (the slot is
    /// `Some` between steps) is violated, which would indicate a bug in
    /// this crate.
    #[inline]
    pub async fn fold<B, F>(mut self, init: B, mut f: F) -> B
    where
        Self: Unpin,
        S: Unpin,
        StepFn: for<'a> FnMut(&mut S, &mut Context<'a>) -> Poll<Gradus<A>> + Unpin,
        F: FnMut(B, A) -> B,
    {
        let mut acc = Some(init);
        // Poll `self.step` directly instead of awaiting `self.next()` per item:
        // this avoids constructing a fresh `poll_fn` future for every element,
        // collapsing the whole drain into a single future. `acc` is threaded
        // through the `FnMut` closure via `Option::take`/reassign.
        poll_fn(|cx| {
            loop {
                match (self.step)(&mut self.state, cx) {
                    Poll::Pending => return Poll::Pending,
                    Poll::Ready(Gradus::Yield(a)) => {
                        let current = acc.take().unwrap();
                        acc = Some(f(current, a));
                    }
                    Poll::Ready(Gradus::Skip) => {}
                    Poll::Ready(Gradus::Done) => return Poll::Ready(()),
                }
            }
        })
        .await;
        acc.unwrap()
    }

    /// Collect the stream into a `Vec` (async).
    #[inline]
    pub async fn collect_vec(self) -> Vec<A>
    where
        Self: Unpin,
        S: Unpin,
        StepFn: for<'a> FnMut(&mut S, &mut Context<'a>) -> Poll<Gradus<A>> + Unpin,
    {
        // Pre-allocate with a small starting capacity to avoid the first
        // several doublings that Vec::new() (capacity 0) would incur.
        self.fold(Vec::with_capacity(16), |mut acc, item| {
            acc.push(item);
            acc
        })
        .await
    }

    /// Collect the stream into a `Vec` with a pre-allocated capacity.
    ///
    /// Use this when you know or can estimate the output size, to avoid
    /// repeated reallocations during collection.
    #[inline]
    pub async fn collect_vec_with_capacity(self, capacity: usize) -> Vec<A>
    where
        Self: Unpin,
        S: Unpin,
        StepFn: for<'a> FnMut(&mut S, &mut Context<'a>) -> Poll<Gradus<A>> + Unpin,
    {
        self.fold(Vec::with_capacity(capacity), |mut acc, item| {
            acc.push(item);
            acc
        })
        .await
    }

    /// Conservative size hint. Returns `(0, None)` since fused streams
    /// generally cannot know their output size without running.
    /// Use `collect_vec_with_capacity` when you have an estimate.
    #[inline]
    pub fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }

    /// Execute a side-effect for each yielded item (async).
    #[inline]
    pub async fn for_each<F>(self, mut f: F)
    where
        Self: Unpin,
        S: Unpin,
        StepFn: for<'a> FnMut(&mut S, &mut Context<'a>) -> Poll<Gradus<A>> + Unpin,
        F: FnMut(A),
    {
        self.fold((), move |(), item| {
            f(item);
        })
        .await;
    }

    /// Count the number of items yielded (async).
    #[inline]
    pub async fn count(self) -> usize
    where
        Self: Unpin,
        S: Unpin,
        StepFn: for<'a> FnMut(&mut S, &mut Context<'a>) -> Poll<Gradus<A>> + Unpin,
    {
        self.fold(0usize, |n, _| n + 1).await
    }

    /// Return the first item satisfying `predicate` (async).
    #[inline]
    pub async fn find<F>(mut self, mut predicate: F) -> Option<A>
    where
        Self: Unpin,
        S: Unpin,
        StepFn: for<'a> FnMut(&mut S, &mut Context<'a>) -> Poll<Gradus<A>> + Unpin,
        F: FnMut(&A) -> bool,
    {
        while let Some(item) = self.next().await {
            if predicate(&item) {
                return Some(item);
            }
        }
        None
    }

    /// Return `true` if any item satisfies `predicate` (async).
    #[inline]
    pub async fn any<F>(self, predicate: F) -> bool
    where
        Self: Unpin,
        S: Unpin,
        StepFn: for<'a> FnMut(&mut S, &mut Context<'a>) -> Poll<Gradus<A>> + Unpin,
        F: FnMut(&A) -> bool,
    {
        self.find(predicate).await.is_some()
    }

    /// Return `true` if every item satisfies `predicate` (async).
    ///
    /// Stops at the first item that does not match; returns `true` for an
    /// empty stream.
    #[inline]
    pub async fn all<F>(mut self, mut predicate: F) -> bool
    where
        Self: Unpin,
        S: Unpin,
        StepFn: for<'a> FnMut(&mut S, &mut Context<'a>) -> Poll<Gradus<A>> + Unpin,
        F: FnMut(&A) -> bool,
    {
        while let Some(item) = self.next().await {
            if !predicate(&item) {
                return false;
            }
        }
        true
    }

    /// Return the last item yielded by the stream (async).
    ///
    /// Drains the entire stream. Returns `None` for an empty stream.
    #[inline]
    pub async fn last(mut self) -> Option<A>
    where
        Self: Unpin,
        S: Unpin,
        StepFn: for<'a> FnMut(&mut S, &mut Context<'a>) -> Poll<Gradus<A>> + Unpin,
    {
        let mut last: Option<A> = None;
        while let Some(item) = self.next().await {
            last = Some(item);
        }
        last
    }

    /// Return the `n`-th item (0-indexed) yielded by the stream (async).
    ///
    /// Returns `None` if the stream yields fewer than `n + 1` items.
    #[inline]
    pub async fn nth(mut self, mut n: usize) -> Option<A>
    where
        Self: Unpin,
        S: Unpin,
        StepFn: for<'a> FnMut(&mut S, &mut Context<'a>) -> Poll<Gradus<A>> + Unpin,
    {
        while let Some(item) = self.next().await {
            if n == 0 {
                return Some(item);
            }
            n -= 1;
        }
        None
    }

    /// Return the index of the first item satisfying `predicate` (async).
    ///
    /// Returns `None` if no item matches.
    #[inline]
    pub async fn position<F>(mut self, mut predicate: F) -> Option<usize>
    where
        Self: Unpin,
        S: Unpin,
        StepFn: for<'a> FnMut(&mut S, &mut Context<'a>) -> Poll<Gradus<A>> + Unpin,
        F: FnMut(&A) -> bool,
    {
        let mut idx = 0usize;
        while let Some(item) = self.next().await {
            if predicate(&item) {
                return Some(idx);
            }
            idx += 1;
        }
        None
    }

    /// Reduce the stream to a single value using a binary function (async).
    ///
    /// Uses the first item as the initial accumulator. Returns `None` for an
    /// empty stream.
    #[inline]
    pub async fn reduce<F>(mut self, mut f: F) -> Option<A>
    where
        Self: Unpin,
        S: Unpin,
        StepFn: for<'a> FnMut(&mut S, &mut Context<'a>) -> Poll<Gradus<A>> + Unpin,
        F: FnMut(A, A) -> A,
    {
        let mut acc = self.next().await?;
        while let Some(item) = self.next().await {
            acc = f(acc, item);
        }
        Some(acc)
    }

    /// Sum all items in the stream (async).
    ///
    /// Requires `A: Add<Output = A> + Default`. Returns `A::default()` for an
    /// empty stream.
    #[inline]
    pub async fn sum(self) -> A
    where
        Self: Unpin,
        S: Unpin,
        StepFn: for<'a> FnMut(&mut S, &mut Context<'a>) -> Poll<Gradus<A>> + Unpin,
        A: core::ops::Add<Output = A> + Default,
    {
        self.fold(A::default(), |acc, x| acc + x).await
    }

    /// Multiply all items in the stream (async).
    ///
    /// Requires `A: Mul<Output = A> + From<u8>`. The identity element is
    /// `A::from(1u8)`. Returns that identity for an empty stream.
    #[inline]
    pub async fn product(self) -> A
    where
        Self: Unpin,
        S: Unpin,
        StepFn: for<'a> FnMut(&mut S, &mut Context<'a>) -> Poll<Gradus<A>> + Unpin,
        A: core::ops::Mul<Output = A> + From<u8>,
    {
        self.fold(A::from(1u8), |acc, x| acc * x).await
    }

    /// Fuse two streams end-to-end (lazy, no intermediate `Vec`).
    ///
    /// Yields all items from `self`, then all items from `other`. The resulting
    /// stream's state is `(S, S2, bool)` where the `bool` flag flips to `true`
    /// when the first stream is exhausted, at which point the step function
    /// delegates to `other`'s step. A `Skip` is emitted on the transition so
    /// the outer `poll_next` loop immediately tries the second stream.
    #[inline(always)]
    pub fn chain<S2, StepFn2>(
        self,
        other: FlumenFusus<S2, StepFn2, A>,
    ) -> FlumenFusus<(S, S2, bool), impl ChainStep<S, S2, A>, A>
    where
        S: Unpin,
        S2: Unpin,
        StepFn: for<'a> FnMut(&mut S, &mut Context<'a>) -> Poll<Gradus<A>> + Unpin,
        StepFn2: for<'a> FnMut(&mut S2, &mut Context<'a>) -> Poll<Gradus<A>> + Unpin,
    {
        let mut step1 = self.step;
        let mut step2 = other.step;
        FlumenFusus::new(
            (self.state, other.state, false),
            move |st: &mut (S, S2, bool), cx: &mut Context<'_>| {
                if st.2 {
                    step2(&mut st.1, cx)
                } else {
                    match step1(&mut st.0, cx) {
                        Poll::Pending => Poll::Pending,
                        Poll::Ready(Gradus::Yield(a)) => Poll::Ready(Gradus::Yield(a)),
                        Poll::Ready(Gradus::Skip) => Poll::Ready(Gradus::Skip),
                        Poll::Ready(Gradus::Done) => {
                            st.2 = true;
                            Poll::Ready(Gradus::Skip)
                        }
                    }
                }
            },
        )
    }

    /// Zip two streams, terminating at the shorter.
    #[inline]
    pub fn zip<S2, StepFn2, B>(
        self,
        other: FlumenFusus<S2, StepFn2, B>,
    ) -> ZipFusus<S, S2, impl ZipStep<S, S2, A, B>, A, B>
    where
        S: Unpin,
        S2: Unpin,
        StepFn: for<'a> FnMut(&mut S, &mut Context<'a>) -> Poll<Gradus<A>> + Unpin,
        StepFn2: for<'a> FnMut(&mut S2, &mut Context<'a>) -> Poll<Gradus<B>> + Unpin,
    {
        let mut step1 = self.step;
        let mut step2 = other.step;
        FlumenFusus::new(
            (self.state, other.state, None),
            move |st: &mut (S, S2, Option<A>), cx: &mut Context<'_>| {
                // Reuse a stashed item from a partial step, or drain step1
                // until Yield or Done (propagate Pending immediately).
                let a = match st.2.take() {
                    Some(a) => a,
                    None => loop {
                        match step1(&mut st.0, cx) {
                            Poll::Pending => return Poll::Pending,
                            Poll::Ready(Gradus::Yield(a)) => break a,
                            Poll::Ready(Gradus::Skip) => {}
                            Poll::Ready(Gradus::Done) => return Poll::Ready(Gradus::Done),
                        }
                    },
                };
                // Drain step2 until Yield or Done; stash `a` if step2 is not
                // ready yet so the item survives until the next poll.
                let b = loop {
                    match step2(&mut st.1, cx) {
                        Poll::Pending => {
                            st.2 = Some(a);
                            return Poll::Pending;
                        }
                        Poll::Ready(Gradus::Yield(b)) => break b,
                        Poll::Ready(Gradus::Skip) => {}
                        Poll::Ready(Gradus::Done) => return Poll::Ready(Gradus::Done),
                    }
                };
                Poll::Ready(Gradus::Yield((a, b)))
            },
        )
    }

    /// Get the next item from the fused stream (async).
    #[inline]
    async fn next(&mut self) -> Option<A>
    where
        Self: Unpin,
        S: Unpin,
        StepFn: for<'a> FnMut(&mut S, &mut Context<'a>) -> Poll<Gradus<A>> + Unpin,
    {
        poll_fn(|cx| Pin::new(&mut *self).poll_next(cx)).await
    }
}

impl<S, StepFn, A> Stream for FlumenFusus<S, StepFn, A>
where
    S: Unpin,
    StepFn: for<'a> FnMut(&mut S, &mut Context<'a>) -> Poll<Gradus<A>> + Unpin,
{
    type Item = A;

    #[inline(always)]
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        loop {
            match (this.step)(&mut this.state, cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(Gradus::Yield(a)) => return Poll::Ready(Some(a)),
                Poll::Ready(Gradus::Skip) => {}
                Poll::Ready(Gradus::Done) => return Poll::Ready(None),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    use alloc::vec::IntoIter;
    use core::future::Future;

    /// Step function over a vec iterator that yields each item immediately.
    fn ready_step(it: &mut IntoIter<i32>, _cx: &mut Context<'_>) -> Poll<Gradus<i32>> {
        Poll::Ready(match it.next() {
            Some(x) => Gradus::Yield(x),
            None => Gradus::Done,
        })
    }

    /// Step function that returns `Pending` once before each item.
    fn pending_once_step(
        st: &mut (IntoIter<i32>, bool),
        cx: &mut Context<'_>,
    ) -> Poll<Gradus<i32>> {
        if !st.1 {
            st.1 = true;
            cx.waker().wake_by_ref();
            return Poll::Pending;
        }
        st.1 = false;
        Poll::Ready(match st.0.next() {
            Some(x) => Gradus::Yield(x),
            None => Gradus::Done,
        })
    }

    /// Busy-poll a future to completion (the test steps wake the noop waker).
    fn drive<F: Future>(fut: F) -> F::Output {
        let mut fut = core::pin::pin!(fut);
        let mut cx = Context::from_waker(core::task::Waker::noop());
        loop {
            if let Poll::Ready(out) = fut.as_mut().poll(&mut cx) {
                return out;
            }
        }
    }

    #[test]
    fn test_zip_keeps_items_when_second_stream_pending() {
        let fast = FlumenFusus::new(vec![1, 2, 3].into_iter(), ready_step);
        let slow = FlumenFusus::new((vec![10, 20, 30].into_iter(), false), pending_once_step);
        let pairs = drive(fast.zip(slow).collect_vec());
        assert_eq!(pairs, vec![(1, 10), (2, 20), (3, 30)]);
    }
}
