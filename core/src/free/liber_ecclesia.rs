//! `LiberEcclesia` - Church-encoded Free Monad
//!
//! > *"Ecclesia non moritur."*
//! > — The Church does not die. (Legal maxim)
//!
//! Church-encoded Free monad that achieves O(n) performance for left-associated
//! binds by using continuation-passing style (CPS) to automatically right-associate
//! operations.
//!
//! # Performance
//!
//! The standard Free monad (`Liber`) has O(n²) complexity for left-associated binds:
//! ```text
//! ((pure a >>= f1) >>= f2) >>= f3
//! ```
//!
//! This is because each bind must traverse the entire tree structure.
//!
//! `LiberEcclesia` uses a simplified Church encoding approach with explicit
//! continuation stacks, giving O(n) performance.
//!
//! ## Memory Coalescing
//!
//! For small continuation stacks (≤ 4 elements), we use inline storage to avoid
//! heap allocation. This improves cache locality and reduces allocation overhead
//! for common use cases.
//!
//! # Latin Etymology
//!
//! *Ecclesia* = church, assembly (from Greek ἐκκλησία)
//! Named for Church encoding, invented by Alonzo Church.
//!
//! # Reference
//!
//! Based on techniques from:
//! - "Reflection without Remorse" (van der Ploeg & Kiselyov, 2014)
//! - "Church Encoding of Data Types" (Mogensen, 1992)

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::boxed::Box;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use super::liber::Liber;
use crate::hints::likely;
use crate::typeclasses::hkt::FunctorHKT;

// =============================================================================
// SmallStack - Inline storage for small collections
// =============================================================================

/// Number of elements to store inline before spilling to heap.
/// Chosen to fit common monadic pipelines (3-4 operations) while keeping
/// stack size reasonable.
pub const INLINE_CAPACITY: usize = 4;

/// A small-vector-style stack that stores elements inline up to a threshold.
///
/// This provides O(1) push for small stacks without heap allocation,
/// improving cache locality for common use cases.
///
/// # Latin Etymology
///
/// *Acervus* = heap, pile, stack
/// *Parvus* = small
#[cfg(feature = "alloc")]
pub struct AcervusParvus<T> {
    /// Storage mode: inline for small counts, heap for larger
    storage: AcervusStorage<T>,
}

#[cfg(feature = "alloc")]
enum AcervusStorage<T> {
    /// Inline storage using `MaybeUninit` for efficiency
    Inline {
        /// Fixed-size array for inline storage
        data: [Option<T>; INLINE_CAPACITY],
        /// Number of elements currently stored
        len: usize,
    },
    /// Heap storage when we exceed inline capacity
    Heap(Vec<T>),
}

#[cfg(feature = "alloc")]
impl<T> AcervusParvus<T> {
    /// Create an empty small stack.
    #[inline(always)]
    pub const fn empty() -> Self {
        AcervusParvus {
            storage: AcervusStorage::Inline {
                data: [const { None }; INLINE_CAPACITY],
                len: 0,
            },
        }
    }

    /// Check if the stack is empty.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the number of elements.
    #[inline(always)]
    pub fn len(&self) -> usize {
        match &self.storage {
            AcervusStorage::Inline { len, .. } => *len,
            AcervusStorage::Heap(vec) => vec.len(),
        }
    }

    /// Check if storage is inline.
    #[inline(always)]
    pub fn is_inline(&self) -> bool {
        matches!(self.storage, AcervusStorage::Inline { .. })
    }

    /// Push an element onto the stack.
    #[inline(always)]
    pub fn push(&mut self, item: T) {
        match &mut self.storage {
            AcervusStorage::Inline { data, len } => {
                // Hot path: most pushes fit inline
                if likely(*len < INLINE_CAPACITY) {
                    data[*len] = Some(item);
                    *len += 1;
                } else {
                    // Cold path: spill to heap (rare)
                    self.spill_to_heap(item);
                }
            }
            AcervusStorage::Heap(vec) => {
                vec.push(item);
            }
        }
    }

    /// Cold path: spill from inline to heap storage.
    #[cold]
    #[inline(never)]
    fn spill_to_heap(&mut self, item: T) {
        if let AcervusStorage::Inline { data, .. } = &mut self.storage {
            let mut vec = Vec::with_capacity(INLINE_CAPACITY * 2);
            for slot in data.iter_mut() {
                if let Some(val) = slot.take() {
                    vec.push(val);
                }
            }
            vec.push(item);
            self.storage = AcervusStorage::Heap(vec);
        }
    }

    /// Pop an element from the end of the stack.
    #[inline(always)]
    pub fn pop(&mut self) -> Option<T> {
        match &mut self.storage {
            AcervusStorage::Inline { data, len } => {
                // Hot path: stack has elements
                if likely(*len > 0) {
                    *len -= 1;
                    data[*len].take()
                } else {
                    None
                }
            }
            AcervusStorage::Heap(vec) => vec.pop(),
        }
    }

    /// Iterate over elements.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        AcervusIter {
            stack: self,
            index: 0,
        }
    }
}

