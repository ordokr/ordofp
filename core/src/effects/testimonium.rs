//! Evidence-Based Effect Handlers - Koka-style evidence passing
//!
//! > *"Testimonium est probatio"*
//! > — Evidence is proof. (Legal maxim)
//!
//! This module implements evidence-based effect handlers inspired by Koka's
//! generalized evidence passing. Evidence allows effect handlers to be passed
//! implicitly through computations, enabling efficient effect handling.
//!
//! # Design
//!
//! Evidence-based effects decompose into:
//! - **Effect Tags**: Unique identifiers for effect types
//! - **Evidence**: Proof that an effect handler is available
//! - **Evidence Vectors**: Collections of evidence for multiple effects
//! - **Handler Clauses**: Different ways to handle operations
//!
//! # Inspired By
//!
//! - Koka's evidence-based algebraic effects
//! - "Generalized Evidence Passing for Effect Handlers" paper
//! - "Effect Handlers, Evidently" paper
//!
//! # Scholastic Naming
//!
//! | English | Latin | Etymology |
//! |---------|-------|-----------|
//! | Evidence | Testimonium | *testimonium* = testimony |
//! | Tag | Signum | *signum* = sign, mark |
//! | Vector | Vector | *vector* = carrier |
//! | Clause | Clausula | *clausula* = small clause |
//! | Resume | Resumere | *resumere* = to take back |

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::any::TypeId;
use core::marker::PhantomData;

use super::algebraic::EffectusAlgebraicus;

// =============================================================================
// Effect Tags
// =============================================================================

/// An effect tag uniquely identifies an effect type.
///
/// `SignumEffectus<E>` serves as a runtime type tag for effect `E`,
/// allowing dynamic effect dispatch.
///
/// > *"Signum effectus"* — Sign of the effect.
#[derive(Debug)]
pub struct SignumEffectus<E: EffectusAlgebraicus> {
    /// Human-readable name for debugging.
    nomen: &'static str,
    /// Type ID for runtime identification.
    type_id: TypeId,
    _phantom: PhantomData<E>,
}

impl<E: EffectusAlgebraicus + 'static> SignumEffectus<E> {
    /// Create a new effect tag with the given name.
    #[inline]
    pub const fn new(nomen: &'static str) -> Self {
        SignumEffectus {
            nomen,
            type_id: TypeId::of::<E>(),
            _phantom: PhantomData,
        }
    }

    /// Get the tag's name.
    #[inline]
    pub fn nomen(&self) -> &'static str {
        self.nomen
    }

    /// Get the type ID.
    #[inline]
    pub fn type_id(&self) -> TypeId {
        self.type_id
    }

    /// Check if this tag matches another effect type.
    #[inline]
    pub fn matches<F: EffectusAlgebraicus + 'static>(&self) -> bool {
        self.type_id == TypeId::of::<F>()
    }
}

impl<E: EffectusAlgebraicus + 'static> Clone for SignumEffectus<E> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<E: EffectusAlgebraicus + 'static> Copy for SignumEffectus<E> {}

// =============================================================================
// Evidence
// =============================================================================

/// Evidence that an effect handler is available.
///
/// `Testimonium<E, R>` proves that effect `E` can be handled in context `R`.
/// Evidence carries the handler and enables implicit handler passing.
///
/// > *"Testimonium tractatoris"* — Evidence of the handler.
///
/// # Example
///
/// ```rust
/// use ordofp_core::effects::algebraic::EffectusAlgebraicus;
/// use ordofp_core::effects::testimonium::{SignumEffectus, Testimonium};
///
/// #[derive(Debug, Clone)]
/// struct MyEffect;
///
/// impl EffectusAlgebraicus for MyEffect {
///     type Result = i32;
/// }
///
/// let tag = SignumEffectus::<MyEffect>::new("MyEffect");
/// let evidence = Testimonium::new(tag, 42i32);
/// assert_eq!(*evidence.tractator(), 42);
/// ```
pub struct Testimonium<E: EffectusAlgebraicus, H> {
    /// The effect tag.
    signum: SignumEffectus<E>,
    /// The effect handler.
    tractator: H,
    /// Handler depth in the handler stack.
    depth: usize,
}

