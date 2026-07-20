//! Aequivalentia (Iso) - Bidirectional lossless transformation between two types.
//!
//! > *"Aequivalentia est mutua convertibilitas inter terminos."*
//! > — Equivalence is the mutual convertibility between terms. (Scholastic)
//!
//! An aequivalentia (isomorphism) represents a bijective mapping between two types.
//! Every value in type `S` can be converted to type `A` and back without loss.
//!
//! Aequivalentiae are useful for:
//! - Newtype wrappers
//! - Equivalent representations of the same data
//! - Refactoring between isomorphic types

/// An aequivalentia (isomorphism) between types `S` and `A`.
///
/// An aequivalentia is defined by two functions:
/// - `forward: &S -> A` - Convert from `S` to `A`
/// - `backward: &A -> S` - Convert from `A` to `S`
///
/// The functions must be inverses of each other:
/// - `backward(forward(s)) == s` for all `s`
/// - `forward(backward(a)) == a` for all `a`
///
/// # Type Parameters
/// - `S` - The source type
/// - `A` - The target type
/// - `ForwardFn` - The forward conversion function type
/// - `BackwardFn` - The backward conversion function type
pub struct Aequivalentia<S, A, ForwardFn, BackwardFn>
where
    ForwardFn: Fn(&S) -> A,
    BackwardFn: Fn(&A) -> S,
{
    forward_fn: ForwardFn,
    backward_fn: BackwardFn,
    _phantom: core::marker::PhantomData<fn(&S) -> A>,
}

