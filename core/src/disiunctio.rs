//! Module that holds Disiunctio (coproduct) data structures, traits, and implementations
//!
//! > *\"Omnis propositio est vera vel falsa.\"*
//! > — Every proposition is true or false. (Principle of bivalence)
//!
//! A Disiunctio (disjunction) represents an ad-hoc sum type, allowing you to express
//! "one of these types" without defining a custom enum. Named after the scholastic
//! logical concept of disjunction.
//!
//! The variants are named:
//! - `Sinister` (left) - the first/head case
//! - `Dexter` (right) - the tail/remaining cases
//!
//! Think of "Disiunctio" as ad-hoc enums; allowing you to do something like this
//!
//! ```rust
//! # fn main() {
//! # use ordofp_core::Disiunctio;
//! // For simplicity, assign our Disiunctio type to a type alias
//! // This is purely optional.
//! type I32Bool = Disiunctio!(i32, bool);
//! // Inject things into our Disiunctio type
//! let co1 = I32Bool::inject(3);
//! let co2 = I32Bool::inject(true);
//!
//! // Getting stuff
//! let get_from_1a: Option<&i32> = co1.get();
//! let get_from_1b: Option<&bool> = co1.get();
//! assert_eq!(get_from_1a, Some(&3));
//! assert_eq!(get_from_1b, None);
//!
//! let get_from_2a: Option<&i32> = co2.get();
//! let get_from_2b: Option<&bool> = co2.get();
//! assert_eq!(get_from_2a, None);
//! assert_eq!(get_from_2b, Some(&true));
//!
//! // *Taking* stuff (by value)
//! let take_from_1a: Option<i32> = co1.take();
//! assert_eq!(take_from_1a, Some(3));
//!
//! // Or with a Result
//! let uninject_from_1a: Result<i32, _> = co1.uninject();
//! let uninject_from_1b: Result<bool, _> = co1.uninject();
//! assert_eq!(uninject_from_1a, Ok(3));
//! assert!(uninject_from_1b.is_err());
//! # }
//! ```
//!
//! Or, if you want to "fold" over all possible values of a Disiunctio
//!
//! ```rust
//! # use ordofp_core::{hlist, functio_poly, Disiunctio};
//! # fn main() {
//! # type I32Bool = Disiunctio!(i32, bool);
//! # let co1 = I32Bool::inject(3);
//! # let co2 = I32Bool::inject(true);
//! // In the below, we use unreachable!() to make it obvious hat we know what type of
//! // item is inside our Disiunctiones co1 and co2 but in real life, you should be writing
//! // complete functions for all the cases when folding Disiunctiones
//! //
//! // to_ref borrows every item so that we can fold without consuming the Disiunctio.
//! assert_eq!(
//!     co1.to_ref().fold(hlist![|&i| format!("i32 {}", i),
//!                              |&b| unreachable!() /* we know this won't happen for co1 */ ]),
//!     "i32 3".to_string());
//! assert_eq!(
//!     co2.to_ref().fold(hlist![|&i| unreachable!() /* we know this won't happen for co2 */,
//!                              |&b| String::from(if b { "t" } else { "f" })]),
//!     "t".to_string());
//!
//! // Here, we use the functio_poly! macro to declare a polymorphic function to avoid caring
//! // about the order in which declare handlers for the types in our Disiunctio
//! let folded = co1.fold(
//!       functio_poly![
//!         |_b: bool| -> String { unreachable!() }, /* we know this won't happen for co1 */
//!         |i:  i32 | -> String { format!("i32 {}", i) },
//!       ]
//!      );
//! assert_eq!(folded, "i32 3".to_string());
//! # }
//! ```

use crate::hlist::{Coniunctio, Nihil};
use crate::indices::{Here, There};
use crate::traits::{Func, Poly, ToMut, ToRef};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Enum type representing a Disiunctio (Disiunctio/Sum type).
///
/// > *\"Disiunctio est oratio in qua ponuntur duo vel plura incompossibilia.\"*
/// > — A disjunction is a statement in which two or more incompatible things are posited.
///
/// Think of this as a Result, but capable of supporting any arbitrary number
/// of types instead of just 2. Named after the scholastic logical term for
/// exclusive alternation.
///
/// To construct a Disiunctio, you would typically declare a type using the `Disiunctio!` type
/// macro and then use the `inject` method.
///
/// # Examples
///
/// ```rust
/// # fn main() {
/// use ordofp_core::Disiunctio;
///
/// type I32Bool = Disiunctio!(i32, bool);
/// let co1 = I32Bool::inject(3);
/// let get_from_1a: Option<&i32> = co1.get();
/// let get_from_1b: Option<&bool> = co1.get();
/// assert_eq!(get_from_1a, Some(&3));
/// assert_eq!(get_from_1b, None);
/// # }
/// ```
#[derive(PartialEq, Debug, Eq, Clone, Copy, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Disiunctio<H, T> {
    /// The left (sinister) case - Disiunctio is H
    Sinister(H),
    /// The right (dexter) case - Disiunctio is T
    Dexter(T),
}

/// Phantom type for signature purposes only (has no value)
///
/// > *\"Ex absurdo quodlibet.\"*
/// > — From the absurd, anything follows. (Principle of explosion)
///
/// `Absurdum` represents the impossible case, the terminator for the
/// Disiunctio type signature. It can never be instantiated.
///
/// Used by the macro to terminate the Disiunctio type signature
#[derive(PartialEq, Debug, Eq, Clone, Copy, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Absurdum {}

// Inherent methods
impl<Head, Tail> Disiunctio<Head, Tail> {
    /// Instantiate a Disiunctio from an element.
    ///
    /// This is generally much nicer than nested usage of `Disiunctio::{Sinister, Dexter}`.
    /// The method uses a trick with type inference to automatically build the correct variant
    /// according to the input type.
    ///
    /// In standard usage, the `Index` type parameter can be ignored,
    /// as it will typically be solved for using type inference.
    ///
    /// # Rules
    ///
    /// If the type does not appear in the Disiunctio, the conversion is forbidden.
    ///
    /// If the type appears multiple times in the Disiunctio, type inference will fail.
    ///
    /// # Example
    ///
    /// ```rust
    /// # fn main() {
    /// use ordofp_core::Disiunctio;
    /// use ordofp_core::disiunctio::Disiunctio;
    ///
    /// type I32F32 = Disiunctio!(i32, f32);
    ///
    /// // Constructing Disiunctiones using inject:
    /// let co1_nice: I32F32 = Disiunctio::inject(1i32);
    /// let co2_nice: I32F32 = Disiunctio::inject(42f32);
    ///
    /// // Compare this to the "hard way":
    /// let co1_ugly: I32F32 = Disiunctio::Sinister(1i32);
    /// let co2_ugly: I32F32 = Disiunctio::Dexter(Disiunctio::Sinister(42f32));
    ///
    /// assert_eq!(co1_nice, co1_ugly);
    /// assert_eq!(co2_nice, co2_ugly);
    ///
    /// // Feel free to use `inject` on a type alias, or even directly on the
    /// // `Disiunctio!` macro. (the latter requires wrapping the type in `<>`)
    /// let _ = I32F32::inject(42f32);
    /// let _ = <Disiunctio!(i32, f32)>::inject(42f32);
    ///
    /// // You can also use a turbofish to specify the type of the input when
    /// // it is ambiguous (e.g. an empty `vec![]`).
    /// // The Index parameter should be left as `_`.
    /// type Vi32Vf32 = Disiunctio!(Vec<i32>, Vec<f32>);
    /// let _: Vi32Vf32 = Disiunctio::inject::<Vec<i32>, _>(vec![]);
    /// # }
    /// ```
    #[inline(always)]
    pub fn inject<T, Index>(to_insert: T) -> Self
    where
        Self: DisiunctioInjector<T, Index>,
    {
        DisiunctioInjector::inject(to_insert)
    }

