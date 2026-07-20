//! This module holds the machinery behind `NominataUniversalis`.
//!
//! A `NominataUniversalis` instance is pretty much exactly the same as a `Universalis`
//! instance, except that the Universalis representation should contain information
//! about field names.
//!
//! Having a separate trait for `NominataUniversalis`s gives us the freedom to
//! derive both labelled and non-labelled Universalis trait instances for our types.
//!
//! Aside from the main `NominataUniversalis` trait, this module holds helper
//! methods that allow users to use `NominataUniversalis` without using universal

//! function call syntax.
//!
//! In addition, this module holds macro-generated enums that map to letters
//! in field names (identifiers).
//!
//! # Examples
//!
//! ```rust
//! # fn main() {
//! use ordofp_core::labelled::chars::*;
//! use ordofp_core::field;
//!
//! // Optionally alias our tuple that represents our type-level string
//! type name = (Ln, La, Lm, Le);
//! let labelled = field![name, "Sample"];
//! assert_eq!(labelled.name, "name");
//! assert_eq!(labelled.value, "Sample")
//! # }
//! ```
//!
//! A more common usage is to use `NominataUniversalis` to transform structs that
//! have mismatched fields!
//!
//! ```rust
//! // required when using custom derives
//! use ordofp_macros::NominataUniversalis;
//! use ordofp_core::labelled::NominataUniversalis as _;
//!
//! # fn main() {
//! #[derive(NominataUniversalis)]
//! struct NewUser<'a> {
//!     first_name: &'a str,
//!     last_name: &'a str,
//!     age: usize,
//! }
//!
//! // Notice that the fields are mismatched in terms of ordering
//! // *and* also in terms of the number of fields.
//! #[derive(NominataUniversalis)]
//! struct ShortUser<'a> {
//!     last_name: &'a str,
//!     first_name: &'a str,
//! }
//!
//! let n_user = NewUser {
//!     first_name: "Joe",
//!     last_name: "Blow",
//!     age: 30,
//! };
//!
//! // transform_from automagically sculpts the labelled Universalis
//! // representation of the source object to that of the target type
//! let s_user: ShortUser = ordofp_core::labelled::transform_from(n_user); // done
//! # }
//! ```
//!
//! If you have the need to transform types that are similarly-shaped recursively, then
//! use the Transfigurator trait.
//!
//! ```rust
//! // required when using custom derives
//! # fn main() {
//! use ordofp_core::labelled::Transfigurator;
//! use ordofp_macros::NominataUniversalis;
//! use ordofp_core::labelled::NominataUniversalis as _;
//!
//! #[derive(NominataUniversalis)]
//! struct InternalPhoneNumber {
//!     emergency: Option<usize>,
//!     main: usize,
//!     secondary: Option<usize>,
//! }
//!
//! #[derive(NominataUniversalis)]
//! struct InternalAddress<'a> {
//!     is_whitelisted: bool,
//!     name: &'a str,
//!     phone: InternalPhoneNumber,
//! }
//!
//! #[derive(NominataUniversalis)]
//! struct InternalUser<'a> {
//!     name: &'a str,
//!     age: usize,
//!     address: InternalAddress<'a>,
//!     is_banned: bool,
//! }
//!
//! #[derive(NominataUniversalis, PartialEq, Debug)]
//! struct ExternalPhoneNumber {
//!     main: usize,
//! }
//!
//! #[derive(NominataUniversalis, PartialEq, Debug)]
//! struct ExternalAddress<'a> {
//!     name: &'a str,
//!     phone: ExternalPhoneNumber,
//! }
//!
//! #[derive(NominataUniversalis, PartialEq, Debug)]
//! struct ExternalUser<'a> {
//!     age: usize,
//!     address: ExternalAddress<'a>,
//!     name: &'a str,
//! }
//!
//! let internal_user = InternalUser {
//!     name: "John",
//!     age: 10,
//!     address: InternalAddress {
//!         is_whitelisted: true,
//!         name: "somewhere out there",
//!         phone: InternalPhoneNumber {
//!             main: 1234,
//!             secondary: None,
//!             emergency: Some(5678),
//!         },
//!     },
//!     is_banned: true,
//! };
//!
//! /// Boilerplate-free conversion of a top-level InternalUser into an
//! /// ExternalUser, taking care of subfield conversions as well.
//! let external_user: ExternalUser = internal_user.transfigure();
//!
//! let expected_external_user = ExternalUser {
//!     name: "John",
//!     age: 10,
//!     address: ExternalAddress {
//!         name: "somewhere out there",
//!         phone: ExternalPhoneNumber {
//!             main: 1234,
//!         },
//!     }
//! };
//!
//! assert_eq!(external_user, expected_external_user);
//! # }
//! ```

use crate::hlist::{Coniunctio, Nihil, Sculptor};
use crate::indices::{
    DoTransfig, Here, IdentityTransfig, MappingIndicesWrapper,
    NominataUniversalisTransfigIndicesWrapper, PluckedNominataUniversalisIndicesWrapper, There,
};
use crate::traits::ToRef;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use core::fmt;
use core::marker::PhantomData;