impl<E: EffectusAlgebraicus + 'static, H> Testimonium<E, H> {
    /// Create new evidence with the given handler.
    #[inline]
    pub fn new(signum: SignumEffectus<E>, tractator: H) -> Self {
        Testimonium {
            signum,
            tractator,
            depth: 0,
        }
    }

    /// Create evidence at a specific depth.
    #[inline]
    pub fn with_depth(signum: SignumEffectus<E>, tractator: H, depth: usize) -> Self {
        Testimonium {
            signum,
            tractator,
            depth,
        }
    }

    /// Get the effect tag.
    #[inline]
    pub fn signum(&self) -> &SignumEffectus<E> {
        &self.signum
    }

    /// Get a reference to the handler.
    #[inline]
    pub fn tractator(&self) -> &H {
        &self.tractator
    }

    /// Get a mutable reference to the handler.
    #[inline]
    pub fn tractator_mut(&mut self) -> &mut H {
        &mut self.tractator
    }

    /// Get the handler depth.
    #[inline]
    pub fn depth(&self) -> usize {
        self.depth
    }

    /// Consume and return the handler.
    #[inline]
    pub fn into_tractator(self) -> H {
        self.tractator
    }
}

// =============================================================================
// Evidence Vector
// =============================================================================

/// A vector of evidence for multiple effects.
///
/// `VectorTestimonium` stores evidence for all effects in scope,
/// allowing dynamic effect lookup and dispatch.
///
/// > *"Vector testimoniorum"* — Vector of evidence.
///
/// # Example
///
/// ```rust
/// use ordofp_core::effects::algebraic::EffectusAlgebraicus;
/// use ordofp_core::effects::testimonium::{SignumEffectus, Testimonium, VectorTestimonium};
///
/// #[derive(Debug, Clone)]
/// struct MyEffect;
///
/// impl EffectusAlgebraicus for MyEffect {
///     type Result = i32;
/// }
///
/// let mut evv = VectorTestimonium::new();
/// let tag = SignumEffectus::<MyEffect>::new("MyEffect");
/// let evidence = Testimonium::new(tag, ());
/// evv.push(evidence);
/// assert_eq!(evv.len(), 1);
/// assert!(evv.has::<MyEffect>());
/// ```
pub struct VectorTestimonium {
    /// Type-erased evidence entries.
    entries: Vec<TestimoniumEntry>,
}

/// A type-erased evidence entry.
struct TestimoniumEntry {
    /// Type ID of the effect.
    type_id: TypeId,
    /// Type-erased evidence. Underscore-named because it is stored but not
    /// yet retrieved — retrieval needs additional type machinery for safe
    /// downcasting in effect handlers.
    _evidence: Box<dyn core::any::Any + Send + Sync>,
}

impl VectorTestimonium {
    /// Create an empty evidence vector.
    #[inline]
    pub fn new() -> Self {
        VectorTestimonium {
            entries: Vec::new(),
        }
    }

    /// Create an evidence vector with capacity.
    #[inline]
    pub fn with_capacity(cap: usize) -> Self {
        VectorTestimonium {
            entries: Vec::with_capacity(cap),
        }
    }

    /// Push evidence onto the vector.
    #[inline]
    pub fn push<E, H>(&mut self, evidence: Testimonium<E, H>)
    where
        E: EffectusAlgebraicus + 'static,
        H: Send + Sync + 'static,
    {
        self.entries.push(TestimoniumEntry {
            type_id: TypeId::of::<E>(),
            _evidence: Box::new(evidence),
        });
    }

    /// Look up evidence for an effect type.
    #[inline]
    pub fn lookup<E: EffectusAlgebraicus + 'static>(&self) -> Option<usize> {
        let target = TypeId::of::<E>();
        self.entries.iter().position(|e| e.type_id == target)
    }

    /// Check if evidence exists for an effect.
    #[inline]
    pub fn has<E: EffectusAlgebraicus + 'static>(&self) -> bool {
        self.lookup::<E>().is_some()
    }

    /// Get the number of evidence entries.
    #[inline]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the vector is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Clear all evidence.
    #[inline]
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Pop the last evidence entry.
    #[inline]
    pub fn pop(&mut self) {
        self.entries.pop();
    }
}