    /// Borrow an element from a Disiunctio by type.
    ///
    /// # Example
    ///
    /// ```rust
    /// # fn main() {
    /// use ordofp_core::Disiunctio;
    ///
    /// type I32F32 = Disiunctio!(i32, f32);
    ///
    /// // You can let type inference find the desired type:
    /// let co1 = I32F32::inject(42f32);
    /// let co1_as_i32: Option<&i32> = co1.get();
    /// let co1_as_f32: Option<&f32> = co1.get();
    /// assert_eq!(co1_as_i32, None);
    /// assert_eq!(co1_as_f32, Some(&42f32));
    ///
    /// // You can also use turbofish syntax to specify the type.
    /// // The Index parameter should be left as `_`.
    /// let co2 = I32F32::inject(1i32);
    /// assert_eq!(co2.get::<i32, _>(), Some(&1));
    /// assert_eq!(co2.get::<f32, _>(), None);
    /// # }
    /// ```
    #[inline(always)]
    pub fn get<S, Index>(&self) -> Option<&S>
    where
        Self: DisiunctioSelector<S, Index>,
    {
        DisiunctioSelector::get(self)
    }

    /// Retrieve an element from a Disiunctio by type, ignoring all others.
    ///
    /// # Example
    ///
    /// ```rust
    /// # fn main() {
    /// use ordofp_core::Disiunctio;
    ///
    /// type I32F32 = Disiunctio!(i32, f32);
    ///
    /// // You can let type inference find the desired type:
    /// let co1 = I32F32::inject(42f32);
    /// let co1_as_i32: Option<i32> = co1.take();
    /// let co1_as_f32: Option<f32> = co1.take();
    /// assert_eq!(co1_as_i32, None);
    /// assert_eq!(co1_as_f32, Some(42f32));
    ///
    /// // You can also use turbofish syntax to specify the type.
    /// // The Index parameter should be left as `_`.
    /// let co2 = I32F32::inject(1i32);
    /// assert_eq!(co2.take::<i32, _>(), Some(1));
    /// assert_eq!(co2.take::<f32, _>(), None);
    /// # }
    /// ```
    #[inline(always)]
    pub fn take<T, Index>(self) -> Option<T>
    where
        Self: DisiunctioTaker<T, Index>,
    {
        DisiunctioTaker::take(self)
    }

    /// Attempt to extract a value from a Disiunctio (or get the remaining possibilities).
    ///
    /// By chaining calls to this, one can exhaustively match all variants of a Disiunctio.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```rust
    /// # fn main() {
    /// use ordofp_core::Disiunctio;
    ///
    /// type I32F32 = Disiunctio!(i32, f32);
    /// type I32 = Disiunctio!(i32); // remainder after uninjecting f32
    /// type F32 = Disiunctio!(f32); // remainder after uninjecting i32
    ///
    /// let co1 = I32F32::inject(42f32);
    ///
    /// // You can let type inference find the desired type.
    /// let co1 = I32F32::inject(42f32);
    /// let co1_as_i32: Result<i32, F32> = co1.uninject();
    /// let co1_as_f32: Result<f32, I32> = co1.uninject();
    /// assert_eq!(co1_as_i32, Err(F32::inject(42f32)));
    /// assert_eq!(co1_as_f32, Ok(42f32));
    ///
    /// // It is not necessary to annotate the type of the remainder:
    /// let res: Result<i32, _> = co1.uninject();
    /// assert!(res.is_err());
    ///
    /// // You can also use turbofish syntax to specify the type.
    /// // The Index parameter should be left as `_`.
    /// let co2 = I32F32::inject(1i32);
    /// assert_eq!(co2.uninject::<i32, _>(), Ok(1));
    /// assert_eq!(co2.uninject::<f32, _>(), Err(I32::inject(1)));
    /// # }
    /// ```
    ///
    /// Chaining calls for an exhaustive match:
    ///
    /// ```rust
    /// # fn main() {
    /// use ordofp_core::Disiunctio;
    ///
    /// type I32F32 = Disiunctio!(i32, f32);
    ///
    /// // Be aware that this particular example could be
    /// // written far more succinctly using `fold`.
    /// fn handle_i32_f32(co: I32F32) -> &'static str {
    ///     // Remove i32 from the Disiunctio
    ///     let co = match co.uninject::<i32, _>() {
    ///         Ok(x) => return "integer!",
    ///         Err(co) => co,
    ///     };
    ///
    ///     // Remove f32 from the Disiunctio
    ///     let co = match co.uninject::<f32, _>() {
    ///         Ok(x) => return "float!",
    ///         Err(co) => co,
    ///     };
    ///
    ///     // Now co is empty
    ///     match co { /* unreachable */ }
    /// }
    ///
    /// assert_eq!(handle_i32_f32(I32F32::inject(3)), "integer!");
    /// assert_eq!(handle_i32_f32(I32F32::inject(3.0)), "float!");
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `Err` holding the remainder Disiunctio — the same value
    /// re-typed with `T` removed from the possibilities — when the
    /// inhabited variant is not `T`. Nothing is lost: the `Err` side is
    /// how exhaustive matching proceeds to the next candidate type.
    #[inline(always)]
    pub fn uninject<T, Index>(
        self,
    ) -> Result<T, <Self as DisiunctioUninjector<T, Index>>::Remainder>
    where
        Self: DisiunctioUninjector<T, Index>,
    {
        DisiunctioUninjector::uninject(self)
    }

