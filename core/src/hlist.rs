//! Module that holds `HList` data structures, implementations, and typeclasses.
//!
//! > *\"Totum est maius sua parte.\"*
//! > — The whole is greater than its part. (Euclid, via Aquinas)
//!
//! An `HList` (heterogeneous list) is a statically typed list where each element
//! can have a different type. The core types are:
//!
//! - [`Nihil`] - The empty list (*privatio*, nothing)
//! - [`Coniunctio`] - The cons cell (*synthesis*, joining head to tail)
//!

//! Typically, you would want to use the `hlist!` macro to make it easier
//! for you to use `HList`.
//!
//! # Examples
//!
//! ```rust
//! # fn main() {
//! use ordofp_core::{hlist, HList, functio_poly};
//!
//! let h = hlist![1, "hi"];
//! assert_eq!(h.len(), 2);
//! let (a, b) = h.into_tuple2();
//! assert_eq!(a, 1);
//! assert_eq!(b, "hi");
//!
//! // Reverse
//! let h1 = hlist![true, "hi"];
//! assert_eq!(h1.into_reverse(), hlist!["hi", true]);
//!
//! // foldr (foldl also available)
//! let h2 = hlist![1, false, 42f32];
//! let folded = h2.foldr(
//!             hlist![|acc, i| i + acc,
//!                    |acc, _| if acc > 42f32 { 9000 } else { 0 },
//!                    |acc, f| f + acc],
//!             1f32
//!     );
//! assert_eq!(folded, 9001);
//!
//! let h3 = hlist![9000, "joe", 41f32];
//! // Mapping over an HList with a polymorphic function,
//! // declared using the functio_poly! macro (you can choose to impl
//! // it manually)
//! let mapped = h3.map(
//!   functio_poly![
//!     |f: f32|   -> f32 { f + 1f32 },
//!     |i: isize| -> isize { i + 1 },
//!     ['a] |s: &'a str| -> &'a str { s }
//!   ]);
//! assert_eq!(mapped, hlist![9001, "joe", 42f32]);
//!
//! // Plucking a value out by type
//! let h4 = hlist![1, "hello", true, 42f32];
//! let (t, remainder): (bool, _) = h4.pluck();
//! assert!(t);
//! assert_eq!(remainder, hlist![1, "hello", 42f32]);
//!
//! // Resculpting an HList
//! let h5 = hlist![9000, "joe", 41f32, true];
//! let (reshaped, remainder2): (HList![f32, i32, &str], _) = h5.sculpt();
//! assert_eq!(reshaped, hlist![41f32, 9000, "joe"]);
//! assert_eq!(remainder2, hlist![true]);
//! # }
//! ```

use crate::indices::{Here, Suffixed, There};
use crate::traits::{Func, IntoReverse, Poly, ToMut, ToRef};
#[cfg(feature = "alloc")]
use alloc::vec::Vec;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use core::ops::Add;

/// Typeclass for HList-y behaviour
///
/// An `HList` is a heterogeneous list, one that is statically typed at compile time. In simple terms,
/// it is just an arbitrarily-nested Tuple2.
pub trait HList: Sized {
    /// Returns the length of a given `HList` type without making use of any references, or
    /// in fact, any values at all.
    ///
    /// # Examples
    /// ```rust
    /// # fn main() {
    /// use ordofp_core::prelude::HList;
    ///
    /// assert_eq!(<HList![i32, bool, f32]>::LEN, 3);
    /// # }
    /// ```
    const LEN: usize;

    /// Returns the length of a given `HList`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # fn main() {
    /// use ordofp_core::hlist;
    ///
    /// let h = hlist![1, "hi"];
    /// assert_eq!(h.len(), 2);
    /// # }
    /// ```
    #[inline]
    fn len(&self) -> usize {
        Self::LEN
    }

    /// Returns whether a given `HList` is empty
    ///
    /// # Examples
    ///
    /// ```rust
    /// # fn main() {
    /// use ordofp_core::hlist;
    ///
    /// let h = hlist![];
    /// assert!(h.is_empty());
    /// # }
    /// ```
    #[inline]
    fn is_empty(&self) -> bool {
        Self::LEN == 0
    }

    /// Prepends an item to the current `HList`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # fn main() {
    /// use ordofp_core::hlist;
    ///
    /// let h1 = hlist![1, "hi"];
    /// let h2 = h1.prepend(true);
    /// let (a, (b, c)) = h2.into_tuple2();
    /// assert_eq!(a, true);
    /// assert_eq!(b, 1);
    /// assert_eq!(c, "hi");
    /// # }
    /// ```
    #[inline]
    fn prepend<H>(self, h: H) -> Coniunctio<H, Self> {
        Coniunctio {
            head: h,
            tail: self,
        }
    }
}

/// Represents the right-most end of a heterogeneous list
///
/// > *\"Ex nihilo nihil fit.\"*
/// > — From nothing, nothing comes. (Parmenides, via Lucretius)
///
/// `Nihil` represents the empty case, the *privatio* (privation) of elements.
/// It is the base case for the recursive `HList` structure.
///
/// # Examples
///
/// ```rust
/// # use ordofp_core::hlist::{coniunctio, Nihil};
/// let h = coniunctio(1, Nihil);
/// let h = h.head;
/// assert_eq!(h, 1);
/// ```
#[derive(PartialEq, Debug, Eq, Clone, Copy, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Nihil;

impl HList for Nihil {
    const LEN: usize = 0;
}

/// Represents the most basic non-empty `HList`. Its value is held in `head`
/// while its tail is another `HList`.
///
/// > *\"Coniunctio est copulatio duorum.\"*
/// > — Conjunction is the coupling of two. (Scholastic definition)
///
/// `Coniunctio` (conjunction/joining) represents the cons cell of an `HList`,
/// joining a head element with a tail. Named after Aristotle's σύνθεσις (synthesis).
#[derive(PartialEq, Debug, Eq, Clone, Copy, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Coniunctio<H, T> {
    /// The first element of this `HList`.
    pub head: H,
    /// The rest of the list: another `HList`, ending in [`Nihil`].
    pub tail: T,
}

impl<H, T: HList> HList for Coniunctio<H, T> {
    const LEN: usize = 1 + <T as HList>::LEN;
}

impl<H, T> Coniunctio<H, T> {
    /// Returns the head of the list and the tail of the list as a tuple2.
    /// The original list is consumed
    ///
    /// # Examples
    ///
    /// ```rust
    /// # fn main() {
    /// use ordofp_core::hlist;
    ///
    /// let h = hlist!("hi");
    /// let (h, tail) = h.pop();
    /// assert_eq!(h, "hi");
    /// assert_eq!(tail, hlist![]);
    /// # }
    /// ```
    #[inline]
    pub fn pop(self) -> (H, T) {
        (self.head, self.tail)
    }
}

/// Takes an element and an Hlist and returns another one with
/// the element prepended to the original list. The original list
/// is consumed
///
/// > *\"Coniungere est in unum redigere.\"*
/// > — To conjoin is to bring into one.
///
/// # Examples
///
/// ```rust
/// # fn main() {
/// use ordofp_core::hlist::{Nihil, coniunctio};
///
/// let h_list = coniunctio("what", coniunctio(1.23f32, Nihil));
/// let (h1, h2) = h_list.into_tuple2();
/// assert_eq!(h1, "what");
/// assert_eq!(h2, 1.23f32);
/// # }
/// ```
#[inline]
pub fn coniunctio<H, T: HList>(h: H, tail: T) -> Coniunctio<H, T> {
    Coniunctio { head: h, tail }
}

