//! Liberior - Freer Monad
//!
//! > *"Liberior est qui sibi imperat."*
//! > — More free is he who commands himself. (Seneca)
//!
//! The Freer monad is more efficient than Free because it doesn't
//! require the underlying type to be a Functor. It uses defunctionalization
//! to represent continuations as data.

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::boxed::Box;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

// =============================================================================
// Continuation Type Aliases
// =============================================================================

/// Terminal typed continuation: maps the type-erased result of an effect (or
/// of the preceding bind step) to the next `Liberior<F, A>` program.
#[cfg(feature = "alloc")]
type ContinuatioFinalis<F, A> = Box<dyn FnOnce(Box<dyn core::any::Any>) -> Liberior<F, A> + Send>;

/// One type-erased intermediate bind step in the flat continuation queue:
/// consumes a boxed intermediate value and produces the next boxed value.
#[cfg(feature = "alloc")]
type GradusErasus = Box<dyn FnOnce(Box<dyn core::any::Any>) -> Box<dyn core::any::Any> + Send>;

// =============================================================================
// Liberior - Freer Monad
// =============================================================================

/// Freer monad - doesn't require Functor constraint.
///
/// Unlike `Liber`, which requires `F` to be a functor, `Liberior` uses
/// defunctionalization to represent the continuation chain as data.
/// This makes it more efficient and applicable to more types.
///
/// The key insight is that instead of:
/// ```text
/// Free f a = Pure a | Free (f (Free f a))
/// ```
///
/// We use:
/// ```text
/// Freer f a = Pure a | Impure (f x) (x -> Freer f a)  -- for some x
/// ```
///
/// The continuation `x -> Freer f a` is stored explicitly, avoiding
/// the need to map over `f`.
///
/// # Latin Etymology
///
/// *Liberior* = more free (comparative of *liber*)
///
/// # Example
///
/// ```rust
/// use ordofp_core::free::{Liberior, mitto_liberior};
///
/// // Define an effect marker type -- no Functor impl required!
/// enum ConsoleOp {}
///
/// // Build a program by sending an effect into the Freer monad
/// let program: Liberior<ConsoleOp, i32> = mitto_liberior::<ConsoleOp, i32>(42);
/// assert!(program.est_impurus());
/// ```
#[cfg(feature = "alloc")]
pub enum Liberior<F, A> {
    /// Pure value - computation completes immediately.
    ///
    /// # Latin Etymology
    /// *Purus* = pure, clean
    Purus(A),

    /// Impure computation with continuation.
    ///
    /// Contains an effect `F<X>` and a continuation `X -> Liberior<F, A>`.
    ///
    /// # Latin Etymology
    /// *Impurus* = impure, not clean
    Impurus(LiberiorSuspensio<F, A>),
}

/// Suspended computation in the Freer monad.
///
/// This uses existential quantification: there exists some type X
/// such that we have `F<X>` and `X -> Liberior<F, A>`.
///
/// Internally, binds are stored as a flat queue of type-erased
/// continuations rather than nested closures, giving O(1) per bind
/// and O(n) overall interpretation instead of the O(n²) closure-wrap
/// approach.
///
/// # Latin Etymology
///
/// *Suspensio* = suspension, hanging
#[cfg(feature = "alloc")]
pub struct LiberiorSuspensio<F, A> {
    /// The effect operation (type-erased).
    effect: Box<dyn core::any::Any + Send + Sync>,
    /// The first (mandatory) continuation: maps the effect's result to
    /// `Liberior<F, A>`.  Stored separately so we never allocate a Vec
    /// for the common single-bind case.
    first_cont: ContinuatioFinalis<F, A>,
    /// Additional continuations accumulated via `flat_map`.  Each entry
    /// is a type-erased `FnOnce(Box<Any>) -> Box<Any>` that maps an
    /// intermediate value to the next intermediate value.  The final
    /// step (returning `Liberior<F, A>`) is always `first_cont` composed
    /// with however many of these steps come before it — but we store
    /// them in forward order and apply them iteratively at run time.
    ///
    /// `None` until the second `flat_map` call (avoids a heap allocation
    /// for programs with a single continuation).
    extra_conts: Option<Vec<GradusErasus>>,
    /// Phantom data for the functor type.
    _f: core::marker::PhantomData<F>,
}

