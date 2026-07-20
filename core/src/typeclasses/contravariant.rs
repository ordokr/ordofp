//! # Contravariant Functor (Contravario)
//!
//! A contravariant functor is a type constructor that allows mapping functions in a way that reverses
//! their direction. While regular functors map functions forward (A -> B), contravariant functors map
//! functions backward (B -> A).
//!
//! ## Mathematical Definition
//!
//! In category theory, a contravariant functor F from category C to category D is a functor that:
//! - Maps objects A in C to objects F(A) in D
//! - Maps morphisms f: A -> B in C to morphisms F(f): F(B) -> F(A) in D, reversing the arrow
//!
//! ## Laws
//!
//! A valid contravariant functor must satisfy these laws:
//!
//! 1. **Identity:**
//!    ```text
//!    contramap(id) = id
//!    ```
//!    Mapping the identity function should produce the identity function.
//!
//! 2. **Composition:**
//!    ```text
//!    contramap(f . g) = contramap(g) . contramap(f)
//!    ```
//!    The mapping of a composition should equal the composition of the mappings in reverse order.
//!
//! ## Common Use Cases
//!
//! 1. **Comparison Functions** - Transform comparisons to work with complex types
//! 2. **Predicates and Validation** - Transform predicates to work with different input types
//! 3. **Callbacks and Event Handlers** - Adapt callback signatures for different contexts
//!
//! ## Scholastic Naming
//!
//! Following `OrdoFP`'s Scholastic naming convention:
//! - `Contravario` - Latin for "contravariant"
//! - `contravertere` - Latin for "to turn against/opposite", the contramap operation

use core::marker::PhantomData;

/// A contravariant functor that allows mapping functions in reverse direction.
///
/// While regular functors (Functor) transform `F<A>` to `F<B>` given `A -> B`,
/// contravariant functors transform `F<A>` to `F<B>` given `B -> A`.
///
/// # Type Parameters
///
/// - `Self`: The type constructor (like `Predicate<A>`)
///
/// # Examples
///
/// ```rust
/// use ordofp_core::typeclasses::contravariant::Praedicatum2;
///
/// // Create a predicate that checks if a number is positive
/// let is_positive = Praedicatum2::new(|x: &i32| *x > 0);
///
/// // Contramap to work with string lengths
/// let is_non_empty = is_positive.contramap_ref(|s: &String| s.len() as i32);
///
/// assert!(is_non_empty.run(&"hello".to_string()));
/// assert!(!is_non_empty.run(&String::new()));
/// ```
pub trait Contravario {
    /// The type parameter of this contravariant functor.
    type Param;

    /// The result type when contramapping to type `B`.
    type Target<B>;

    /// Maps a function that transforms values of type B into values of type `Self::Param`,
    /// producing a new contravariant functor that works with type B.
    ///
    /// Named `contravertere` (Latin for "to turn against/opposite") following Scholastic naming.
    ///
    /// # Type Parameters
    ///
    /// * `B`: The new input type for the resulting functor
    ///
    /// # Arguments
    ///
    /// * `f`: Function that converts from the new type B to `Self::Param`
    fn contravertere<B, F>(self, f: F) -> Self::Target<B>
    where
        F: Fn(B) -> Self::Param;

    /// Alias for `contravertere` using standard naming.
    #[inline]
    fn contramap<B, F>(self, f: F) -> Self::Target<B>
    where
        Self: Sized,
        F: Fn(B) -> Self::Param,
    {
        self.contravertere(f)
    }
}

/// Implementation for `PhantomData` - trivial contravariant functor.
impl<A> Contravario for PhantomData<A> {
    type Param = A;
    type Target<B> = PhantomData<B>;

    #[inline]
    fn contravertere<B, F>(self, _f: F) -> PhantomData<B>
    where
        F: Fn(B) -> Self::Param,
    {
        PhantomData
    }
}

/// A predicate wrapper that implements Contravario.
///
/// Named `Praedicatum` (Latin for "predicate") following Scholastic naming.
///
/// # Examples
///
/// ```rust
/// use ordofp_core::typeclasses::contravariant::Praedicatum;
///
/// let is_even = Praedicatum::new(|x: &i32| x % 2 == 0);
/// assert!(is_even.run(&4));
/// assert!(!is_even.run(&3));
/// ```
#[derive(Clone)]
pub struct Praedicatum<A, F>
where
    F: Fn(&A) -> bool,
{
    predicate: F,
    _marker: PhantomData<A>,
}

impl<A, F> Praedicatum<A, F>
where
    F: Fn(&A) -> bool,
{
    /// Creates a new Praedicatum from a predicate function.
    #[inline]
    pub fn new(predicate: F) -> Self {
        Praedicatum {
            predicate,
            _marker: PhantomData,
        }
    }

    /// Runs the predicate on the given value.
    #[inline]
    pub fn run(&self, value: &A) -> bool {
        (self.predicate)(value)
    }
}