#[cfg(feature = "alloc")]
impl<T> IntoIterator for AcervusParvus<T> {
    type Item = T;
    type IntoIter = AcervusIntoIter<T>;

    /// Consume the stack and return an iterator over its elements in
    /// bottom-to-top order.
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        AcervusIntoIter {
            stack: self,
            inline_cursor: 0,
        }
    }
}

#[cfg(feature = "alloc")]
impl<T> Default for AcervusParvus<T> {
    #[inline(always)]
    fn default() -> Self {
        Self::empty()
    }
}

/// Iterator over `AcervusParvus` elements.
#[cfg(feature = "alloc")]
struct AcervusIter<'a, T> {
    stack: &'a AcervusParvus<T>,
    index: usize,
}

#[cfg(feature = "alloc")]
impl<'a, T> Iterator for AcervusIter<'a, T> {
    type Item = &'a T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match &self.stack.storage {
            AcervusStorage::Inline { data, len } => {
                if self.index < *len {
                    let item = data[self.index].as_ref();
                    self.index += 1;
                    item
                } else {
                    None
                }
            }
            AcervusStorage::Heap(vec) => {
                if self.index < vec.len() {
                    let item = &vec[self.index];
                    self.index += 1;
                    Some(item)
                } else {
                    None
                }
            }
        }
    }
}

/// Consuming iterator over [`AcervusParvus`] elements (see
/// [`IntoIterator for AcervusParvus`](AcervusParvus#impl-IntoIterator-for-AcervusParvus<T>)).
#[cfg(feature = "alloc")]
pub struct AcervusIntoIter<T> {
    stack: AcervusParvus<T>,
    /// Cursor for the inline path: index of the next element to yield.
    /// For the heap path the Vec is reversed on first use so we can pop
    /// from the back in O(1), avoiding the O(n) `Vec::remove(0)`.
    inline_cursor: usize,
}

#[cfg(feature = "alloc")]
impl<T> Iterator for AcervusIntoIter<T> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.stack.storage {
            AcervusStorage::Inline { data, len } => {
                if self.inline_cursor >= *len {
                    return None;
                }
                // Advance cursor instead of shifting elements — O(1) per call.
                let item = data[self.inline_cursor].take();
                self.inline_cursor += 1;
                item
            }
            AcervusStorage::Heap(vec) => {
                // Pop from the back is O(1).  On the first call we reverse
                // so that the original front becomes the new back.
                if vec.is_empty() {
                    None
                } else {
                    // `inline_cursor` doubles as a "reversed" flag here: 0
                    // means not yet reversed, 1 means already reversed.
                    if self.inline_cursor == 0 {
                        vec.reverse();
                        self.inline_cursor = 1;
                    }
                    vec.pop()
                }
            }
        }
    }
}

// =============================================================================
// LiberEcclesia - Church-encoded Free Monad (Simplified Implementation)
// =============================================================================

/// Church-encoded Free monad for O(n) bind performance.
///
/// This implementation uses a stack-based approach to achieve O(n) performance
/// for left-associated binds. The key insight is that we defer continuation
/// composition until interpretation time.
///
/// # Type Parameters
///
/// * `F` - The functor type (HKT witness)
/// * `A` - The result type
///
/// # Example
///
/// ```rust
/// use ordofp_core::free::*;
///
/// // Create a pure value
/// let pure_val: LiberEcclesia<OptionFWitness, i32> = LiberEcclesia::purus(42);
///
/// // Chain operations with O(n) performance
/// let result = pure_val
///     .flat_map(|x| LiberEcclesia::purus(x + 1))
///     .flat_map(|x| LiberEcclesia::purus(x * 2));
/// assert_eq!(result.run_pure(), 86);
/// ```
#[cfg(feature = "alloc")]
pub enum LiberEcclesia<F: FunctorHKT + 'static, A: 'static> {
    /// Pure value - computation completes immediately.
    Purus(A),

    /// Suspended computation with continuation stack.
    /// The stack ensures O(n) performance by deferring composition.
    Suspensus {
        /// The suspended functor value (type-erased)
        effect: Box<dyn core::any::Any + Send + Sync>,
        /// Stack of continuations (type-erased)
        /// Each continuation is: `Box<dyn Any + Send + Sync> -> LiberEcclesiaErased<F>`
        stack: ContinuatioStack<F>,
    },
}

/// A stack of continuations for O(n) bind performance.
///
/// This is the key data structure that enables efficient left-associated binds.
/// Instead of building nested closures, we maintain a flat stack.
///
/// Uses inline storage for small continuation stacks (≤ 4 elements) to avoid
/// heap allocation for common use cases.
///
/// # Latin Etymology
///
/// *Continuatio* = continuation, sequence
#[cfg(feature = "alloc")]
pub struct ContinuatioStack<F: FunctorHKT + 'static> {
    /// Type-erased continuation functions with inline storage optimization
    steps: AcervusParvus<Box<dyn ContinuatioStep<F> + Send + Sync>>,
}