#[cfg(feature = "alloc")]
impl<F: 'static, A: 'static> Liberior<F, A> {
    /// Create a pure value.
    #[inline]
    pub fn purus(a: A) -> Self {
        Liberior::Purus(a)
    }

    /// Check if this is a pure value.
    #[inline]
    pub fn est_purus(&self) -> bool {
        matches!(self, Liberior::Purus(_))
    }

    /// Check if this is an impure computation.
    #[inline]
    pub fn est_impurus(&self) -> bool {
        matches!(self, Liberior::Impurus(_))
    }

    /// Map a function over the result type.
    #[inline]
    pub fn map<B: 'static, G>(self, f: G) -> Liberior<F, B>
    where
        G: FnOnce(A) -> B + Send + 'static,
    {
        self.flat_map(move |a| Liberior::purus(f(a)))
    }

    /// Monadic bind (flatMap).
    ///
    /// This is the core operation that sequences Freer computations.
    ///
    /// # Complexity
    ///
    /// O(1) per call — pushes the new continuation onto the flat queue
    /// stored inside `LiberiorSuspensio` instead of wrapping the old
    /// continuation in a new closure.  This avoids the classic O(n²)
    /// left-nesting problem where each bind added another stack frame
    /// that had to be traversed at interpretation time.
    ///
    /// # Panics
    ///
    /// The queued continuation panics only if a step in the internal
    /// continuation queue yields a value of the wrong type-erased type,
    /// which the queue construction makes impossible — such a panic
    /// indicates a bug in this crate.
    #[inline]
    pub fn flat_map<B: 'static, G>(self, f: G) -> Liberior<F, B>
    where
        G: FnOnce(A) -> Liberior<F, B> + Send + 'static,
    {
        match self {
            Liberior::Purus(a) => f(a),
            Liberior::Impurus(suspensio) => {
                // Re-type the existing first_cont from  `Any -> Liberior<F, A>`
                // to `Any -> Any` so it can be pushed into `extra_conts`,
                // and make `f` the new typed terminal continuation.
                //
                // Layout after this call:
                //   extra_conts: [...previous extras..., old_first_cont_as_any]
                //   first_cont:  f   (the new terminal step)
                //
                // At interpretation time we drain extra_conts left-to-right,
                // then call first_cont on the final intermediate value.

                let old_first: ContinuatioFinalis<F, A> = suspensio.first_cont;

                // Erase `Liberior<F, A>` to `Box<dyn Any>` so it can live in
                // the homogeneous `extra_conts` Vec.
                let erased_old: GradusErasus = Box::new(move |x| {
                    // We box the whole Liberior so it can be re-extracted by
                    // the next step or the terminal continuation.
                    Box::new(old_first(x)) as Box<dyn core::any::Any>
                });

                let new_first: ContinuatioFinalis<F, B> = Box::new(move |boxed_any| {
                    // The previous step boxed a `Liberior<F, A>`; unwrap it.
                    let intermediate: Liberior<F, A> = *boxed_any
                        .downcast::<Liberior<F, A>>()
                        .expect("Liberior: type mismatch in flat continuation queue");
                    // Apply the user's continuation to the intermediate
                    // Liberior.  If it's Purus we call f directly; if it's
                    // Impurus we recurse — but that recursion is bounded to
                    // a single level (the result of one user function), not
                    // the whole chain.
                    match intermediate {
                        Liberior::Purus(a) => f(a),
                        Liberior::Impurus(inner) => {
                            // The intermediate computation itself is impure.
                            // Bind f onto it so that its own continuation
                            // queue absorbs f rather than nesting closures.
                            Liberior::Impurus(inner).flat_map(f)
                        }
                    }
                });

                let mut extra = suspensio.extra_conts.unwrap_or_default();
                extra.push(erased_old);

                Liberior::Impurus(LiberiorSuspensio {
                    effect: suspensio.effect,
                    first_cont: new_first,
                    extra_conts: Some(extra),
                    _f: core::marker::PhantomData,
                })
            }
        }
    }
}

// =============================================================================
// LiberiorSuspensio - continuation queue runner
// =============================================================================