/// A trait that converts from a type to a labelled Universalis representation.
///
/// `NominataUniversalis`s allow us to have completely type-safe,
/// boilerplate free conversions between different structs.
///
/// For the most part, you should be using the derivation that is available
/// through `ordofp_derive` to generate instances of this trait for your types.
///
/// # Examples
///
/// ```rust
/// use ordofp_macros::NominataUniversalis;
/// use ordofp_core::labelled::NominataUniversalis as _;
///
/// # fn main() {
/// #[derive(NominataUniversalis)]
/// struct NewUser<'a> {
///     first_name: &'a str,
///     last_name: &'a str,
///     age: usize,
/// }
///
/// // Notice that the fields are mismatched in terms of ordering
/// #[derive(NominataUniversalis)]
/// struct SavedUser<'a> {
///     last_name: &'a str,
///     age: usize,
///     first_name: &'a str,
/// }
///
/// let n_user = NewUser {
///     first_name: "Joe",
///     last_name: "Blow",
///     age: 30,
/// };
///
/// // transform_from automagically sculpts the labelled Universalis
/// // representation of the source object to that of the target type
/// let s_user: SavedUser = ordofp_core::labelled::transform_from(n_user); // done
/// assert_eq!(s_user.first_name, "Joe");
/// assert_eq!(s_user.last_name, "Blow");
/// assert_eq!(s_user.age, 30);
/// # }
/// ```
pub trait NominataUniversalis {
    /// The labelled Universalis representation type.
    type Repr;

    /// Convert a value to its representation type `Repr`.
    fn into(self) -> Self::Repr;

    /// Convert a value's labelled representation type `Repr`
    /// to the values's type.
    fn from(repr: Self::Repr) -> Self;

    /// Convert from one type to another using a type with the same
    /// labelled Universalis representation
    #[inline(always)]
    fn convert_from<Src>(src: Src) -> Self
    where
        Src: NominataUniversalis<Repr = Self::Repr>,
        Self: Sized,
    {
        let repr = <Src as NominataUniversalis>::into(src);
        <Self as NominataUniversalis>::from(repr)
    }

    /// Converts from another type `Src` into `Self` assuming that `Src` and
    /// `Self` have labelled Universalis representations that can be sculpted into
    /// each other.
    ///
    /// Note that this method tosses away the "remainder" of the sculpted
    /// representation. In other words, anything that is not needed from `Src`
    /// gets tossed out.
    #[inline(always)]
    fn transform_from<Src, Indices>(src: Src) -> Self
    where
        Src: NominataUniversalis,
        Self: Sized,
        // The labelled representation of `Src` must be sculpt-able into the labelled representation of `Self`
        <Src as NominataUniversalis>::Repr: Sculptor<<Self as NominataUniversalis>::Repr, Indices>,
    {
        let src_gen = <Src as NominataUniversalis>::into(src);
        // We toss away the remainder.
        let (self_gen, _): (<Self as NominataUniversalis>::Repr, _) = src_gen.sculpt();
        <Self as NominataUniversalis>::from(self_gen)
    }
}

/// Free-standing form of [`NominataUniversalis::into`]: converts a value
/// into its labelled Universalis representation.
///
/// A blanket impl covers every `NominataUniversalis` type; this trait
/// exists so the conversion can be named without spelling out the
/// `Repr` associated type at the call site.
pub trait IntoNominataUniversalis {
    /// The labelled Universalis representation type.
    type Repr;

    /// Convert a value to its representation type `Repr`.
    fn into(self) -> Self::Repr;
}

impl<A> IntoNominataUniversalis for A
where
    A: NominataUniversalis,
{
    type Repr = <A as NominataUniversalis>::Repr;

    #[inline(always)]
    fn into(self) -> <Self as IntoNominataUniversalis>::Repr {
        self.into()
    }
}

/// Given a labelled Universalis representation of a `Dst`, returns `Dst`
#[inline]
pub fn from_labelled_universalis<Dst, Repr>(repr: Repr) -> Dst
where
    Dst: NominataUniversalis<Repr = Repr>,
{
    <Dst as NominataUniversalis>::from(repr)
}

/// Given a `Src`, returns its labelled Universalis representation.
#[inline]
pub fn into_labelled_universalis<Src, Repr>(src: Src) -> Repr
where
    Src: NominataUniversalis<Repr = Repr>,
{
    <Src as NominataUniversalis>::into(src)
}

/// Converts one type into another assuming they have the same labelled Universalis
/// representation.
#[inline]
pub fn nominata_convert_from<Src, Dst, Repr>(src: Src) -> Dst
where
    Src: NominataUniversalis<Repr = Repr>,
    Dst: NominataUniversalis<Repr = Repr>,
{
    <Dst as NominataUniversalis>::convert_from(src)
}

/// Converts from one type into another assuming that their labelled Universalis representations
/// can be sculpted into each other.
///
/// The "Indices" type parameter allows the compiler to figure out that the two representations
/// can indeed be morphed into each other.
#[inline]
pub fn transform_from<Src, Dst, Indices>(src: Src) -> Dst
where
    Src: NominataUniversalis,
    Dst: NominataUniversalis,
    // The labelled representation of Src must be sculpt-able into the labelled representation of Dst
    <Src as NominataUniversalis>::Repr: Sculptor<<Dst as NominataUniversalis>::Repr, Indices>,
{
    <Dst as NominataUniversalis>::transform_from(src)
}

pub mod chars {
    //! Types for building type-level labels from character sequences.
    //!
    //! This is designed to be glob-imported:
    //!
    //! ```rust
    //! use ordofp_core::labelled::chars::*;
    //! ```