// Inherent methods shared by Nihil and Coniunctio.
macro_rules! gen_inherent_methods {
    (impl<$($TyPar:ident),*> $Struct:ty { ... })
    => {
        impl<$($TyPar),*> $Struct {
            /// Returns the length of a given HList
            ///
            /// # Examples
            ///
            /// ```rust
            /// # fn main() {
            /// use ordofp_core::hlist;
            ///
            /// let h = hlist![1, "hi"];
            /// assert_eq!(h.len(), 2);
            /// # }
            /// ```
            #[inline(always)]
            pub fn len(&self) -> usize
            where Self: HList,
            {
                HList::len(self)
            }

            /// Returns whether a given HList is empty
            ///
            /// # Examples
            ///
            /// ```rust
            /// # fn main() {
            /// use ordofp_core::hlist;
            ///
            /// let h = hlist![];
            /// assert!(h.is_empty());
            /// # }
            /// ```
            #[inline(always)]
            pub fn is_empty(&self) -> bool
            where Self: HList,
            {
                HList::is_empty(self)
            }

            /// Prepend an item to the current HList
            ///
            /// # Examples
            ///
            /// ```rust
            /// # fn main() {
            /// use ordofp_core::hlist;
            ///
            /// let h1 = hlist![1, "hi"];
            /// let h2 = h1.prepend(true);
            /// let (a, (b, c)) = h2.into_tuple2();
            /// assert_eq!(a, true);
            /// assert_eq!(b, 1);
            /// assert_eq!(c, "hi");
            /// # }
            /// ```
            #[inline(always)]
            pub fn prepend<H>(self, h: H) -> Coniunctio<H, Self>
            where Self: HList,
            {
                HList::prepend(self, h)
            }

            /// Consume the current HList and return an HList with the requested shape.
            ///
            /// `sculpt` allows us to extract/reshape/sculpt the current HList into another shape,
            /// provided that the requested shape's types are are contained within the current HList.
            ///
            /// The `Indices` type parameter allows the compiler to figure out that `Ts`
            /// and `Self` can be morphed into each other.
            ///
            /// # Examples
            ///
            /// ```rust
            /// # fn main() {
            /// use ordofp_core::{hlist, HList};
            ///
            /// let h = hlist![9000, "joe", 41f32, true];
            /// let (reshaped, remainder): (HList![f32, i32, &str], _) = h.sculpt();
            /// assert_eq!(reshaped, hlist![41f32, 9000, "joe"]);
            /// assert_eq!(remainder, hlist![true]);
            /// # }
            /// ```
            // #[inline] is enough — sculpt is a reshape, not a hot inner-loop wrapper; let the compiler decide.
            #[inline]
            pub fn sculpt<Ts, Indices>(self) -> (Ts, <Self as Sculptor<Ts, Indices>>::Remainder)
            where Self: Sculptor<Ts, Indices>,
            {
                Sculptor::sculpt(self)
            }

            /// Reverse the HList.
            ///
            /// # Examples
            ///
            /// ```rust
            /// # fn main() {
            /// use ordofp_core::hlist;
            ///
            /// assert_eq!(hlist![].into_reverse(), hlist![]);
            ///
            /// assert_eq!(
            ///     hlist![1, "hello", true, 42f32].into_reverse(),
            ///     hlist![42f32, true, "hello", 1],
            /// )
            /// # }
            /// ```
            // #[inline] is enough — into_reverse is typically called once, not in a hot loop.
            #[inline]
            pub fn into_reverse(self) -> <Self as IntoReverse>::Output
            where Self: IntoReverse,
            {
                IntoReverse::into_reverse(self)
            }

            /// Return an HList where the contents are references to
            /// the original HList on which this method was called.
            ///
            /// # Examples
            ///
            /// ```rust
            /// # fn main() {
            /// use ordofp_core::hlist;
            ///
            /// assert_eq!(hlist![].to_ref(), hlist![]);
            ///
            /// assert_eq!(hlist![1, true].to_ref(), hlist![&1, &true]);
            /// # }
            /// ```
            #[inline(always)]
            pub fn to_ref<'a>(&'a self) -> <Self as ToRef<'a>>::Output
                where Self: ToRef<'a>,
            {
                ToRef::to_ref(self)
            }

            /// Return an `HList` where the contents are mutable references
            /// to the original `HList` on which this method was called.
            ///
            /// # Examples
            ///
            /// ```rust
            /// # fn main() {
            /// use ordofp_core::hlist;
            ///
            /// assert_eq!(hlist![].to_mut(), hlist![]);
            ///
            /// assert_eq!(hlist![1, true].to_mut(), hlist![&mut 1, &mut true]);
            /// # }
            /// ```
            #[inline(always)]
            pub fn to_mut<'a>(&'a mut self) -> <Self as ToMut<'a>>::Output
            where
                Self: ToMut<'a>,
            {
                ToMut::to_mut(self)
            }

            /// Apply a function to each element of an HList.
            ///
            /// This transforms some `HList![A, B, C, ..., E]` into some
            /// `HList![T, U, V, ..., Z]`.  A variety of types are supported
            /// for the folder argument:
            ///
            /// * An `hlist![]` of closures (one for each element).
            /// * A single closure (for mapping an HList that is homogenous).
            /// * A single [`Poly`].
            ///
            /// [`Poly`]: ../traits/struct.Poly.html
            ///
            /// # Examples
            ///
            /// ```rust
            /// # fn main() {
            /// use ordofp_core::hlist::Nihil;
            /// use ordofp_core::hlist;
            ///
            /// assert_eq!(Nihil.map(Nihil), Nihil);
            ///
            /// let h = hlist![1, false, 42f32];
            ///
            /// // Sadly we need to help the compiler understand the bool type in our mapper
            ///
            /// let mapped = h.to_ref().map(hlist![
            ///     |&n| n + 1,
            ///     |b: &bool| !b,
            ///     |&f| f + 1f32]);
            /// assert_eq!(mapped, hlist![2, true, 43f32]);
            ///
            /// // There is also a value-consuming version that passes values to your functions
            /// // instead of just references:
            ///
            /// let mapped2 = h.map(hlist![
            ///     |n| n + 3,
            ///     |b: bool| !b,
            ///     |f| f + 8959f32]);
            /// assert_eq!(mapped2, hlist![4, true, 9001f32]);
            /// # }
            /// ```
            // #[inline] is enough — higher-order with heavy monomorphization; let the compiler decide per call site.
            #[inline]
            pub fn map<F>(self, mapper: F) -> <Self as HMappable<F>>::Output
            where Self: HMappable<F>,
            {
                HMappable::map(self, mapper)
            }

            /// Zip two HLists together.
            ///
            /// This zips a `HList![A1, B1, ..., C1]` with a `HList![A2, B2, ..., C2]`
            /// to make a `HList![(A1, A2), (B1, B2), ..., (C1, C2)]`
            ///
            /// # Example
            ///
            /// ```rust
            /// # fn main() {
            /// use ordofp_core::hlist::Nihil;
            /// use ordofp_core::hlist;
            ///
            /// assert_eq!(Nihil.zip(Nihil), Nihil);
            ///
            /// let h1 = hlist![1, false, 42f32];
            /// let h2 = hlist![true, "foo", 2];
            ///
            /// let zipped = h1.zip(h2);
            /// assert_eq!(zipped, hlist![
            ///     (1, true),
            ///     (false, "foo"),
            ///     (42f32, 2),
            /// ]);
            /// # }
            /// ```
            // #[inline] is enough — higher-order with heavy monomorphization; let the compiler decide per call site.
            #[inline]
            pub fn zip<Other>(self, other: Other) -> <Self as HZippable<Other>>::Zipped
            where Self: HZippable<Other>,
            {
                HZippable::zip(self, other)
            }

            /// Perform a left fold over an HList.
            ///
            /// This transforms some `HList![A, B, C, ..., E]` into a single
            /// value by visiting all of the elements in left-to-right order.
            /// A variety of types are supported for the mapper argument:
            ///
            /// * An `hlist![]` of closures (one for each element).
            /// * A single closure (for folding an HList that is homogenous).
            /// * A single [`Poly`].
            ///
            /// The accumulator can freely change type over the course of the call.
            /// When called with a list of `N` functions, an expanded form of the
            /// implementation with type annotations might look something like this:
            ///
            /// (pseudo-code — not compilable by design:)
            /// ```ignore
            /// let acc: Acc0 = init_value;
            /// let acc: Acc1 = f1(acc, x1);
            /// let acc: Acc2 = f2(acc, x2);
            /// let acc: Acc3 = f3(acc, x3);
            /// ...
            /// let acc: AccN = fN(acc, xN);
            /// acc
            /// ```
            ///
            /// [`Poly`]: ../traits/struct.Poly.html
            ///
            /// # Examples
            ///
            /// ```rust
            /// # fn main() {
            /// use ordofp_core::hlist;
            ///
            /// let nil = hlist![];
            ///
            /// assert_eq!(nil.foldl(hlist![], 0), 0);
            ///
            /// let h = hlist![1, false, 42f32];
            ///
            /// let folded = h.to_ref().foldl(
            ///     hlist![
            ///         |acc, &i| i + acc,
            ///         |acc, b: &bool| if !b && acc > 42 { 9000f32 } else { 0f32 },
            ///         |acc, &f| f + acc
            ///     ],
            ///     1
            /// );
            ///
            /// assert_eq!(42f32, folded);
            ///
            /// // There is also a value-consuming version that passes values to your folding
            /// // functions instead of just references:
            ///
            /// let folded2 = h.foldl(
            ///     hlist![
            ///         |acc, i| i + acc,
            ///         |acc, b: bool| if !b && acc > 42 { 9000f32 } else { 0f32 },
            ///         |acc, f| f + acc
            ///     ],
            ///     8918
            /// );
            ///
            /// assert_eq!(9042f32, folded2)
            /// # }
            /// ```
            // #[inline] is enough — higher-order fold with heavy monomorphization; let the compiler decide.
            #[inline]
            pub fn foldl<Folder, Acc>(
                self,
                folder: Folder,
                acc: Acc,
            ) -> <Self as HFoldLeftable<Folder, Acc>>::Output
            where Self: HFoldLeftable<Folder, Acc>,
            {
                HFoldLeftable::foldl(self, folder, acc)
            }

            /// Perform a right fold over an HList.
            ///
            /// This transforms some `HList![A, B, C, ..., E]` into a single
            /// value by visiting all of the elements in reverse order.
            /// A variety of types are supported for the mapper argument:
            ///
            /// * An `hlist![]` of closures (one for each element).
            /// * A single closure (for folding an HList that is homogenous),
            ///   taken by reference.
            /// * A single [`Poly`].
            ///
            /// The accumulator can freely change type over the course of the call.
            ///
            /// [`Poly`]: ../traits/struct.Poly.html
            ///
            /// # Comparison to `foldl`
            ///
            /// While the order of element traversal in `foldl` may seem more natural,
            /// `foldr` does have its use cases, in particular when it is used to build
            /// something that reflects the structure of the original HList (such as
            /// folding an HList of `Option`s into an `Option` of an HList).
            /// An implementation of such a function using `foldl` will tend to
            /// reverse the list, while `foldr` will tend to preserve its order.
            ///
            /// The reason for this is because `foldr` performs what is known as
            /// "structural induction;" it can be understood as follows:
            ///
            /// * Write out the HList in terms of [`coniunctio`] and [`Nihil`].
            /// * Substitute each [`coniunctio`] with a function,
            ///   and substitute [`Nihil`] with `init`
            ///
            /// ```text
            /// the list:
            ///     coniunctio(x1, coniunctio(x2, coniunctio(x3, ...coniunctio(xN, Nihil)...)))
            ///
            /// becomes:
            ///        f1( x1,    f2( x2,    f3( x3, ...   fN( xN, init)...)))
            /// ```
            ///
            /// [`Nihil`]: struct.Nihil.html
            /// [`coniunctio`]: fn.coniunctio.html
            ///
            /// # Examples
            ///
            /// ```rust
            /// # fn main() {
            /// use ordofp_core::hlist;
            ///
            /// let nil = hlist![];
            ///
            /// assert_eq!(nil.foldr(hlist![], 0), 0);
            ///
            /// let h = hlist![1, false, 42f32];
            ///
            /// let folded = h.foldr(
            ///     hlist![
            ///         |acc, i| i + acc,
            ///         |acc, b: bool| if !b && acc > 42f32 { 9000 } else { 0 },
            ///         |acc, f| f + acc
            ///     ],
            ///     1f32
            /// );
            ///
            /// assert_eq!(9001, folded)
            /// # }
            /// ```
            // #[inline] is enough — higher-order fold with heavy monomorphization; let the compiler decide.
            #[inline]
            pub fn foldr<Folder, Init>(
                self,
                folder: Folder,
                init: Init,
            ) -> <Self as HFoldRightable<Folder, Init>>::Output
            where Self: HFoldRightable<Folder, Init>,
            {
                HFoldRightable::foldr(self, folder, init)
            }

            /// Extend the contents of this HList with another HList
            ///
            /// This exactly the same as the [`Add`][Add] impl.
            ///
            /// [Add]: struct.Coniunctio.html#impl-Add%3CRHS%3E-for-Coniunctio%3CH,+T%3E
            ///
            /// # Examples
            ///
            /// ```rust
            /// use ordofp_core::hlist;
            ///
            /// let first = hlist![0u8, 1u16];
            /// let second = hlist![2u32, 3u64];
            ///
            /// assert_eq!(first.extend(second), hlist![0u8, 1u16, 2u32, 3u64]);
            /// ```
            pub fn extend<Other>(
                self,
                other: Other
            ) -> <Self as Add<Other>>::Output
            where
                Self: Add<Other>,
                Other: HList,
            {
                self + other
            }
        }
    };
}

