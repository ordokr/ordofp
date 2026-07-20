//! Effect-Aware Fusion
//!
//! This module provides **explicit** fusion combinators that combine multiple
//! operations into single, more efficient operations. The type system enforces
//! that fusion is only allowed when effects commute.
//!
//! # Fusion Rules
//!
//! Fusion is possible when effects **commute** - meaning the order of execution
//! doesn't affect the result. The type system enforces this property.
//!
//! ## Examples of Fusible Operations
//!
//! - Multiple `Reader` operations → Single environment access
//! - Multiple `map` operations → Single function composition
//! - Adjacent `filter` and `map` → Single pass
//!
//! ## Examples of Non-Fusible Operations
//!
//! - `State` followed by `State` → Order matters
//! - `Writer` followed by `Writer` → Log order matters
//! - `IO` operations → Side effects can't be reordered
//!
//! # Example
//!
//! ```rust
//! use ordofp_core::nexus::pure;
//! use ordofp_core::nexus::optim::*;
//!
//! // These pure operations can be fused
//! let result = fuse(
//!     pure(5).map(|x| x + 1),
//!     pure(3).map(|x| x * 2),
//!     |a, b| a + b,
//! );
//! assert_eq!(result.run_pure(), 12);
//! ```

use core::marker::PhantomData;

use crate::nexus::effect::Eff;
use crate::nexus::row::READER_BIT;
use crate::nexus::row::{EffectRow, Pure, Row};

// =============================================================================
// Fusion Marker Traits
// =============================================================================

/// Marker trait for effects that can be fused together.
///
/// Two effects can be fused if they commute - meaning their
/// execution order doesn't affect the final result.
pub trait CanFuse<E2>: EffectRow
where
    E2: EffectRow,
{
    /// The resulting effect row after fusion.
    type Fused: EffectRow;
}

/// Pure effects can be fused with anything.
impl<E2: EffectRow> CanFuse<E2> for Pure {
    type Fused = E2;
}

/// Reader effects can be fused with Reader effects.
impl CanFuse<Row<READER_BIT>> for Row<READER_BIT> {
    type Fused = Row<READER_BIT>;
}

// =============================================================================
// Fusion Rules
// =============================================================================

/// A fusion rule that combines two operations.
pub trait FusionRule<A, B, C> {
    /// The fused operation.
    fn fuse(a: A, b: B) -> C;
}

/// Identity fusion - no actual fusion, just combine results.
pub struct IdentityFusion;

impl<A, B> FusionRule<A, B, (A, B)> for IdentityFusion {
    fn fuse(a: A, b: B) -> (A, B) {
        (a, b)
    }
}

/// Map fusion - compose two mapping functions.
///
/// This is a marker type for map fusion rules.
/// Use `compose_maps` for the actual function composition.
pub struct MapFusion;

/// Filter-map fusion - combine filter and map into single pass.
pub struct FilterMapFusion;

// =============================================================================
// Fusion Combinators
// =============================================================================

/// Fuse two pure computations with a combining function.
///
/// Since both computations are pure, they can be executed in any order
/// and their results combined.
///
/// # Example
///
/// ```rust
/// use ordofp_core::nexus::pure;
/// use ordofp_core::nexus::optim::fusion::fuse;
///
/// let result = fuse(
///     pure(5),
///     pure(3),
///     |a, b| a + b,
/// );
/// assert_eq!(result.run_pure(), 8);
/// ```
#[inline]
pub fn fuse<A, B, C, F>(first: Eff<Pure, A>, second: Eff<Pure, B>, combine: F) -> Eff<Pure, C>
where
    F: FnOnce(A, B) -> C,
{
    let a = first.run_pure();
    let b = second.run_pure();
    Eff::from_value(combine(a, b))
}

// =============================================================================
// Map Fusion
// =============================================================================