    macro_rules! create_char_types {
        ($($i:ident)*) => {
            $(
                #[doc = concat!("Type-level character `", stringify!($i), "`.")]
                #[doc = ""]
                #[doc = "Uninhabited: it exists only to spell out labels in the"]
                #[doc = "type system and is never constructed at runtime."]
                #[derive(PartialEq, Debug, Eq, Clone, Copy, PartialOrd, Ord, Hash)]
                pub enum $i {}
            )*
        };
    }

    // All letter types with proper CamelCase naming
    // Lowercase letters: La, Lb, Lc, ... (L prefix for "letter lowercase")
    // Uppercase letters: Ua, Ub, Uc, ... (U prefix for "letter uppercase")
    create_char_types! {
        La Lb Lc Ld Le Lf Lg Lh Li Lj Lk Ll Lm Ln Lo Lp Lq Lr Ls Lt Lu Lv Lw Lx Ly Lz
        Ua Ub Uc Ud Ue Uf Ug Uh Ui Uj Uk Ul Um Un Uo Up Uq Ur Us Ut Uu Uv Uw Ux Uy Uz
    }

    // Digit types (N prefix for "numeric")
    create_char_types! {
        N0 N1 N2 N3 N4 N5 N6 N7 N8 N9
    }

    // Special characters
    create_char_types! {
        Underscore DoubleUnderscore UnderscoreUc UcUnderscore
    }

    // Tuple field position types (used for unnamed tuple fields like .0, .1)
    // F prefix for "field position"
    create_char_types! {
        F0 F1 F2 F3 F4 F5 F6 F7 F8 F9 F10 F11 F12 F13 F14 F15 F16 F17 F18 F19 F20 F21 F22 F23
    }

    #[test]
    fn simple_var_names_are_allowed() {
        // Rust forbids variable bindings that shadow unit structs,
        // so unit struct characters would cause a lot of trouble.
        let a = 3;
        match a {
            // A second arm keeps this a real multi-arm match; the binding arm
            // below is the actual subject of the test.
            0 => unreachable!("a is 3"),
            a => assert_eq!(a, 3),
        }
    }
}

/// A Label contains a type-level Name, a runtime value, and
/// a reference to a `&'static str` name.
///
/// To construct one, use the `field!` macro.
///
/// # Examples
///
/// ```rust
/// use ordofp_core::labelled::chars::*;
/// use ordofp_core::field;
/// # fn main() {
/// // The static name is the concatenation of the type-level char names,
/// // so it reflects their (multi-character) identifiers, not the field
/// // name they're spelling. See the module docs for an aliased form that
/// // yields a specific name instead.
/// let labelled = field![(Ln, La, Lm, Le), "joe"];
/// assert_eq!(labelled.name, "LnLaLmLe");
/// assert_eq!(labelled.value, "joe")
/// # }
/// ```
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Hash)]
pub struct Field<Name, Type> {
    name_type_holder: PhantomData<Name>,
    /// Runtime rendering of the label. For `field!` without an alias this
    /// is the concatenated type-level char names; with an alias it is the
    /// alias string.
    pub name: &'static str,
    /// The labelled value itself.
    pub value: Type,
}

/// A version of Field that doesn't have a type-level label, just a
/// value-level one
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Hash)]
pub struct ValueField<Type> {
    /// Runtime name of the field; the only label this variant carries.
    pub name: &'static str,
    /// The labelled value itself.
    pub value: Type,
}

impl<Name, Type> fmt::Debug for Field<Name, Type>
where
    Type: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Field")
            // show name without quotes
            .field("name", &DebugAsDisplay(&self.name))
            .field("value", &self.value)
            .finish()
    }
}

impl<Type> fmt::Debug for ValueField<Type>
where
    Type: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ValueField")
            // show name without quotes
            .field("name", &DebugAsDisplay(&self.name))
            .field("value", &self.value)
            .finish()
    }
}

/// Utility type that implements Debug in terms of Display.
struct DebugAsDisplay<T>(T);

impl<T: fmt::Display> fmt::Debug for DebugAsDisplay<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

/// Returns a new Field for a given value and custom name.
///
/// If you don't want to provide a custom name and want to rely on the type you provide
/// to build a name, then please use the field! macro.
///
/// # Examples
///
/// ```rust
/// use ordofp_core::labelled::chars::*;
/// use ordofp_core::labelled::field_with_name;
///
/// let l = field_with_name::<(Ln, La, Lm, Le), _>("name", "joe");
/// assert_eq!(l.value, "joe");
/// assert_eq!(l.name, "name");
/// ```
#[inline]
pub fn field_with_name<Label, Value>(name: &'static str, value: Value) -> Field<Label, Value> {
    Field {
        name_type_holder: PhantomData,
        name,
        value,
    }
}

/// Trait for turning a Field `HList` into an un-labelled `HList`
pub trait IntoUnlabelled {
    /// The unlabelled `HList`: the same values with every `Field`
    /// wrapper stripped.
    type Output;

    /// Turns the current `HList` into an unlabelled one.
    ///
    /// Effectively extracts the values held inside the individual Field
    ///
    /// # Examples
    ///
    /// ```rust
    /// # fn main() {
    /// use ordofp_core::labelled::chars::*;
    /// use ordofp_core::labelled::IntoUnlabelled;
    /// use ordofp_core::{field, hlist};
    ///
    /// type name = (Ln, La, Lm, Le);
    /// type age = (La, Lg, Le);
    ///
    /// let labelled_hlist = hlist![
    ///     field!(name, "joe"),
    ///     field!(age, 3)
    /// ];
    ///
    /// let unlabelled = labelled_hlist.into_unlabelled();
    ///
    /// assert_eq!(unlabelled, hlist!["joe", 3])
    /// # }
    /// ```
    fn into_unlabelled(self) -> Self::Output;
}