impl Default for VectorTestimonium {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Handler Clauses (Clausula)
// =============================================================================

/// Handler clause types for effect operations.
///
/// `Clausula` describes how an effect operation should be handled:
/// - `Fun`: Tail-resumptive (operation resumes immediately)
/// - `Ctl`: Control (operation may manipulate continuation)
/// - `Final`: Final (operation never resumes)
///
/// > *"Clausula tractatoris"* — Handler clause.
///
/// # Koka Correspondence
///
/// These correspond to Koka's handler clause types:
/// - `fun` -> `Fun` (tail-resumptive)
/// - `control` -> `Ctl` (control operation)
/// - `final` -> `Final` (never resumes)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClausulaGenus {
    /// Tail-resumptive: operation returns directly.
    Fun,
    /// Control: operation has access to continuation.
    Ctl,
    /// Final: operation never resumes.
    Final,
}

/// A handler clause that handles an operation.
///
/// `Clausula<A, B, E, R>` handles operation input `A` to produce `B`,
/// possibly using effect `E` in context `R`.
pub enum Clausula<A, B, E: EffectusAlgebraicus, R> {
    /// Tail-resumptive clause: simple function.
    Fun(Box<dyn Fn(A) -> B + Send + Sync>),

    /// Control clause: has access to resumption.
    Ctl(Box<dyn Fn(A, Resumptio<B, R>) -> R + Send + Sync>),

    /// Final clause: never resumes.
    Final(Box<dyn Fn(A) -> R + Send + Sync>),

    /// Carries `Infallible` so this variant can never actually be
    /// constructed; it exists solely so `E` is used in the enum body.
    #[doc(hidden)]
    _Phantom(PhantomData<E>, core::convert::Infallible),
}

impl<A, B, E: EffectusAlgebraicus, R> Clausula<A, B, E, R> {
    /// Create a tail-resumptive clause.
    #[inline]
    pub fn fun<F>(f: F) -> Self
    where
        F: Fn(A) -> B + Send + Sync + 'static,
    {
        Clausula::Fun(Box::new(f))
    }

    /// Create a control clause.
    #[inline]
    pub fn ctl<F>(f: F) -> Self
    where
        F: Fn(A, Resumptio<B, R>) -> R + Send + Sync + 'static,
    {
        Clausula::Ctl(Box::new(f))
    }

    /// Create a final clause.
    #[inline]
    pub fn final_<F>(f: F) -> Self
    where
        F: Fn(A) -> R + Send + Sync + 'static,
    {
        Clausula::Final(Box::new(f))
    }

    /// Get the clause type.
    #[inline]
    pub fn genus(&self) -> ClausulaGenus {
        match self {
            Clausula::Fun(_) => ClausulaGenus::Fun,
            Clausula::Ctl(_) => ClausulaGenus::Ctl,
            Clausula::Final(_) => ClausulaGenus::Final,
            Clausula::_Phantom(_, never) => match *never {},
        }
    }
}

// =============================================================================
// Resumption
// =============================================================================

/// A resumption (continuation) for control operations.
///
/// `Resumptio<A, R>` represents the continuation from an effect operation.
/// It can be called to resume the computation with a value.
///
/// > *"Resumptio computationis"* — Resumption of computation.
///
/// # One-Shot Semantics
///
/// Resumptions are one-shot: they can only be called once.
/// This matches OCaml 5.0 and Koka's continuation semantics.
pub struct Resumptio<A, R> {
    /// The continuation function.
    continuation: Box<dyn FnOnce(A) -> R + Send>,
}

impl<A: 'static, R: 'static> Resumptio<A, R> {
    /// Create a new resumption.
    #[inline]
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce(A) -> R + Send + 'static,
    {
        Resumptio {
            continuation: Box::new(f),
        }
    }

    /// Resume the computation with a value.
    ///
    /// One-shot semantics are enforced statically: `resume` consumes the
    /// resumption by move, so it cannot be called twice.
    #[inline]
    pub fn resume(self, value: A) -> R {
        (self.continuation)(value)
    }

    /// Transform the resumption's output.
    #[inline]
    pub fn map<S: 'static, F>(self, f: F) -> Resumptio<A, S>
    where
        F: FnOnce(R) -> S + Send + 'static,
    {
        Resumptio::new(move |a| f(self.resume(a)))
    }

    /// Transform the resumption's input.
    #[inline]
    pub fn contramap<B: 'static, F>(self, f: F) -> Resumptio<B, R>
    where
        F: FnOnce(B) -> A + Send + 'static,
    {
        Resumptio::new(move |b| self.resume(f(b)))
    }
}

