//! Enhanced Arrow Type Classes (Sagitta)
//!
//! > *"Sagitta volat, tempus fugit."*
//! > — The arrow flies, time flees. (Medieval proverb)
//!
//! This module extends the basic Arrow type class with additional capabilities:
//!
//! - **`ArrowChoice` (`SagittaElectio`)**: Arrows that can make choices based on sum types
//! - **`ArrowApply` (`SagittaApplicatio`)**: Arrows that can apply themselves (equivalent to monads)
//! - **`ArrowLoop` (`SagittaCirculus`)**: Arrows with feedback/recursion capability
//!
//! # Type Class Hierarchy
//!
//! ```text
//! Category
//!    ↓
//! Arrow (Sagitta)
//!    ↓
//! ArrowChoice (SagittaElectio)
//!    ↓
//! ArrowApply (SagittaApplicatio)
//!
//! Arrow
//!    ↓
//! ArrowLoop (SagittaCirculus)
//! ```

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::boxed::Box;

use crate::datatypes::Aut;

// Note: The SagittaElectio, SagittaApplicatio, and SagittaCirculus traits
// extend the Arrow trait from typeclasses. However, due to Rust's trait system
// limitations with lifetime bounds on GATs, we provide standalone function
// implementations in the fn_arrows module instead of implementing these traits
// for boxed functions.
use crate::typeclasses::arrow::Arrow;

// =============================================================================
// SagittaElectio - ArrowChoice
// =============================================================================

/// `ArrowChoice` - Arrows with choice capability.
///
/// > *"Electio est actus voluntatis."*
/// > — Choice is an act of will. (Thomas Aquinas)
///
/// `ArrowChoice` extends Arrow with the ability to handle sum types (Either/Aut).
/// This allows arrows to make decisions based on the variant of an input.
///
/// # Laws
///
/// 1. **Left identity**: `left (arr f) = arr (left f)` where `left f` on Either means `bimap f id`
/// 2. **Left composition**: `left (f >>> g) = left f >>> left g`
/// 3. **Left absorption**: `left f >>> arr (id +++ g) = arr (id +++ g) >>> left f`
///
/// # Latin Etymology
///
/// *Electio* = choice, selection (from *eligere* = to choose)
///
/// # Example
///
/// No boxed-function `Arrow`/`SagittaElectio` trait instance ships in this
/// crate yet -- the module-level `fn_arrows` submodule provides the same
/// operation as a standalone function instead (see its doc comment for why).
///
/// ```rust
/// use ordofp_core::category::fn_arrows::{self, BoxedFn};
/// use ordofp_core::datatypes::Aut;
///
/// // Handle either a number or a string
/// let double: BoxedFn<i32, i32> = Box::new(|x| x * 2);
/// let arrow = fn_arrows::sinister::<i32, i32, &str>(double);
/// assert_eq!(arrow(Aut::sinister(21)), Aut::sinister(42));
/// assert_eq!(arrow(Aut::dexter("unchanged")), Aut::dexter("unchanged"));
/// ```
pub trait SagittaElectio: Arrow {
    /// Lift an arrow to work on the left (Sinister) side of an Aut.
    ///
    /// If the input is `Sinister(a)`, applies the arrow and returns `Sinister(b)`.
    /// If the input is `Dexter(c)`, passes it through unchanged as `Dexter(c)`.
    ///
    /// # Type Signature
    ///
    /// ```text
    /// left :: Arrow a b -> Arrow (Aut a c) (Aut b c)
    /// ```
    fn sinister<A, B, C>(f: Self::Hom<A, B>) -> Self::Hom<Aut<A, C>, Aut<B, C>>;

    /// Lift an arrow to work on the right (Dexter) side of an Aut.
    ///
    /// If the input is `Dexter(a)`, applies the arrow and returns `Dexter(b)`.
    /// If the input is `Sinister(c)`, passes it through unchanged as `Sinister(c)`.
    ///
    /// Default implementation uses `sinister` with swapping.
    ///
    /// # Type Signature
    ///
    /// ```text
    /// right :: Arrow a b -> Arrow (Aut c a) (Aut c b)
    /// ```
    fn dexter<A, B, C>(f: Self::Hom<A, B>) -> Self::Hom<Aut<C, A>, Aut<C, B>>
    where
        A: 'static,
        B: 'static,
        C: 'static,
    {
        // swap >>> left f >>> swap
        let swap_in: Self::Hom<Aut<C, A>, Aut<A, C>> = Self::arr(|aut: Aut<C, A>| aut.swap());
        let left_f: Self::Hom<Aut<A, C>, Aut<B, C>> = Self::sinister(f);
        let swap_out: Self::Hom<Aut<B, C>, Aut<C, B>> = Self::arr(|aut: Aut<B, C>| aut.swap());
        Self::compose(swap_out, Self::compose(left_f, swap_in))
    }