gen_inherent_methods! {
    impl<> Nihil { ... }
}
gen_inherent_methods! {
    impl<Head, Tail> Coniunctio<Head, Tail> { ... }
}

// Coniunctio-only inherent methods.
impl<Head, Tail> Coniunctio<Head, Tail> {
    /// Borrow an element by type from an `HList`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # fn main() {
    /// use ordofp_core::hlist;
    ///
    /// let h = hlist![1i32, 2u32, "hello", true, 42f32];
    ///
    /// // Often, type inference can figure out the type you want.
    /// // You can help guide type inference when necessary by
    /// // using type annotations.
    /// let b: &bool = h.get();
    /// if !b { panic!("no way!") };
    ///
    /// // If space is tight, you can also use turbofish syntax.
    /// // The Index is still left to type inference by using `_`.
    /// match *h.get::<u32, _>() {
    ///     2 => { }
    ///     _ => panic!("it can't be!!"),
    /// }
    /// # }
    /// ```
    #[inline(always)]
    pub fn get<T, Index>(&self) -> &T
    where
        Self: Selector<T, Index>,
    {
        Selector::get(self)
    }

    /// Mutably borrow an element by type from an `HList`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # fn main() {
    /// use ordofp_core::hlist;
    ///
    /// let mut h = hlist![1i32, true];
    ///
    /// // Type inference ensures we fetch the correct type.
    /// *h.get_mut() = false;
    /// *h.get_mut() = 2;
    /// // *h.get_mut() = "neigh";  // Won't compile.
    ///
    /// assert_eq!(h, hlist![2i32, false]);
    /// # }
    /// ```
    #[inline(always)]
    pub fn get_mut<T, Index>(&mut self) -> &mut T
    where
        Self: Selector<T, Index>,
    {
        Selector::get_mut(self)
    }

    /// Remove an element by type from an `HList`.
    ///
    /// The remaining elements are returned along with it.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # fn main() {
    /// use ordofp_core::hlist;
    ///
    /// let list = hlist![1, "hello", true, 42f32];
    ///
    /// // Often, type inference can figure out the target type.
    /// let (b, list): (bool, _) = list.pluck();
    /// assert!(b);
    ///
    /// // When type inference will not suffice, you can use a turbofish.
    /// // The Index is still left to type inference by using `_`.
    /// let (s, list) = list.pluck::<i32, _>();
    ///
    /// // Each time we plucked, we got back a remainder.
    /// // Let's check what's left:
    /// assert_eq!(list, hlist!["hello", 42.0])
    /// # }
    /// ```
    #[inline(always)]
    pub fn pluck<T, Index>(self) -> (T, <Self as Plucker<T, Index>>::Remainder)
    where
        Self: Plucker<T, Index>,
    {
        Plucker::pluck(self)
    }

    /// Turns an `HList` into nested Tuple2s, which are less troublesome to pattern match
    /// and have a nicer type signature.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # fn main() {
    /// use ordofp_core::hlist;
    ///
    /// let h = hlist![1, "hello", true, 42f32];
    ///
    /// // We now have a much nicer pattern matching experience
    /// let (first,(second,(third, fourth))) = h.into_tuple2();
    ///
    /// assert_eq!(first ,       1);
    /// assert_eq!(second, "hello");
    /// assert_eq!(third ,    true);
    /// assert_eq!(fourth,   42f32);
    /// # }
    /// ```
    #[inline(always)]
    pub fn into_tuple2(
        self,
    ) -> (
        <Self as IntoTuple2>::HeadType,
        <Self as IntoTuple2>::TailOutput,
    )
    where
        Self: IntoTuple2,
    {
        IntoTuple2::into_tuple2(self)
    }
}

impl<RHS> Add<RHS> for Nihil
where
    RHS: HList,
{
    type Output = RHS;

    #[inline]
    fn add(self, rhs: RHS) -> RHS {
        rhs
    }
}

