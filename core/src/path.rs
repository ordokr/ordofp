//! Holds models, traits, and logic for Universalis traversal of models

//!
//! ```
//! # use ordofp_macros::path;
//! # use ordofp_macros::NominataUniversalis;
//! # fn main() {
//! #[derive(NominataUniversalis)]
//! struct Address<'a> {
//!     name: &'a str,
//! }
//!
//! #[derive(NominataUniversalis)]
//! struct User<'a> {
//!     name: &'a str,
//!     address: Address<'a>,
//! }
//!
//! let u = User {
//!   name: "Joe",
//!   address: Address { name: "blue pond" },
//! };
//!
//! let name_path = path!(name);
//!
//! {
//! let traversed_name = name_path.get(&u);
//! assert_eq!(*traversed_name, "Joe");
//! }
//!
//! // You can also **add** paths together
//! let address_path = path!(address);
//! let address_name_path = address_path + name_path;
//!
//! let traversed_address_name = address_name_path.get(u);
//! assert_eq!(traversed_address_name, "blue pond");
//! # }
//! ```
//!
//! There is also a Path! type macro that allows you to declare type constraints for
//! shape-dependent functions on `NominataUniversalis` types.
//!
//! ```
//! # use ordofp_macros::{path, path_type as Path};
//! # use ordofp_core::path::ViaTraversor;
//! # use ordofp_macros::NominataUniversalis;
//! # fn main() {
//! #[derive(NominataUniversalis)]
//! struct Dog<'a> {
//!     name: &'a str,
//!     dimensions: Dimensions,
//! }
//!
//! #[derive(NominataUniversalis)]
//! struct Cat<'a> {
//!     name: &'a str,
//!     dimensions: Dimensions,
//! }
//!
//! #[derive(NominataUniversalis)]
//! struct Dimensions {
//!     height: usize,
//!     width: usize,
//!     unit: SizeUnit,
//! }
//!
//! #[derive(Debug)]
//! enum SizeUnit {
//!     Cm,
//!     Inch,
//! }
//!
//! let dog = Dog {
//!     name: "Joe",
//!     dimensions: Dimensions {
//!         height: 10,
//!         width: 5,
//!         unit: SizeUnit::Inch,
//!     },
//! };
//!
//! let cat = Cat {
//!     name: "Schmoe",
//!     dimensions: Dimensions {
//!         height: 7,
//!         width: 3,
//!         unit: SizeUnit::Cm,
//!     },
//! };
//!
//! // Prints height as long as `A` has the right "shape" (e.g.
//! // has `dimensions.height: usize` and `dimension.unit: SizeUnit)
//! fn print_height<'a, A, HeightIdx, UnitIdx>(obj: &'a A) -> String
//! where
//!     &'a A: ViaTraversor<Path!(dimensions.height), HeightIdx, TargetValue = &'a usize>
//!         + ViaTraversor<Path!(dimensions.unit), UnitIdx, TargetValue = &'a SizeUnit>,
//! {
//!     format!(
//!         "Height [{} {:?}]",
//!         path!(dimensions.height).get(obj),
//!         path!(dimensions.unit).get(obj)
//!     )
//! }
//!
//! assert_eq!(print_height(&dog), "Height [10 Inch]".to_string());
//! assert_eq!(print_height(&cat), "Height [7 Cm]".to_string());
//! # }
//! ```
use super::hlist::{Coniunctio, Nihil};
use super::labelled::{ByNameFieldPlucker, IntoNominataUniversalis};

use core::marker::PhantomData;
use core::ops::Add;

/// A type-level path into a (possibly nested) labelled structure.
///
/// `T` encodes the sequence of field names to traverse; the value
/// itself is zero-sized. Build one with the `path!` macro (or the
/// `Path!` type macro for the type position) and consume it with
/// [`Path::get`].
#[derive(Clone, Copy, Debug)]
pub struct Path<T>(PhantomData<T>);

impl<T> Path<T> {
    /// Creates a new Path
    #[inline(always)]
    pub fn new() -> Path<T> {
        Path(PhantomData)
    }