    /// Merge two arrows that produce the same output type.
    ///
    /// Given an arrow from A to C and an arrow from B to C,
    /// produces an arrow from `Aut<A, B>` to C.
    ///
    /// This is also known as `fanin` or `(|||)` in Haskell.
    ///
    /// # Type Signature
    ///
    /// ```text
    /// fanin :: Arrow a c -> Arrow b c -> Arrow (Aut a b) c
    /// ```
    fn confluo<A, B, C>(f: Self::Hom<A, C>, g: Self::Hom<B, C>) -> Self::Hom<Aut<A, B>, C>
    where
        A: 'static,
        B: 'static,
        C: 'static;

    /// Apply different arrows to left and right sides.
    ///
    /// Given `Arrow a c` and `Arrow b d`, produces `Arrow (Aut a b) (Aut c d)`.
    ///
    /// This is also known as `(+++)` in Haskell.
    ///
    /// # Type Signature
    ///
    /// ```text
    /// (+++) :: Arrow a c -> Arrow b d -> Arrow (Aut a b) (Aut c d)
    /// ```
    fn addo<A, B, C, D>(f: Self::Hom<A, C>, g: Self::Hom<B, D>) -> Self::Hom<Aut<A, B>, Aut<C, D>>
    where
        A: 'static,
        B: 'static,
        C: 'static,
        D: 'static,
    {
        // left f >>> right g
        let left_f: Self::Hom<Aut<A, B>, Aut<C, B>> = Self::sinister(f);
        let right_g: Self::Hom<Aut<C, B>, Aut<C, D>> = Self::dexter(g);
        Self::compose(right_g, left_f)
    }
}

// =============================================================================
// SagittaApplicatio - ArrowApply
// =============================================================================

/// `ArrowApply` - Arrows that can apply themselves.
///
/// > *"Applicatio est actio applicandi."*
/// > — Application is the act of applying. (Scholastic)
///
/// `ArrowApply` provides an `app` arrow that takes a pair of an arrow and its input,
/// and applies the arrow to the input. This makes arrows as powerful as monads.
///
/// # Laws
///
/// 1. **Application**: `first (arr (b,)) >>> app = f` for any `f :: a ~> b`
/// 2. **arr and app**: `arr f >>> app = first (arr (arr . f)) >>> app`
///
/// # Significance
///
/// `ArrowApply` is equivalent in power to Monad. Any `ArrowApply` can implement Monad,
/// and any Monad can implement `ArrowApply`. This makes `ArrowApply` the "ultimate"
/// arrow type class.
///
/// # Latin Etymology
///
/// *Applicatio* = application, joining (from *applicare* = to attach, apply)
///
/// # Note on Implementation
///
/// Due to Rust's type system limitations, we use boxed function arrows
/// for the `app` implementation.
pub trait SagittaApplicatio: Arrow {
    /// The application arrow.
    ///
    /// Takes a pair of (arrow from A to B, value of A) and produces B.
    ///
    /// # Type Signature
    ///
    /// ```text
    /// app :: Arrow (Arrow a b, a) b
    /// ```
    fn applicatio<A, B>() -> Self::Hom<(Self::Hom<A, B>, A), B>
    where
        A: 'static,
        B: 'static;
}

// =============================================================================
// SagittaCirculus - ArrowLoop
// =============================================================================

