//! This module holds the machinery behind `Universalis`.
//!
//! It contains the `Universalis` trait and some helper methods for using the
//! `Universalis` trait without having to use universal function call syntax.
//!
//! # Examples
//!
//! ```rust
//! use ordofp_macros::Universalis;
//! use ordofp_core::universalis::convert_from;
//!
//! # fn main() {
//! // Universalis maps by shape (an HList of field types, in order), not by
//! // field name -- so mismatched names on either side of the conversion are fine.
//! #[derive(Universalis)]
//! struct ApiPerson<'a> {
//!     first_name: &'a str,
//!     last_name: &'a str,
//!     age: usize,
//! }
//!
//! #[derive(Universalis)]
//! struct DomainPerson<'a> {
//!     given_name: &'a str,
//!     family_name: &'a str,
//!     age: usize,
//! }
//!
//! let a_person = ApiPerson {
//!     first_name: "Joe",
//!     last_name: "Blow",
//!     age: 30,
//! };
//! let d_person: DomainPerson = convert_from(a_person); // done
//! assert_eq!(d_person.given_name, "Joe");
//! assert_eq!(d_person.family_name, "Blow");
//! assert_eq!(d_person.age, 30);
//! # }
//! ```

/// A trait that converts from a type to a Universalis representation.
///
/// For the most part, you should be using the derivation that is available
/// through `ordofp_derive` to generate instances of this trait for your types.
///
/// # Laws
///
/// Any implementation of `Universalis` must satisfy the following two laws:
///
/// 1. `forall x : Self. x == Universalis::from(Universalis::into(x))`
/// 2. `forall y : Repr. y == Universalis::into(Universalis::from(y))`
///
/// That is, `from` and `into` should make up an isomorphism between
/// `Self` and the representation type `Repr`.
///
/// # Examples
///
/// ```rust
/// use ordofp_macros::Universalis;
/// use ordofp_core::universalis::convert_from;
///
/// # fn main() {
/// #[derive(Universalis)]
/// struct ApiPerson<'a> {
///     first_name: &'a str,
///     last_name: &'a str,
///     age: usize,
/// }
///
/// #[derive(Universalis)]
/// struct DomainPerson<'a> {
///     given_name: &'a str,
///     family_name: &'a str,
///     age: usize,
/// }
///
/// let a_person = ApiPerson {
///     first_name: "Joe",
///     last_name: "Blow",
///     age: 30,
/// };
/// let d_person: DomainPerson = convert_from(a_person); // done
/// assert_eq!(d_person.given_name, "Joe");
/// assert_eq!(d_person.family_name, "Blow");
/// assert_eq!(d_person.age, 30);
/// # }
/// ```
pub trait Universalis {
    /// The Universalis representation type.
    type Repr;

    /// Convert a value to its representation type `Repr`.
    fn into(self) -> Self::Repr;

    /// Convert a value's representation type `Repr` to the value's type.
    fn from(repr: Self::Repr) -> Self;

    /// Convert a value to another type provided that they have
    /// the same representation type.
    #[inline]
    fn convert_from<Src>(src: Src) -> Self
    where
        Self: Sized,
        Src: Universalis<Repr = Self::Repr>,
    {
        let repr = <Src as Universalis>::into(src);
        <Self as Universalis>::from(repr)
    }

    /// Maps the given value of type `Self` by first transforming it to
    /// the representation type `Repr`, then applying a `mapper` function
    /// on `Repr` and finally transforming it back to a value of type `Self`.
    #[inline]
    fn map_repr<Mapper>(self, mapper: Mapper) -> Self
    where
        Self: Sized,
        Mapper: FnOnce(Self::Repr) -> Self::Repr,
    {
        Self::from(mapper(self.into()))
    }

    /// Maps the given value of type `Self` by first transforming it
    /// a type `Inter` that has the same representation type as `Self`,
    /// then applying a `mapper` function on `Inter` and finally transforming
    /// it back to a value of type `Self`.
    #[inline]
    fn map_inter<Inter, Mapper>(self, mapper: Mapper) -> Self
    where
        Self: Sized,
        Inter: Universalis<Repr = Self::Repr>,
        Mapper: FnOnce(Inter) -> Inter,
    {
        Self::convert_from(mapper(Inter::convert_from(self)))
    }
}