    /// Extract a subset of the possible types in a Disiunctio (or get the remaining possibilities)
    ///
    /// This is basically [`uninject`] on steroids.  It lets you remove a number
    /// of types from a Disiunctio at once, leaving behind the remainder in an `Err`.
    /// For instance, one can extract `Disiunctio!(C, A)` from `Disiunctio!(A, B, C, D)`
    /// to produce `Result<Disiunctio!(C, A), Disiunctio!(B, D)>`.
    ///
    /// Each type in the extracted subset is required to be part of the input Disiunctio.
    ///
    /// [`uninject`]: #method.uninject
    ///
    /// # Example
    ///
    /// Basic usage:
    ///
    /// ```rust
    /// # fn main() {
    /// use ordofp_core::Disiunctio;
    ///
    /// type I32BoolF32 = Disiunctio!(i32, bool, f32);
    /// type I32F32 = Disiunctio!(i32, f32);
    ///
    /// let co1 = I32BoolF32::inject(42_f32);
    /// let co2 = I32BoolF32::inject(true);
    ///
    /// let sub1: Result<Disiunctio!(i32, f32), _> = co1.subset();
    /// let sub2: Result<Disiunctio!(i32, f32), _> = co2.subset();
    /// assert!(sub1.is_ok());
    /// assert!(sub2.is_err());
    ///
    /// // Turbofish syntax for specifying the target subset is also supported.
    /// // The Indices parameter should be left to type inference using `_`.
    /// assert!(co1.subset::<Disiunctio!(i32, f32), _>().is_ok());
    /// assert!(co2.subset::<Disiunctio!(i32, f32), _>().is_err());
    ///
    /// // Order doesn't matter.
    /// assert!(co1.subset::<Disiunctio!(f32, i32), _>().is_ok());
    /// # }
    /// ```
    ///
    /// Like `uninject`, `subset` can be used for exhaustive matching,
    /// with the advantage that it can remove more than one type at a time:
    ///
    /// ```rust
    /// # fn main() {
    /// use ordofp_core::{Disiunctio, hlist};
    /// use ordofp_core::disiunctio::Disiunctio;
    ///
    /// fn handle_stringly_things(co: Disiunctio!(&'static str, String)) -> String {
    ///     co.fold(hlist![
    ///         |s| format!("&str {}", s),
    ///         |s| format!("String {}", s),
    ///     ])
    /// }
    ///
    /// fn handle_countly_things(co: Disiunctio!(u32)) -> String {
    ///     co.fold(hlist![
    ///         |n| vec!["."; n as usize].concat(),
    ///     ])
    /// }
    ///
    /// fn handle_all(co: Disiunctio!(String, u32, &'static str)) -> String {
    ///     // co is currently Disiunctio!(String, u32, &'static str)
    ///     let co = match co.subset().map(handle_stringly_things) {
    ///         Ok(s) => return s,
    ///         Err(co) => co,
    ///     };
    ///
    ///     // Now co is Disiunctio!(u32).
    ///     let co = match co.subset().map(handle_countly_things) {
    ///         Ok(s) => return s,
    ///         Err(co) => co,
    ///     };
    ///
    ///     // Now co is empty.
    ///     match co { /* unreachable */ }
    /// }
    ///
    /// assert_eq!(handle_all(Disiunctio::inject("hello")), "&str hello");
    /// assert_eq!(handle_all(Disiunctio::inject(String::from("World!"))), "String World!");
    /// assert_eq!(handle_all(Disiunctio::inject(4)), "....");
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `Err` holding the remainder Disiunctio — the same value
    /// re-typed with every `Targets` type removed — when the inhabited
    /// variant is not one of the `Targets`. Nothing is lost: the `Err`
    /// side is how exhaustive matching proceeds on the leftover types.
    #[inline(always)]
    pub fn subset<Targets, Indices>(
        self,
    ) -> Result<Targets, <Self as DisiunctioSubsetter<Targets, Indices>>::Remainder>
    where
        Self: DisiunctioSubsetter<Targets, Indices>,
    {
        DisiunctioSubsetter::subset(self)
    }

    /// Convert a Disiunctio into another that can hold its variants.
    ///
    /// This converts a Disiunctio into another one which is capable of holding each
    /// of its types. The most well-supported use-cases (i.e. those where type inference
    /// is capable of solving for the indices) are:
    ///
    /// * Reordering variants: `Disiunctio!(C, A, B) -> Disiunctio!(A, B, C)`
    /// * Embedding into a superset: `Disiunctio!(B, D) -> Disiunctio!(A, B, C, D, E)`
    /// * Coalescing duplicate inputs: `Disiunctio!(B, B, B, B) -> Disiunctio!(A, B, C)`
    ///
    /// and of course any combination thereof.
    ///
    /// # Rules
    ///
    /// If any type in the input does not appear in the output, the conversion is forbidden.
    ///
    /// If any type in the input appears multiple times in the output, type inference will fail.
    ///
    /// All of these rules fall naturally out of its fairly simple definition,
    /// which is equivalent to:
    ///
    /// ```text
    /// disiunctio.fold(hlist![
    ///     |x| Disiunctio::inject(x),
    ///     |x| Disiunctio::inject(x),
    ///             ...
    ///     |x| Disiunctio::inject(x),
    /// ])
    /// ```
    ///
    /// # Example
    ///
    /// ```rust
    /// # fn main() {
    /// use ordofp_core::Disiunctio;
    ///
    /// type I32BoolF32 = Disiunctio!(i32, bool, f32);
    /// type BoolI32 = Disiunctio!(bool, i32);
    ///
    /// let co = BoolI32::inject(true);
    /// let embedded: I32BoolF32 = co.embed();
    /// assert_eq!(embedded, I32BoolF32::inject(true));
    ///
    /// // Turbofish syntax for specifying the output type is also supported.
    /// // The Indices parameter should be left to type inference using `_`.
    /// let embedded = co.embed::<I32BoolF32, _>();
    /// assert_eq!(embedded, I32BoolF32::inject(true));
    /// # }
    /// ```
    #[inline(always)]
    pub fn embed<Targets, Indices>(self) -> Targets
    where
        Self: DisiunctioEmbedder<Targets, Indices>,
    {
        DisiunctioEmbedder::embed(self)
    }