impl<H, T, RHS> Add<RHS> for Coniunctio<H, T>
where
    T: Add<RHS>,
    RHS: HList,
{
    type Output = Coniunctio<H, <T as Add<RHS>>::Output>;

    #[inline]
    fn add(self, rhs: RHS) -> Self::Output {
        Coniunctio {
            head: self.head,
            tail: self.tail + rhs,
        }
    }
}

/// Trait for borrowing an `HList` element by type
///
/// This trait is part of the implementation of the inherent method
/// [`Coniunctio::get`]. Please see that method for more information.
///
/// You only need to import this trait when working with Universalis
/// `HLists` of unknown type. If you have an `HList` of known type,
/// then `list.get()` should "just work" even without the trait.
///
/// [`Coniunctio::get`]: struct.Coniunctio.html#method.get
pub trait Selector<S, I> {
    /// Borrow an element by type from an `HList`.
    ///
    /// Please see the [inherent method] for more information.
    ///
    /// The only difference between that inherent method and this
    /// trait method is the location of the type parameters
    /// (here, they are on the trait rather than the method).
    ///
    /// [inherent method]: struct.Coniunctio.html#method.get
    fn get(&self) -> &S;

    /// Mutably borrow an element by type from an `HList`.
    ///
    /// Please see the [inherent method] for more information.
    ///
    /// The only difference between that inherent method and this
    /// trait method is the location of the type parameters
    /// (here, they are on the trait rather than the method).
    ///
    /// [inherent method]: struct.Coniunctio.html#method.get_mut
    fn get_mut(&mut self) -> &mut S;
}

impl<T, Tail> Selector<T, Here> for Coniunctio<T, Tail> {
    #[inline]
    fn get(&self) -> &T {
        &self.head
    }

    #[inline]
    fn get_mut(&mut self) -> &mut T {
        &mut self.head
    }
}

impl<Head, Tail, FromTail, TailIndex> Selector<FromTail, There<TailIndex>>
    for Coniunctio<Head, Tail>
where
    Tail: Selector<FromTail, TailIndex>,
{
    #[inline]
    fn get(&self) -> &FromTail {
        self.tail.get()
    }

    #[inline]
    fn get_mut(&mut self) -> &mut FromTail {
        self.tail.get_mut()
    }
}

/// Trait defining extraction from a given `HList`
///
/// This trait is part of the implementation of the inherent method
/// [`Coniunctio::pluck`]. Please see that method for more information.
///
/// You only need to import this trait when working with Universalis
/// `HLists` of unknown type. If you have an `HList` of known type,
/// then `list.pluck()` should "just work" even without the trait.
///
/// [`Coniunctio::pluck`]: struct.Coniunctio.html#method.pluck
pub trait Plucker<Target, Index> {
    /// What is left after you pluck the target from the Self
    type Remainder;

    /// Remove an element by type from an `HList`.
    ///
    /// Please see the [inherent method] for more information.
    ///
    /// The only difference between that inherent method and this
    /// trait method is the location of the type parameters.
    /// (here, they are on the trait rather than the method)
    ///
    /// [inherent method]: struct.Coniunctio.html#method.pluck
    fn pluck(self) -> (Target, Self::Remainder);
}

/// Implementation when the pluck target is in head
impl<T, Tail> Plucker<T, Here> for Coniunctio<T, Tail> {
    type Remainder = Tail;

    #[inline]
    fn pluck(self) -> (T, Self::Remainder) {
        (self.head, self.tail)
    }
}

/// Implementation when the pluck target is in the tail
impl<Head, Tail, FromTail, TailIndex> Plucker<FromTail, There<TailIndex>> for Coniunctio<Head, Tail>
where
    Tail: Plucker<FromTail, TailIndex>,
{
    type Remainder = Coniunctio<Head, <Tail as Plucker<FromTail, TailIndex>>::Remainder>;

    fn pluck(self) -> (FromTail, Self::Remainder) {
        let (target, tail_remainder): (
            FromTail,
            <Tail as Plucker<FromTail, TailIndex>>::Remainder,
        ) = <Tail as Plucker<FromTail, TailIndex>>::pluck(self.tail);
        (
            target,
            Coniunctio {
                head: self.head,
                tail: tail_remainder,
            },
        )
    }
}

/// Implementation when target is reference and  the pluck target is in head
impl<'a, T, Tail: ToRef<'a>> Plucker<&'a T, Here> for &'a Coniunctio<T, Tail> {
    type Remainder = <Tail as ToRef<'a>>::Output;

    #[inline]
    fn pluck(self) -> (&'a T, Self::Remainder) {
        (&self.head, self.tail.to_ref())
    }
}

/// Implementation when target is reference the pluck target is in the tail
impl<'a, Head, Tail, FromTail, TailIndex> Plucker<&'a FromTail, There<TailIndex>>
    for &'a Coniunctio<Head, Tail>
where
    &'a Tail: Plucker<&'a FromTail, TailIndex>,
{
    type Remainder =
        Coniunctio<&'a Head, <&'a Tail as Plucker<&'a FromTail, TailIndex>>::Remainder>;

    fn pluck(self) -> (&'a FromTail, Self::Remainder) {
        let (target, tail_remainder): (
            &'a FromTail,
            <&'a Tail as Plucker<&'a FromTail, TailIndex>>::Remainder,
        ) = <&'a Tail as Plucker<&'a FromTail, TailIndex>>::pluck(&self.tail);
        (
            target,
            Coniunctio {
                head: &self.head,
                tail: tail_remainder,
            },
        )
    }
}

/// Trait for pulling out some subset of an `HList`, using type inference.
///
/// This trait is part of the implementation of the inherent method
/// [`Coniunctio::sculpt`]. Please see that method for more information.
///
/// You only need to import this trait when working with Universalis
/// `HLists` of unknown type. If you have an `HList` of known type,
/// then `list.sculpt()` should "just work" even without the trait.
///
/// [`Coniunctio::sculpt`]: struct.Coniunctio.html#method.sculpt
pub trait Sculptor<Target, Indices> {
    /// The `HList` of elements left over after the `Target` shape has
    /// been extracted.
    type Remainder;

    /// Consumes the current `HList` and returns an `HList` with the requested shape.
    ///
    /// Please see the [inherent method] for more information.
    ///
    /// The only difference between that inherent method and this
    /// trait method is the location of the type parameters.
    /// (here, they are on the trait rather than the method)
    ///
    /// [inherent method]: struct.Coniunctio.html#method.sculpt
    fn sculpt(self) -> (Target, Self::Remainder);
}

/// Implementation for when the target is an empty `HList` (Nihil)
///
/// Index type is Nihil because we don't need an index for finding Nihil
impl<Source> Sculptor<Nihil, Nihil> for Source {
    type Remainder = Source;

    #[inline(always)]
    fn sculpt(self) -> (Nihil, Self::Remainder) {
        (Nihil, self)
    }
}

/// Implementation for when we have a non-empty Coniunctio target
///
/// Indices is Coniunctio<`IndexHead`, `IndexTail`> here because the compiler is being asked to figure out the
/// Index for Plucking the first item of type `THead` out of Self and the rest (`IndexTail`) is for the
/// Plucker's remainder induce.
impl<THead, TTail, SHead, STail, IndexHead, IndexTail>
    Sculptor<Coniunctio<THead, TTail>, Coniunctio<IndexHead, IndexTail>>
    for Coniunctio<SHead, STail>
where
    Coniunctio<SHead, STail>: Plucker<THead, IndexHead>,
    <Coniunctio<SHead, STail> as Plucker<THead, IndexHead>>::Remainder: Sculptor<TTail, IndexTail>,
{
    type Remainder =
        <<Coniunctio<SHead, STail> as Plucker<THead, IndexHead>>::Remainder as Sculptor<
            TTail,
            IndexTail,
        >>::Remainder;

    // #[inline] is enough — recursive sculpt body is larger than a trivial wrapper; let the compiler decide.
    #[inline]
    fn sculpt(self) -> (Coniunctio<THead, TTail>, Self::Remainder) {
        let (p, r): (
            THead,
            <Coniunctio<SHead, STail> as Plucker<THead, IndexHead>>::Remainder,
        ) = self.pluck();
        let (tail, tail_remainder): (TTail, Self::Remainder) = r.sculpt();
        (Coniunctio { head: p, tail }, tail_remainder)
    }
}

impl IntoReverse for Nihil {
    type Output = Nihil;
    #[inline]
    fn into_reverse(self) -> Self::Output {
        self
    }
}