/// `ArrowLoop` - Arrows with feedback/loop capability.
///
/// > *"In circulus perfectus est."*
/// > — In the circle, there is perfection. (Alchemical)
///
/// `ArrowLoop` provides the ability to create feedback loops in arrow computations.
/// This is similar to the `loop` construct in circuit descriptions or the
/// fixed-point combinator in lambda calculus.
///
/// # Laws
///
/// 1. **Extension**: `loop (arr f) = arr (\ b -> fst (fix (\ (c,d) -> f (b,d))))`
/// 2. **Left tightening**: `loop (first h >>> f) = h >>> loop f`
/// 3. **Right tightening**: `loop (f >>> first h) = loop f >>> h`
/// 4. **Sliding**: `loop (f >>> arr (id *** k)) = loop (arr (id *** k) >>> f)`
/// 5. **Vanishing**: `loop (loop f) = loop (arr unassoc >>> f >>> arr assoc)`
/// 6. **Superposing**: `second (loop f) = loop (arr assoc >>> second f >>> arr unassoc)`
///
/// # Latin Etymology
///
/// *Circulus* = circle, ring (from *circus* = ring)
///
/// # Example
///
/// This example uses the standalone `fn_arrows::circulus` (see the module-level
/// note on why boxed functions get free functions instead of trait impls).
/// Each call is fed `D::default()` as the initial feedback value and the
/// output feedback is discarded -- state does not persist across calls.
///
/// ```rust
/// use ordofp_core::category::fn_arrows;
///
/// let f: fn_arrows::BoxedFn<(i32, i32), (i32, i32)> =
///     Box::new(|(input, state)| (input + state, state + 1));
/// let looped = fn_arrows::circulus(f);
///
/// // With default state (0), result is input + 0 = input
/// assert_eq!(looped(5), 5);
/// ```
pub trait SagittaCirculus: Arrow {
    /// Create a feedback loop.
    ///
    /// Given an arrow from `(B, D)` to `(C, D)`, produces an arrow from B to C.
    /// The D component is fed back as input in the next iteration.
    ///
    /// # Type Signature
    ///
    /// ```text
    /// loop :: Arrow (b, d) (c, d) -> Arrow b c
    /// ```
    ///
    /// # Safety Note
    ///
    /// The implementation must be careful about evaluation order to avoid
    /// infinite loops. The feedback value D must be computed lazily.
    fn circulus<B, C, D>(f: Self::Hom<(B, D), (C, D)>) -> Self::Hom<B, C>
    where
        D: Default;
}

// =============================================================================
// Function Arrow Operations
// =============================================================================

/// Standalone arrow operations for plain functions.
///
/// These functions implement arrow operations without requiring trait bounds
/// that conflict with the existing Category/Arrow traits.
///
/// # Latin Etymology
///
/// *Operationes Sagittae Functionis* = operations of the function arrow
#[cfg(feature = "alloc")]
pub mod fn_arrows {
    use super::{Aut, Box};

    /// Boxed function type for dynamic dispatch.
    pub type BoxedFn<A, B> = Box<dyn Fn(A) -> B + Send + Sync>;

    /// Identity function.
    #[inline]
    pub fn id<A: 'static>() -> BoxedFn<A, A> {
        Box::new(|a| a)
    }

    /// Compose two boxed functions.
    #[inline]
    pub fn compose<A: 'static, B: 'static, C: 'static>(
        f: BoxedFn<B, C>,
        g: BoxedFn<A, B>,
    ) -> BoxedFn<A, C> {
        Box::new(move |a| f(g(a)))
    }

    /// Lift a function to a boxed function.
    #[inline]
    pub fn arr<A: 'static, B: 'static, F>(f: F) -> BoxedFn<A, B>
    where
        F: Fn(A) -> B + Send + Sync + 'static,
    {
        Box::new(f)
    }

    /// Apply arrow to first element of pair.
    #[inline]
    pub fn first<A: 'static, B: 'static, C: 'static>(f: BoxedFn<A, B>) -> BoxedFn<(A, C), (B, C)> {
        Box::new(move |(a, c)| (f(a), c))
    }

    /// Apply arrow to second element of pair.
    #[inline]
    pub fn second<A: 'static, B: 'static, C: 'static>(f: BoxedFn<B, C>) -> BoxedFn<(A, B), (A, C)> {
        Box::new(move |(a, b)| (a, f(b)))
    }

    /// Split - apply two arrows in parallel.
    #[inline]
    pub fn split<A: 'static, B: 'static, C: 'static, D: 'static>(
        f: BoxedFn<A, B>,
        g: BoxedFn<C, D>,
    ) -> BoxedFn<(A, C), (B, D)> {
        Box::new(move |(a, c)| (f(a), g(c)))
    }

    /// Left choice - apply arrow to left side of Aut.
    #[inline]
    pub fn sinister<A: 'static, B: 'static, C: 'static>(
        f: BoxedFn<A, B>,
    ) -> BoxedFn<Aut<A, C>, Aut<B, C>> {
        Box::new(move |aut| match aut {
            Aut::Sinister(a) => Aut::Sinister(f(a)),
            Aut::Dexter(c) => Aut::Dexter(c),
        })
    }

    /// Right choice - apply arrow to right side of Aut.
    #[inline]
    pub fn dexter<A: 'static, B: 'static, C: 'static>(
        f: BoxedFn<B, C>,
    ) -> BoxedFn<Aut<A, B>, Aut<A, C>> {
        Box::new(move |aut| match aut {
            Aut::Sinister(a) => Aut::Sinister(a),
            Aut::Dexter(b) => Aut::Dexter(f(b)),
        })
    }

    /// Fanin - merge two arrows that produce the same type.
    #[inline]
    pub fn confluo<A: 'static, B: 'static, C: 'static>(
        f: BoxedFn<A, C>,
        g: BoxedFn<B, C>,
    ) -> BoxedFn<Aut<A, B>, C> {
        Box::new(move |aut| match aut {
            Aut::Sinister(a) => f(a),
            Aut::Dexter(b) => g(b),
        })
    }

    /// Sum - apply different arrows to both sides of Aut.
    #[inline]
    pub fn addo<A: 'static, B: 'static, C: 'static, D: 'static>(
        f: BoxedFn<A, C>,
        g: BoxedFn<B, D>,
    ) -> BoxedFn<Aut<A, B>, Aut<C, D>> {
        Box::new(move |aut| match aut {
            Aut::Sinister(a) => Aut::Sinister(f(a)),
            Aut::Dexter(b) => Aut::Dexter(g(b)),
        })
    }

    /// Application arrow - apply an arrow to a value.
    #[inline]
    pub fn applicatio<A: 'static, B: 'static>() -> BoxedFn<(BoxedFn<A, B>, A), B> {
        Box::new(|(f, a): (BoxedFn<A, B>, A)| f(a))
    }

    /// Loop with default feedback value.
    #[inline]
    pub fn circulus<B: 'static, C: 'static, D: Default + 'static>(
        f: BoxedFn<(B, D), (C, D)>,
    ) -> BoxedFn<B, C> {
        Box::new(move |b| {
            let d = D::default();
            let (c, _) = f((b, d));
            c
        })
    }
}