/// A single step in the continuation stack.
#[cfg(feature = "alloc")]
pub trait ContinuatioStep<F: FunctorHKT>: Send + Sync {
    /// Apply this continuation step to a value.
    fn applica(&self, value: Box<dyn core::any::Any + Send + Sync>) -> LiberEcclesiaErased<F>;
}

/// Type-erased version of `LiberEcclesia` for internal use.
#[cfg(feature = "alloc")]
pub enum LiberEcclesiaErased<F: FunctorHKT + 'static> {
    /// Pure value (type-erased)
    Purus(Box<dyn core::any::Any + Send + Sync>),
    /// Suspended computation
    Suspensus {
        /// The pending effect operation, type-erased to `Any`; downcast by
        /// the interpreter that knows the concrete functor payload.
        effect: Box<dyn core::any::Any + Send + Sync>,
        /// The continuation stack to run (left to right) once the effect
        /// produces a value — the Church-encoded bind chain.
        stack: ContinuatioStack<F>,
    },
}

// =============================================================================
// Implementation
// =============================================================================

#[cfg(feature = "alloc")]
impl<F: FunctorHKT + 'static> ContinuatioStack<F> {
    /// Create an empty continuation stack.
    ///
    /// Uses inline storage - no heap allocation for the first 4 continuations.
    #[inline(always)]
    pub fn empty() -> Self {
        ContinuatioStack {
            steps: AcervusParvus::empty(),
        }
    }

    /// Check if the stack is empty.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    /// Get the length of the stack.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.steps.len()
    }

    /// Check if the stack is using inline storage (no heap allocation).
    #[inline(always)]
    pub fn is_inline(&self) -> bool {
        self.steps.is_inline()
    }

    /// Push a new continuation onto the stack.
    ///
    /// O(1) amortized - uses inline storage for first 4 elements.
    #[inline(always)]
    fn push(&mut self, step: Box<dyn ContinuatioStep<F> + Send + Sync>) {
        self.steps.push(step);
    }
}

/// Concrete continuation step implementation.
#[cfg(feature = "alloc")]
struct ContinuatioStepImpl<F, A, B, G>
where
    F: FunctorHKT + 'static,
    A: Send + Sync + 'static,
    B: Send + Sync + 'static,
    G: Fn(A) -> LiberEcclesia<F, B> + Send + Sync,
{
    f: G,
    _phantom: core::marker::PhantomData<fn(A) -> B>,
}

#[cfg(feature = "alloc")]
impl<F, A, B, G> ContinuatioStep<F> for ContinuatioStepImpl<F, A, B, G>
where
    F: FunctorHKT + 'static,
    A: Send + Sync + 'static,
    B: Send + Sync + 'static,
    G: Fn(A) -> LiberEcclesia<F, B> + Send + Sync,
{
    fn applica(&self, value: Box<dyn core::any::Any + Send + Sync>) -> LiberEcclesiaErased<F> {
        let a: A = *value.downcast().expect("Type mismatch in continuation");
        let result = (self.f)(a);
        result.erase()
    }
}

#[cfg(feature = "alloc")]
impl<F: FunctorHKT + 'static, A: Send + Sync + 'static> LiberEcclesia<F, A> {
    /// Erase the type for internal use.
    #[inline]
    fn erase(self) -> LiberEcclesiaErased<F> {
        match self {
            LiberEcclesia::Purus(a) => LiberEcclesiaErased::Purus(Box::new(a)),
            LiberEcclesia::Suspensus { effect, stack } => {
                LiberEcclesiaErased::Suspensus { effect, stack }
            }
        }
    }
}

#[cfg(feature = "alloc")]
impl<F: FunctorHKT + 'static, A: 'static> LiberEcclesia<F, A> {
    /// Create a pure (immediate) value.
    ///
    /// This is the `return` / `pure` of the monad.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::free::{LiberEcclesia, OptionFWitness};
    ///
    /// let x: LiberEcclesia<OptionFWitness, i32> = LiberEcclesia::purus(42);
    /// assert!(x.est_purus());
    /// ```
    #[inline(always)]
    pub fn purus(a: A) -> Self {
        LiberEcclesia::Purus(a)
    }

    /// Check if this is a pure value.
    #[inline(always)]
    pub fn est_purus(&self) -> bool {
        matches!(self, LiberEcclesia::Purus(_))
    }

    /// Check if this is a suspended computation.
    #[inline(always)]
    pub fn est_suspensus(&self) -> bool {
        matches!(self, LiberEcclesia::Suspensus { .. })
    }

    /// Extract the value if this is a pure computation.
    ///
    /// Returns `None` if the computation is suspended (impure).
    #[inline(always)]
    pub fn extract_pure(self) -> Option<A> {
        match self {
            LiberEcclesia::Purus(a) => Some(a),
            LiberEcclesia::Suspensus { .. } => None,
        }
    }