impl<H, Tail> IntoReverse for Coniunctio<H, Tail>
where
    Tail: IntoReverse,
    <Tail as IntoReverse>::Output: Add<Coniunctio<H, Nihil>>,
{
    type Output = <<Tail as IntoReverse>::Output as Add<Coniunctio<H, Nihil>>>::Output;

    #[inline]
    fn into_reverse(self) -> Self::Output {
        self.tail.into_reverse()
            + Coniunctio {
                head: self.head,
                tail: Nihil,
            }
    }
}

impl<P, H, Tail> HMappable<Poly<P>> for Coniunctio<H, Tail>
where
    P: Func<H>,
    Tail: HMappable<Poly<P>>,
{
    type Output = Coniunctio<<P as Func<H>>::Output, <Tail as HMappable<Poly<P>>>::Output>;
    #[inline]
    fn map(self, poly: Poly<P>) -> Self::Output {
        Coniunctio {
            head: P::call(self.head),
            tail: self.tail.map(poly),
        }
    }
}

/// Trait for mapping over an `HList`
///
/// This trait is part of the implementation of the inherent method
/// [`Coniunctio::map`]. Please see that method for more information.
///
/// You only need to import this trait when working with Universalis
/// `HLists` or Mappers of unknown type. If the type of everything is known,
/// then `list.map(f)` should "just work" even without the trait.
///
/// [`Coniunctio::map`]: struct.Coniunctio.html#method.map
pub trait HMappable<Mapper> {
    /// The `HList` produced by applying the mapper to every element;
    /// same length as the input, element types transformed pointwise.
    type Output;

    /// Apply a function to each element of an `HList`.
    ///
    /// Please see the [inherent method] for more information.
    ///
    /// The only difference between that inherent method and this
    /// trait method is the location of the type parameters.
    /// (here, they are on the trait rather than the method)
    ///
    /// [inherent method]: struct.Coniunctio.html#method.map
    fn map(self, mapper: Mapper) -> Self::Output;
}

impl<F> HMappable<F> for Nihil {
    type Output = Nihil;

    #[inline]
    fn map(self, _: F) -> Self::Output {
        Nihil
    }
}

impl<F, R, H, Tail> HMappable<F> for Coniunctio<H, Tail>
where
    F: Fn(H) -> R,
    Tail: HMappable<F>,
{
    type Output = Coniunctio<R, <Tail as HMappable<F>>::Output>;

    #[inline]
    fn map(self, f: F) -> Self::Output {
        let Coniunctio { head, tail } = self;
        Coniunctio {
            head: f(head),
            tail: tail.map(f),
        }
    }
}

impl<F, R, MapperTail, H, Tail> HMappable<Coniunctio<F, MapperTail>> for Coniunctio<H, Tail>
where
    F: FnOnce(H) -> R,
    Tail: HMappable<MapperTail>,
{
    type Output = Coniunctio<R, <Tail as HMappable<MapperTail>>::Output>;

    #[inline]
    fn map(self, mapper: Coniunctio<F, MapperTail>) -> Self::Output {
        let Coniunctio { head, tail } = self;
        Coniunctio {
            head: (mapper.head)(head),
            tail: tail.map(mapper.tail),
        }
    }
}

/// Trait for zipping `HLists`
///
/// This trait is part of the implementation of the inherent method
/// [`Coniunctio::zip`]. Please see that method for more information.
///
/// You only need to import this trait when working with Universalis
/// `HLists` of unknown type. If the type of everything is known,
/// then `list.zip(list2)` should "just work" even without the trait.
///
/// [`Coniunctio::zip`]: struct.Coniunctio.html#method.zip
pub trait HZippable<Other> {
    /// The `HList` of pairs produced by zipping; the two lists must have
    /// the same length, so no elements are dropped.
    type Zipped: HList;

    /// Zip this `HList` with another one.
    ///
    /// Please see the [inherent method] for more information.
    ///
    /// [inherent method]: struct.Coniunctio.html#method.zip
    fn zip(self, other: Other) -> Self::Zipped;
}

impl HZippable<Nihil> for Nihil {
    type Zipped = Nihil;
    #[inline]
    fn zip(self, _other: Nihil) -> Self::Zipped {
        Nihil
    }
}

impl<H1, T1, H2, T2> HZippable<Coniunctio<H2, T2>> for Coniunctio<H1, T1>
where
    T1: HZippable<T2>,
{
    type Zipped = Coniunctio<(H1, H2), T1::Zipped>;
    #[inline]
    fn zip(self, other: Coniunctio<H2, T2>) -> Self::Zipped {
        Coniunctio {
            head: (self.head, other.head),
            tail: self.tail.zip(other.tail),
        }
    }
}

/// Trait for performing a right fold over an `HList`
///
/// This trait is part of the implementation of the inherent method
/// [`Coniunctio::foldr`]. Please see that method for more information.
///
/// You only need to import this trait when working with Universalis
/// `HLists` or Folders of unknown type. If the type of everything is known,
/// then `list.foldr(f, init)` should "just work" even without the trait.
///
/// [`Coniunctio::foldr`]: struct.Coniunctio.html#method.foldr
pub trait HFoldRightable<Folder, Init> {
    /// The final accumulator type after folding every element from the
    /// right; for the empty list this is `Init` itself.
    type Output;

    /// Perform a right fold over an `HList`.
    ///
    /// Please see the [inherent method] for more information.
    ///
    /// The only difference between that inherent method and this
    /// trait method is the location of the type parameters.
    /// (here, they are on the trait rather than the method)
    ///
    /// [inherent method]: struct.Coniunctio.html#method.foldr
    fn foldr(self, folder: Folder, i: Init) -> Self::Output;
}

impl<F, Init> HFoldRightable<F, Init> for Nihil {
    type Output = Init;

    #[inline]
    fn foldr(self, _: F, i: Init) -> Self::Output {
        i
    }
}

impl<F, FolderHeadR, FolderTail, H, Tail, Init> HFoldRightable<Coniunctio<F, FolderTail>, Init>
    for Coniunctio<H, Tail>
where
    Tail: HFoldRightable<FolderTail, Init>,
    F: FnOnce(<Tail as HFoldRightable<FolderTail, Init>>::Output, H) -> FolderHeadR,
{
    type Output = FolderHeadR;

    #[inline]
    fn foldr(self, folder: Coniunctio<F, FolderTail>, init: Init) -> Self::Output {
        let folded_tail = self.tail.foldr(folder.tail, init);
        (folder.head)(folded_tail, self.head)
    }
}

impl<F, R, H, Tail, Init> HFoldRightable<F, Init> for Coniunctio<H, Tail>
where
    Tail: foldr_owned::HFoldRightableOwned<F, Init>,
    F: Fn(<Tail as HFoldRightable<F, Init>>::Output, H) -> R,
{
    type Output = R;

    #[inline]
    fn foldr(self, folder: F, init: Init) -> Self::Output {
        foldr_owned::HFoldRightableOwned::real_foldr(self, folder, init).0
    }
}

/// [`HFoldRightable`] inner mechanics for folding with a folder that needs to be owned.
pub mod foldr_owned {
    use super::{Coniunctio, HFoldRightable, Nihil};

    /// A real `foldr` for the folder that must be owned to fold.
    ///
    /// Due to `HList` being a recursive struct and not linear array,
    /// the only way to fold it is recursive.
    ///
    /// However, there are differences in the `foldl` and `foldr` traversing
    /// the `HList`:
    ///
    /// 1. `foldl` calls `folder(head)` and then passes the ownership
    ///    of the folder to the next recursive call.
    /// 2. `foldr` passes the ownership of the folder to the next recursive call,
    ///    and then tries to call `folder(head)`; but the ownership is already gone!
    pub trait HFoldRightableOwned<Folder, Init>: HFoldRightable<Folder, Init> {
        /// Fold from the right, threading the owned folder through the
        /// recursion and handing it back alongside the result so the
        /// caller's frame can reuse it.
        fn real_foldr(self, folder: Folder, init: Init) -> (Self::Output, Folder);
    }

    impl<F, Init> HFoldRightableOwned<F, Init> for Nihil {
        #[inline]
        fn real_foldr(self, f: F, i: Init) -> (Self::Output, F) {
            (i, f)
        }
    }

    impl<F, H, Tail, Init> HFoldRightableOwned<F, Init> for Coniunctio<H, Tail>
    where
        Self: HFoldRightable<F, Init>,
        Tail: HFoldRightableOwned<F, Init>,
        F: Fn(<Tail as HFoldRightable<F, Init>>::Output, H) -> Self::Output,
    {
        #[inline]
        fn real_foldr(self, folder: F, init: Init) -> (Self::Output, F) {
            let (folded_tail, folder) = self.tail.real_foldr(folder, init);
            ((folder)(folded_tail, self.head), folder)
        }
    }
}