/// Fuse multiple map operations into a single traversal.
///
/// Instead of: items.map(f).map(g).map(h)
/// Compute:    items.map(|x| h(g(f(x))))
///
/// # Example
///
/// ```rust
/// use ordofp_core::nexus::optim::fusion::fused_map;
///
/// let items = vec![1, 2, 3];
/// let result = fused_map(items, |x| {
///     let a = x + 1;
///     let b = a * 2;
///     let c = b.to_string();
///     c
/// });
/// assert_eq!(result, vec!["4".to_string(), "6".to_string(), "8".to_string()]);
/// ```
#[inline]
pub fn fused_map<I, A, B, F>(items: I, f: F) -> alloc::vec::Vec<B>
where
    I: IntoIterator<Item = A>,
    F: Fn(A) -> B,
{
    items.into_iter().map(f).collect()
}

/// Compose multiple functions for fused mapping.
///
/// # Example
///
/// ```rust
/// use ordofp_core::nexus::optim::fusion::compose_maps;
///
/// let composed = compose_maps(|x: i32| x + 1, |x| x * 2);
/// assert_eq!(composed(5), 12); // (5 + 1) * 2
/// ```
#[inline]
pub fn compose_maps<A, B, C, F, G>(f: F, g: G) -> impl Fn(A) -> C
where
    F: Fn(A) -> B,
    G: Fn(B) -> C,
{
    move |a| g(f(a))
}

/// Compose three functions for fused mapping in a single pass.
///
/// Equivalent to `|a| h(g(f(a)))` but expressed as a combinator.  Useful when
/// you want to describe a three-stage transformation pipeline and pass it as a
/// single callable.
///
/// # Example
///
/// ```rust
/// use ordofp_core::nexus::optim::fusion::compose_maps3;
///
/// let pipeline = compose_maps3(
///     |x: i32| x + 1,        // stage 1: increment
///     |x: i32| x * 2,        // stage 2: double
///     |x: i32| x.to_string(),// stage 3: stringify
/// );
/// assert_eq!(pipeline(5), "12"); // (5 + 1) * 2 = 12
/// ```
#[inline]
pub fn compose_maps3<A, B, C, D, F, G, H>(f: F, g: G, h: H) -> impl Fn(A) -> D
where
    F: Fn(A) -> B,
    G: Fn(B) -> C,
    H: Fn(C) -> D,
{
    move |a| h(g(f(a)))
}

// =============================================================================
// Filter-Map Fusion
// =============================================================================

/// Fuse filter and map into a single pass.
///
/// # Example
///
/// ```rust
/// use ordofp_core::nexus::optim::fusion::filter_map_fused;
///
/// let items = vec![1, 2, 3, 4, 5];
/// let result = filter_map_fused(
///     items,
///     |x| x % 2 == 0,  // Keep evens
///     |x| x * 10,       // Multiply by 10
/// );
/// assert_eq!(result, vec![20, 40]);
/// ```
#[inline]
pub fn filter_map_fused<I, A, B, P, F>(items: I, predicate: P, transform: F) -> alloc::vec::Vec<B>
where
    I: IntoIterator<Item = A>,
    P: Fn(&A) -> bool,
    F: Fn(A) -> B,
{
    items
        .into_iter()
        .filter(|x| predicate(x))
        .map(transform)
        .collect()
}

/// Combined filter-map operation (like Rust's `filter_map`).
///
/// Applies `f` to each element of `items`, collecting only the `Some` values
/// into a new `Vec`. Filter and transform happen in a single pass with no
/// intermediate allocation, unlike chaining `.filter(..).map(..)`.
///
/// # Parameters
///
/// * `items` – Any iterable of elements of type `A`.
/// * `f` – A function that returns `Some(B)` to keep a transformed value, or
///   `None` to discard the element.
///
/// # Example
///
/// ```rust
/// use ordofp_core::nexus::optim::fusion::fused_filter_map;
///
/// let nums = vec![1i32, -2, 3, -4, 5];
/// let positive_squares = fused_filter_map(nums, |x| {
///     if x > 0 { Some(x * x) } else { None }
/// });
/// assert_eq!(positive_squares, vec![1, 9, 25]);
/// ```
#[inline]
pub fn fused_filter_map<I, A, B, F>(items: I, f: F) -> alloc::vec::Vec<B>
where
    I: IntoIterator<Item = A>,
    F: Fn(A) -> Option<B>,
{
    items.into_iter().filter_map(f).collect()
}

// =============================================================================
// Fold Fusion
// =============================================================================