impl<S, A, ForwardFn, BackwardFn> Clone for Aequivalentia<S, A, ForwardFn, BackwardFn>
where
    ForwardFn: Fn(&S) -> A + Clone,
    BackwardFn: Fn(&A) -> S + Clone,
{
    fn clone(&self) -> Self {
        Self {
            forward_fn: self.forward_fn.clone(),
            backward_fn: self.backward_fn.clone(),
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<S, A, ForwardFn, BackwardFn> Aequivalentia<S, A, ForwardFn, BackwardFn>
where
    ForwardFn: Fn(&S) -> A,
    BackwardFn: Fn(&A) -> S,
{
    /// Create a new aequivalentia from forward and backward functions.
    ///
    /// # Arguments
    /// - `forward_fn` - Function to convert from `S` to `A`
    /// - `backward_fn` - Function to convert from `A` to `S`
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::optics::Aequivalentia;
    ///
    /// // Aequivalentia between Celsius and Fahrenheit
    /// let celsius_fahrenheit = Aequivalentia::new(
    ///     |c: &f64| *c * 9.0 / 5.0 + 32.0,
    ///     |f: &f64| (*f - 32.0) * 5.0 / 9.0,
    /// );
    ///
    /// let celsius = 0.0;
    /// let fahrenheit = celsius_fahrenheit.forward(&celsius);
    /// assert!((fahrenheit - 32.0).abs() < 0.001);
    /// ```
    #[inline]
    pub fn new(forward_fn: ForwardFn, backward_fn: BackwardFn) -> Self {
        Self {
            forward_fn,
            backward_fn,
            _phantom: core::marker::PhantomData,
        }
    }

    /// Convert from `S` to `A`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::optics::aequivalentia;
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct Meters(f64);
    ///
    /// let meters_to_feet = aequivalentia(
    ///     |m: &Meters| m.0 * 3.28084,
    ///     |f: &f64| Meters(*f / 3.28084),
    /// );
    ///
    /// let meters = Meters(1.0);
    /// let feet = meters_to_feet.forward(&meters);
    /// assert!((feet - 3.28084).abs() < 0.001);
    /// ```
    #[inline]
    pub fn forward(&self, source: &S) -> A {
        (self.forward_fn)(source)
    }

    /// Convert from `A` to `S`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::optics::aequivalentia;
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct Meters(f64);
    ///
    /// let meters_to_feet = aequivalentia(
    ///     |m: &Meters| m.0 * 3.28084,
    ///     |f: &f64| Meters(*f / 3.28084),
    /// );
    ///
    /// let feet = 3.28084;
    /// let meters = meters_to_feet.backward(&feet);
    /// assert!((meters.0 - 1.0).abs() < 0.001);
    /// ```
    #[inline]
    pub fn backward(&self, target: &A) -> S {
        (self.backward_fn)(target)
    }

    /// Modify a value by converting to `A`, applying a function, and converting back.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::optics::aequivalentia;
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct Celsius(f64);
    ///
    /// let celsius_to_fahrenheit = aequivalentia(
    ///     |c: &Celsius| c.0 * 9.0 / 5.0 + 32.0,
    ///     |f: &f64| Celsius((*f - 32.0) * 5.0 / 9.0),
    /// );
    ///
    /// let temp = Celsius(0.0);
    /// // Add 10 degrees Fahrenheit
    /// let warmer = celsius_to_fahrenheit.modify(&temp, |f| f + 10.0);
    /// // 32°F + 10°F = 42°F = (42-32) * 5/9 = 5.555...°C
    /// assert!((warmer.0 - 5.555555).abs() < 0.001);
    /// ```
    #[inline]
    pub fn modify<F>(&self, source: &S, f: F) -> S
    where
        F: FnOnce(A) -> A,
    {
        let a = self.forward(source);
        self.backward(&f(a))
    }

    /// Reverse the aequivalentia, swapping forward and backward.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::optics::aequivalentia;
    ///
    /// let swap = aequivalentia(
    ///     |(a, b): &(i32, String)| (b.clone(), *a),
    ///     |(b, a): &(String, i32)| (*a, b.clone()),
    /// );
    ///
    /// let reversed = swap.reverse();
    ///
    /// let pair = ("hello".to_string(), 42);
    /// let result = reversed.forward(&pair);
    /// assert_eq!(result, (42, "hello".to_string()));
    /// ```
    #[inline]
    pub fn reverse(&self) -> Aequivalentia<A, S, BackwardFn, ForwardFn>
    where
        ForwardFn: Clone,
        BackwardFn: Clone,
    {
        Aequivalentia {
            forward_fn: self.backward_fn.clone(),
            backward_fn: self.forward_fn.clone(),
            _phantom: core::marker::PhantomData,
        }
    }

    /// Compose this aequivalentia with another to form a chain.
    ///
    /// If `self` converts `S` to `A`, and `other` converts `A` to `B`,
    /// the composed aequivalentia converts `S` to `B`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ordofp_core::optics::aequivalentia;
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct Meters(f64);
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct Feet(f64);
    ///
    /// // Meters <-> f64
    /// let unwrap_meters = aequivalentia(
    ///     |m: &Meters| m.0,
    ///     |f: &f64| Meters(*f),
    /// );
    ///
    /// // f64 <-> Feet (with conversion)
    /// let to_feet = aequivalentia(
    ///     |f: &f64| Feet(*f * 3.28084),
    ///     |feet: &Feet| feet.0 / 3.28084,
    /// );
    ///
    /// let meters_to_feet = unwrap_meters.compose(&to_feet);
    ///
    /// let meters = Meters(1.0);
    /// let feet = meters_to_feet.forward(&meters);
    /// assert!((feet.0 - 3.28084).abs() < 0.001);
    /// ```
    #[inline]
    pub fn compose<B, ForwardFn2, BackwardFn2>(
        &self,
        other: &Aequivalentia<A, B, ForwardFn2, BackwardFn2>,
    ) -> ComposedAequivalentia<S, A, B, ForwardFn, BackwardFn, ForwardFn2, BackwardFn2>
    where
        ForwardFn: Clone,
        BackwardFn: Clone,
        ForwardFn2: Fn(&A) -> B + Clone,
        BackwardFn2: Fn(&B) -> A + Clone,
    {
        ComposedAequivalentia {
            outer: self.clone(),
            inner: other.clone(),
        }
    }
}

/// A composed aequivalentia from `S` to `B` through an intermediate `A`.
#[derive(Clone)]
pub struct ComposedAequivalentia<S, A, B, ForwardFn1, BackwardFn1, ForwardFn2, BackwardFn2>
where
    ForwardFn1: Fn(&S) -> A,
    BackwardFn1: Fn(&A) -> S,
    ForwardFn2: Fn(&A) -> B,
    BackwardFn2: Fn(&B) -> A,
{
    outer: Aequivalentia<S, A, ForwardFn1, BackwardFn1>,
    inner: Aequivalentia<A, B, ForwardFn2, BackwardFn2>,
}

impl<S, A, B, ForwardFn1, BackwardFn1, ForwardFn2, BackwardFn2>
    ComposedAequivalentia<S, A, B, ForwardFn1, BackwardFn1, ForwardFn2, BackwardFn2>
where
    ForwardFn1: Fn(&S) -> A,
    BackwardFn1: Fn(&A) -> S,
    ForwardFn2: Fn(&A) -> B,
    BackwardFn2: Fn(&B) -> A,
{
    /// Convert from `S` to `B`.
    #[inline]
    pub fn forward(&self, source: &S) -> B {
        let a = self.outer.forward(source);
        self.inner.forward(&a)
    }

    /// Convert from `B` to `S`.
    #[inline]
    pub fn backward(&self, target: &B) -> S {
        let a = self.inner.backward(target);
        self.outer.backward(&a)
    }

    /// Modify a value through the composed aequivalentia.
    #[inline]
    pub fn modify<F>(&self, source: &S, f: F) -> S
    where
        F: FnOnce(B) -> B,
    {
        let b = self.forward(source);
        self.backward(&f(b))
    }
}

/// Create a new aequivalentia from forward and backward functions.
///
/// This is a convenience function equivalent to `Aequivalentia::new`.
///
/// # Example
///
/// ```rust
/// use ordofp_core::optics::aequivalentia;
///
/// // Aequivalentia for a newtype wrapper
/// #[derive(Clone, Debug, PartialEq)]
/// struct UserId(u64);
///
/// let user_id_aeq = aequivalentia(
///     |id: &UserId| id.0,
///     |n: &u64| UserId(*n),
/// );
///
/// let id = UserId(12345);
/// assert_eq!(user_id_aeq.forward(&id), 12345);
/// assert_eq!(user_id_aeq.backward(&12345), UserId(12345));
/// ```
#[inline]
pub fn aequivalentia<S, A, ForwardFn, BackwardFn>(
    forward_fn: ForwardFn,
    backward_fn: BackwardFn,
) -> Aequivalentia<S, A, ForwardFn, BackwardFn>
where
    ForwardFn: Fn(&S) -> A,
    BackwardFn: Fn(&A) -> S,
{
    Aequivalentia::new(forward_fn, backward_fn)
}

/// An aequivalentia that works with references.
///
/// This variant avoids cloning but can only be used for inspection.
pub struct AequivalentiaRef<S, A, ForwardFn>
where
    ForwardFn: Fn(&S) -> &A,
{
    forward_fn: ForwardFn,
    // `fn(&S) -> &A` is always `Clone`/`Copy`, so (unlike `PhantomData<(S, A)>`
    // with `#[derive(Clone)]`) cloning does not spuriously require `S: Clone`.
    _phantom: core::marker::PhantomData<fn(&S) -> &A>,
}

impl<S, A, ForwardFn> Clone for AequivalentiaRef<S, A, ForwardFn>
where
    ForwardFn: Fn(&S) -> &A + Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            forward_fn: self.forward_fn.clone(),
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<S, A, ForwardFn> AequivalentiaRef<S, A, ForwardFn>
where
    ForwardFn: Fn(&S) -> &A,
{
    /// Create a new reference aequivalentia.
    #[inline]
    pub fn new(forward_fn: ForwardFn) -> Self {
        Self {
            forward_fn,
            _phantom: core::marker::PhantomData,
        }
    }

    /// Get a reference to the converted value.
    #[inline]
    pub fn forward<'a>(&self, source: &'a S) -> &'a A {
        (self.forward_fn)(source)
    }
}

// ==================== Common Aequivalentiae ====================

/// Identity aequivalentia - converts any type to itself.
///
/// # Example
///
/// ```rust
/// use ordofp_core::optics::identitas;
///
/// let id_aeq = identitas::<i32>();
/// assert_eq!(id_aeq.forward(&42), 42);
/// assert_eq!(id_aeq.backward(&42), 42);
/// ```
#[inline]
pub fn identitas<T: Clone>() -> Aequivalentia<T, T, impl Fn(&T) -> T, impl Fn(&T) -> T> {
    aequivalentia(|t: &T| t.clone(), |t: &T| t.clone())
}

/// The pair-swapping [`Aequivalentia`] returned by [`permutatio`].
///
/// Both conversion directions are capture-free closures, which coerce to plain
/// `fn` pointers — keeping the four-parameter `Aequivalentia` type nameable.
pub type PermutatioAequivalentia<A, B> =
    Aequivalentia<(A, B), (B, A), fn(&(A, B)) -> (B, A), fn(&(B, A)) -> (A, B)>;

/// Swap aequivalentia for pairs - swaps the elements of a tuple.
///
/// # Example
///
/// ```rust
/// use ordofp_core::optics::permutatio;
///
/// let swap_aeq = permutatio::<i32, &str>();
/// assert_eq!(swap_aeq.forward(&(1, "hello")), ("hello", 1));
/// assert_eq!(swap_aeq.backward(&("world", 2)), (2, "world"));
/// ```
#[inline]
pub fn permutatio<A: Clone, B: Clone>() -> PermutatioAequivalentia<A, B> {
    aequivalentia(
        |(a, b): &(A, B)| (b.clone(), a.clone()),
        |(b, a): &(B, A)| (a.clone(), b.clone()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    extern crate alloc;
    use alloc::string::{String, ToString};

    #[derive(Clone, Debug, PartialEq)]
    struct Celsius(f64);

    #[derive(Clone, Debug, PartialEq)]
    struct Fahrenheit(f64);

    #[derive(Clone, Debug, PartialEq)]
    struct UserId(u64);

    #[test]
    fn test_aequivalentia_basic() {
        let celsius_fahrenheit = aequivalentia(
            |c: &Celsius| Fahrenheit(c.0 * 9.0 / 5.0 + 32.0),
            |f: &Fahrenheit| Celsius((f.0 - 32.0) * 5.0 / 9.0),
        );

        let freezing_c = Celsius(0.0);
        let freezing_f = celsius_fahrenheit.forward(&freezing_c);
        assert!((freezing_f.0 - 32.0).abs() < 0.001);

        let back = celsius_fahrenheit.backward(&freezing_f);
        assert!((back.0 - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_aequivalentia_roundtrip() {
        let user_id_aeq = aequivalentia(|id: &UserId| id.0, |n: &u64| UserId(*n));

        let original = UserId(12345);

        // Forward then backward
        let forward = user_id_aeq.forward(&original);
        let roundtrip = user_id_aeq.backward(&forward);
        assert_eq!(roundtrip, original);

        // Backward then forward
        let n = 67890u64;
        let backward = user_id_aeq.backward(&n);
        let roundtrip2 = user_id_aeq.forward(&backward);
        assert_eq!(roundtrip2, n);
    }

    #[test]
    fn test_aequivalentia_modify() {
        let user_id_aeq = aequivalentia(|id: &UserId| id.0, |n: &u64| UserId(*n));

        let id = UserId(100);
        let incremented = user_id_aeq.modify(&id, |n| n + 1);
        assert_eq!(incremented, UserId(101));
    }

    #[test]
    fn test_aequivalentia_reverse() {
        let user_id_aeq = aequivalentia(|id: &UserId| id.0, |n: &u64| UserId(*n));

        let reversed = user_id_aeq.reverse();

        let n = 42u64;
        assert_eq!(reversed.forward(&n), UserId(42));
        assert_eq!(reversed.backward(&UserId(42)), 42);
    }

    #[test]
    fn test_aequivalentia_composition() {
        // UserId -> u64
        let unwrap = aequivalentia(|id: &UserId| id.0, |n: &u64| UserId(*n));

        // u64 -> String
        let to_string_aeq = aequivalentia(
            |n: &u64| n.to_string(),
            |s: &String| s.parse::<u64>().unwrap_or(0),
        );

        let composed = unwrap.compose(&to_string_aeq);

        let id = UserId(42);
        assert_eq!(composed.forward(&id), "42".to_string());
        assert_eq!(composed.backward(&"42".to_string()), UserId(42));
    }

    #[test]
    fn test_identitas() {
        let id_aeq = identitas::<i32>();
        assert_eq!(id_aeq.forward(&42), 42);
        assert_eq!(id_aeq.backward(&42), 42);
    }

    #[test]
    fn test_permutatio() {
        let swap_aeq = permutatio::<i32, String>();

        let pair = (42, "hello".to_string());
        let swapped = swap_aeq.forward(&pair);
        assert_eq!(swapped, ("hello".to_string(), 42));

        let back = swap_aeq.backward(&swapped);
        assert_eq!(back, pair);
    }

    #[test]
    fn test_aequivalentia_ref() {
        #[derive(Clone)]
        struct Wrapper(String);

        let aeq_ref = AequivalentiaRef::new(|w: &Wrapper| &w.0);

        let wrapper = Wrapper("hello".to_string());
        assert_eq!(aeq_ref.forward(&wrapper), "hello");
    }
}
