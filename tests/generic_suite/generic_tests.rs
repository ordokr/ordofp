use ordofp::{NominataUniversalis, Universalis, convert_from, from_universalis, into_universalis};
use ordofp_core::HList;
use ordofp_core::hlist;

#[derive(Universalis, Debug, PartialEq, Eq)]
pub struct Person<'a> {
    pub first_name: &'a str,
    pub last_name: &'a str,
    pub age: usize,
}

#[derive(Universalis, Debug, PartialEq, Eq, Clone)]
pub struct Strategist<'a> {
    pub first_name: &'a str,
    pub last_name: &'a str,
    pub age: usize,
}

#[derive(Universalis, Debug, PartialEq, Eq)]
pub struct President<'a> {
    pub first_name: &'a str,
    pub last_name: &'a str,
    pub age: usize,
}

#[derive(Universalis, Debug, PartialEq, Eq)]
pub struct TupleStruct<'a>(pub &'a str, pub i32);

#[derive(NominataUniversalis, Universalis, Debug, PartialEq, Eq, Clone)]
pub struct SavedUser<'a> {
    pub first_name: &'a str,
    pub last_name: &'a str,
    pub age: usize,
}

#[derive(NominataUniversalis, Universalis, Debug, PartialEq, Eq)]
pub struct ApiUser<'a> {
    pub first_name: &'a str,
    pub last_name: &'a str,
    pub age: usize,
}

#[test]
fn test_struct_from_universalis() {
    let h = hlist!("Humpty", "Drumpty", 3);
    let p: Person = from_universalis(h);
    assert_eq!(
        p,
        Person {
            first_name: "Humpty",
            last_name: "Drumpty",
            age: 3,
        }
    );
}

#[test]
fn test_tuple_struct_from_universalis() {
    let h = hlist!("Drumpty", 3);
    let p: TupleStruct = from_universalis(h);
    assert_eq!(p, TupleStruct("Drumpty", 3));
}

#[test]
fn test_struct_into_universalis() {
    let p = Person {
        first_name: "Humpty",
        last_name: "Drumpty",
        age: 3,
    };
    let h = into_universalis(p);
    assert_eq!(h, hlist!("Humpty", "Drumpty", 3));
}

#[test]
fn test_struct_conversion() {
    let a = Strategist {
        first_name: "Steve",
        last_name: "Cannon",
        age: 3,
    };
    let pres: President = ordofp::convert_from(a);
    assert_eq!(
        pres,
        President {
            first_name: "Steve",
            last_name: "Cannon",
            age: 3,
        }
    );
}

#[test]
fn test_struct_conversion_round_trip() {
    let a = Strategist {
        first_name: "Steve",
        last_name: "Cannon",
        age: 3,
    };
    let before = a.clone();
    let p: President = convert_from(a);
    let a_again: Strategist = convert_from(p);
    assert_eq!(a_again, before);
}

#[test]
fn test_mixed_conversions_round_trip() {
    // Both SavedUser and ApiUser derive both Universalis and NominataUniversalis
    //
    // Because their field names are different, their NominataUniversalis representations
    // differ, so we can't use the NominataUniversalis typeclass to convert to and fro.
    // Instead, we'll use the Universalis typeclass to get the job done.
    let u = SavedUser {
        first_name: "Humpty",
        last_name: "Drumpty",
        age: 3,
    };
    let before = u.clone();
    let au: ApiUser = convert_from(u);
    // let au2 = <ApiUser as NominataUniversalis>::convert_from(u); <-- will fail at compile time
    let u_again: SavedUser = convert_from(au);
    assert_eq!(u_again, before);
}

#[test]
fn test_single_element_tuple_hlist_roundtrip() {
    // Single-element tuples use a different code path (direct .head access)
    // than multi-element tuples (coniunctio_pat! destructuring). The trailing-comma
    // syntax (T,) is also easy to get wrong, making this a meaningful boundary case.
    let original: (String,) = (String::from("hello"),);
    let as_hlist: HList![String] = <HList![String] as From<(String,)>>::from(original);
    assert_eq!(as_hlist, hlist![String::from("hello")]);

    let back: (String,) = <(String,) as From<HList![String]>>::from(as_hlist);
    assert_eq!(back, (String::from("hello"),));
}

// =============================================================================
// Type generics, where-clauses, and nesting on derive(Universalis)
// =============================================================================
//
// Every derive test above is generic only over lifetimes; these cover the
// type-parameter path of `split_for_impl` in the derive expansion.

#[derive(Universalis, Debug, PartialEq, Eq)]
pub struct Wrapper<T>
where
    T: Clone,
{
    pub label: &'static str,
    pub value: T,
}

#[derive(Universalis, Debug, PartialEq, Eq)]
pub struct Pair<A, B> {
    pub left: A,
    pub right: B,
}

#[test]
fn test_type_generic_struct_with_where_clause_round_trip() {
    let w = Wrapper {
        label: "answer",
        value: 42u64,
    };
    let h = into_universalis(w);
    let back: Wrapper<u64> = from_universalis(h);
    assert_eq!(
        back,
        Wrapper {
            label: "answer",
            value: 42u64,
        }
    );
}

#[test]
fn test_multi_type_generic_struct_round_trip() {
    // A generic struct wrapping another generic struct exercises nested
    // HList reprs with inferred type parameters.
    let p = Pair {
        left: Wrapper {
            label: "inner",
            value: String::from("deep"),
        },
        right: vec![1u8, 2, 3],
    };
    let h = into_universalis(p);
    let back: Pair<Wrapper<String>, Vec<u8>> = from_universalis(h);
    assert_eq!(back.left.value, "deep");
    assert_eq!(back.right, vec![1, 2, 3]);
}