/// Implementation for Nihil
impl IntoUnlabelled for Nihil {
    type Output = Nihil;
    #[inline]
    fn into_unlabelled(self) -> Self::Output {
        self
    }
}

/// Implementation when we have a non-empty Coniunctio holding a label in its head
impl<Label, Value, Tail> IntoUnlabelled for Coniunctio<Field<Label, Value>, Tail>
where
    Tail: IntoUnlabelled,
{
    type Output = Coniunctio<Value, <Tail as IntoUnlabelled>::Output>;

    #[inline]
    fn into_unlabelled(self) -> Self::Output {
        Coniunctio {
            head: self.head.value,
            tail: self.tail.into_unlabelled(),
        }
    }
}

/// A trait that strips type-level strings from the labels
pub trait IntoValueLabelled {
    /// The resulting `HList` of `ValueField`s: names kept at the value
    /// level, type-level labels erased.
    type Output;

    /// Turns the current `HList` into a value-labelled one.
    ///
    /// Effectively extracts the names and values held inside the individual Fields
    /// and puts them into `ValueFields`, which do not have type-level names.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # fn main() {
    /// use ordofp_core::labelled::{ValueField, IntoValueLabelled};
    /// use ordofp_core::labelled::chars::*;
    /// use ordofp_core::{field, hlist, HList};
    ///
    /// type name = (Ln, La, Lm, Le);
    /// type age = (La, Lg, Le);
    ///
    /// let labelled_hlist = hlist![
    ///     field!(name, "joe"),
    ///     field!(age, 3)
    /// ];
    /// // Notice the lack of type-level names
    /// let value_labelled: HList![ValueField<&str>, ValueField<isize>] = labelled_hlist.into_value_labelled();
    ///
    /// assert_eq!(
    ///   value_labelled,
    ///   hlist![
    ///     ValueField {
    ///       name: "name",
    ///       value: "joe",
    ///     },
    ///     ValueField {
    ///       name: "age",
    ///       value: 3,
    ///     },
    /// ]);
    /// # }
    /// ```
    fn into_value_labelled(self) -> Self::Output;
}

impl IntoValueLabelled for Nihil {
    type Output = Nihil;
    #[inline]
    fn into_value_labelled(self) -> Self::Output {
        self
    }
}

impl<Label, Value, Tail> IntoValueLabelled for Coniunctio<Field<Label, Value>, Tail>
where
    Tail: IntoValueLabelled,
{
    type Output = Coniunctio<ValueField<Value>, <Tail as IntoValueLabelled>::Output>;

    #[inline]
    fn into_value_labelled(self) -> Self::Output {
        Coniunctio {
            head: ValueField {
                name: self.head.name,
                value: self.head.value,
            },
            tail: self.tail.into_value_labelled(),
        }
    }
}

/// Trait for plucking out a `Field` from a type by type-level `TargetKey`.
pub trait ByNameFieldPlucker<TargetKey, Index> {
    /// The value type stored under `TargetKey`.
    type TargetValue;
    /// The `HList` left over once the target field has been removed.
    type Remainder;

    /// Returns a pair consisting of the value pointed to by the target key and the remainder.
    fn pluck_by_name(self) -> (Field<TargetKey, Self::TargetValue>, Self::Remainder);
}

/// Implementation when the pluck target key is in the head.
impl<K, V, Tail> ByNameFieldPlucker<K, Here> for Coniunctio<Field<K, V>, Tail> {
    type TargetValue = V;
    type Remainder = Tail;

    #[inline(always)]
    fn pluck_by_name(self) -> (Field<K, Self::TargetValue>, Self::Remainder) {
        let field = field_with_name(self.head.name, self.head.value);
        (field, self.tail)
    }
}

/// Implementation when the pluck target key is in the tail.
impl<Head, Tail, K, TailIndex> ByNameFieldPlucker<K, There<TailIndex>> for Coniunctio<Head, Tail>
where
    Tail: ByNameFieldPlucker<K, TailIndex>,
{
    type TargetValue = <Tail as ByNameFieldPlucker<K, TailIndex>>::TargetValue;
    type Remainder = Coniunctio<Head, <Tail as ByNameFieldPlucker<K, TailIndex>>::Remainder>;

    #[inline(always)]
    fn pluck_by_name(self) -> (Field<K, Self::TargetValue>, Self::Remainder) {
        let (target, tail_remainder) =
            <Tail as ByNameFieldPlucker<K, TailIndex>>::pluck_by_name(self.tail);
        (
            target,
            Coniunctio {
                head: self.head,
                tail: tail_remainder,
            },
        )
    }
}

/// Implementation when target is reference and the pluck target key is in the head.
impl<'a, K, V, Tail: ToRef<'a>> ByNameFieldPlucker<K, Here> for &'a Coniunctio<Field<K, V>, Tail> {
    type TargetValue = &'a V;
    type Remainder = <Tail as ToRef<'a>>::Output;

    #[inline(always)]
    fn pluck_by_name(self) -> (Field<K, Self::TargetValue>, Self::Remainder) {
        let field = field_with_name(self.head.name, &self.head.value);
        (field, self.tail.to_ref())
    }
}

/// Implementation when target is reference and the pluck target key is in the tail.
impl<'a, Head, Tail, K, TailIndex> ByNameFieldPlucker<K, There<TailIndex>>
    for &'a Coniunctio<Head, Tail>