impl<P, R, H, Tail, Init> HFoldRightable<Poly<P>, Init> for Coniunctio<H, Tail>
where
    Tail: HFoldRightable<Poly<P>, Init>,
    P: Func<(<Tail as HFoldRightable<Poly<P>, Init>>::Output, H), Output = R>,
{
    type Output = R;

    #[inline]
    fn foldr(self, poly: Poly<P>, init: Init) -> Self::Output {
        let Coniunctio { head, tail } = self;
        let folded_tail = tail.foldr(poly, init);
        P::call((folded_tail, head))
    }
}

impl<'a> ToRef<'a> for Nihil {
    type Output = Nihil;

    #[inline(always)]
    fn to_ref(&'a self) -> Self::Output {
        Nihil
    }
}

impl<'a, H, Tail> ToRef<'a> for Coniunctio<H, Tail>
where
    H: 'a,
    Tail: ToRef<'a>,
{
    type Output = Coniunctio<&'a H, <Tail as ToRef<'a>>::Output>;

    #[inline(always)]
    fn to_ref(&'a self) -> Self::Output {
        Coniunctio {
            head: &self.head,
            tail: self.tail.to_ref(),
        }
    }
}

impl<'a> ToMut<'a> for Nihil {
    type Output = Nihil;

    #[inline(always)]
    fn to_mut(&'a mut self) -> Self::Output {
        Nihil
    }
}

impl<'a, H, Tail> ToMut<'a> for Coniunctio<H, Tail>
where
    H: 'a,
    Tail: ToMut<'a>,
{
    type Output = Coniunctio<&'a mut H, <Tail as ToMut<'a>>::Output>;

    #[inline(always)]
    fn to_mut(&'a mut self) -> Self::Output {
        Coniunctio {
            head: &mut self.head,
            tail: self.tail.to_mut(),
        }
    }
}

/// Trait for performing a left fold over an `HList`
///
/// This trait is part of the implementation of the inherent method
/// [`Coniunctio::foldl`]. Please see that method for more information.
///
/// You only need to import this trait when working with Universalis
/// `HLists` or Mappers of unknown type. If the type of everything is known,
/// then `list.foldl(f, acc)` should "just work" even without the trait.
///
/// [`Coniunctio::foldl`]: struct.Coniunctio.html#method.foldl
pub trait HFoldLeftable<Folder, Acc> {
    /// The final accumulator type after folding every element from the
    /// left; for the empty list this is `Acc` itself.
    type Output;

    /// Perform a left fold over an `HList`.
    ///
    /// Please see the [inherent method] for more information.
    ///
    /// The only difference between that inherent method and this
    /// trait method is the location of the type parameters.
    /// (here, they are on the trait rather than the method)
    ///
    /// [inherent method]: struct.Coniunctio.html#method.foldl
    fn foldl(self, folder: Folder, acc: Acc) -> Self::Output;
}

impl<F, Acc> HFoldLeftable<F, Acc> for Nihil {
    type Output = Acc;

    #[inline]
    fn foldl(self, _: F, acc: Acc) -> Self::Output {
        acc
    }
}

impl<F, R, FTail, H, Tail, Acc> HFoldLeftable<Coniunctio<F, FTail>, Acc> for Coniunctio<H, Tail>
where
    Tail: HFoldLeftable<FTail, R>,
    F: FnOnce(Acc, H) -> R,
{
    type Output = <Tail as HFoldLeftable<FTail, R>>::Output;

    #[inline]
    fn foldl(self, folder: Coniunctio<F, FTail>, acc: Acc) -> Self::Output {
        let Coniunctio { head, tail } = self;
        tail.foldl(folder.tail, (folder.head)(acc, head))
    }
}

impl<P, R, H, Tail, Acc> HFoldLeftable<Poly<P>, Acc> for Coniunctio<H, Tail>
where
    Tail: HFoldLeftable<Poly<P>, R>,
    P: Func<(Acc, H), Output = R>,
{
    type Output = <Tail as HFoldLeftable<Poly<P>, R>>::Output;

    #[inline]
    fn foldl(self, poly: Poly<P>, acc: Acc) -> Self::Output {
        let Coniunctio { head, tail } = self;
        let r = P::call((acc, head));
        tail.foldl(poly, r)
    }
}

/// Implementation for folding over an `HList` using a single function that
/// can handle all cases
///
/// ```rust
/// # fn main() {
/// use ordofp_core::hlist;
///
/// let h = hlist![1, 2, 3, 4, 5];
///
/// let r: isize = h.foldl(|acc, next| acc + next, 0);
/// assert_eq!(r, 15);
/// # }
/// ```
impl<F, H, Tail, Acc> HFoldLeftable<F, Acc> for Coniunctio<H, Tail>
where
    Tail: HFoldLeftable<F, Acc>,
    F: Fn(Acc, H) -> Acc,
{
    type Output = <Tail as HFoldLeftable<F, Acc>>::Output;

    #[inline]
    fn foldl(self, f: F, acc: Acc) -> Self::Output {
        let Coniunctio { head, tail } = self;
        let acc = f(acc, head);
        tail.foldl(f, acc)
    }
}

/// Trait for transforming an `HList` into a nested tuple.
///
/// This trait is part of the implementation of the inherent method
/// [`Coniunctio::into_tuple2`]. Please see that method for more information.
///
/// This operation is not useful in Universalis contexts, so it is unlikely
/// that you should ever need to import this trait. Do not worry;
/// if you have an `HList` of known type, then `list.into_tuple2()`
/// should "just work," even without the trait.
///
/// [`Coniunctio::into_tuple2`]: struct.Coniunctio.html#method.into_tuple2
pub trait IntoTuple2 {
    /// The 0 element in the output tuple
    type HeadType;

    /// The 1 element in the output tuple
    type TailOutput;

    /// Turns an `HList` into nested Tuple2s, which are less troublesome to pattern match
    /// and have a nicer type signature.
    ///
    /// Please see the [inherent method] for more information.
    ///
    /// [inherent method]: struct.Coniunctio.html#method.into_tuple2
    fn into_tuple2(self) -> (Self::HeadType, Self::TailOutput);
}

impl<T1, T2> IntoTuple2 for Coniunctio<T1, Coniunctio<T2, Nihil>> {
    type HeadType = T1;
    type TailOutput = T2;

    #[inline]
    fn into_tuple2(self) -> (Self::HeadType, Self::TailOutput) {
        (self.head, self.tail.head)
    }
}

impl<T, Tail> IntoTuple2 for Coniunctio<T, Tail>
where
    Tail: IntoTuple2,
{
    type HeadType = T;
    type TailOutput = (
        <Tail as IntoTuple2>::HeadType,
        <Tail as IntoTuple2>::TailOutput,
    );

    #[inline]
    fn into_tuple2(self) -> (Self::HeadType, Self::TailOutput) {
        (self.head, self.tail.into_tuple2())
    }
}

#[cfg(feature = "alloc")]
impl<H, Tail> From<Coniunctio<H, Tail>> for Vec<H>
where
    Tail: Into<Vec<H>> + HList,
{
    fn from(hlist: Coniunctio<H, Tail>) -> Self {
        let h = hlist.head;
        let t = hlist.tail;
        let mut v = Vec::with_capacity(<Coniunctio<H, Tail> as HList>::LEN);
        v.push(h);
        let mut t_vec: Vec<H> = t.into();
        v.append(&mut t_vec);
        v
    }
}

#[cfg(feature = "alloc")]
impl<T> From<Nihil> for Vec<T> {
    fn from(_: Nihil) -> Self {
        Vec::new()
    }
}

impl Default for Nihil {
    #[inline]
    fn default() -> Self {
        Nihil
    }
}

impl<T: Default, Tail: Default + HList> Default for Coniunctio<T, Tail> {
    #[inline]
    fn default() -> Self {
        coniunctio(T::default(), Tail::default())
    }
}