    /// Run a pure computation, panicking if it's impure.
    ///
    /// # Panics
    ///
    /// Panics if the computation is `Suspensus` (it still contains at least
    /// one unhandled effect). Use [`Self::extract_pure`] for a non-panicking
    /// variant.
    #[inline(always)]
    pub fn run_pure(self) -> A {
        match self {
            LiberEcclesia::Purus(a) => a,
            LiberEcclesia::Suspensus { .. } => {
                panic!("Cannot run impure LiberEcclesia as pure")
            }
        }
    }
}

#[cfg(feature = "alloc")]
impl<F: FunctorHKT + 'static, A: Send + Sync + 'static> LiberEcclesia<F, A> {
    /// Map a function over the result type.
    ///
    /// Time complexity: O(1) - just appends to continuation stack.
    #[inline(always)]
    pub fn map<B: Send + Sync + 'static, G>(self, f: G) -> LiberEcclesia<F, B>
    where
        G: Fn(A) -> B + Send + Sync + 'static,
    {
        self.flat_map(move |a| LiberEcclesia::purus(f(a)))
    }

    /// Monadic bind (flatMap) with O(1) amortized performance.
    ///
    /// This operation appends to the continuation stack instead of
    /// traversing the structure, achieving O(1) amortized time.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::free::{LiberEcclesia, OptionFWitness};
    ///
    /// let program: LiberEcclesia<OptionFWitness, i32> = LiberEcclesia::purus(42)
    ///     .flat_map(|x| LiberEcclesia::purus(x + 1))
    ///     .flat_map(|x| LiberEcclesia::purus(x * 2));
    /// assert_eq!(program.run_pure(), 86);
    /// ```
    #[inline(always)]
    pub fn flat_map<B: Send + Sync + 'static, G>(self, f: G) -> LiberEcclesia<F, B>
    where
        G: Fn(A) -> LiberEcclesia<F, B> + Send + Sync + 'static,
    {
        match self {
            LiberEcclesia::Purus(a) => f(a),
            LiberEcclesia::Suspensus { effect, mut stack } => {
                // Append the new continuation to the stack - O(1) amortized
                stack.push(Box::new(ContinuatioStepImpl {
                    f,
                    _phantom: core::marker::PhantomData,
                }));

                LiberEcclesia::Suspensus { effect, stack }
            }
        }
    }

    /// Lift a functor value into the Church-encoded Free monad.
    ///
    /// This is the fundamental operation for building Free monad programs.
    #[inline(always)]
    pub fn lift_f<X: Send + Sync + 'static>(fx: F::Target<X>) -> LiberEcclesia<F, X>
    where
        F::Target<X>: Send + Sync,
    {
        LiberEcclesia::Suspensus {
            effect: Box::new(fx),
            stack: ContinuatioStack::empty(),
        }
    }
}

// =============================================================================
// Codensity Transform - Pure Rust Implementation
// =============================================================================

/// Codensity monad for guaranteed O(1) bind performance.
///
/// The Codensity monad transforms any monad to have O(1) bind operations
/// by using continuation-passing style.
///
/// This is a simpler version that works with concrete types rather than
/// trait objects, avoiding the dyn-compatibility issues.
///
/// # Latin Etymology
///
/// *Densitas* = density, thickness (Co-density = dual of density)
///
/// # Example
///
/// ```rust
/// use ordofp_core::free::*;
///
/// // Create a Codensity-wrapped computation
/// let cod = CodOption::purus(42);
///
/// // Chain with guaranteed O(1) per operation
/// let result = cod
///     .flat_map(|x| CodOption::purus(x + 1))
///     .flat_map(|x| CodOption::purus(x * 2))
///     .lower(); // Convert back to Option
/// assert_eq!(result, Some(86));
/// ```
#[cfg(feature = "alloc")]
pub struct CodOption<A> {
    /// The computation represented as a continuation.
    /// For pure values, we store the value directly for efficiency.
    inner: CodOptionInner<A>,
}

/// Type-erased resumption for `CodOption`'s CPS encoding: consumes a boxed
/// intermediate value and yields the next boxed value (or `None` for failure).
#[cfg(feature = "alloc")]
type CodOptionKont<'a> = dyn Fn(Box<dyn core::any::Any>) -> Option<Box<dyn core::any::Any>> + 'a;

/// A suspended `CodOption` computation: given a resumption, runs to completion.
#[cfg(feature = "alloc")]
type CodOptionRun =
    Box<dyn for<'a> FnOnce(&'a CodOptionKont<'a>) -> Option<Box<dyn core::any::Any>> + Send>;

#[cfg(feature = "alloc")]
enum CodOptionInner<A> {
    /// A pure value (optimization for the common case)
    Pure(A),
    /// A composed continuation
    Composed(CodOptionRun),
}