/// Fuse map and fold into single traversal.
///
/// Instead of: items.map(f).fold(init, combine)
/// Compute:    items.fold(init, |acc, x| combine(acc, f(x)))
///
/// # Example
///
/// ```rust
/// use ordofp_core::nexus::optim::fusion::map_fold_fused;
///
/// let items = vec![1, 2, 3];
/// let sum_of_squares = map_fold_fused(
///     items,
///     |x| x * x,
///     0,
///     |acc, sq| acc + sq,
/// );
/// assert_eq!(sum_of_squares, 14); // 1 + 4 + 9
/// ```
#[inline]
pub fn map_fold_fused<I, A, B, C, M, F>(items: I, map: M, init: C, fold: F) -> C
where
    I: IntoIterator<Item = A>,
    M: Fn(A) -> B,
    F: Fn(C, B) -> C,
{
    items.into_iter().fold(init, |acc, x| fold(acc, map(x)))
}

// =============================================================================
// Reader Fusion
// =============================================================================

/// Extract two values from a reader environment in a single access.
///
/// Combines two projection functions `f1` and `f2` into one closure that
/// reads `env` once and returns both results as a tuple. This avoids the
/// overhead of two separate reader computations and keeps related projections
/// co-located at the call site.
///
/// For a three-value variant see [`reader_extract3`].
///
/// # Type Parameters
///
/// * `E` – The shared environment type (e.g. a configuration struct).
/// * `A` – Type of the first extracted value.
/// * `B` – Type of the second extracted value.
/// * `F1` – Projection closure `&E -> A`.
/// * `F2` – Projection closure `&E -> B`.
///
/// # Example
///
/// ```rust
/// use ordofp_core::nexus::optim::fusion::reader_extract2;
///
/// struct Config { host: String, port: u16 }
///
/// let extract = reader_extract2(
///     |cfg: &Config| cfg.host.clone(),
///     |cfg: &Config| cfg.port,
/// );
///
/// let cfg = Config { host: "localhost".into(), port: 8080 };
/// assert_eq!(extract(&cfg), ("localhost".to_string(), 8080));
/// ```
#[inline]
pub fn reader_extract2<E, A, B, F1, F2>(f1: F1, f2: F2) -> impl Fn(&E) -> (A, B)
where
    F1: Fn(&E) -> A,
    F2: Fn(&E) -> B,
{
    move |env| (f1(env), f2(env))
}

/// Extract three values from a reader environment.
#[inline]
pub fn reader_extract3<E, A, B, C, F1, F2, F3>(f1: F1, f2: F2, f3: F3) -> impl Fn(&E) -> (A, B, C)
where
    F1: Fn(&E) -> A,
    F2: Fn(&E) -> B,
    F3: Fn(&E) -> C,
{
    move |env| (f1(env), f2(env), f3(env))
}

// =============================================================================
// Writer Fusion
// =============================================================================

/// Fuse multiple writer operations into batched writes.
///
/// Collect all log entries and write them in a single operation.
pub struct WriterBatch<W> {
    entries: alloc::vec::Vec<W>,
}

impl<W> WriterBatch<W> {
    /// Create a new empty batch.
    #[inline]
    pub fn new() -> Self {
        WriterBatch {
            entries: alloc::vec::Vec::new(),
        }
    }

    /// Create a new batch with a pre-allocated capacity.
    #[inline]
    pub fn with_capacity(cap: usize) -> Self {
        WriterBatch {
            entries: alloc::vec::Vec::with_capacity(cap),
        }
    }

    /// Add an entry to the batch.
    #[inline]
    pub fn add(&mut self, entry: W) {
        self.entries.push(entry);
    }

    /// Get all entries.
    #[inline]
    pub fn entries(self) -> alloc::vec::Vec<W> {
        self.entries
    }