    /// Borrow each variant of the Disiunctio.
    ///
    /// # Example
    ///
    /// Composing with `subset` to match a subset of variants without
    /// consuming the Disiunctio:
    ///
    /// ```rust
    /// # fn main() {
    /// use ordofp_core::Disiunctio;
    /// use ordofp_core::disiunctio::Disiunctio;
    ///
    /// let co: Disiunctio!(i32, bool, String) = Disiunctio::inject(true);
    ///
    /// assert!(co.to_ref().subset::<Disiunctio!(&bool, &String), _>().is_ok());
    /// # }
    /// ```
    #[inline(always)]
    pub fn to_ref<'a>(&'a self) -> <Self as ToRef<'a>>::Output
    where
        Self: ToRef<'a>,
    {
        ToRef::to_ref(self)
    }

    /// Borrow each variant of the `Disiunctio` mutably.
    ///
    /// # Example
    ///
    /// Composing with `subset` to match a subset of variants without
    /// consuming the Disiunctio:
    ///
    /// ```rust
    /// # fn main() {
    /// use ordofp_core::Disiunctio;
    /// use ordofp_core::disiunctio::Disiunctio;
    ///
    /// let mut co: Disiunctio!(i32, bool, String) = Disiunctio::inject(true);
    ///
    /// assert!(co.to_mut().subset::<Disiunctio!(&mut bool, &mut String), _>().is_ok());
    /// # }
    /// ```
    #[inline(always)]
    pub fn to_mut<'a>(&'a mut self) -> <Self as ToMut<'a>>::Output
    where
        Self: ToMut<'a>,
    {
        ToMut::to_mut(self)
    }

    /// Use functions to transform a Disiunctio into a single value.
    ///
    /// A variety of types are supported for the `Folder` argument:
    ///
    /// * An `hlist![]` of closures (one for each type, in order).
    /// * A single closure (for a Disiunctio that is homogenous).
    /// * A single [`Poly`].
    ///
    /// [`Poly`]: ../traits/struct.Poly.html
    ///
    /// # Example
    ///
    /// ```rust
    /// # fn main() {
    /// use ordofp_core::{Disiunctio, hlist};
    ///
    /// type I32F32Bool = Disiunctio!(i32, f32, bool);
    ///
    /// let co1 = I32F32Bool::inject(3);
    /// let co2 = I32F32Bool::inject(true);
    /// let co3 = I32F32Bool::inject(42f32);
    ///
    /// let folder = hlist![|&i| format!("int {}", i),
    ///                     |&f| format!("float {}", f),
    ///                     |&b| (if b { "t" } else { "f" }).to_string()];
    ///
    /// assert_eq!(co1.to_ref().fold(folder), "int 3".to_string());
    /// # }
    /// ```
    ///
    /// Using a polymorphic function type has the advantage of not
    /// forcing you to care about the order in which you declare
    /// handlers for the types in your Disiunctio.
    ///
    /// ```rust
    /// # fn main() {
    /// use ordofp_core::traits::{Poly, Func};
    /// use ordofp_core::Disiunctio;
    ///
    /// type I32F32Bool = Disiunctio!(i32, f32, bool);
    ///
    /// impl Func<i32> for P {
    ///     type Output = bool;
    ///     fn call(args: i32) -> Self::Output {
    ///         args > 100
    ///     }
    /// }
    /// impl Func<bool> for P {
    ///     type Output = bool;
    ///     fn call(args: bool) -> Self::Output {
    ///         args
    ///     }
    /// }
    /// impl Func<f32> for P {
    ///     type Output = bool;
    ///     fn call(args: f32) -> Self::Output {
    ///         args > 9000f32
    ///     }
    /// }
    /// struct P;
    ///
    /// let co1 = I32F32Bool::inject(3);
    /// let folded = co1.fold(Poly(P));
    /// # }
    /// ```
    #[inline(always)]
    pub fn fold<Output, Folder>(self, folder: Folder) -> Output
    where
        Self: DisiunctioFoldable<Folder, Output>,
    {
        DisiunctioFoldable::fold(self, folder)
    }

    /// Apply a function to each variant of a Disiunctio.
    ///
    /// The transforms some `Disiunctio!(A, B, C, ..., E)` into some
    /// `Disiunctio!(T, U, V, ..., Z)`. A variety of types are supported for the
    /// mapper argument:
    ///
    /// * An `hlist![]` of closures (one for each variant).
    /// * A single closure (for mapping a Disiunctio that is homogenous).
    /// * A single [`Poly`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ordofp_core::{hlist, Disiunctio};
    ///
    /// type I32F32Bool = Disiunctio!(i32, f32, bool);
    /// type BoolStrU8 = Disiunctio!(bool, &'static str, u8);
    ///
    /// let co1 = I32F32Bool::inject(3);
    /// let co2 = I32F32Bool::inject(42f32);
    /// let co3 = I32F32Bool::inject(true);
    ///
    /// let mapper = hlist![
    ///     |n| n > 0,
    ///     |f| if f == 42f32 { "😀" } else { "🤨" },
    ///     |b| if b { 1u8 } else { 0u8 },
    /// ];
    ///
    /// assert_eq!(co1.map(&mapper), BoolStrU8::inject(true));
    /// assert_eq!(co2.map(&mapper), BoolStrU8::inject("😀"));
    /// assert_eq!(co3.map(&mapper), BoolStrU8::inject(1u8));
    /// ```
    ///
    /// Using a polymorphic function type has the advantage of not forcing you
    /// to care about the order in which you declare handlers for the types in
    /// your Disiunctio.
    ///
    /// ```rust
    /// use ordofp_core::{functio_poly, Disiunctio};
    ///
    /// type I32F32Bool = Disiunctio!(i32, f32, bool);
    ///
    /// let co1 = I32F32Bool::inject(3);
    /// let co2 = I32F32Bool::inject(42f32);
    /// let co3 = I32F32Bool::inject(true);
    ///
    /// let mapper = functio_poly![
    ///     |b: bool| -> bool { !b },
    ///     |n: i32| -> i32 { n + 3 },
    ///     |f: f32| -> f32 { -f },
    /// ];
    ///
    /// assert_eq!(co1.map(&mapper), I32F32Bool::inject(6));
    /// assert_eq!(co2.map(&mapper), I32F32Bool::inject(-42f32));
    /// assert_eq!(co3.map(&mapper), I32F32Bool::inject(false));
    /// ```
    ///
    /// You can also use a singular closure if the Disiunctio variants are all
    /// the same.
    ///
    /// ```rust
    /// use ordofp_core::Disiunctio;
    ///
    /// type IntInt = Disiunctio!(i32, i32);
    /// type BoolBool = Disiunctio!(bool, bool);
    ///
    /// let mapper = |n| n > 0;
    ///
    /// let co = IntInt::Sinister(42);
    /// assert_eq!(co.map(mapper), BoolBool::Sinister(true));
    /// ```
    #[inline(always)]
    pub fn map<F>(self, mapper: F) -> <Self as DisiunctioMappable<F>>::Output
    where
        Self: DisiunctioMappable<F>,
    {
        DisiunctioMappable::map(self, mapper)
    }
}

impl<T> Disiunctio<T, Absurdum> {
    /// Extract the value from a Disiunctio with only one variant.
    ///
    /// # Example
    ///
    /// ```rust
    /// # fn main() {
    /// use ordofp_core::Disiunctio;
    ///
    /// type I32Only = Disiunctio!(i32);
    /// let co = I32Only::inject(5);
    ///
    /// assert_eq!(co.extract(), 5);
    /// # }
    /// ```
    #[inline(always)]
    pub fn extract(self) -> T {
        match self {
            Disiunctio::Sinister(v) => v,
            Disiunctio::Dexter(never) => match never {},
        }
    }
}

/// Trait for instantiating a Disiunctio from an element
///
/// This trait is part of the implementation of the inherent static method
/// [`Disiunctio::inject`]. Please see that method for more information.
///
/// You only need to import this trait when working with Universalis
/// Disiunctiones of unknown type. In most code, `Disiunctio::inject` will
/// "just work," with or without this trait.
///
/// [`Disiunctio::inject`]: enum.Disiunctio.html#method.inject
pub trait DisiunctioInjector<InjectType, Index> {
    /// Instantiate a Disiunctio from an element.
    ///
    /// Please see the [inherent static method] for more information.
    ///
    /// The only difference between that inherent method and this
    /// trait method is the location of the type parameters.
    /// (here, they are on the trait rather than the method)
    ///
    /// [inherent static method]: enum.Disiunctio.html#method.inject
    fn inject(to_insert: InjectType) -> Self;
}

impl<I, Tail> DisiunctioInjector<I, Here> for Disiunctio<I, Tail> {
    #[inline]
    fn inject(to_insert: I) -> Self {
        Disiunctio::Sinister(to_insert)
    }
}

impl<Head, I, Tail, TailIndex> DisiunctioInjector<I, There<TailIndex>> for Disiunctio<Head, Tail>
where
    Tail: DisiunctioInjector<I, TailIndex>,
{
    #[inline]
    fn inject(to_insert: I) -> Self {
        let tail_inserted = <Tail as DisiunctioInjector<I, TailIndex>>::inject(to_insert);
        Disiunctio::Dexter(tail_inserted)
    }
}

// For turning something into a Disiunctio -->

/// Trait for borrowing a Disiunctio element by type
///
/// This trait is part of the implementation of the inherent method
/// [`Disiunctio::get`]. Please see that method for more information.
///
/// You only need to import this trait when working with Universalis
/// Disiunctiones of unknown type. If you have a Disiunctio of known type,
/// then `co.get()` should "just work" even without the trait.
///
/// [`Disiunctio::get`]: enum.Disiunctio.html#method.get
pub trait DisiunctioSelector<S, I> {
    /// Borrow an element from a Disiunctio by type.
    ///
    /// Please see the [inherent method] for more information.
    ///
    /// The only difference between that inherent method and this
    /// trait method is the location of the type parameters.
    /// (here, they are on the trait rather than the method)
    ///
    /// [inherent method]: enum.Disiunctio.html#method.get
    fn get(&self) -> Option<&S>;
}

impl<Head, Tail> DisiunctioSelector<Head, Here> for Disiunctio<Head, Tail> {
    #[inline]
    fn get(&self) -> Option<&Head> {
        use Disiunctio::Sinister;
        match *self {
            Sinister(ref thing) => Some(thing),
            Disiunctio::Dexter(_) => None, // Impossible
        }
    }
}