/// Indexed type conversions of `T -> Self` with index `I`.
/// This is a generalized version of `From` which for example allows the caller
/// to use default values for parts of `Self` and thus "fill in the blanks".
///
/// `LiftFrom` is the reciprocal of `LiftInto`.
///
/// ```rust
/// # fn main() {
/// use ordofp_core::hlist::{lift_from, LiftFrom};
/// use ordofp_core::{HList, hlist};
///
/// type H = HList![(), usize, f64, (), bool];
///
/// let x = H::lift_from(42.0);
/// assert_eq!(x, hlist![(), 0, 42.0, (), false]);
///
/// let x: H = lift_from(true);
/// assert_eq!(x, hlist![(), 0, 0.0, (), true]);
/// # }
/// ```
pub trait LiftFrom<T, I> {
    /// Performs the indexed conversion.
    fn lift_from(part: T) -> Self;
}

/// Free function version of `LiftFrom::lift_from`.
pub fn lift_from<I, T, PF: LiftFrom<T, I>>(part: T) -> PF {
    PF::lift_from(part)
}

/// An indexed conversion that consumes `self`, and produces a `T`. To produce
/// `T`, the index `I` may be used to for example "fill in the blanks".
/// `LiftInto` is the reciprocal of `LiftFrom`.
///
/// ```rust
/// # fn main() {
/// use ordofp_core::hlist::LiftInto;
/// use ordofp_core::{HList, hlist};
///
/// type H = HList![(), usize, f64, (), bool];
///
/// // Type inference works as expected:
/// let x: H = 1337.lift_into();
/// assert_eq!(x, hlist![(), 1337, 0.0, (), false]);
///
/// // Sublists:
/// let x: H = hlist![(), true].lift_into();
/// assert_eq!(x, hlist![(), 0, 0.0, (), true]);
///
/// let x: H = hlist![3.0, ()].lift_into();
/// assert_eq!(x, hlist![(), 0, 3.0, (), false]);
///
/// let x: H = hlist![(), 1337].lift_into();
/// assert_eq!(x, hlist![(), 1337, 0.0, (), false]);
///
/// let x: H = hlist![(), 1337, 42.0, (), true].lift_into();
/// assert_eq!(x, hlist![(), 1337, 42.0, (), true]);
/// # }
/// ```
pub trait LiftInto<T, I> {
    /// Performs the indexed conversion.
    fn lift_into(self) -> T;
}

impl<T, U, I> LiftInto<U, I> for T
where
    U: LiftFrom<T, I>,
{
    fn lift_into(self) -> U {
        LiftFrom::lift_from(self)
    }
}

impl<T, Tail> LiftFrom<T, Here> for Coniunctio<T, Tail>
where
    Tail: Default + HList,
{
    #[inline]
    fn lift_from(part: T) -> Self {
        coniunctio(part, Tail::default())
    }
}

impl<Head, Tail, ValAtIx, TailIx> LiftFrom<ValAtIx, There<TailIx>> for Coniunctio<Head, Tail>
where
    Head: Default,
    Tail: HList + LiftFrom<ValAtIx, TailIx>,
{
    #[inline]
    fn lift_from(part: ValAtIx) -> Self {
        coniunctio(Head::default(), Tail::lift_from(part))
    }
}