// =============================================================================
// Utility Functions
// =============================================================================

/// Create a choice arrow that routes based on a predicate.
///
/// Given a predicate and an input, routes to Sinister if true, Dexter if false.
///
/// # Example
///
/// ```rust
/// use ordofp_core::category::via_praedicatum;
/// use ordofp_core::datatypes::Aut;
///
/// let router = via_praedicatum(|x: &i32| *x > 0);
/// assert_eq!(router(5), Aut::sinister(5));
/// assert_eq!(router(-3), Aut::dexter(-3));
/// ```
#[inline]
pub fn via_praedicatum<A, F>(predicate: F) -> impl Fn(A) -> Aut<A, A>
where
    F: Fn(&A) -> bool,
{
    move |a| {
        if predicate(&a) {
            Aut::sinister(a)
        } else {
            Aut::dexter(a)
        }
    }
}

/// Coalesce an Aut where both sides have the same type.
///
/// This is useful after processing both branches of a choice.
#[inline]
pub fn coalesco<A>(aut: Aut<A, A>) -> A {
    match aut {
        Aut::Sinister(a) => a,
        Aut::Dexter(a) => a,
    }
}

/// Inject a value into the left side of an Aut.
#[inline]
pub fn inicio_sinister<A, B>(a: A) -> Aut<A, B> {
    Aut::sinister(a)
}