// Note: Praedicatum does not implement Contravario directly due to Rust type system
// limitations with impl Trait in associated types. Use Praedicatum2 for contravariant
// operations instead.

/// A safer version of Praedicatum that works with reference-based contramapping.
#[derive(Clone)]
pub struct Praedicatum2<A> {
    predicate: alloc::sync::Arc<dyn Fn(&A) -> bool + Send + Sync>,
}

#[cfg(feature = "alloc")]
impl<A: 'static> Praedicatum2<A> {
    /// Creates a new Praedicatum2 from a predicate function.
    #[inline]
    pub fn new<F>(predicate: F) -> Self
    where
        F: Fn(&A) -> bool + Send + Sync + 'static,
    {
        Praedicatum2 {
            predicate: alloc::sync::Arc::new(predicate),
        }
    }

    /// Runs the predicate on the given value.
    #[inline]
    pub fn run(&self, value: &A) -> bool {
        (self.predicate)(value)
    }

    /// Contramaps with a function from &B to A.
    #[inline]
    pub fn contramap_ref<B: 'static, F>(self, f: F) -> Praedicatum2<B>
    where
        F: Fn(&B) -> A + Send + Sync + 'static,
    {
        let pred = self.predicate;
        Praedicatum2::new(move |b: &B| pred(&f(b)))
    }
}

/// Shared comparison function stored by [`Comparatio`].
type CompareFn<A> = alloc::sync::Arc<dyn Fn(&A, &A) -> core::cmp::Ordering + Send + Sync>;

/// A comparison wrapper that implements Contravario.
///
/// Named `Comparatio` (Latin for "comparison") following Scholastic naming.
#[derive(Clone)]
pub struct Comparatio<A> {
    compare: CompareFn<A>,
}

#[cfg(feature = "alloc")]
impl<A: 'static> Comparatio<A> {
    /// Creates a new Comparatio from a comparison function.
    #[inline]
    pub fn new<F>(compare: F) -> Self
    where
        F: Fn(&A, &A) -> core::cmp::Ordering + Send + Sync + 'static,
    {
        Comparatio {
            compare: alloc::sync::Arc::new(compare),
        }
    }

    /// Creates a Comparatio from a type's natural ordering.
    #[inline]
    pub fn natural() -> Self
    where
        A: Ord,
    {
        Self::new(core::cmp::Ord::cmp)
    }

    /// Compares two values using this comparison.
    #[inline]
    pub fn compare(&self, a: &A, b: &A) -> core::cmp::Ordering {
        (self.compare)(a, b)
    }

    /// Contramaps with a function from &B to A.
    #[inline]
    pub fn contramap_ref<B: 'static, F>(self, f: F) -> Comparatio<B>
    where
        F: Fn(&B) -> A + Send + Sync + 'static,
    {
        let cmp = self.compare;
        Comparatio::new(move |a: &B, b: &B| cmp(&f(a), &f(b)))
    }

    /// Reverses the comparison order.
    #[inline]
    pub fn reverse(self) -> Self {
        let cmp = self.compare;
        Comparatio::new(move |a, b| cmp(b, a))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::{String, ToString};

    #[test]
    fn test_phantom_data_contravariant() {
        let phantom: PhantomData<i32> = PhantomData;
        let _contramapped: PhantomData<String> = phantom.contravertere(|s: String| s.len() as i32);
    }

    #[test]
    fn test_praedicatum2_new() {
        let is_positive = Praedicatum2::new(|x: &i32| *x > 0);
        assert!(is_positive.run(&5));
        assert!(!is_positive.run(&-3));
    }

    #[test]
    fn test_praedicatum2_contramap() {
        let is_positive = Praedicatum2::new(|x: &i32| *x > 0);
        let is_non_empty = is_positive.contramap_ref(|s: &String| s.len() as i32);

        assert!(is_non_empty.run(&"hello".to_string()));
        assert!(!is_non_empty.run(&String::new()));
    }

    #[test]
    fn test_comparatio_natural() {
        let cmp: Comparatio<i32> = Comparatio::natural();
        assert_eq!(cmp.compare(&1, &2), core::cmp::Ordering::Less);
        assert_eq!(cmp.compare(&2, &1), core::cmp::Ordering::Greater);
        assert_eq!(cmp.compare(&1, &1), core::cmp::Ordering::Equal);
    }

    #[test]
    fn test_comparatio_contramap() {
        let cmp: Comparatio<usize> = Comparatio::natural();
        let by_len = cmp.contramap_ref(|s: &String| s.len());

        assert_eq!(
            by_len.compare(&"hi".to_string(), &"hello".to_string()),
            core::cmp::Ordering::Less
        );
    }

    #[test]
    fn test_comparatio_reverse() {
        let cmp: Comparatio<i32> = Comparatio::natural();
        let rev = cmp.reverse();

        assert_eq!(rev.compare(&1, &2), core::cmp::Ordering::Greater);
        assert_eq!(rev.compare(&2, &1), core::cmp::Ordering::Less);
    }
}