/// Given a Universalis representation `Repr` of a `Dst`, returns `Dst`.
#[inline]
pub fn from_universalis<Dst, Repr>(repr: Repr) -> Dst
where
    Dst: Universalis<Repr = Repr>,
{
    <Dst as Universalis>::from(repr)
}

/// Given a value of type `Src`, returns its Universalis representation `Repr`.
#[inline]
pub fn into_universalis<Src, Repr>(src: Src) -> Repr
where
    Src: Universalis<Repr = Repr>,
{
    <Src as Universalis>::into(src)
}

/// Converts one type `Src` into another type `Dst` assuming they have the same
/// representation type `Repr`.
#[inline]
pub fn convert_from<Src, Dst, Repr>(src: Src) -> Dst
where
    Src: Universalis<Repr = Repr>,
    Dst: Universalis<Repr = Repr>,
{
    <Dst as Universalis>::convert_from(src)
}

/// Maps a value of a given type `Origin` using a function on
/// the representation type `Repr` of `Origin`.
#[inline]
pub fn map_repr<Origin, Mapper>(val: Origin, mapper: Mapper) -> Origin
where
    Origin: Universalis,
    Mapper: FnOnce(Origin::Repr) -> Origin::Repr,
{
    <Origin as Universalis>::map_repr(val, mapper)
}

/// Maps a value of a given type `Origin` using a function on
/// a type `Inter` which has the same representation type of `Origin`.
///
/// Note that the compiler will have a hard time inferring the type variable
/// `Inter`. Thus, using `map_inter` is mostly effective if the type is
/// constrained by the input function or by the body of a lambda.
#[inline]
pub fn map_inter<Inter, Origin, Mapper>(val: Origin, mapper: Mapper) -> Origin
where
    Origin: Universalis,
    Inter: Universalis<Repr = Origin::Repr>,
    Mapper: FnOnce(Inter) -> Inter,
{
    <Origin as Universalis>::map_inter(val, mapper)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Two distinct newtypes that share the same Repr so we can test cross-type
    // conversion and the isomorphism laws without pulling in the derive macro.
    #[derive(Debug, PartialEq, Clone)]
    struct Meters(f64);

    #[derive(Debug, PartialEq, Clone)]
    struct Feet(f64);

    impl Universalis for Meters {
        type Repr = f64;
        fn into(self) -> f64 {
            self.0
        }
        fn from(repr: f64) -> Self {
            Meters(repr)
        }
    }

    impl Universalis for Feet {
        type Repr = f64;
        fn into(self) -> f64 {
            self.0
        }
        fn from(repr: f64) -> Self {
            Feet(repr)
        }
    }

    #[test]
    fn isomorphism_law_into_then_from() {
        // forall x: Self. from(into(x)) == x
        let original = Meters(2.5);
        let repr = Universalis::into(original.clone());
        let recovered = <Meters as Universalis>::from(repr);
        assert_eq!(recovered, original);
    }

    #[test]
    fn isomorphism_law_from_then_into() {
        // forall r: Repr. into(from(r)) == r
        let repr: f64 = 1.75;
        let value = <Meters as Universalis>::from(repr);
        let recovered: f64 = Universalis::into(value);
        assert_eq!(recovered, repr);
    }

    #[test]
    fn convert_from_between_distinct_types_with_same_repr() {
        // Meters and Feet share Repr = f64; convert_from must transfer the
        // raw value unchanged (unit semantics are the caller's concern).
        let m = Meters(1.0);
        let f: Feet = convert_from(m);
        assert_eq!(f, Feet(1.0));
    }

    #[test]
    fn map_repr_identity_is_noop() {
        // Applying the identity function through map_repr must leave the value
        // unchanged — this is the edge case where the mapper does nothing.
        let m = Meters(42.0);
        let result = m.clone().map_repr(|r| r);
        assert_eq!(result, m);
    }

    #[test]
    fn map_inter_with_same_type_is_identity() {
        // When Inter == Origin the round-trip is: into -> from -> id -> into -> from.
        // The value must survive the double conversion intact.
        let m = Meters(7.0);
        let result = map_inter::<Meters, Meters, _>(m.clone(), |x| x);
        assert_eq!(result, m);
    }
}
