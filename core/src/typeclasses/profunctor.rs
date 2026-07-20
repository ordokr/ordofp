//! # Profunctor (Profunctio)
//!
//! A profunctor is a bifunctor that is contravariant in its first argument and covariant
//! in its second argument. Profunctors generalize the concept of a function, where the
//! input type varies contravariantly and the output type varies covariantly.
//!
//! ## Mathematical Definition
//!
//! A profunctor P : C^op × D → Set is a functor where:
//! - The first argument varies contravariantly (opposite direction)
//! - The second argument varies covariantly (same direction)
//!
//! ## Laws
//!
//! A valid profunctor must satisfy:
//!
//! 1. **Identity:**
//!    ```text
//!    dimap id id = id
//!    ```
//!
//! 2. **Composition:**
//!    ```text
//!    dimap (f . g) (h . i) = dimap g h . dimap f i
//!    ```
//!
//! ## Common Use Cases
//!
//! 1. **Optics** - Profunctor optics unify lens/prism/traversal under `dimap`
//! 2. **Arrows** - Arrow-like computations with explicit input/output transformations
//! 3. **Adapters** - Transforming interfaces between incompatible types
//!
//! ## Scholastic Naming
//!
//! Following `OrdoFP`'s Scholastic naming convention:
//! - `Profunctio` - Latin form of "profunctor"
//! - `divertere` - Latin for "to turn different ways", the dimap operation

use alloc::sync::Arc;

/// A profunctor that is contravariant in its first type parameter and covariant in its second.
///
/// Profunctors generalize functions - they consume one type and produce another.
/// The key operation `dimap` transforms both the input type (contravariantly) and
/// the output type (covariantly) simultaneously.
///
/// # Type Parameters
///
/// - `A`: The contravariant input type (consumed)
/// - `B`: The covariant output type (produced)
///
/// # Examples
///
/// ```rust
/// use ordofp_core::typeclasses::profunctor::{FnProfunctio, Profunctio};
///
/// // A function is a profunctor
/// let add_one: FnProfunctio<i32, i32> = FnProfunctio::new(|x| x + 1);
///
/// // dimap: contramap the input, map the output
/// let parsed_and_stringified = add_one.divertere(
///     |s: &str| s.parse::<i32>().unwrap_or(0),  // contramap input
///     |n: i32| n.to_string()                    // map output
/// );
///
/// assert_eq!(parsed_and_stringified.run("5"), "6");
/// ```
pub trait Profunctio {
    /// The input (contravariant) type parameter.
    type Input;
    /// The output (covariant) type parameter.
    type Output;
    /// The result type when transforming both type parameters.
    type Target<A, B>;

    /// Transform both type parameters simultaneously.
    ///
    /// Named `divertere` (Latin for "to turn different ways") following Scholastic naming.
    ///
    /// # Arguments
    ///
    /// * `f` - Contravariant transformation for the input (B -> A)
    /// * `g` - Covariant transformation for the output (C -> D)
    fn divertere<A: 'static, D: 'static, F, G>(self, f: F, g: G) -> Self::Target<A, D>
    where
        F: Fn(A) -> Self::Input + Send + Sync + 'static,
        G: Fn(Self::Output) -> D + Send + Sync + 'static;

    /// Alias for `divertere` using standard naming.
    #[inline]
    fn dimap<A: 'static, D: 'static, F, G>(self, f: F, g: G) -> Self::Target<A, D>
    where
        Self: Sized,
        F: Fn(A) -> Self::Input + Send + Sync + 'static,
        G: Fn(Self::Output) -> D + Send + Sync + 'static,
    {
        self.divertere(f, g)
    }

    /// Transform only the input type (contravariant).
    ///
    /// Named `sinistro` (Latin for "on the left") following Scholastic naming.
    #[inline]
    fn sinistro<A: 'static, F>(self, f: F) -> Self::Target<A, Self::Output>
    where
        Self: Sized,
        F: Fn(A) -> Self::Input + Send + Sync + 'static,
        Self::Output: Clone + Send + Sync + 'static,
    {
        self.divertere(f, |b| b)
    }

    /// Alias for `sinistro` - transform only the input (left/contravariant).
    #[inline]
    fn lmap<A: 'static, F>(self, f: F) -> Self::Target<A, Self::Output>
    where
        Self: Sized,
        F: Fn(A) -> Self::Input + Send + Sync + 'static,
        Self::Output: Clone + Send + Sync + 'static,
    {
        self.sinistro(f)
    }

    /// Transform only the output type (covariant).
    ///
    /// Named `dextro` (Latin for "on the right") following Scholastic naming.
    #[inline]
    fn dextro<D: 'static, G>(self, g: G) -> Self::Target<Self::Input, D>
    where
        Self: Sized,
        G: Fn(Self::Output) -> D + Send + Sync + 'static,
        Self::Input: Clone + Send + Sync + 'static,
    {
        self.divertere(|a| a, g)
    }

    /// Alias for `dextro` - transform only the output (right/covariant).
    #[inline]
    fn rmap<D: 'static, G>(self, g: G) -> Self::Target<Self::Input, D>
    where
        Self: Sized,
        G: Fn(Self::Output) -> D + Send + Sync + 'static,
        Self::Input: Clone + Send + Sync + 'static,
    {
        self.dextro(g)
    }
}

/// A function wrapper that implements Profunctio.
///
/// Named `FnProfunctio` - a profunctor based on functions.
#[derive(Clone)]
pub struct FnProfunctio<A, B> {
    f: Arc<dyn Fn(A) -> B + Send + Sync>,
}