#[cfg(feature = "alloc")]
impl<A: Clone + Send + 'static> CodOption<A> {
    /// Create a pure Codensity value.
    #[inline(always)]
    pub fn purus(a: A) -> Self {
        CodOption {
            inner: CodOptionInner::Pure(a),
        }
    }

    /// Monadic bind with O(1) performance.
    #[inline(always)]
    pub fn flat_map<B: Clone + Send + 'static, G>(self, f: G) -> CodOption<B>
    where
        G: Fn(A) -> CodOption<B> + Send + 'static,
    {
        match self.inner {
            CodOptionInner::Pure(a) => f(a),
            CodOptionInner::Composed(run) => CodOption {
                inner: CodOptionInner::Composed(Box::new(move |k| {
                    run(&|any_val| {
                        let val: A = *any_val.downcast().ok()?;
                        let next = f(val);
                        match next.inner {
                            CodOptionInner::Pure(b) => k(Box::new(b)),
                            CodOptionInner::Composed(next_run) => next_run(k),
                        }
                    })
                })),
            },
        }
    }

    /// Map a function over the value.
    #[inline(always)]
    pub fn map<B: Clone + Send + 'static, G>(self, f: G) -> CodOption<B>
    where
        G: Fn(A) -> B + Send + 'static,
    {
        self.flat_map(move |a| CodOption::purus(f(a)))
    }

    /// Lower back to Option.
    #[inline(always)]
    pub fn lower(self) -> Option<A> {
        match self.inner {
            CodOptionInner::Pure(a) => Some(a),
            CodOptionInner::Composed(run) => run(&|any_val| Some(any_val))
                .and_then(|boxed| boxed.downcast::<A>().ok())
                .map(|boxed| *boxed),
        }
    }

    /// Create a None value in Codensity.
    #[inline(always)]
    pub fn none() -> Self {
        // Use a sentinel value that will be converted to None on lower
        CodOption {
            inner: CodOptionInner::Composed(Box::new(|_k| None)),
        }
    }

    /// Lift an Option into Codensity.
    #[inline(always)]
    pub fn from_option(opt: Option<A>) -> Self {
        match opt {
            Some(a) => Self::purus(a),
            None => Self::none(),
        }
    }

    /// Filter values based on a predicate.
    #[inline(always)]
    pub fn filter<P>(self, predicate: P) -> Self
    where
        P: Fn(&A) -> bool + Send + 'static,
    {
        self.flat_map(move |a| {
            if predicate(&a) {
                CodOption::purus(a)
            } else {
                CodOption::none()
            }
        })
    }

    /// Get the value or a default.
    #[inline(always)]
    pub fn get_or_else<F>(self, default: F) -> A
    where
        F: FnOnce() -> A,
    {
        self.lower().unwrap_or_else(default)
    }

    /// Combine with another `CodOption` using a function.
    #[inline(always)]
    pub fn map2<B, C, F>(self, other: CodOption<B>, f: F) -> CodOption<C>
    where
        B: Clone + Send + 'static,
        C: Clone + Send + 'static,
        F: Fn(A, B) -> C + Send + 'static + Clone,
    {
        self.flat_map(move |a| {
            let f = f.clone();
            other.clone().map(move |b| f(a.clone(), b))
        })
    }
}

#[cfg(feature = "alloc")]
impl<A: Clone + Send + 'static> Clone for CodOption<A> {
    /// # Panics
    ///
    /// Panics if the computation is in the composed (CPS) state — only
    /// pure (`purus`) values can be cloned with the current design.
    fn clone(&self) -> Self {
        match &self.inner {
            CodOptionInner::Pure(a) => CodOption::purus(a.clone()),
            CodOptionInner::Composed(_) => {
                // For composed values, we can't easily clone
                // This is a limitation of the current design
                panic!("Cannot clone composed CodOption")
            }
        }
    }
}

/// Codensity monad specialized for Result.
///
/// This is a simplified Codensity representation optimized for the common case
/// where values are either pure successes or errors. For Result types,
/// direct pattern matching is efficient, so the full CPS transformation
/// provides little benefit while adding complexity.
#[cfg(feature = "alloc")]
pub struct CodResult<A, E> {
    inner: CodResultInner<A, E>,
}

#[cfg(feature = "alloc")]
enum CodResultInner<A, E> {
    Pure(A),
    Error(E),
}

#[cfg(feature = "alloc")]
impl<A: Clone + Send + 'static, E: Clone + Send + 'static> CodResult<A, E> {
    /// Create a pure Ok value.
    #[inline(always)]
    pub fn ok(a: A) -> Self {
        CodResult {
            inner: CodResultInner::Pure(a),
        }
    }

    /// Create an Err value.
    #[inline(always)]
    pub fn err(e: E) -> Self {
        CodResult {
            inner: CodResultInner::Error(e),
        }
    }

    /// Monadic bind with O(1) performance.
    #[inline(always)]
    pub fn flat_map<B: Clone + Send + 'static, G>(self, f: G) -> CodResult<B, E>
    where
        G: Fn(A) -> CodResult<B, E> + Send + 'static,
    {
        match self.inner {
            CodResultInner::Pure(a) => f(a),
            CodResultInner::Error(e) => CodResult {
                inner: CodResultInner::Error(e),
            },
        }
    }