impl<Head, FromTail, Tail, TailIndex> DisiunctioSelector<FromTail, There<TailIndex>>
    for Disiunctio<Head, Tail>
where
    Tail: DisiunctioSelector<FromTail, TailIndex>,
{
    #[inline]
    fn get(&self) -> Option<&FromTail> {
        use Disiunctio::Dexter;
        match *self {
            Dexter(ref rest) => rest.get(),
            Disiunctio::Sinister(_) => None, // Impossible
        }
    }
}

/// Trait for retrieving a Disiunctio element by type
///
/// This trait is part of the implementation of the inherent method
/// [`Disiunctio::take`]. Please see that method for more information.
///
/// You only need to import this trait when working with Universalis
/// Disiunctiones of unknown type. If you have a Disiunctio of known type,
/// then `co.take()` should "just work" even without the trait.
///
/// [`Disiunctio::take`]: enum.Disiunctio.html#method.take
pub trait DisiunctioTaker<S, I> {
    /// Retrieve an element from a Disiunctio by type, ignoring all others.
    ///
    /// Please see the [inherent method] for more information.
    ///
    /// The only difference between that inherent method and this
    /// trait method is the location of the type parameters.
    /// (here, they are on the trait rather than the method)
    ///
    /// [inherent method]: enum.Disiunctio.html#method.take
    fn take(self) -> Option<S>;
}

impl<Head, Tail> DisiunctioTaker<Head, Here> for Disiunctio<Head, Tail> {
    #[inline]
    fn take(self) -> Option<Head> {
        use Disiunctio::Sinister;
        match self {
            Sinister(thing) => Some(thing),
            Disiunctio::Dexter(_) => None, // Impossible
        }
    }
}

impl<Head, FromTail, Tail, TailIndex> DisiunctioTaker<FromTail, There<TailIndex>>
    for Disiunctio<Head, Tail>
where
    Tail: DisiunctioTaker<FromTail, TailIndex>,
{
    #[inline]
    fn take(self) -> Option<FromTail> {
        use Disiunctio::Dexter;
        match self {
            Dexter(rest) => rest.take(),
            Disiunctio::Sinister(_) => None, // Impossible
        }
    }
}

/// Trait for folding a Disiunctio into a single value.
///
/// This trait is part of the implementation of the inherent method
/// [`Disiunctio::fold`]. Please see that method for more information.
///
/// You only need to import this trait when working with Universalis
/// Disiunctiones or Folders of unknown type. If the type of everything is known,
/// then `co.fold(folder)` should "just work" even without the trait.
///
/// [`Disiunctio::fold`]: enum.Disiunctio.html#method.fold
pub trait DisiunctioFoldable<Folder, Output> {
    /// Use functions to fold a Disiunctio into a single value.
    ///
    /// Please see the [inherent method] for more information.
    ///
    /// The only difference between that inherent method and this
    /// trait method is the location of the type parameters.
    /// (here, they are on the trait rather than the method)
    ///
    /// [inherent method]: enum.Disiunctio.html#method.fold
    fn fold(self, f: Folder) -> Output;
}

impl<P, R, CH, CTail> DisiunctioFoldable<Poly<P>, R> for Disiunctio<CH, CTail>
where
    P: Func<CH, Output = R>,
    CTail: DisiunctioFoldable<Poly<P>, R>,
{
    #[inline]
    fn fold(self, f: Poly<P>) -> R {
        use Disiunctio::{Dexter, Sinister};
        match self {
            Sinister(r) => P::call(r),
            Dexter(rest) => rest.fold(f),
        }
    }
}

impl<F, R, FTail, CH, CTail> DisiunctioFoldable<Coniunctio<F, FTail>, R> for Disiunctio<CH, CTail>
where
    F: FnOnce(CH) -> R,
    CTail: DisiunctioFoldable<FTail, R>,
{
    #[inline]
    fn fold(self, f: Coniunctio<F, FTail>) -> R {
        use Disiunctio::{Dexter, Sinister};
        let f_head = f.head;
        let f_tail = f.tail;
        match self {
            Sinister(r) => (f_head)(r),
            Dexter(rest) => rest.fold(f_tail),
        }
    }
}

/// This is literally impossible; Absurdum is not instantiable
impl<F, R> DisiunctioFoldable<F, R> for Absurdum {
    #[inline(always)]
    fn fold(self, _: F) -> R {
        match self {}
    }
}

/// Trait for mapping over a Disiunctio's variants.
///
/// This trait is part of the implementation of the inherent method
/// [`Disiunctio::map`]. Please see that method for more information.
///
/// You only need to import this trait when working with Universalis Disiunctiones or
/// mappers of unknown type. If the type of everything is known, then
/// `co.map(mapper)` should "just work" even without the trait.
pub trait DisiunctioMappable<Mapper> {
    /// The Disiunctio produced by the mapping: the same variant
    /// structure with each variant's type transformed by its function.
    type Output;

    /// Use functions to map each variant of a Disiunctio.
    ///
    /// Please see the [inherent method] for more information.
    ///
    /// The only difference between that inherent method and this
    /// trait method is the location of the type parameters.
    /// (here, they are on the trait rather than the method)
    ///
    /// [inherent method]: Disiunctio::map
    fn map(self, f: Mapper) -> Self::Output;
}

/// Implementation for mapping a Disiunctio using an `hlist!`.
impl<F, R, MapperTail, CH, CTail> DisiunctioMappable<Coniunctio<F, MapperTail>>
    for Disiunctio<CH, CTail>
where
    F: FnOnce(CH) -> R,
    CTail: DisiunctioMappable<MapperTail>,
{
    type Output = Disiunctio<R, <CTail as DisiunctioMappable<MapperTail>>::Output>;

    #[inline(always)]
    fn map(self, mapper: Coniunctio<F, MapperTail>) -> Self::Output {
        match self {
            Disiunctio::Sinister(l) => Disiunctio::Sinister((mapper.head)(l)),
            Disiunctio::Dexter(rest) => Disiunctio::Dexter(rest.map(mapper.tail)),
        }
    }
}

/// Implementation for mapping a Disiunctio using a `&hlist!`.
impl<'a, F, R, MapperTail, CH, CTail> DisiunctioMappable<&'a Coniunctio<F, MapperTail>>
    for Disiunctio<CH, CTail>
where
    F: Fn(CH) -> R,
    CTail: DisiunctioMappable<&'a MapperTail>,
{
    type Output = Disiunctio<R, <CTail as DisiunctioMappable<&'a MapperTail>>::Output>;

    #[inline(always)]
    fn map(self, mapper: &'a Coniunctio<F, MapperTail>) -> Self::Output {
        match self {
            Disiunctio::Sinister(l) => Disiunctio::Sinister((mapper.head)(l)),
            Disiunctio::Dexter(rest) => Disiunctio::Dexter(rest.map(&mapper.tail)),
        }
    }
}

/// Implementation for mapping a Disiunctio using a `&mut hlist!`.
impl<'a, F, R, MapperTail, CH, CTail> DisiunctioMappable<&'a mut Coniunctio<F, MapperTail>>
    for Disiunctio<CH, CTail>