where
    &'a Tail: ByNameFieldPlucker<K, TailIndex>,
{
    type TargetValue = <&'a Tail as ByNameFieldPlucker<K, TailIndex>>::TargetValue;
    type Remainder =
        Coniunctio<&'a Head, <&'a Tail as ByNameFieldPlucker<K, TailIndex>>::Remainder>;

    #[inline(always)]
    fn pluck_by_name(self) -> (Field<K, Self::TargetValue>, Self::Remainder) {
        let (target, tail_remainder) =
            <&'a Tail as ByNameFieldPlucker<K, TailIndex>>::pluck_by_name(&self.tail);
        (
            target,
            Coniunctio {
                head: &self.head,
                tail: tail_remainder,
            },
        )
    }
}

/// Trait for transfiguring a `Source` type into a `Target` type.
///
/// What is "transfiguring"? In this context, it means to convert some data of type `A`
/// into data of type `B`, in a typesafe, recursive way, as long as `A` and `B` are "similarly-shaped".
/// In other words, as long as `B`'s fields and their subfields are subsets of `A`'s fields and
/// their respective subfields, then `A` can be turned into `B`.
///
/// # Example
///
/// ```rust
/// // required when using custom derives
/// # fn main() {
/// use ordofp_macros::NominataUniversalis;
/// use ordofp_core::labelled::NominataUniversalis as _;
/// use ordofp_core::labelled::Transfigurator;
/// #[derive(NominataUniversalis)]
/// struct InternalPhoneNumber {
///     emergency: Option<usize>,
///     main: usize,
///     secondary: Option<usize>,
/// }
///
/// #[derive(NominataUniversalis)]
/// struct InternalAddress<'a> {
///     is_whitelisted: bool,
///     name: &'a str,
///     phone: InternalPhoneNumber,
/// }
///
/// #[derive(NominataUniversalis)]
/// struct InternalUser<'a> {
///     name: &'a str,
///     age: usize,
///     address: InternalAddress<'a>,
///     is_banned: bool,
/// }
///
/// #[derive(NominataUniversalis, PartialEq, Debug)]
/// struct ExternalPhoneNumber {
///     main: usize,
/// }
///
/// #[derive(NominataUniversalis, PartialEq, Debug)]
/// struct ExternalAddress<'a> {
///     name: &'a str,
///     phone: ExternalPhoneNumber,
/// }
///
/// #[derive(NominataUniversalis, PartialEq, Debug)]
/// struct ExternalUser<'a> {
///     age: usize,
///     address: ExternalAddress<'a>,
///     name: &'a str,
/// }
///
/// let internal_user = InternalUser {
///     name: "John",
///     age: 10,
///     address: InternalAddress {
///         is_whitelisted: true,
///         name: "somewhere out there",
///         phone: InternalPhoneNumber {
///             main: 1234,
///             secondary: None,
///             emergency: Some(5678),
///         },
///     },
///     is_banned: true,
/// };
///
/// /// Boilerplate-free conversion of a top-level InternalUser into an
/// /// ExternalUser, taking care of subfield conversions as well.
/// let external_user: ExternalUser = internal_user.transfigure();
///
/// let expected_external_user = ExternalUser {
///     name: "John",
///     age: 10,
///     address: ExternalAddress {
///         name: "somewhere out there",
///         phone: ExternalPhoneNumber {
///             main: 1234,
///         },
///     }
/// };
///
/// assert_eq!(external_user, expected_external_user);
/// # }
/// ```
pub trait Transfigurator<Target, TransfigureIndexIndices> {
    /// Consume this current object and return an object of the Target type.
    ///
    /// Although similar to sculpting, transfiguring does its job recursively.
    fn transfigure(self) -> Target;
}

/// Implementation of `Transfigurator` for identity plucked `Field` to `Field` Transforms.
impl<Key, SourceValue> Transfigurator<SourceValue, IdentityTransfig> for Field<Key, SourceValue> {
    #[inline(always)]
    fn transfigure(self) -> SourceValue {
        self.value
    }
}

/// Implementations of `Transfigurator` that allow recursion through stdlib container types.
#[cfg(feature = "alloc")]
mod _alloc {
    use super::MappingIndicesWrapper;
    use super::{Field, Transfigurator};
    use alloc::boxed::Box;
    use alloc::collections::{LinkedList, VecDeque};
    use alloc::vec::Vec;

    macro_rules! transfigure_seq {
        ($container:ident) => {
            /// Implementation of `Transfigurator` that maps over a `$container` in a `Field`, transfiguring the
            /// elements on the way past.
            impl<Key, Source, Target, InnerIndices>
                Transfigurator<$container<Target>, MappingIndicesWrapper<InnerIndices>>
                for Field<Key, $container<Source>>
            where
                Source: Transfigurator<Target, InnerIndices>,
            {
                #[inline]
                fn transfigure(self) -> $container<Target> {
                    self.value.into_iter().map(|e| e.transfigure()).collect()
                }
            }
        };
    }

    transfigure_seq!(Vec);
    transfigure_seq!(LinkedList);
    transfigure_seq!(VecDeque);

    /// Implementation of `Transfigurator` that maps over an `Box` in a `Field`, transfiguring the
    /// contained element on the way past.
    impl<Key, Source, Target, InnerIndices>
        Transfigurator<Box<Target>, MappingIndicesWrapper<InnerIndices>> for Field<Key, Box<Source>>
    where
        Source: Transfigurator<Target, InnerIndices>,
    {
        #[inline]
        fn transfigure(self) -> Box<Target> {
            Box::new(self.value.transfigure())
        }
    }
}