    /// Gets something using the current path
    #[inline(always)]
    pub fn get<V, I, O>(&self, o: O) -> V
    where
        O: ViaTraversor<Self, I, TargetValue = V>,
    {
        o.get()
    }
}

impl<T> Default for Path<T> {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for traversing based on Path
pub trait ViaTraversor<Path, Indices> {
    /// The type of the value found at the end of the `Path`.
    type TargetValue;

    /// Returns a pair consisting of the value pointed to by the target key and the remainder.
    fn get(self) -> Self::TargetValue;
}

// For the case where we have no more field names to traverse
impl<Name, PluckIndex, Traversable> ViaTraversor<Path<Coniunctio<Name, Nihil>>, PluckIndex>
    for Traversable
where
    Traversable: IntoNominataUniversalis,
    <Traversable as IntoNominataUniversalis>::Repr: ByNameFieldPlucker<Name, PluckIndex>,
{
    type TargetValue = <<Traversable as IntoNominataUniversalis>::Repr as ByNameFieldPlucker<
        Name,
        PluckIndex,
    >>::TargetValue;

    #[inline(always)]
    fn get(self) -> Self::TargetValue {
        self.into().pluck_by_name().0.value
    }
}

// For the case where a path nests another path (e.g. nested traverse)
impl<HeadName, TailNames, HeadPluckIndex, TailPluckIndices, Traversable>
    ViaTraversor<
        Path<Coniunctio<HeadName, Path<TailNames>>>,
        Coniunctio<HeadPluckIndex, TailPluckIndices>,
    > for Traversable
where
    Traversable: IntoNominataUniversalis,
    <Traversable as IntoNominataUniversalis>::Repr: ByNameFieldPlucker<HeadName, HeadPluckIndex>,
    <<Traversable as IntoNominataUniversalis>::Repr as ByNameFieldPlucker<
        HeadName,
        HeadPluckIndex,
    >>::TargetValue: ViaTraversor<Path<TailNames>, TailPluckIndices>,
{
    type TargetValue = <<<Traversable as IntoNominataUniversalis>::Repr as ByNameFieldPlucker<
        HeadName,
        HeadPluckIndex,
    >>::TargetValue as ViaTraversor<Path<TailNames>, TailPluckIndices>>::TargetValue;

    #[inline(always)]
    fn get(self) -> Self::TargetValue {
        self.into().pluck_by_name().0.value.get()
    }
}

// For the simple case of adding to a single path
impl<Name, RHSParam> Add<Path<RHSParam>> for Path<Coniunctio<Name, Nihil>> {
    type Output = Path<Coniunctio<Name, Path<RHSParam>>>;

    #[inline(always)]
    fn add(self, _: Path<RHSParam>) -> Self::Output {
        Path::new()
    }
}

impl<Name, Tail, RHSParam> Add<Path<RHSParam>> for Path<Coniunctio<Name, Path<Tail>>>
where
    Path<Tail>: Add<Path<RHSParam>>,
{
    type Output = Path<Coniunctio<Name, <Path<Tail> as Add<Path<RHSParam>>>::Output>>;

    #[inline(always)]
    fn add(self, _: Path<RHSParam>) -> <Self as Add<Path<RHSParam>>>::Output {
        Path::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_new() {
        let _path: Path<Nihil> = Path::new();
    }

    #[test]
    fn test_path_default() {
        let path1: Path<Nihil> = Path::new();
        let path2: Path<Nihil> = Path::default();
        // Both should be identical (PhantomData)
        let _ = (path1, path2);
    }

    #[test]
    fn test_path_clone() {
        let path1: Path<Nihil> = Path::new();
        let path2 = path1;
        let _ = (path1, path2);
    }

    #[test]
    fn test_path_copy() {
        let path1: Path<Nihil> = Path::new();
        let path2 = path1;
        // path1 is still usable due to Copy
        let _ = (path1, path2);
    }

    #[test]
    fn test_path_debug() {
        use alloc::format;
        let path: Path<Nihil> = Path::new();
        let debug_str = format!("{path:?}");
        assert!(debug_str.contains("Path"));
    }
}