#[cfg(feature = "alloc")]
impl<F: 'static, A: 'static> LiberiorSuspensio<F, A> {
    /// Run the full continuation queue against an effect result.
    ///
    /// This drives the O(n) iterative unwinding: first the `extra_conts`
    /// steps are applied left-to-right (each converting one `Box<Any>` to
    /// the next), then `first_cont` is applied to yield the final
    /// `Liberior<F, A>`.
    ///
    /// Interpreters should call this once they have resolved the `effect`
    /// into its result value.
    #[inline]
    pub fn resume(self, effect_result: Box<dyn core::any::Any>) -> Liberior<F, A> {
        let mut value: Box<dyn core::any::Any> = effect_result;

        // Drive the intermediate steps iteratively — O(n), no recursion.
        if let Some(extras) = self.extra_conts {
            for step in extras {
                value = step(value);
            }
        }

        // Apply the terminal typed continuation.
        (self.first_cont)(value)
    }

    /// Access the raw effect (type-erased) for inspection by an interpreter.
    #[inline]
    pub fn effect(&self) -> &(dyn core::any::Any + Send + Sync) {
        &*self.effect
    }

    /// Consume the suspension and take ownership of the effect box.
    ///
    /// Returns `(effect, suspensio_without_effect)` — the caller can
    /// downcast the effect, then call `resume` on the rest.
    #[inline]
    pub fn take_effect(self) -> (Box<dyn core::any::Any + Send + Sync>, Self) {
        // We need to reconstruct without the effect; replace with a unit box.
        let dummy_effect: Box<dyn core::any::Any + Send + Sync> = Box::new(());
        let real_effect = self.effect;
        let rest = LiberiorSuspensio {
            effect: dummy_effect,
            first_cont: self.first_cont,
            extra_conts: self.extra_conts,
            _f: core::marker::PhantomData,
        };
        (real_effect, rest)
    }
}

// =============================================================================
// Send Operations
// =============================================================================

/// Send an effect to the Freer monad.
///
/// This is the primary way to inject effects into Freer.
///
/// # Latin Etymology
///
/// *Mitto* = to send
///
/// # Panics
///
/// The installed continuation panics only if the interpreter resumes it with
/// a value that fails to downcast back to `X` — an internal invariant that
/// cannot fire absent a bug in this crate or a hand-written interpreter that
/// resumes with the wrong type.
#[cfg(feature = "alloc")]
#[inline]
pub fn mitto_liberior<F: 'static, X: Send + Sync + 'static>(effect: X) -> Liberior<F, X> {
    Liberior::Impurus(LiberiorSuspensio {
        effect: Box::new(effect),
        first_cont: Box::new(|x| {
            let value = x.downcast::<X>().expect("Type mismatch in Liberior");
            Liberior::Purus(*value)
        }),
        extra_conts: None,
        _f: core::marker::PhantomData,
    })
}

// =============================================================================
// Interpretation
// =============================================================================

/// Run a pure Liberior computation.
///
/// This only works if the computation contains no effects.
///
/// # Panics
///
/// Panics if the computation is not pure.
#[cfg(feature = "alloc")]
#[inline]
pub fn curro_purus_liberior<F, A>(liberior: Liberior<F, A>) -> A {
    match liberior {
        Liberior::Purus(a) => a,
        Liberior::Impurus(_) => panic!("Cannot run impure Liberior as pure"),
    }
}

// =============================================================================
// Simple Effect DSL Example Types
// =============================================================================

/// A simple state effect operation.
///
/// # Latin Etymology
///
/// *Operatio Status* = state operation
#[cfg(feature = "alloc")]
#[derive(Debug, Clone)]
pub enum StatusOperatio<S> {
    /// Get the current state.
    Lego(core::marker::PhantomData<S>),
    /// Set a new state.
    Scribo(S),
}

/// A simple reader effect operation.
///
/// # Latin Etymology
///
/// *Operatio Lectoris* = reader operation
#[cfg(feature = "alloc")]
#[derive(Debug, Clone)]
pub enum LectorOperatio<E> {
    /// Read the environment.
    Lego(core::marker::PhantomData<E>),
}