/// Inject a value into the right side of an Aut.
#[inline]
pub fn inicio_dexter<A, B>(b: B) -> Aut<A, B> {
    Aut::dexter(b)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;
    use fn_arrows::*;

    #[test]
    fn test_fn_arrows_id() {
        let id_fn: BoxedFn<i32, i32> = id();
        assert_eq!(id_fn(42), 42);
    }

    #[test]
    fn test_fn_arrows_compose() {
        let f: BoxedFn<i32, i32> = Box::new(|x| x + 1);
        let g: BoxedFn<i32, i32> = Box::new(|x| x * 2);
        let composed = compose(f, g);
        // (x * 2) + 1
        assert_eq!(composed(5), 11);
    }

    #[test]
    fn test_fn_arrows_arr() {
        let f = arr(|x: i32| x.to_string());
        assert_eq!(f(42), "42");
    }

    #[test]
    fn test_fn_arrows_first() {
        let f: BoxedFn<i32, i32> = Box::new(|x| x * 2);
        let first_f = first::<i32, i32, &str>(f);
        assert_eq!(first_f((5, "hello")), (10, "hello"));
    }

    #[test]
    fn test_fn_arrows_second() {
        let f: BoxedFn<i32, i32> = Box::new(|x| x * 2);
        let second_f = second::<&str, i32, i32>(f);
        assert_eq!(second_f(("hello", 5)), ("hello", 10));
    }

    #[test]
    fn test_fn_arrows_sinister() {
        let f: BoxedFn<i32, i32> = Box::new(|x| x * 2);
        let left_f = sinister::<i32, i32, &str>(f);

        assert_eq!(left_f(Aut::sinister(21)), Aut::sinister(42));
        assert_eq!(left_f(Aut::dexter("unchanged")), Aut::dexter("unchanged"));
    }

    #[test]
    fn test_fn_arrows_dexter() {
        let f: BoxedFn<i32, i32> = Box::new(|x| x * 2);
        let right_f = dexter::<&str, i32, i32>(f);

        assert_eq!(right_f(Aut::dexter(21)), Aut::dexter(42));
        assert_eq!(
            right_f(Aut::sinister("unchanged")),
            Aut::sinister("unchanged")
        );
    }

    #[test]
    fn test_fn_arrows_confluo() {
        let f: BoxedFn<i32, i32> = Box::new(|x| x * 2);
        let g: BoxedFn<&str, i32> = Box::new(|s| s.len() as i32);
        let fanin = confluo(f, g);

        assert_eq!(fanin(Aut::sinister(21)), 42);
        assert_eq!(fanin(Aut::dexter("hello")), 5);
    }

    #[test]
    fn test_fn_arrows_addo() {
        let f: BoxedFn<i32, i32> = Box::new(|x| x * 2);
        let g: BoxedFn<&str, usize> = Box::new(str::len);
        let plus = addo(f, g);

        assert_eq!(plus(Aut::sinister(21)), Aut::sinister(42));
        assert_eq!(plus(Aut::dexter("hello")), Aut::dexter(5));
    }

    #[test]
    fn test_fn_arrows_applicatio() {
        let app = applicatio::<i32, i32>();
        let double: BoxedFn<i32, i32> = Box::new(|x| x * 2);

        assert_eq!(app((double, 21)), 42);
    }

    #[test]
    fn test_fn_arrows_circulus() {
        let f: BoxedFn<(i32, i32), (i32, i32)> =
            Box::new(|(input, state)| (input + state, state + 1));
        let looped = circulus(f);

        // With default state (0), result is input + 0 = input
        assert_eq!(looped(5), 5);
    }

    #[test]
    fn test_via_praedicatum() {
        let router = via_praedicatum(|x: &i32| *x > 0);

        assert_eq!(router(5), Aut::sinister(5));
        assert_eq!(router(-3), Aut::dexter(-3));
        assert_eq!(router(0), Aut::dexter(0));
    }

    #[test]
    fn test_coalesco() {
        assert_eq!(coalesco(Aut::<i32, i32>::sinister(42)), 42);
        assert_eq!(coalesco(Aut::<i32, i32>::dexter(100)), 100);
    }

    #[test]
    fn test_inicio() {
        let left: Aut<i32, &str> = inicio_sinister(42);
        let right: Aut<i32, &str> = inicio_dexter("hello");

        assert_eq!(left, Aut::sinister(42));
        assert_eq!(right, Aut::dexter("hello"));
    }

    // Arrow Laws Tests

    #[test]
    fn test_arrow_identity_law() {
        // arr id = id
        let arr_id = arr(|x: i32| x);
        let id_fn: BoxedFn<i32, i32> = id();

        for x in [0, 1, -1, 42, 100] {
            assert_eq!(arr_id(x), id_fn(x));
        }
    }

    #[test]
    fn test_arrow_composition_law() {
        // arr (f >>> g) = arr f >>> arr g
        let f = |x: i32| x + 1;
        let g = |x: i32| x * 2;

        let arr_composed = arr(move |x| g(f(x)));
        let arr_f = arr(f);
        let arr_g = arr(g);
        let composed_arr = compose(arr_g, arr_f);

        for x in [0, 1, -1, 42, 100] {
            assert_eq!(arr_composed(x), composed_arr(x));
        }
    }

    #[test]
    fn test_arrow_first_composition() {
        // first (f >>> g) = first f >>> first g
        let f: BoxedFn<i32, i32> = Box::new(|x| x + 1);
        let g: BoxedFn<i32, i32> = Box::new(|x| x * 2);

        let composed = compose(Box::new(g) as BoxedFn<i32, i32>, f);
        let first_composed = first::<i32, i32, &str>(composed);

        let f2: BoxedFn<i32, i32> = Box::new(|x| x + 1);
        let g2: BoxedFn<i32, i32> = Box::new(|x| x * 2);
        let first_f = first::<i32, i32, &str>(f2);
        let first_g = first::<i32, i32, &str>(g2);
        let composed_first = compose(first_g, first_f);

        let input = (5, "test");
        assert_eq!(first_composed(input), composed_first(input));
    }
}