impl<Prefix, Suffix> LiftFrom<Prefix, Suffixed<Suffix>> for <Prefix as Add<Suffix>>::Output
where
    Prefix: HList + Add<Suffix>,
    Suffix: Default,
{
    #[inline]
    fn lift_from(part: Prefix) -> Self {
        part + Suffix::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use alloc::string::ToString;
    use alloc::vec;

    #[test]
    fn test_hcons() {
        let hlist1 = coniunctio(1, Nihil);
        let (h, _) = hlist1.pop();
        assert_eq!(h, 1);

        let hlist2 = coniunctio("hello", coniunctio(1, Nihil));
        let (h2, tail2) = hlist2.pop();
        let (h1, _): (i32, _) = tail2.pop();
        assert_eq!(h2, "hello");
        assert_eq!(h1, 1);
    }

    struct HasHList<T: HList>(T);

    #[test]
    fn test_contained_list() {
        let c = HasHList(coniunctio(1, Nihil));
        let retrieved = c.0;
        assert_eq!(retrieved.len(), 1);
        let new_list = coniunctio(2, retrieved);
        assert_eq!(new_list.len(), 2);
    }

    #[test]
    fn test_pluck() {
        let h = hlist![1, "hello".to_string(), true, 42f32];
        let (t, r): (f32, _) = h.clone().pluck();
        assert_eq!(t, 42f32);
        assert_eq!(r, hlist![1, "hello".to_string(), true]);
    }

    #[test]
    fn test_ref_pluck() {
        let h = &hlist![1, "hello".to_string(), true, 42f32];
        let (t, r): (&f32, _) = h.pluck();
        assert_eq!(t, &42f32);
        assert_eq!(r, hlist![&1, &"hello".to_string(), &true]);
    }

    #[test]
    fn test_hlist_macro() {
        assert_eq!(hlist![], Nihil);
        let h: HList!(i32, &str, i32) = hlist![1, "2", 3];
        let (h1, tail1) = h.pop();
        assert_eq!(h1, 1);
        assert_eq!(tail1, hlist!["2", 3]);
        let (h2, tail2) = tail1.pop();
        assert_eq!(h2, "2");
        assert_eq!(tail2, hlist![3]);
        let (h3, tail3) = tail2.pop();
        assert_eq!(h3, 3);
        assert_eq!(tail3, Nihil);
    }

    #[test]
    fn test_hlist_macro_trailing_comma() {
        let h1: HList!(i32, &str, i32) = hlist![1, "2", 3];
        let h2: HList!(i32, &str, i32,) = hlist![1, "2", 3];
        let h3: HList!(i32) = hlist![1];
        let h4: HList!(i32,) = hlist![1,];
        assert_eq!(h1, h2);
        assert_eq!(h3, h4);
    }

    #[test]
    fn test_pattern_matching() {
        let coniunctio_pat!(one1) = hlist!["one"];
        assert_eq!(one1, "one");
        let coniunctio_pat!(one2,) = hlist!["one"];
        assert_eq!(one2, "one");

        let h = hlist![5, 3.2f32, true, "blue"];
        let coniunctio_pat!(five, float, right, s) = h;
        assert_eq!(five, 5);
        assert_eq!(float, 3.2f32);
        assert!(right);
        assert_eq!(s, "blue");

        let h2 = hlist![13.5f32, "hello", Some(41)];
        let coniunctio_pat![a, b, c,] = h2;
        assert_eq!(a, 13.5f32);
        assert_eq!(b, "hello");
        assert_eq!(c, Some(41));
    }

    #[test]
    fn test_add() {
        let h1 = hlist![true, "hi"];
        let h2 = hlist![1, 32f32];
        let combined = h1 + h2;
        assert_eq!(combined, hlist![true, "hi", 1, 32f32]);
    }

    #[test]
    fn test_into_reverse() {
        let h1 = hlist![true, "hi"];
        let h2 = hlist![1, 32f32];
        assert_eq!(h1.into_reverse(), hlist!["hi", true]);
        assert_eq!(h2.into_reverse(), hlist![32f32, 1]);
    }

    #[test]
    fn test_foldr_consuming() {
        let h = hlist![1, false, 42f32];
        let folded = h.foldr(
            hlist![
                |acc, i| i + acc,
                |acc, _| if acc > 42f32 { 9000 } else { 0 },
                |acc, f| f + acc,
            ],
            1f32,
        );
        assert_eq!(folded, 9001);
    }

    #[test]
    fn test_single_func_foldr_consuming() {
        let h = hlist![1, 2, 3];
        let folded = h.foldr(&|acc, i| i * acc, 1);
        assert_eq!(folded, 6);
    }

    #[test]
    fn test_foldr_non_consuming() {
        let h = hlist![1, false, 42f32];
        let folder = hlist![
            |acc, &i| i + acc,
            |acc, &_| if acc > 42f32 { 9000 } else { 0 },
            |acc, &f| f + acc
        ];
        let folded = h.to_ref().foldr(folder, 1f32);
        assert_eq!(folded, 9001);
    }

    #[test]
    fn test_poly_foldr_consuming() {
        trait Dummy {
            fn dummy(&self) -> i32 {
                1
            }
        }
        impl<T: ?Sized> Dummy for T {}

        struct Dummynator;
        impl<T: Dummy, I: IntoIterator<Item = T>> Func<(i32, I)> for Dummynator {
            type Output = i32;
            fn call(args: (i32, I)) -> Self::Output {
                let (init, i) = args;
                i.into_iter().fold(init, |init, x| init + x.dummy())
            }
        }

        let h = hlist![0..10, 0..=10, &[0, 1, 2], &['a', 'b', 'c']];
        assert_eq!(
            h.foldr(Poly(Dummynator), 0),
            (0..10)
                .map(|d| d.dummy())
                .chain((0..=10).map(|d| d.dummy()))
                .chain([0_i32, 1, 2].iter().map(Dummy::dummy))
                .chain(['a', 'b', 'c'].iter().map(Dummy::dummy))
                .sum::<i32>()
        );
    }

    #[test]
    fn test_foldl_consuming() {
        let h = hlist![1, false, 42f32];
        let folded = h.foldl(
            hlist![
                |acc, i| i + acc,
                |acc, b: bool| if !b && acc > 42 { 9000f32 } else { 0f32 },
                |acc, f| f + acc,
            ],
            1,
        );
        assert_eq!(42f32, folded);
    }

    #[test]
    fn test_foldl_non_consuming() {
        let h = hlist![1, false, 42f32];
        let folded = h.to_ref().foldl(
            hlist![
                |acc, &i| i + acc,
                |acc, b: &bool| if !b && acc > 42 { 9000f32 } else { 0f32 },
                |acc, &f| f + acc,
            ],
            1,
        );
        assert_eq!(42f32, folded);
        assert_eq!((&h.head), &1);
    }

    #[test]
    fn test_poly_foldl_consuming() {
        trait Dummy {
            fn dummy(&self) -> i32 {
                1
            }
        }
        impl<T: ?Sized> Dummy for T {}

        struct Dummynator;
        impl<T: Dummy, I: IntoIterator<Item = T>> Func<(i32, I)> for Dummynator {
            type Output = i32;
            fn call(args: (i32, I)) -> Self::Output {
                let (acc, i) = args;
                i.into_iter().fold(acc, |acc, x| acc + x.dummy())
            }
        }

        let h = hlist![0..10, 0..=10, &[0, 1, 2], &['a', 'b', 'c']];
        assert_eq!(
            h.foldl(Poly(Dummynator), 0),
            (0..10)
                .map(|d| d.dummy())
                .chain((0..=10).map(|d| d.dummy()))
                .chain([0_i32, 1, 2].iter().map(Dummy::dummy))
                .chain(['a', 'b', 'c'].iter().map(Dummy::dummy))
                .sum::<i32>()
        );
    }

    #[test]
    fn test_map_consuming() {
        let h = hlist![9000, "joe", 41f32];
        let mapped = h.map(hlist![|n| n + 1, |s| s, |f| f + 1f32]);
        assert_eq!(mapped, hlist![9001, "joe", 42f32]);
    }

    #[test]
    fn test_poly_map_consuming() {
        let h = hlist![9000, "joe", 41f32, "schmoe", 50];
        impl Func<i32> for P {
            type Output = bool;
            fn call(args: i32) -> Self::Output {
                args > 100
            }
        }
        impl<'a> Func<&'a str> for P {
            type Output = usize;
            fn call(args: &'a str) -> Self::Output {
                args.len()
            }
        }
        impl Func<f32> for P {
            type Output = &'static str;
            fn call(_: f32) -> Self::Output {
                "dummy"
            }
        }
        struct P;
        assert_eq!(h.map(Poly(P)), hlist![true, 3, "dummy", 6, false]);
    }

    #[test]
    fn test_poly_map_non_consuming() {
        let h = hlist![9000, "joe", 41f32, "schmoe", 50];
        impl<'a> Func<&'a i32> for P {
            type Output = bool;
            fn call(args: &'a i32) -> Self::Output {
                *args > 100
            }
        }
        impl<'a> Func<&'a &'a str> for P {
            type Output = usize;
            fn call(args: &'a &'a str) -> Self::Output {
                args.len()
            }
        }
        impl<'a> Func<&'a f32> for P {
            type Output = &'static str;
            fn call(_: &'a f32) -> Self::Output {
                "dummy"
            }
        }
        struct P;
        assert_eq!(h.to_ref().map(Poly(P)), hlist![true, 3, "dummy", 6, false]);
    }

    #[test]
    fn test_map_single_func_consuming() {
        let h = hlist![9000, 9001, 9002];
        let mapped = h.map(|v| v + 1);
        assert_eq!(mapped, hlist![9001, 9002, 9003]);
    }

    #[test]
    fn test_map_single_func_non_consuming() {
        let h = hlist![9000, 9001, 9002];
        let mapped = h.to_ref().map(|v| v + 1);
        assert_eq!(mapped, hlist![9001, 9002, 9003]);
    }

    #[test]
    fn test_map_non_consuming() {
        let h = hlist![9000, "joe", 41f32];
        let mapped = h.to_ref().map(hlist![|&n| n + 1, |&s| s, |&f| f + 1f32]);
        assert_eq!(mapped, hlist![9001, "joe", 42f32]);
    }

    #[test]
    fn test_zip_easy() {
        let h1 = hlist![9000, "joe", 41f32];
        let h2 = hlist!["joe", 9001, 42f32];
        let zipped = h1.zip(h2);
        assert_eq!(
            zipped,
            hlist![(9000, "joe"), ("joe", 9001), (41f32, 42f32),]
        );
    }

    #[test]
    fn test_zip_composes() {
        let h1 = hlist![1, "1", 1.0];
        let h2 = hlist![2, "2", 2.0];
        let h3 = hlist![3, "3", 3.0];
        let zipped = h1.zip(h2).zip(h3);
        assert_eq!(
            zipped,
            hlist![((1, 2), 3), (("1", "2"), "3"), ((1.0, 2.0), 3.0)],
        );
    }

    #[test]
    fn test_sculpt() {
        let h = hlist![9000, "joe", 41f32];
        let (reshaped, remainder): (HList!(f32, i32), _) = h.sculpt();
        assert_eq!(reshaped, hlist![41f32, 9000]);
        assert_eq!(remainder, hlist!["joe"]);
    }

    #[test]
    fn test_len_const() {
        assert_eq!(<HList![usize, &str, f32] as HList>::LEN, 3);
    }

    #[test]
    fn test_single_func_foldl_consuming() {
        use std::collections::HashMap;

        let h = hlist![
            ("one", 1),
            ("two", 2),
            ("three", 3),
            ("four", 4),
            ("five", 5),
        ];
        let r = h.foldl(
            |mut acc: HashMap<&'static str, isize>, (k, v)| {
                acc.insert(k, v);
                acc
            },
            HashMap::with_capacity(5),
        );
        let expected: HashMap<_, _> = {
            vec![
                ("one", 1),
                ("two", 2),
                ("three", 3),
                ("four", 4),
                ("five", 5),
            ]
            .into_iter()
            .collect()
        };
        assert_eq!(r, expected);
    }

    #[test]
    fn test_single_func_foldl_non_consuming() {
        let h = hlist![1, 2, 3, 4, 5];
        let r: isize = h.to_ref().foldl(|acc, &next| acc + next, 0isize);
        assert_eq!(r, 15);
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_into_vec() {
        let h = hlist![1, 2, 3, 4, 5];
        let as_vec: Vec<_> = h.into();
        assert_eq!(as_vec, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_lift() {
        type H = HList![(), usize, f64, (), bool];

        // Ensure type inference works as expected first:
        let x: H = 1337.lift_into();
        assert_eq!(x, hlist![(), 1337, 0.0, (), false]);

        let x = H::lift_from(42.0);
        assert_eq!(x, hlist![(), 0, 42.0, (), false]);

        let x: H = lift_from(true);
        assert_eq!(x, hlist![(), 0, 0.0, (), true]);

        // Sublists:
        let x: H = hlist![(), true].lift_into();
        assert_eq!(x, hlist![(), 0, 0.0, (), true]);

        let x: H = hlist![3.0, ()].lift_into();
        assert_eq!(x, hlist![(), 0, 3.0, (), false]);

        let x: H = hlist![(), 1337].lift_into();
        assert_eq!(x, hlist![(), 1337, 0.0, (), false]);

        let x: H = hlist![(), 1337, 42.0, (), true].lift_into();
        assert_eq!(x, hlist![(), 1337, 42.0, (), true]);
    }

    #[test]
    fn test_coniunctio_extend_nihil() {
        let first = hlist![0];
        let second = hlist![];

        assert_eq!(first.extend(second), hlist![0]);
    }

    #[test]
    fn test_nihil_extend_coniunctio() {
        let first = hlist![];
        let second = hlist![0];

        assert_eq!(first.extend(second), hlist![0]);
    }

    #[test]
    fn test_nihil_extend_nihil() {
        let first = hlist![];
        let second = hlist![];

        assert_eq!(first.extend(second), hlist![]);
    }
}