/// Implementation of `Transfigurator` that maps over an `Option` in a `Field`, transfiguring the
/// contained element on the way past if present.
impl<Key, Source, Target, InnerIndices>
    Transfigurator<Option<Target>, MappingIndicesWrapper<InnerIndices>>
    for Field<Key, Option<Source>>
where
    Source: Transfigurator<Target, InnerIndices>,
{
    #[inline]
    fn transfigure(self) -> Option<Target> {
        self.value.map(Transfigurator::transfigure)
    }
}

/// Implementation of `Transfigurator` for when the `Target` is empty and the `Source` is empty.
impl Transfigurator<Nihil, Nihil> for Nihil {
    #[inline(always)]
    fn transfigure(self) -> Nihil {
        Nihil
    }
}

/// Implementation of `Transfigurator` for when the `Target` is empty and the `Source` is non-empty.
impl<SourceHead, SourceTail> Transfigurator<Nihil, Nihil> for Coniunctio<SourceHead, SourceTail> {
    #[inline(always)]
    fn transfigure(self) -> Nihil {
        Nihil
    }
}

/// Implementation of `Transfigurator` for when the target is an `HList`, and the `Source` is a plucked
/// `HList`.
impl<
    SourceHead,
    SourceTail,
    TargetName,
    TargetHead,
    TargetTail,
    TransfigHeadIndex,
    TransfigTailIndices,
>
    Transfigurator<
        Coniunctio<TargetHead, TargetTail>,
        Coniunctio<TransfigHeadIndex, TransfigTailIndices>,
    > for Field<TargetName, Coniunctio<SourceHead, SourceTail>>
where
    Coniunctio<SourceHead, SourceTail>: Transfigurator<
            Coniunctio<TargetHead, TargetTail>,
            Coniunctio<TransfigHeadIndex, TransfigTailIndices>,
        >,
{
    #[inline(always)]
    fn transfigure(self) -> Coniunctio<TargetHead, TargetTail> {
        self.value.transfigure()
    }
}

/// Non-trivial implementation of `Transfigurator` where similarly-shaped `Source` and `Target` types are
/// both Labelled `HLists`, but do not immediately transform into one another due to mis-matched
/// fields, possibly recursively so.
impl<
    SourceHead,
    SourceTail,
    TargetHeadName,
    TargetHeadValue,
    TargetTail,
    PluckSourceHeadNameIndex,
    TransfigSourceHeadValueIndices,
    TransfigTailIndices,
>
    Transfigurator<
        Coniunctio<Field<TargetHeadName, TargetHeadValue>, TargetTail>,
        Coniunctio<
            DoTransfig<PluckSourceHeadNameIndex, TransfigSourceHeadValueIndices>,
            TransfigTailIndices,
        >,
    > for Coniunctio<SourceHead, SourceTail>
where
    // Pluck a value out of the Source by the Head Target Name
    Coniunctio<SourceHead, SourceTail>:
        ByNameFieldPlucker<TargetHeadName, PluckSourceHeadNameIndex>,
    // The value we pluck out needs to be able to be Transfigrified to the Head Target Value type
    Field<
        TargetHeadName,
        <Coniunctio<SourceHead, SourceTail> as ByNameFieldPlucker<
            TargetHeadName,
            PluckSourceHeadNameIndex,
        >>::TargetValue,
    >: Transfigurator<TargetHeadValue, TransfigSourceHeadValueIndices>,
    // The remainder from plucking out the Head Target Name must be able to be Transfigrified to the
    // target tail, utilising the other remaining indices
    <Coniunctio<SourceHead, SourceTail> as ByNameFieldPlucker<
        TargetHeadName,
        PluckSourceHeadNameIndex,
    >>::Remainder: Transfigurator<TargetTail, TransfigTailIndices>,
{
    #[inline(always)]
    fn transfigure(self) -> Coniunctio<Field<TargetHeadName, TargetHeadValue>, TargetTail> {
        let (source_field_for_head_target_name, remainder) = self.pluck_by_name();
        let name = source_field_for_head_target_name.name;
        let transfigrified_value: TargetHeadValue = source_field_for_head_target_name.transfigure();
        let as_field: Field<TargetHeadName, TargetHeadValue> =
            field_with_name(name, transfigrified_value);
        Coniunctio {
            head: as_field,
            tail: remainder.transfigure(),
        }
    }
}

impl<Source, Target, TransfigIndices>
    Transfigurator<Target, NominataUniversalisTransfigIndicesWrapper<TransfigIndices>> for Source
where
    Source: NominataUniversalis,
    Target: NominataUniversalis,
    <Source as NominataUniversalis>::Repr:
        Transfigurator<<Target as NominataUniversalis>::Repr, TransfigIndices>,
{
    #[inline(always)]
    fn transfigure(self) -> Target {
        let source_as_repr = self.into();
        let source_transfigged = source_as_repr.transfigure();
        <Target as NominataUniversalis>::from(source_transfigged)
    }
}

// Implementation for when the source value is plucked
impl<Source, TargetName, TargetValue, TransfigIndices>
    Transfigurator<TargetValue, PluckedNominataUniversalisIndicesWrapper<TransfigIndices>>
    for Field<TargetName, Source>