    /// Number of entries.
    #[inline]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl<W> Default for WriterBatch<W> {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Fusion Proof Types
// =============================================================================

/// Proof that two effect rows can be fused.
pub struct FusionProof<R1: EffectRow, R2: EffectRow> {
    _marker: PhantomData<(R1, R2)>,
}

impl<R1: EffectRow, R2: EffectRow> FusionProof<R1, R2> {
    /// Create a fusion proof.
    ///
    /// # Safety
    ///
    /// Only create when effects actually commute.
    pub const unsafe fn new() -> Self {
        FusionProof {
            _marker: PhantomData,
        }
    }
}

/// Prove fusion is safe for pure effects.
pub fn prove_pure_fusion() -> FusionProof<Pure, Pure> {
    // SAFETY: `Pure` effects carry no observable side effects and impose no
    // ordering constraints on each other, so fusing two `Pure` effect rows is
    // always sound. The `FusionProof` invariant ("only create when effects
    // actually commute") is satisfied unconditionally for `Pure × Pure`.
    unsafe { FusionProof::new() }
}

// =============================================================================
// Optimization Pipeline
// =============================================================================

/// An optimization pipeline that applies fusion rules.
pub struct FusionPipeline<A> {
    value: A,
}

impl<A> FusionPipeline<A> {
    /// Create a new pipeline.
    #[inline]
    pub fn new(value: A) -> Self {
        FusionPipeline { value }
    }

    /// Apply a transformation.
    #[inline]
    pub fn map<B, F: FnOnce(A) -> B>(self, f: F) -> FusionPipeline<B> {
        FusionPipeline {
            value: f(self.value),
        }
    }

    /// Finish the pipeline and extract the value.
    #[inline]
    pub fn finish(self) -> A {
        self.value
    }
}

/// Create a fusion pipeline.
#[inline]
pub fn pipeline<A>(value: A) -> FusionPipeline<A> {
    FusionPipeline::new(value)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nexus::ops::pure;
    use alloc::string::ToString;

    #[test]
    fn test_fuse_pure() {
        let result = fuse(pure(5), pure(3), |a, b| a + b);
        assert_eq!(result.run_pure(), 8);
    }

    #[test]
    fn test_compose_maps() {
        let f = compose_maps(|x: i32| x + 1, |x| x * 2);
        assert_eq!(f(5), 12);
    }

    #[test]
    fn test_compose_maps3() {
        let f = compose_maps3(|x: i32| x + 1, |x| x * 2, |x: i32| x.to_string());
        assert_eq!(f(5), "12");
    }

    #[test]
    fn test_fused_map() {
        let items = alloc::vec![1, 2, 3];
        let result = fused_map(items, |x| (x + 1) * 2);
        assert_eq!(result, alloc::vec![4, 6, 8]);
    }

    #[test]
    fn test_filter_map_fused() {
        let items = alloc::vec![1, 2, 3, 4, 5];
        let result = filter_map_fused(items, |x| x % 2 == 0, |x| x * 10);
        assert_eq!(result, alloc::vec![20, 40]);
    }

    #[test]
    fn test_fused_filter_map() {
        let items = alloc::vec![1, 2, 3, 4, 5];
        let result = fused_filter_map(items, |x| if x % 2 == 0 { Some(x * 10) } else { None });
        assert_eq!(result, alloc::vec![20, 40]);
    }

    #[test]
    fn test_map_fold_fused() {
        let items = alloc::vec![1, 2, 3];
        let sum_of_squares = map_fold_fused(items, |x| x * x, 0, |acc, sq| acc + sq);
        assert_eq!(sum_of_squares, 14);
    }

    #[test]
    fn test_reader_extract2() {
        struct Config {
            a: i32,
            b: i32,
        }
        let extract = reader_extract2(|c: &Config| c.a, |c: &Config| c.b);
        let config = Config { a: 10, b: 20 };
        assert_eq!(extract(&config), (10, 20));
    }

    #[test]
    fn test_writer_batch() {
        let mut batch = WriterBatch::new();
        batch.add("log1");
        batch.add("log2");
        batch.add("log3");
        assert_eq!(batch.len(), 3);
        assert_eq!(batch.entries(), alloc::vec!["log1", "log2", "log3"]);
    }

    #[test]
    fn test_pipeline() {
        let result = pipeline(5)
            .map(|x| x + 1)
            .map(|x| x * 2)
            .map(|x: i32| x.to_string())
            .finish();
        assert_eq!(result, "12");
    }
}