/// A simple writer effect operation.
///
/// # Latin Etymology
///
/// *Operatio Scriptoris* = writer operation
#[cfg(feature = "alloc")]
#[derive(Debug, Clone)]
pub enum ScriptorOperatio<W> {
    /// Write/tell a value.
    Dico(W),
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_purus() {
        let free: Liberior<(), i32> = Liberior::purus(42);
        assert!(free.est_purus());
        assert!(!free.est_impurus());
    }

    #[test]
    fn test_map_purus() {
        let free: Liberior<(), i32> = Liberior::purus(42);
        let mapped = free.map(|x| x * 2);

        match mapped {
            Liberior::Purus(x) => assert_eq!(x, 84),
            _ => panic!("Expected Purus"),
        }
    }

    #[test]
    fn test_flat_map_purus() {
        let free: Liberior<(), i32> = Liberior::purus(42);
        let chained = free.flat_map(|x| Liberior::purus(x + 1));

        match chained {
            Liberior::Purus(x) => assert_eq!(x, 43),
            _ => panic!("Expected Purus"),
        }
    }

    #[test]
    fn test_chain_operations() {
        let free: Liberior<(), i32> = Liberior::purus(10);
        let result = free
            .flat_map(|x| Liberior::purus(x + 5))
            .flat_map(|x| Liberior::purus(x * 2))
            .map(|x| x - 10);

        match result {
            Liberior::Purus(x) => assert_eq!(x, 20), // ((10 + 5) * 2) - 10 = 20
            _ => panic!("Expected Purus"),
        }
    }

    #[test]
    fn test_monad_left_identity() {
        let a = 42;
        let f = |x: i32| Liberior::<(), i32>::purus(x * 2);

        let left: Liberior<(), i32> = Liberior::purus(a).flat_map(f);
        let right: Liberior<(), i32> = f(a);

        match (left, right) {
            (Liberior::Purus(l), Liberior::Purus(r)) => assert_eq!(l, r),
            _ => panic!("Both should be Purus"),
        }
    }

    #[test]
    fn test_monad_right_identity() {
        let m: Liberior<(), i32> = Liberior::purus(42);
        let result = m.flat_map(Liberior::purus);

        match result {
            Liberior::Purus(x) => assert_eq!(x, 42),
            _ => panic!("Expected Purus"),
        }
    }

    #[test]
    fn test_curro_purus() {
        let free: Liberior<(), i32> = Liberior::purus(42);
        let result = curro_purus_liberior(free);
        assert_eq!(result, 42);
    }

    #[test]
    #[should_panic(expected = "Cannot run impure")]
    fn test_curro_purus_panics_on_impure() {
        let free: Liberior<(), i32> = mitto_liberior::<(), i32>(42);
        let _ = curro_purus_liberior(free);
    }

    // Minimal interpreter for tests: the only effect is an i64 that the handler
    // echoes back as its result. Drives the suspension's continuation queue.
    fn run_i64(mut prog: Liberior<(), i64>) -> i64 {
        loop {
            match prog {
                Liberior::Purus(a) => return a,
                Liberior::Impurus(susp) => {
                    let (effect, rest) = susp.take_effect();
                    let val = *effect.downcast::<i64>().expect("test effect is always i64");
                    prog = rest.resume(Box::new(val));
                }
            }
        }
    }

    /// Regression test for the O(n²)->O(n) flat-continuation-queue rewrite.
    ///
    /// Builds a deeply left-nested bind chain on a SINGLE effect and interprets
    /// it. Every `flat_map` appends to `extra_conts`; `resume` must drain them
    /// left-to-right in the right order. A naive ordering or off-by-one in the
    /// queue would yield the wrong total here.
    #[test]
    fn test_deep_bind_chain_interpretation() {
        let mut prog: Liberior<(), i64> = mitto_liberior::<(), i64>(0);
        for _ in 0..1000 {
            prog = prog.flat_map(|x| Liberior::purus(x + 1));
        }
        assert_eq!(run_i64(prog), 1000);
    }

    /// A continuation that itself yields a fresh suspension (a two-effect
    /// program) must resume correctly across both effects and combine results.
    #[test]
    fn test_multi_effect_sequencing() {
        let prog: Liberior<(), i64> = mitto_liberior::<(), i64>(10)
            .flat_map(|x| mitto_liberior::<(), i64>(20).flat_map(move |y| Liberior::purus(x + y)));
        assert_eq!(run_i64(prog), 30);
    }
}