where
    F: FnMut(CH) -> R,
    CTail: DisiunctioMappable<&'a mut MapperTail>,
{
    type Output = Disiunctio<R, <CTail as DisiunctioMappable<&'a mut MapperTail>>::Output>;

    #[inline(always)]
    fn map(self, mapper: &'a mut Coniunctio<F, MapperTail>) -> Self::Output {
        match self {
            Disiunctio::Sinister(l) => Disiunctio::Sinister((mapper.head)(l)),
            Disiunctio::Dexter(rest) => Disiunctio::Dexter(rest.map(&mut mapper.tail)),
        }
    }
}

/// Implementation for mapping a Disiunctio using a `functio_poly!`.
impl<P, CH, CTail> DisiunctioMappable<Poly<P>> for Disiunctio<CH, CTail>
where
    P: Func<CH>,
    CTail: DisiunctioMappable<Poly<P>>,
{
    type Output =
        Disiunctio<<P as Func<CH>>::Output, <CTail as DisiunctioMappable<Poly<P>>>::Output>;

    #[inline(always)]
    fn map(self, poly: Poly<P>) -> Self::Output {
        match self {
            Disiunctio::Sinister(l) => Disiunctio::Sinister(P::call(l)),
            Disiunctio::Dexter(rest) => Disiunctio::Dexter(rest.map(poly)),
        }
    }
}

/// Implementation for mapping a Disiunctio using a `&functio_poly!`.
impl<'a, P, CH, CTail> DisiunctioMappable<&'a Poly<P>> for Disiunctio<CH, CTail>
where
    P: Func<CH>,
    CTail: DisiunctioMappable<&'a Poly<P>>,
{
    type Output =
        Disiunctio<<P as Func<CH>>::Output, <CTail as DisiunctioMappable<&'a Poly<P>>>::Output>;

    #[inline(always)]
    fn map(self, poly: &'a Poly<P>) -> Self::Output {
        match self {
            Disiunctio::Sinister(l) => Disiunctio::Sinister(P::call(l)),
            Disiunctio::Dexter(rest) => Disiunctio::Dexter(rest.map(poly)),
        }
    }
}

/// Implementation for mapping a Disiunctio using a `&mut functio_poly!`.
impl<'a, P, CH, CTail> DisiunctioMappable<&'a mut Poly<P>> for Disiunctio<CH, CTail>
where
    P: Func<CH>,
    CTail: DisiunctioMappable<&'a mut Poly<P>>,
{
    type Output =
        Disiunctio<<P as Func<CH>>::Output, <CTail as DisiunctioMappable<&'a mut Poly<P>>>::Output>;

    #[inline(always)]
    fn map(self, poly: &'a mut Poly<P>) -> Self::Output {
        match self {
            Disiunctio::Sinister(l) => Disiunctio::Sinister(P::call(l)),
            Disiunctio::Dexter(rest) => Disiunctio::Dexter(rest.map(poly)),
        }
    }
}

/// Implementation for mapping a Disiunctio using a single function that can
/// handle all variants.
impl<F, R, CH, CTail> DisiunctioMappable<F> for Disiunctio<CH, CTail>
where
    F: FnMut(CH) -> R,
    CTail: DisiunctioMappable<F>,
{
    type Output = Disiunctio<R, <CTail as DisiunctioMappable<F>>::Output>;

    #[inline(always)]
    fn map(self, mut f: F) -> Self::Output {
        match self {
            Disiunctio::Sinister(l) => Disiunctio::Sinister(f(l)),
            Disiunctio::Dexter(rest) => Disiunctio::Dexter(rest.map(f)),
        }
    }
}

/// Base case map impl.
impl<F> DisiunctioMappable<F> for Absurdum {
    type Output = Absurdum;

    #[inline(always)]
    fn map(self, _: F) -> Self::Output {
        match self {}
    }
}

impl<'a, CH: 'a, CTail> ToRef<'a> for Disiunctio<CH, CTail>
where
    CTail: ToRef<'a>,
{
    type Output = Disiunctio<&'a CH, <CTail as ToRef<'a>>::Output>;

    #[inline(always)]
    fn to_ref(&'a self) -> Self::Output {
        match *self {
            Disiunctio::Sinister(ref r) => Disiunctio::Sinister(r),
            Disiunctio::Dexter(ref rest) => Disiunctio::Dexter(rest.to_ref()),
        }
    }
}

impl<'a> ToRef<'a> for Absurdum {
    type Output = Absurdum;

    #[inline(always)]
    fn to_ref(&'a self) -> Absurdum {
        match *self {}
    }
}

impl<'a, CH: 'a, CTail> ToMut<'a> for Disiunctio<CH, CTail>
where
    CTail: ToMut<'a>,
{
    type Output = Disiunctio<&'a mut CH, <CTail as ToMut<'a>>::Output>;

    #[inline(always)]
    fn to_mut(&'a mut self) -> Self::Output {
        match *self {
            Disiunctio::Sinister(ref mut r) => Disiunctio::Sinister(r),
            Disiunctio::Dexter(ref mut rest) => Disiunctio::Dexter(rest.to_mut()),
        }
    }
}

impl<'a> ToMut<'a> for Absurdum {
    type Output = Absurdum;

    #[inline(always)]
    fn to_mut(&'a mut self) -> Absurdum {
        match *self {}
    }
}

/// Trait for extracting a value from a Disiunctio in an exhaustive way.
///
/// This trait is part of the implementation of the inherent method
/// [`Disiunctio::uninject`]. Please see that method for more information.
///
/// You only need to import this trait when working with Universalis
/// Disiunctiones of unknown type. If you have a Disiunctio of known type,
/// then `co.uninject()` should "just work" even without the trait.
///
/// [`Disiunctio::uninject`]: enum.Disiunctio.html#method.uninject
pub trait DisiunctioUninjector<T, Idx>: DisiunctioInjector<T, Idx> {
    /// The Disiunctio of the possibilities left over once `T` has been
    /// removed from the candidate types.
    type Remainder;

    /// Attempt to extract a value from a Disiunctio (or get the remaining possibilities).
    ///
    /// Please see the [inherent method] for more information.
    ///
    /// The only difference between that inherent method and this
    /// trait method is the location of the type parameters.
    /// (here, they are on the trait rather than the method)
    ///
    /// # Errors
    ///
    /// Returns `Err` holding the remainder Disiunctio when the inhabited
    /// variant is not `T`; no information is lost.
    ///
    /// [inherent method]: enum.Disiunctio.html#method.uninject
    fn uninject(self) -> Result<T, Self::Remainder>;
}

impl<Hd, Tl> DisiunctioUninjector<Hd, Here> for Disiunctio<Hd, Tl> {
    type Remainder = Tl;

    #[inline]
    fn uninject(self) -> Result<Hd, Tl> {
        match self {
            Disiunctio::Sinister(h) => Ok(h),
            Disiunctio::Dexter(t) => Err(t),
        }
    }
}

impl<Hd, Tl, T, N> DisiunctioUninjector<T, There<N>> for Disiunctio<Hd, Tl>
where
    Tl: DisiunctioUninjector<T, N>,
{
    type Remainder = Disiunctio<Hd, Tl::Remainder>;

    #[inline]
    fn uninject(self) -> Result<T, Self::Remainder> {
        match self {
            Disiunctio::Sinister(h) => Err(Disiunctio::Sinister(h)),
            Disiunctio::Dexter(t) => t.uninject().map_err(Disiunctio::Dexter),
        }
    }
}