    /// Map a function over the value.
    #[inline(always)]
    pub fn map<B: Clone + Send + 'static, G>(self, f: G) -> CodResult<B, E>
    where
        G: Fn(A) -> B + Send + 'static,
    {
        self.flat_map(move |a| CodResult::ok(f(a)))
    }

    /// Lower back to Result.
    ///
    /// # Errors
    ///
    /// Returns `Err` carrying the stored error exactly when the codensity
    /// computation short-circuited (was constructed via `err` or an earlier
    /// step failed); a pure value lowers to `Ok`.
    #[inline(always)]
    pub fn lower(self) -> Result<A, E> {
        match self.inner {
            CodResultInner::Pure(a) => Ok(a),
            CodResultInner::Error(e) => Err(e),
        }
    }

    /// Lift a Result into Codensity.
    #[inline(always)]
    pub fn from_result(result: Result<A, E>) -> Self {
        match result {
            Ok(a) => Self::ok(a),
            Err(e) => Self::err(e),
        }
    }

    /// Map the error type.
    #[inline(always)]
    pub fn map_err<E2: Clone + Send + 'static, G>(self, f: G) -> CodResult<A, E2>
    where
        G: Fn(E) -> E2 + Send + 'static,
    {
        match self.inner {
            CodResultInner::Pure(a) => CodResult::ok(a),
            CodResultInner::Error(e) => CodResult::err(f(e)),
        }
    }

    /// Get the Ok value or transform the error.
    #[inline(always)]
    pub fn unwrap_or_else<F>(self, f: F) -> A
    where
        F: FnOnce(E) -> A,
    {
        self.lower().unwrap_or_else(f)
    }

    /// Combine with another `CodResult`.
    #[inline(always)]
    pub fn and_then<B: Clone + Send + 'static, F>(self, f: F) -> CodResult<B, E>
    where
        F: Fn(A) -> CodResult<B, E> + Send + 'static,
    {
        self.flat_map(f)
    }
}

/// Codensity monad specialized for Identity (pure computations).
///
/// This is useful for demonstrating O(n) performance without any
/// actual effects.
#[cfg(feature = "alloc")]
pub struct CodIdentity<A> {
    value: A,
}

#[cfg(feature = "alloc")]
impl<A: Clone + 'static> CodIdentity<A> {
    /// Create a pure value.
    #[inline(always)]
    pub fn purus(a: A) -> Self {
        CodIdentity { value: a }
    }

    /// Monadic bind.
    #[inline(always)]
    pub fn flat_map<B: Clone + 'static, G>(self, f: G) -> CodIdentity<B>
    where
        G: FnOnce(A) -> CodIdentity<B>,
    {
        f(self.value)
    }

    /// Map a function.
    #[inline(always)]
    pub fn map<B: Clone + 'static, G>(self, f: G) -> CodIdentity<B>
    where
        G: FnOnce(A) -> B,
    {
        CodIdentity {
            value: f(self.value),
        }
    }

    /// Extract the value.
    #[inline(always)]
    pub fn run(self) -> A {
        self.value
    }
}

// =============================================================================
// Conversion Utilities
// =============================================================================