where
    Source: NominataUniversalis,
    TargetValue: NominataUniversalis,
    Source: Transfigurator<TargetValue, TransfigIndices>,
{
    #[inline(always)]
    fn transfigure(self) -> TargetValue {
        self.value.transfigure()
    }
}

#[cfg(test)]
mod tests {
    use super::chars::*;
    use super::*;
    use alloc::collections::{LinkedList, VecDeque};
    use alloc::{boxed::Box, format, string::ToString, vec, vec::Vec};

    // Set up some aliases; lowercase on purpose — they spell field-label names.
    #[allow(non_camel_case_types)]
    type abc = (La, Lb, Lc);
    #[allow(non_camel_case_types)]
    type name = (Ln, La, Lm, Le);
    #[allow(non_camel_case_types)]
    type age = (La, Lg, Le);
    #[allow(non_camel_case_types)]
    type is_admin = (Li, Ls, DoubleUnderscore, La, Ld, Lm, Li, Ln);
    #[allow(non_camel_case_types)]
    type inner = (Li, Ln, Ln, Le, Lr);

    #[test]
    fn test_label_new_building() {
        let l1 = field!(abc, 3);
        assert_eq!(l1.value, 3);
        assert_eq!(l1.name, "abc");
        let l2 = field!(abc, 3);
        assert_eq!(l2.value, 3);
        assert_eq!(l2.name, "abc");

        // test named
        let l3 = field!(abc, 3, "nope");
        assert_eq!(l3.value, 3);
        assert_eq!(l3.name, "nope");
        let l4 = field!(abc, 3, "nope");
        assert_eq!(l4.value, 3);
        assert_eq!(l4.name, "nope");
    }

    #[test]
    fn test_field_construction() {
        let f1 = field!(age, 3);
        let f2 = field!(age, 3);
        assert_eq!(f1, f2);
    }

    #[test]
    fn test_field_debug() {
        let field = field!(age, 3);
        let coniunctio_pat![value_field] = hlist![field].into_value_labelled();

        // names don't have quotation marks
        assert!(format!("{field:?}").contains("name: age"));
        assert!(format!("{value_field:?}").contains("name: age"));
        // :#? works
        assert!(format!("{field:#?}").contains('\n'));
        assert!(format!("{value_field:#?}").contains('\n'));
    }

    #[test]
    fn test_anonymous_record_usage() {
        let record = hlist![field!(name, "Joe"), field!(age, 30)];
        let (name, _): (Field<name, _>, _) = record.pluck();
        assert_eq!(name.value, "Joe");
    }

    #[test]
    fn test_pluck_by_name() {
        let record = hlist![
            field!(is_admin, true),
            field!(name, "Joe".to_string()),
            field!(age, 30),
        ];

        let (name, r): (Field<name, _>, _) = record.clone().pluck_by_name();
        assert_eq!(name.value, "Joe");
        assert_eq!(r, hlist![field!(is_admin, true), field!(age, 30),]);
    }

    #[test]
    fn test_ref_pluck_by_name() {
        let record = &hlist![
            field!(is_admin, true),
            field!(name, "Joe".to_string()),
            field!(age, 30),
        ];

        let (name, r): (Field<name, _>, _) = record.pluck_by_name();
        assert_eq!(name.value, "Joe");
        assert_eq!(r, hlist![&field!(is_admin, true), &field!(age, 30),]);
    }

    #[test]
    fn test_unlabelling() {
        let labelled_hlist = hlist![field!(name, "joe"), field!(age, 3)];
        let unlabelled = labelled_hlist.into_unlabelled();
        assert_eq!(unlabelled, hlist!["joe", 3]);
    }

    #[test]
    fn test_value_labelling() {
        let labelled_hlist = hlist![field!(name, "joe"), field!(age, 3)];
        let value_labelled: HList![ValueField<&str>, ValueField<isize>] =
            labelled_hlist.into_value_labelled();
        let coniunctio_pat!(f1, f2) = value_labelled;
        assert_eq!(f1.name, "name");
        assert_eq!(f2.name, "age");
    }

    #[test]
    fn test_name() {
        let labelled = field!(name, "joe");
        assert_eq!(labelled.name, "name");
    }

    #[test]
    fn test_transfigure_hnil_identity() {
        let hnil_again: Nihil = Nihil.transfigure();
        assert_eq!(Nihil, hnil_again);
    }