// =============================================================================
// Evidence-Based Handler
// =============================================================================

/// An evidence-based effect handler.
///
/// `TractatorEvidentia<E>` uses evidence passing for efficient effect handling.
pub trait TractatorEvidentia<E: EffectusAlgebraicus>: Sized {
    /// The handler's result type.
    type Output;

    /// Get the effect tag for this handler.
    fn signum(&self) -> SignumEffectus<E>;

    /// Handle a return value.
    fn handle_return(&self, value: Self::Output) -> Self::Output;

    /// Handle an effect operation with evidence.
    fn handle_operation(
        &mut self,
        evv: &VectorTestimonium,
        op: E,
        k: Resumptio<E::Result, Self::Output>,
    ) -> Self::Output;
}

/// Run a computation with evidence-based handling.
///
/// Note: the handler is currently recorded as a unit placeholder in the
/// evidence vector; full handler threading is not yet implemented.
// Takes the handler by value now so that wiring it into the evidence vector
// later is not a breaking signature change.
#[allow(clippy::needless_pass_by_value)]
#[inline]
pub fn run_with_evidence<E, H, A, F>(handler: H, evv: &mut VectorTestimonium, computation: F) -> A
where
    E: EffectusAlgebraicus + 'static,
    H: TractatorEvidentia<E, Output = A> + Send + Sync + 'static,
    F: FnOnce(&VectorTestimonium) -> A,
    A: 'static,
{
    // Push evidence for this handler
    let signum = handler.signum();
    evv.push(Testimonium::new(signum, ()));

    let result = computation(evv);
    let result = handler.handle_return(result);

    // Pop evidence
    evv.pop();

    result
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Test effect
    #[derive(Debug, Clone)]
    struct TestEffect;

    impl EffectusAlgebraicus for TestEffect {
        type Result = i32;
    }

    #[test]
    fn test_signum_effectus() {
        let tag: SignumEffectus<TestEffect> = SignumEffectus::new("TestEffect");
        assert_eq!(tag.nomen(), "TestEffect");
        assert!(tag.matches::<TestEffect>());
    }

    #[test]
    fn test_testimonium() {
        let tag = SignumEffectus::<TestEffect>::new("TestEffect");
        let evidence = Testimonium::new(tag, 42i32);

        assert_eq!(evidence.depth(), 0);
        assert_eq!(*evidence.tractator(), 42);
    }

    #[test]
    fn test_vector_testimonium() {
        let mut evv = VectorTestimonium::new();
        assert!(evv.is_empty());

        let tag = SignumEffectus::<TestEffect>::new("TestEffect");
        let evidence = Testimonium::new(tag, ());
        evv.push(evidence);

        assert_eq!(evv.len(), 1);
        assert!(evv.has::<TestEffect>());
    }

    #[test]
    fn test_clausula_genus() {
        let fun_clause: Clausula<i32, i32, TestEffect, i32> = Clausula::fun(|x| x * 2);
        assert_eq!(fun_clause.genus(), ClausulaGenus::Fun);

        let final_clause: Clausula<i32, i32, TestEffect, i32> = Clausula::final_(|x| x);
        assert_eq!(final_clause.genus(), ClausulaGenus::Final);
    }

    #[test]
    fn test_resumptio() {
        let k = Resumptio::new(|x: i32| x * 2);
        let result = k.resume(21);
        assert_eq!(result, 42);
    }

    #[test]
    fn test_resumptio_map() {
        let k = Resumptio::new(|x: i32| x * 2);
        let mapped = k.map(|r| r + 1);
        assert_eq!(mapped.resume(20), 41);
    }

    #[test]
    fn test_resumptio_contramap() {
        let k = Resumptio::new(|x: i32| x * 2);
        let contramapped = k.contramap(|s: &str| s.len() as i32);
        assert_eq!(contramapped.resume("hello"), 10);
    }

    #[test]
    fn test_resumptio_one_shot() {
        // One-shot semantics are enforced by Rust's move semantics.
        // After calling resume, `k` is consumed and cannot be used again.
        let k = Resumptio::new(|x: i32| x);
        let result = k.resume(42);
        assert_eq!(result, 42);
        // Attempting to call k.resume(1) again would be a compile error:
        // "error: use of moved value: `k`"
    }
}