/// Convert a standard Free monad to Church-encoded form.
///
/// This is useful when you have an existing `Liber` and want O(n) performance
/// for subsequent left-associated binds.
///
/// This is the standard fold from the initial encoding into the Church
/// encoding: `Purus` maps to `purus`, and each `Suspensus` layer maps the
/// conversion over the functor and re-wraps via `lift_f` + `flat_map`
/// (`fromFree (Free fa) = wrap (fmap fromFree fa)`).
///
/// Recursion depth equals the `Suspensus` nesting depth of the input —
/// the same bound as `Liber::map` itself.
#[cfg(feature = "alloc")]
pub fn ad_ecclesiam<F, A>(liber: Liber<F, A>) -> LiberEcclesia<F, A>
where
    F: FunctorHKT + 'static,
    A: Send + Sync + 'static,
    F::Target<LiberEcclesia<F, A>>: Send + Sync,
    LiberEcclesia<F, A>: Send + Sync,
{
    match liber {
        Liber::Purus(a) => LiberEcclesia::purus(a),
        Liber::Suspensus(fa) => {
            LiberEcclesia::<F, A>::lift_f(F::map(*fa, ad_ecclesiam)).flat_map(|inner| inner)
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::super::nat::OptionFWitness;
    use super::*;
    use alloc::vec;
    use alloc::vec::Vec;

    #[test]
    fn test_purus() {
        let free: LiberEcclesia<OptionFWitness, i32> = LiberEcclesia::purus(42);
        assert!(free.est_purus());
        let extracted = free.extract_pure();
        assert_eq!(extracted, Some(42));
    }

    #[test]
    fn test_flat_map_purus() {
        let free: LiberEcclesia<OptionFWitness, i32> = LiberEcclesia::purus(42);
        let chained = free.flat_map(|x| LiberEcclesia::purus(x + 1));
        let extracted = chained.extract_pure();
        assert_eq!(extracted, Some(43));
    }

    #[test]
    fn test_map_purus() {
        let free: LiberEcclesia<OptionFWitness, i32> = LiberEcclesia::purus(42);
        let mapped = free.map(|x| x * 2);
        let extracted = mapped.extract_pure();
        assert_eq!(extracted, Some(84));
    }

    #[test]
    fn test_chain_many_operations() {
        // This would be O(n²) with standard Liber, but O(n) with LiberEcclesia
        let free: LiberEcclesia<OptionFWitness, i32> = LiberEcclesia::purus(0);
        let result = (0..100).fold(free, |acc, i| {
            acc.flat_map(move |x| LiberEcclesia::purus(x + i))
        });
        let extracted = result.extract_pure();
        // Sum of 0..100 = 4950
        assert_eq!(extracted, Some(4950));
    }

    #[test]
    fn test_monad_left_identity() {
        // pure a >>= f  ≡  f a
        let a = 42;
        let f = |x: i32| LiberEcclesia::<OptionFWitness, i32>::purus(x * 2);

        let left = LiberEcclesia::purus(a).flat_map(f);
        let right = f(a);

        assert_eq!(left.extract_pure(), right.extract_pure());
    }

    #[test]
    fn test_monad_right_identity() {
        // m >>= pure  ≡  m
        let m: LiberEcclesia<OptionFWitness, i32> = LiberEcclesia::purus(42);
        let result = m.flat_map(LiberEcclesia::purus);

        assert_eq!(result.extract_pure(), Some(42));
    }

    // CodOption tests

    #[test]
    fn test_cod_option_purus() {
        let cod: CodOption<i32> = CodOption::purus(42);
        let result = cod.lower();
        assert_eq!(result, Some(42));
    }

    #[test]
    fn test_cod_option_flat_map() {
        let cod: CodOption<i32> = CodOption::purus(42);
        let chained = cod.flat_map(|x| CodOption::purus(x + 1));
        let result = chained.lower();
        assert_eq!(result, Some(43));
    }

    #[test]
    fn test_cod_option_map() {
        let cod: CodOption<i32> = CodOption::purus(42);
        let mapped = cod.map(|x| x * 2);
        let result = mapped.lower();
        assert_eq!(result, Some(84));
    }

    #[test]
    fn test_cod_option_chain_many() {
        // Verify O(n) performance for many chained operations
        let cod: CodOption<i32> = CodOption::purus(0);
        let result = (0..100).fold(cod, |acc, i| acc.flat_map(move |x| CodOption::purus(x + i)));
        let lowered = result.lower();
        assert_eq!(lowered, Some(4950));
    }

    // CodResult tests

    #[test]
    fn test_cod_result_ok() {
        let cod: CodResult<i32, &str> = CodResult::ok(42);
        let result = cod.lower();
        assert_eq!(result, Ok(42));
    }

    #[test]
    fn test_cod_result_err() {
        let cod: CodResult<i32, &str> = CodResult::err("error");
        let result = cod.lower();
        assert_eq!(result, Err("error"));
    }

    #[test]
    fn test_cod_result_flat_map() {
        let cod: CodResult<i32, &str> = CodResult::ok(42);
        let chained = cod.flat_map(|x| CodResult::ok(x + 1));
        let result = chained.lower();
        assert_eq!(result, Ok(43));
    }

    #[test]
    fn test_cod_result_chain_many() {
        let cod: CodResult<i32, &str> = CodResult::ok(0);
        let result = (0..100).fold(cod, |acc, i| acc.flat_map(move |x| CodResult::ok(x + i)));
        let lowered = result.lower();
        assert_eq!(lowered, Ok(4950));
    }

    // CodIdentity tests

    #[test]
    fn test_cod_identity_purus() {
        let cod: CodIdentity<i32> = CodIdentity::purus(42);
        let result = cod.run();
        assert_eq!(result, 42);
    }

    #[test]
    fn test_cod_identity_flat_map() {
        let cod: CodIdentity<i32> = CodIdentity::purus(42);
        let chained = cod.flat_map(|x| CodIdentity::purus(x + 1));
        let result = chained.run();
        assert_eq!(result, 43);
    }

    #[test]
    fn test_cod_identity_chain_many() {
        let cod: CodIdentity<i32> = CodIdentity::purus(0);
        let result = (0..100).fold(cod, |acc, i| {
            acc.flat_map(move |x| CodIdentity::purus(x + i))
        });
        let lowered = result.run();
        assert_eq!(lowered, 4950);
    }

    #[test]
    fn test_cod_identity_map() {
        let cod: CodIdentity<i32> = CodIdentity::purus(42);
        let result = cod
            .flat_map(|x| CodIdentity::purus(x + 8))
            .map(|x| x * 2)
            .run();
        assert_eq!(result, 100);
    }

    // ==========================================================================
    // AcervusParvus (SmallStack) tests
    // ==========================================================================

    #[test]
    fn test_acervus_parvus_empty() {
        let stack: AcervusParvus<i32> = AcervusParvus::empty();
        assert!(stack.is_empty());
        assert_eq!(stack.len(), 0);
        assert!(stack.is_inline());
    }

    #[test]
    fn test_acervus_parvus_push_inline() {
        let mut stack: AcervusParvus<i32> = AcervusParvus::empty();

        // Push up to inline capacity
        for i in 0..INLINE_CAPACITY {
            stack.push(i as i32);
            assert!(stack.is_inline(), "Should remain inline at {i}");
        }

        assert_eq!(stack.len(), INLINE_CAPACITY);
        assert!(stack.is_inline());
    }

    #[test]
    fn test_acervus_parvus_push_spill() {
        let mut stack: AcervusParvus<i32> = AcervusParvus::empty();

        // Push past inline capacity
        for i in 0..=INLINE_CAPACITY {
            stack.push(i as i32);
        }

        assert_eq!(stack.len(), INLINE_CAPACITY + 1);
        assert!(!stack.is_inline(), "Should have spilled to heap");
    }

    #[test]
    fn test_acervus_parvus_pop() {
        let mut stack: AcervusParvus<i32> = AcervusParvus::empty();
        stack.push(1);
        stack.push(2);
        stack.push(3);

        assert_eq!(stack.pop(), Some(3));
        assert_eq!(stack.pop(), Some(2));
        assert_eq!(stack.pop(), Some(1));
        assert_eq!(stack.pop(), None);
    }

    #[test]
    fn test_acervus_parvus_iter() {
        let mut stack: AcervusParvus<i32> = AcervusParvus::empty();
        stack.push(1);
        stack.push(2);
        stack.push(3);

        let collected: Vec<i32> = stack.iter().copied().collect();
        assert_eq!(collected, vec![1, 2, 3]);
    }

    #[test]
    fn test_acervus_parvus_into_iter() {
        let mut stack: AcervusParvus<i32> = AcervusParvus::empty();
        stack.push(1);
        stack.push(2);
        stack.push(3);

        let collected: Vec<i32> = stack.into_iter().collect();
        assert_eq!(collected, vec![1, 2, 3]);
    }

    #[test]
    fn test_continuation_stack_inline() {
        // Verify that ContinuatioStack uses inline storage
        let stack: ContinuatioStack<OptionFWitness> = ContinuatioStack::empty();
        assert!(stack.is_empty());
        assert!(stack.is_inline());
    }

    #[test]
    fn test_liber_ecclesia_inline_storage() {
        // Small chains should use inline storage
        let free: LiberEcclesia<OptionFWitness, i32> = LiberEcclesia::purus(0);

        // Chain up to inline capacity operations
        let result = (0..INLINE_CAPACITY).fold(free, |acc, i| {
            acc.flat_map(move |x| LiberEcclesia::purus(x + i as i32))
        });

        // Should still work correctly
        assert_eq!(
            result.extract_pure(),
            Some((0..INLINE_CAPACITY as i32).sum())
        );
    }

    #[test]
    fn test_ad_ecclesiam_purus() {
        let liber: Liber<OptionFWitness, i32> = Liber::purus(42);
        assert_eq!(ad_ecclesiam(liber).run_pure(), 42);
    }

    #[test]
    fn test_ad_ecclesiam_suspensus_roundtrip() {
        // One functor layer: Some(pure 41).
        let liber: Liber<OptionFWitness, i32> = Liber::suspensus(Some(Liber::purus(41)));
        let ecclesia = ad_ecclesiam(liber);

        // The converted form is lift_f(F::map(fa, convert)).flat_map(id):
        // effect holds Option<LiberEcclesia<_, i32>>, stack holds the one
        // identity bind.
        match ecclesia {
            LiberEcclesia::Suspensus { effect, stack } => {
                assert_eq!(stack.len(), 1);
                let inner: Option<LiberEcclesia<OptionFWitness, i32>> = *effect
                    .downcast()
                    .expect("effect must hold the mapped functor layer");
                let inner = inner.expect("the Some layer is preserved");
                assert_eq!(inner.run_pure(), 41);
            }
            LiberEcclesia::Purus(_) => panic!("suspended input must stay suspended"),
        }
    }

    #[test]
    fn test_ad_ecclesiam_nested_suspensus() {
        // Two nested layers: Some(Some(pure 7)) — exercises the recursion.
        let liber: Liber<OptionFWitness, i32> =
            Liber::suspensus(Some(Liber::suspensus(Some(Liber::purus(7)))));

        let LiberEcclesia::Suspensus { effect, .. } = ad_ecclesiam(liber) else {
            panic!("layer 1 must be suspended");
        };
        let inner1: Option<LiberEcclesia<OptionFWitness, i32>> =
            *effect.downcast().expect("layer 1 effect type");
        let LiberEcclesia::Suspensus { effect, .. } = inner1.expect("Some preserved") else {
            panic!("layer 2 must be suspended");
        };
        let inner2: Option<LiberEcclesia<OptionFWitness, i32>> =
            *effect.downcast().expect("layer 2 effect type");
        assert_eq!(inner2.expect("Some preserved").run_pure(), 7);
    }
}