impl<A: 'static, B: 'static> FnProfunctio<A, B> {
    /// Creates a new `FnProfunctio` from a function.
    #[inline]
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(A) -> B + Send + Sync + 'static,
    {
        FnProfunctio { f: Arc::new(f) }
    }

    /// Runs the profunctor on an input value.
    #[inline]
    pub fn run(&self, a: A) -> B {
        (self.f)(a)
    }
}

impl<A: 'static, B: 'static> Profunctio for FnProfunctio<A, B> {
    type Input = A;
    type Output = B;
    type Target<X, Y> = FnProfunctio<X, Y>;

    #[inline]
    fn divertere<X: 'static, Y: 'static, F, G>(self, f: F, g: G) -> FnProfunctio<X, Y>
    where
        F: Fn(X) -> A + Send + Sync + 'static,
        G: Fn(B) -> Y + Send + Sync + 'static,
    {
        let inner = self.f;
        FnProfunctio::new(move |x| g(inner(f(x))))
    }
}

/// A Star-shaped wrapper - a function `A -> F<B>` where F is a functor.
///
/// Named `Stella` (Latin for "star") following Scholastic naming.
///
/// **Honesty note:** despite the profunctor framing, `Stella` does **not**
/// implement [`Profunctio`] — it has no `dimap`. It is a named container
/// for an `A -> F` function (`new` + `run_star`), and the `B` type
/// parameter is phantom-only. A lawful `Star` profunctor instance would
/// require mapping inside `F`, which needs functor machinery this wrapper
/// does not carry.
#[derive(Clone)]
pub struct Stella<F, A, B> {
    run: Arc<dyn Fn(A) -> F + Send + Sync>,
    _marker: core::marker::PhantomData<(A, B)>,
}

impl<F: 'static, A: 'static, B: 'static> Stella<F, A, B> {
    /// Creates a new Stella from a function.
    #[inline]
    pub fn new<Func>(run: Func) -> Self
    where
        Func: Fn(A) -> F + Send + Sync + 'static,
    {
        Stella {
            run: Arc::new(run),
            _marker: core::marker::PhantomData,
        }
    }

    /// Runs the star on an input value.
    #[inline]
    pub fn run_star(&self, a: A) -> F {
        (self.run)(a)
    }
}

/// A Costar-shaped wrapper - a function `F<A> -> B` where F is a functor.
///
/// Named `Cometa` (Latin for "comet", as opposite to star) following Scholastic naming.
///
/// **Honesty note:** like [`Stella`], `Cometa` does **not** implement
/// [`Profunctio`] — it is a named container for an `F -> B` function
/// (`new` + `run_costar`) with a phantom `A` parameter, not a lawful
/// Costar profunctor instance.
#[derive(Clone)]
pub struct Cometa<F, A, B> {
    run: Arc<dyn Fn(F) -> B + Send + Sync>,
    _marker: core::marker::PhantomData<(A, B)>,
}

impl<F: 'static, A: 'static, B: 'static> Cometa<F, A, B> {
    /// Creates a new Cometa from a function.
    #[inline]
    pub fn new<Func>(run: Func) -> Self
    where
        Func: Fn(F) -> B + Send + Sync + 'static,
    {
        Cometa {
            run: Arc::new(run),
            _marker: core::marker::PhantomData,
        }
    }

    /// Runs the costar on an input value.
    #[inline]
    pub fn run_costar(&self, f: F) -> B {
        (self.run)(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::{String, ToString};

    #[test]
    fn test_fn_profunctio_new() {
        let add_one: FnProfunctio<i32, i32> = FnProfunctio::new(|x| x + 1);
        assert_eq!(add_one.run(5), 6);
    }

    #[test]
    fn test_fn_profunctio_dimap() {
        let add_one: FnProfunctio<i32, i32> = FnProfunctio::new(|x| x + 1);

        // Contramap input (parse string), map output (to string)
        let string_to_string =
            add_one.divertere(|s: String| s.parse::<i32>().unwrap_or(0), |n| n.to_string());

        assert_eq!(string_to_string.run("5".to_string()), "6");
        assert_eq!(string_to_string.run("invalid".to_string()), "1"); // 0 + 1
    }

    #[test]
    fn test_fn_profunctio_lmap() {
        let double: FnProfunctio<i32, i32> = FnProfunctio::new(|x| x * 2);

        // Only transform input
        let parse_and_double = double.lmap(|s: String| s.parse::<i32>().unwrap_or(0));

        assert_eq!(parse_and_double.run("5".to_string()), 10);
    }

    #[test]
    fn test_fn_profunctio_rmap() {
        let double: FnProfunctio<i32, i32> = FnProfunctio::new(|x| x * 2);

        // Only transform output
        let double_to_string = double.rmap(|n| n.to_string());

        assert_eq!(double_to_string.run(5), "10");
    }

    #[test]
    fn test_profunctor_identity_law() {
        // dimap id id == id
        let add_one: FnProfunctio<i32, i32> = FnProfunctio::new(|x| x + 1);
        let identity_mapped = add_one.clone().dimap(|x: i32| x, |x| x);

        assert_eq!(add_one.run(5), identity_mapped.run(5));
    }

    #[test]
    fn test_stella_new() {
        let to_option: Stella<Option<i32>, i32, i32> = Stella::new(|x: i32| Some(x * 2));
        assert_eq!(to_option.run_star(5), Some(10));
    }

    #[test]
    fn test_cometa_new() {
        let from_option: Cometa<Option<i32>, i32, i32> =
            Cometa::new(|opt: Option<i32>| opt.unwrap_or(0));
        assert_eq!(from_option.run_costar(Some(5)), 5);
        assert_eq!(from_option.run_costar(None), 0);
    }
}