/// Trait for extracting a subset of the possible types in a Disiunctio.
///
/// This trait is part of the implementation of the inherent method
/// [`Disiunctio::subset`]. Please see that method for more information.
///
/// You only need to import this trait when working with Universalis
/// Disiunctiones of unknown type. If you have a Disiunctio of known type,
/// then `co.subset()` should "just work" even without the trait.
///
/// [`Disiunctio::subset`]: enum.Disiunctio.html#method.subset
pub trait DisiunctioSubsetter<Targets, Indices>: Sized {
    /// The Disiunctio of the possibilities left over once every type in
    /// `Targets` has been removed from the candidate types.
    type Remainder;

    /// Extract a subset of the possible types in a Disiunctio (or get the remaining possibilities)
    ///
    /// Please see the [inherent method] for more information.
    ///
    /// The only difference between that inherent method and this
    /// trait method is the location of the type parameters.
    /// (here, they are on the trait rather than the method)
    ///
    /// # Errors
    ///
    /// Returns `Err` holding the remainder Disiunctio when the inhabited
    /// variant is not one of the `Targets`; no information is lost.
    ///
    /// [inherent method]: enum.Disiunctio.html#method.subset
    fn subset(self) -> Result<Targets, Self::Remainder>;
}

impl<Choices, THead, TTail, NHead, NTail, Rem>
    DisiunctioSubsetter<Disiunctio<THead, TTail>, Coniunctio<NHead, NTail>> for Choices
where
    Self: DisiunctioUninjector<THead, NHead, Remainder = Rem>,
    Rem: DisiunctioSubsetter<TTail, NTail>,
{
    type Remainder = <Rem as DisiunctioSubsetter<TTail, NTail>>::Remainder;

    /// Attempt to extract a value from a subset of the types.
    #[inline]
    fn subset(self) -> Result<Disiunctio<THead, TTail>, Self::Remainder> {
        match self.uninject() {
            Ok(good) => Ok(Disiunctio::Sinister(good)),
            Err(bads) => match bads.subset() {
                Ok(goods) => Ok(Disiunctio::Dexter(goods)),
                Err(bads) => Err(bads),
            },
        }
    }
}

impl<Choices> DisiunctioSubsetter<Absurdum, Nihil> for Choices {
    type Remainder = Self;

    #[inline(always)]
    fn subset(self) -> Result<Absurdum, Self::Remainder> {
        Err(self)
    }
}

/// Trait for converting a Disiunctio into another that can hold its variants.
///
/// This trait is part of the implementation of the inherent method
/// [`Disiunctio::embed`]. Please see that method for more information.
///
/// You only need to import this trait when working with Universalis
/// Disiunctiones of unknown type. If you have a Disiunctio of known type,
/// then `co.embed()` should "just work" even without the trait.
///
/// [`Disiunctio::embed`]: enum.Disiunctio.html#method.embed
pub trait DisiunctioEmbedder<Out, Indices> {
    /// Convert a Disiunctio into another that can hold its variants.
    ///
    /// Please see the [inherent method] for more information.
    ///
    /// The only difference between that inherent method and this
    /// trait method is the location of the type parameters.
    /// (here, they are on the trait rather than the method)
    ///
    /// [inherent method]: enum.Disiunctio.html#method.embed
    fn embed(self) -> Out;
}

impl DisiunctioEmbedder<Absurdum, Nihil> for Absurdum {
    #[inline(always)]
    fn embed(self) -> Absurdum {
        match self {
        // impossible!
    }
    }
}

impl<Head, Tail> DisiunctioEmbedder<Disiunctio<Head, Tail>, Nihil> for Absurdum
where
    Absurdum: DisiunctioEmbedder<Tail, Nihil>,
{
    #[inline(always)]
    fn embed(self) -> Disiunctio<Head, Tail> {
        match self {
        // impossible!
    }
    }
}

impl<Head, Tail, Out, NHead, NTail> DisiunctioEmbedder<Out, Coniunctio<NHead, NTail>>
    for Disiunctio<Head, Tail>