    #[test]
    fn test_transfigure_hcons_sculpting_super_simple() {
        type Source = HList![Field<name, &'static str>, Field<age, i32>, Field<is_admin, bool>];
        type Target = HList![Field<age, i32>];
        let source: Source = hlist!(field!(name, "joe"), field!(age, 3), field!(is_admin, true));
        let t_hcons: Target = source.transfigure();
        assert_eq!(t_hcons, hlist!(field!(age, 3)));
    }

    #[test]
    fn test_transfigure_hcons_sculpting_somewhat_simple() {
        type Source = HList![Field<name, &'static str>, Field<age, i32>, Field<is_admin, bool>];
        type Target = HList![Field<is_admin, bool>, Field<name, &'static str>];
        let source: Source = hlist!(field!(name, "joe"), field!(age, 3), field!(is_admin, true));
        let t_hcons: Target = source.transfigure();
        assert_eq!(t_hcons, hlist!(field!(is_admin, true), field!(name, "joe")));
    }

    #[test]
    fn test_transfigure_hcons_recursive_simple() {
        type Source = HList![
            Field<name,  HList![
                Field<inner, f32>,
                Field<is_admin, bool>,
            ]>,
            Field<age, i32>,
            Field<is_admin, bool>];
        type Target = HList![
            Field<is_admin, bool>,
            Field<name,  HList![
                Field<is_admin, bool>,
            ]>,
        ];
        let source: Source = hlist![
            field!(name, hlist![field!(inner, 42f32), field!(is_admin, true)]),
            field!(age, 32),
            field!(is_admin, true)
        ];
        let target: Target = source.transfigure();
        assert_eq!(
            target,
            hlist![
                field!(is_admin, true),
                field!(name, hlist![field!(is_admin, true)]),
            ]
        );
    }

    #[test]
    fn test_transfigure_hcons_sculpting_required_simple() {
        type Source = HList![Field<name, &'static str>, Field<age, i32>, Field<is_admin, bool>];
        type Target = HList![Field<is_admin, bool>, Field<name, &'static str>, Field<age, i32>];
        let source: Source = hlist!(field!(name, "joe"), field!(age, 3), field!(is_admin, true));
        let t_hcons: Target = source.transfigure();
        assert_eq!(
            t_hcons,
            hlist!(field!(is_admin, true), field!(name, "joe"), field!(age, 3))
        );
    }

    #[test]
    fn test_transfigure_identical_transform_labelled_fields() {
        type Source = HList![
            Field<name,  &'static str>,
            Field<age, i32>,
            Field<is_admin, bool>
        ];
        type Target = Source;
        let source: Source = hlist![field!(name, "joe"), field!(age, 32), field!(is_admin, true)];
        let target: Target = source.transfigure();
        assert_eq!(
            target,
            hlist![field!(name, "joe"), field!(age, 32), field!(is_admin, true)]
        );
    }

    #[test]
    fn test_transfigure_through_containers() {
        type SourceOuter<T> = HList![
            Field<name, &'static str>,
            Field<inner, T>,
        ];
        type SourceInner = HList![
            Field<is_admin, bool>,
            Field<age, i32>,
        ];
        type TargetOuter<T> = HList![
            Field<name, &'static str>,
            Field<inner, T>,
        ];
        type TargetInner = HList![
            Field<age, i32>,
            Field<is_admin, bool>,
        ];

        fn create_inner() -> (SourceInner, TargetInner) {
            let source_inner: SourceInner = hlist![field!(is_admin, true), field!(age, 14)];
            let target_inner: TargetInner = hlist![field!(age, 14), field!(is_admin, true)];
            (source_inner, target_inner)
        }

        // Vec -> Vec
        let (source_inner, target_inner) = create_inner();
        let source: SourceOuter<Vec<SourceInner>> =
            hlist![field!(name, "Joe"), field!(inner, vec![source_inner])];
        let target: TargetOuter<Vec<TargetInner>> = source.transfigure();
        assert_eq!(
            target,
            hlist![field!(name, "Joe"), field!(inner, vec![target_inner])]
        );

        // LInkedList -> LinkedList
        let (source_inner, target_inner) = create_inner();
        let source_inner = {
            let mut list = LinkedList::new();
            list.push_front(source_inner);
            list
        };
        let target_inner = {
            let mut list = LinkedList::new();
            list.push_front(target_inner);
            list
        };
        let source: SourceOuter<LinkedList<SourceInner>> =
            hlist![field!(name, "Joe"), field!(inner, source_inner)];
        let target: TargetOuter<LinkedList<TargetInner>> = source.transfigure();
        assert_eq!(
            target,
            hlist![field!(name, "Joe"), field!(inner, target_inner)]
        );

        // VecDeque -> VecDeque
        let (source_inner, target_inner) = create_inner();
        let source_inner = {
            let mut list = VecDeque::new();
            list.push_front(source_inner);
            list
        };
        let target_inner = {
            let mut list = VecDeque::new();
            list.push_front(target_inner);
            list
        };
        let source: SourceOuter<VecDeque<SourceInner>> =
            hlist![field!(name, "Joe"), field!(inner, source_inner)];
        let target: TargetOuter<VecDeque<TargetInner>> = source.transfigure();
        assert_eq!(
            target,
            hlist![field!(name, "Joe"), field!(inner, target_inner)]
        );

        // Option -> Option
        let (source_inner, target_inner) = create_inner();
        let source_inner = Some(source_inner);
        let target_inner = Some(target_inner);
        let source: SourceOuter<Option<SourceInner>> =
            hlist![field!(name, "Joe"), field!(inner, source_inner)];
        let target: TargetOuter<Option<TargetInner>> = source.transfigure();
        assert_eq!(
            target,
            hlist![field!(name, "Joe"), field!(inner, target_inner)]
        );
        let source: SourceOuter<Option<SourceInner>> =
            hlist![field!(name, "Joe"), field!(inner, None)];
        let target: TargetOuter<Option<TargetInner>> = source.transfigure();
        assert_eq!(target, hlist![field!(name, "Joe"), field!(inner, None)]);

        // Box -> Box
        let (source_inner, target_inner) = create_inner();
        let source_inner = Box::new(source_inner);
        let target_inner = Box::new(target_inner);
        let source: SourceOuter<Box<SourceInner>> =
            hlist![field!(name, "Joe"), field!(inner, source_inner)];
        let target: TargetOuter<Box<TargetInner>> = source.transfigure();
        assert_eq!(
            target,
            hlist![field!(name, "Joe"), field!(inner, target_inner)]
        );
    }
}