where
    Out: DisiunctioInjector<Head, NHead>,
    Tail: DisiunctioEmbedder<Out, NTail>,
{
    #[inline]
    fn embed(self) -> Out {
        match self {
            Disiunctio::Sinister(this) => Out::inject(this),
            Disiunctio::Dexter(those) => those.embed(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Disiunctio::*;
    use super::*;

    use std::format;
    use std::string::{String, ToString};

    #[test]
    fn test_disiunctio_inject() {
        type I32StrBool = Disiunctio!(i32, &'static str, bool);

        let co1 = I32StrBool::inject(3);
        assert_eq!(co1, Sinister(3));
        let get_from_1a: Option<&i32> = co1.get();
        let get_from_1b: Option<&bool> = co1.get();
        assert_eq!(get_from_1a, Some(&3));
        assert_eq!(get_from_1b, None);

        let co2 = I32StrBool::inject(false);
        assert_eq!(co2, Dexter(Dexter(Sinister(false))));
        let get_from_2a: Option<&i32> = co2.get();
        let get_from_2b: Option<&bool> = co2.get();
        assert_eq!(get_from_2a, None);
        assert_eq!(get_from_2b, Some(&false));
    }

    #[test]
    fn test_disiunctio_fold_consuming() {
        type I32F32StrBool = Disiunctio!(i32, f32, bool);

        let co1 = I32F32StrBool::inject(3);
        let folded = co1.fold(hlist![
            |i| format!("int {i}"),
            |f| format!("float {f}"),
            |b| (if b { "t" } else { "f" }).to_string(),
        ]);

        assert_eq!(folded, "int 3".to_string());
    }

    #[test]
    fn test_disiunctio_poly_fold_consuming() {
        type I32F32StrBool = Disiunctio!(i32, f32, bool);

        impl Func<i32> for P {
            type Output = bool;
            fn call(args: i32) -> Self::Output {
                args > 100
            }
        }
        impl Func<bool> for P {
            type Output = bool;
            fn call(args: bool) -> Self::Output {
                args
            }
        }
        impl Func<f32> for P {
            type Output = bool;
            fn call(args: f32) -> Self::Output {
                args > 9000f32
            }
        }
        struct P;

        let co1 = I32F32StrBool::inject(3);
        let folded = co1.fold(Poly(P));

        assert!(!folded);
    }

    #[test]
    fn test_disiunctio_fold_non_consuming() {
        type I32F32Bool = Disiunctio!(i32, f32, bool);

        let co1 = I32F32Bool::inject(3);
        let co2 = I32F32Bool::inject(true);
        let co3 = I32F32Bool::inject(42f32);

        assert_eq!(
            co1.to_ref().fold(hlist![
                |&i| format!("int {i}"),
                |&f| format!("float {f}"),
                |&b| (if b { "t" } else { "f" }).to_string(),
            ]),
            "int 3".to_string()
        );
        assert_eq!(
            co2.to_ref().fold(hlist![
                |&i| format!("int {i}"),
                |&f| format!("float {f}"),
                |&b| (if b { "t" } else { "f" }).to_string(),
            ]),
            "t".to_string()
        );
        assert_eq!(
            co3.to_ref().fold(hlist![
                |&i| format!("int {i}"),
                |&f| format!("float {f}"),
                |&b| (if b { "t" } else { "f" }).to_string(),
            ]),
            "float 42".to_string()
        );
    }

    #[test]
    fn test_disiunctio_uninject() {
        type I32StrBool = Disiunctio!(i32, &'static str, bool);

        let co1 = I32StrBool::inject(3);
        let co2 = I32StrBool::inject("hello");
        let co3 = I32StrBool::inject(false);

        let uninject_i32_co1: Result<i32, _> = co1.uninject();
        let uninject_str_co1: Result<&'static str, _> = co1.uninject();
        let uninject_bool_co1: Result<bool, _> = co1.uninject();
        assert_eq!(uninject_i32_co1, Ok(3));
        assert!(uninject_str_co1.is_err());
        assert!(uninject_bool_co1.is_err());

        let uninject_i32_co2: Result<i32, _> = co2.uninject();
        let uninject_str_co2: Result<&'static str, _> = co2.uninject();
        let uninject_bool_co2: Result<bool, _> = co2.uninject();
        assert!(uninject_i32_co2.is_err());
        assert_eq!(uninject_str_co2, Ok("hello"));
        assert!(uninject_bool_co2.is_err());

        let uninject_i32_co3: Result<i32, _> = co3.uninject();
        let uninject_str_co3: Result<&'static str, _> = co3.uninject();
        let uninject_bool_co3: Result<bool, _> = co3.uninject();
        assert!(uninject_i32_co3.is_err());
        assert!(uninject_str_co3.is_err());
        assert_eq!(uninject_bool_co3, Ok(false));
    }

    #[test]
    fn test_disiunctio_subset() {
        type I32StrBool = Disiunctio!(i32, &'static str, bool);

        // Absurdum can be extracted from anything.
        let res: Result<Absurdum, _> = I32StrBool::inject(3).subset();
        assert!(res.is_err());

        // Compile-only proof: ...including from Absurdum itself. Never called —
        // Absurdum has no values — but the call must type-check. (Written as
        // a tail expression: binding the uninhabited result would trip the
        // unreachable_code lint.)
        fn _absurdum_subset_typechecks(absurdum: Absurdum) -> Result<Absurdum, Absurdum> {
            absurdum.subset()
        }

        {
            // Order does not matter.
            let co = I32StrBool::inject(3);
            let res: Result<Disiunctio!(bool, i32), _> = co.subset();
            assert_eq!(res, Ok(Disiunctio::Dexter(Disiunctio::Sinister(3))));

            let co = I32StrBool::inject("4");
            let res: Result<Disiunctio!(bool, i32), _> = co.subset();
            assert_eq!(res, Err(Disiunctio::Sinister("4")));
        }
    }

    #[test]
    fn test_disiunctio_embed() {
        // Compile-only proofs: Absurdum can be embedded into any Disiunctio
        // (including itself). Never called — Absurdum has no values — but the
        // calls must type-check. (Split into separate fns with tail
        // expressions: sequencing two uninhabited expressions in one body
        // would trip the unreachable_code/unused_variables lints.)
        fn _absurdum_embeds_into_self(a: Absurdum) -> Absurdum {
            a.embed()
        }
        fn _absurdum_embeds_into_disiunctio(a: Absurdum) -> Disiunctio!(i32, bool) {
            a.embed()
        }

        #[derive(Debug, PartialEq)]
        struct A;
        #[derive(Debug, PartialEq)]
        struct B;
        #[derive(Debug, PartialEq)]
        struct C;

        {
            // Order does not matter.
            let co_a = <Disiunctio!(C, A, B)>::inject(A);
            let co_b = <Disiunctio!(C, A, B)>::inject(B);
            let co_c = <Disiunctio!(C, A, B)>::inject(C);
            let out_a: Disiunctio!(A, B, C) = co_a.embed();
            let out_b: Disiunctio!(A, B, C) = co_b.embed();
            let out_c: Disiunctio!(A, B, C) = co_c.embed();
            assert_eq!(out_a, Disiunctio::Sinister(A));
            assert_eq!(out_b, Disiunctio::Dexter(Disiunctio::Sinister(B)));
            assert_eq!(
                out_c,
                Disiunctio::Dexter(Disiunctio::Dexter(Disiunctio::Sinister(C)))
            );
        }

        {
            // Multiple variants can resolve to the same output w/o type annotations
            type Abc = Disiunctio!(A, B, C);
            type Bbb = Disiunctio!(B, B, B);

            let b1 = Bbb::inject::<_, Here>(B);
            let b2 = Bbb::inject::<_, There<Here>>(B);
            let out1: Abc = b1.embed();
            let out2: Abc = b2.embed();
            assert_eq!(out1, Disiunctio::Dexter(Disiunctio::Sinister(B)));
            assert_eq!(out2, Disiunctio::Dexter(Disiunctio::Sinister(B)));
        }
    }

    #[test]
    fn test_disiunctio_map_ref() {
        type I32Bool = Disiunctio!(i32, bool);
        type I32BoolRef<'a> = Disiunctio!(i32, &'a bool);

        fn map_it(co: &I32Bool) -> I32BoolRef<'_> {
            // For some reason rustc complains about lifetimes if you try to
            // inline the closure literal into the hlist.
            let map_bool: fn(&bool) -> &bool = |b| b;

            let mapper = hlist![|n: &i32| *n + 3, map_bool];

            co.to_ref().map(mapper)
        }

        let co = I32Bool::inject(3);
        let new = map_it(&co);
        assert_eq!(new, I32BoolRef::inject(6));
    }

    #[test]
    fn test_disiunctio_map_with_ref_mapper() {
        type I32Bool = Disiunctio!(i32, bool);

        // HList mapper

        let mapper = hlist![|n| n + 3, |b: bool| !b];

        let co = I32Bool::inject(3);
        let co = co.map(&mapper);
        let co = co.map(&mapper);

        assert_eq!(co, I32Bool::inject(9));

        // Poly mapper

        let mapper = functio_poly!(|n: i32| -> i32 { n + 3 }, |b: bool| -> bool { !b });

        let co = I32Bool::inject(3);
        let co = co.map(&mapper);
        let co = co.map(&mapper);

        assert_eq!(co, I32Bool::inject(9));

        // Fn mapper

        type StrStr = Disiunctio!(String, String);

        let captured = String::from("!");
        let mapper = |s: String| format!("{s}{captured}");

        let co = StrStr::Sinister(String::from("hi"));
        let co = co.map(&mapper);
        let co = co.map(&mapper);

        assert_eq!(co, StrStr::Sinister(String::from("hi!!")));
    }

    #[test]
    fn test_disiunctio_map_with_mut_mapper() {
        type I32Bool = Disiunctio!(i32, bool);

        // HList mapper

        let mut number = None;
        let mut boolean = None;

        let mut mapper = hlist![
            |n: i32| {
                number = Some(n);
                n
            },
            |b: bool| {
                boolean = Some(b);
                b
            },
        ];

        let co = I32Bool::inject(3);
        let co = co.map(&mut mapper);
        assert_eq!(co, I32Bool::inject(3));
        assert_eq!(number, Some(3));
        assert_eq!(boolean, None);

        // Poly mapper

        let mut mapper = functio_poly!(
            |n: i32| -> i32 {
                // Poly doesn't support capturing values.
                /* number = Some(n); */
                n
            },
            |b: bool| -> bool {
                // Poly doesn't support capturing values.
                /* boolean = Some(b) */
                b
            },
        );

        let co = I32Bool::inject(3);
        let co = co.map(&mut mapper);
        assert_eq!(co, I32Bool::inject(3));

        // Fn mapper

        type StrStr = Disiunctio!(String, String);

        let mut captured = String::new();
        let mut mapper = |s: String| {
            let s = format!("{s}!");
            captured.push_str(&s);
            s
        };

        let co = StrStr::Sinister(String::from("hi"));
        let co = co.map(&mut mapper);
        let co = co.map(&mut mapper);

        assert_eq!(co, StrStr::Sinister(String::from("hi!!")));
        assert_eq!(captured, String::from("hi!hi!!"));
    }
}
