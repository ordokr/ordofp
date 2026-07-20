//! Tests for `OrdoFP` 4.0 Phase 5: Row Polymorphism
//!
//! Tests extensible records (Registrum) and variants (Variatio).

#![cfg(feature = "rows")]

extern crate alloc;

use alloc::format;
use alloc::string::String;

use ordofp_core::hlist::{Coniunctio, Nihil};
use ordofp_core::indices::{Here, There};
use ordofp_core::labelled::Field;
use ordofp_core::labelled::chars::*;
use ordofp_core::{field, hlist};

use ordofp_core::rows::{
    Bool,
    Casus,
    Confluo,
    Extendo,
    ExtendoCasum,
    Falsum,
    // Field operations
    HabetCampum,
    Muto,
    // Core row traits
    Ordo,
    OrdoOps,
    // Record type
    Registrum,
    RegistrumExt,
    Renomino,
    Restricto,
    // Variant type
    Variatio,
    Verum,
    inject,
};

// =============================================================================
// TYPE ALIASES
// =============================================================================

type Name = (Ln, La, Lm, Le);
type Age = (La, Lg, Le);
type Active = (La, Lc, Lt, Li, Lv, Le);

type Success = (Ls, Lu, Lc, Lc, Le, Ls, Ls);
type Error = (Le, Lr, Lr, Lo, Lr);

// =============================================================================
// ORDO TRAIT TESTS
// =============================================================================

#[test]
fn test_nihil_is_ordo() {
    fn assert_ordo<T: Ordo>() {}
    assert_ordo::<Nihil>();
}

#[test]
fn test_coniunctio_is_ordo() {
    fn assert_ordo<T: Ordo>() {}
    assert_ordo::<Coniunctio<(), Nihil>>();
    assert_ordo::<Coniunctio<i32, Coniunctio<String, Nihil>>>();
}

#[test]
fn test_ordo_ops_numerus() {
    assert_eq!(<Nihil as OrdoOps>::NUMERUS, 0);
    assert_eq!(<Coniunctio<(), Nihil> as OrdoOps>::NUMERUS, 1);
    assert_eq!(
        <Coniunctio<i32, Coniunctio<bool, Nihil>> as OrdoOps>::NUMERUS,
        2
    );
}

#[test]
fn test_bool_types() {
    // Compile-time constants - verify with const assertions
    const _: () = assert!(Verum::VALUE);
    const _: () = assert!(!Falsum::VALUE);
}

// =============================================================================
// HABET CAMPUM TESTS
// =============================================================================

#[test]
fn test_habet_campum_head() {
    let record = hlist![field!(Name, "Alice")];
    let name: &str = HabetCampum::<Name, &str, Here>::get_field(&record);
    assert_eq!(name, "Alice");
}

#[test]
fn test_habet_campum_tail() {
    let record = hlist![field!(Age, 30i32), field!(Name, "Bob")];
    let name: &str = HabetCampum::<Name, &str, There<Here>>::get_field(&record);
    assert_eq!(name, "Bob");
}

#[test]
fn test_habet_campum_mut() {
    let mut record = hlist![field!(Age, 30i32)];
    {
        let age: &mut i32 = HabetCampum::<Age, i32, Here>::get_field_mut(&mut record);
        *age += 1;
    }
    assert_eq!(record.head.value, 31);
}

// =============================================================================
// EXTENDO TESTS
// =============================================================================

#[test]
fn test_extendo_nihil() {
    let record: Coniunctio<Field<Name, &str>, Nihil> =
        Extendo::<Name, &str>::extend(Nihil, "name", "Alice");
    assert_eq!(record.head.value, "Alice");
    assert_eq!(record.head.name, "name");
}

#[test]
fn test_extendo_non_empty() {
    let record = hlist![field!(Name, "Alice")];
    let extended: Coniunctio<Field<Age, i32>, _> = Extendo::<Age, i32>::extend(record, "age", 30);
    assert_eq!(extended.head.value, 30);
    assert_eq!(extended.tail.head.value, "Alice");
}

#[test]
fn test_extendo_chained() {
    // Use the Extendo trait explicitly for chaining
    let r1: Coniunctio<Field<Name, &str>, Nihil> =
        Extendo::<Name, &str>::extend(Nihil, "name", "Charlie");
    let r2 = Extendo::<Age, i32>::extend(r1, "age", 25i32);
    let r3 = Extendo::<Active, bool>::extend(r2, "active", true);

    assert!(r3.head.value);
    assert_eq!(r3.tail.head.value, 25);
    assert_eq!(r3.tail.tail.head.value, "Charlie");
}

// =============================================================================
// RESTRICTO TESTS
// =============================================================================

#[test]
fn test_restricto_head() {
    let record = hlist![field!(Name, "Alice"), field!(Age, 30i32)];
    let (name, rest): (&str, _) = Restricto::<Name, Here>::restrict(record);
    assert_eq!(name, "Alice");
    assert_eq!(rest.head.value, 30);
}

#[test]
fn test_restricto_tail() {
    let record = hlist![field!(Age, 30i32), field!(Name, "Bob")];
    let (name, rest): (&str, _) = Restricto::<Name, There<Here>>::restrict(record);
    assert_eq!(name, "Bob");
    assert_eq!(rest.head.value, 30);
}

// =============================================================================
// CONFLUO (MERGE) TESTS
// =============================================================================

#[test]
fn test_confluo_empty_left() {
    let r1 = Nihil;
    let r2 = hlist![field!(Name, "Alice")];
    let merged: Coniunctio<Field<Name, &str>, Nihil> = Confluo::merge(r1, r2);
    assert_eq!(merged.head.value, "Alice");
}

#[test]
fn test_confluo_empty_right() {
    let r1 = hlist![field!(Name, "Alice")];
    let r2 = Nihil;
    let merged: Coniunctio<Field<Name, &str>, Nihil> = Confluo::merge(r1, r2);
    assert_eq!(merged.head.value, "Alice");
}

#[test]
fn test_confluo_two_records() {
    let r1 = hlist![field!(Name, "Alice")];
    let r2 = hlist![field!(Age, 30i32)];
    let merged = Confluo::merge(r1, r2);
    assert_eq!(merged.head.value, "Alice");
    assert_eq!(merged.tail.head.value, 30);
}

// =============================================================================
// MUTO (MODIFY) TESTS
// =============================================================================

#[test]
fn test_muto_head() {
    let record = hlist![field!(Age, 30i32)];
    let modified: Coniunctio<Field<Age, i32>, Nihil> =
        Muto::<Age, i32, Here>::modify(record, |age| age + 1);
    assert_eq!(modified.head.value, 31);
}

#[test]
fn test_muto_tail() {
    let record = hlist![field!(Name, "Alice"), field!(Age, 30i32)];
    let modified = Muto::<Age, i32, There<Here>>::modify(record, |age| age * 2);
    assert_eq!(modified.head.value, "Alice");
    assert_eq!(modified.tail.head.value, 60);
}

#[test]
fn test_muto_type_change() {
    let record = hlist![field!(Age, 30i32)];
    let modified: Coniunctio<Field<Age, String>, Nihil> =
        Muto::<Age, String, Here>::modify(record, |age| format!("{age} years"));
    assert_eq!(modified.head.value, "30 years");
}

// =============================================================================
// RENOMINO (RENAME) TESTS
// =============================================================================

#[test]
fn test_renomino_head() {
    type NewName = (Ln, Le, Lw, Underscore, Ln, La, Lm, Le);
    let record = hlist![field!(Name, "Alice")];
    let renamed: Coniunctio<Field<NewName, &str>, Nihil> =
        Renomino::<Name, NewName, Here>::rename(record, "new_name");
    assert_eq!(renamed.head.name, "new_name");
    assert_eq!(renamed.head.value, "Alice");
}

// =============================================================================
// REGISTRUM TESTS
// =============================================================================

#[test]
fn test_registrum_new() {
    let record = Registrum::new();
    assert!(record.is_empty());
    assert_eq!(record.len(), 0);
}

#[test]
fn test_registrum_from_hlist() {
    let hlist = hlist![field!(Name, "Alice"), field!(Age, 30i32)];
    let record = Registrum::from_hlist(hlist);
    assert!(!record.is_empty());
    assert_eq!(record.len(), 2);
}

#[test]
fn test_registrum_into_hlist() {
    let original = hlist![field!(Name, "Alice")];
    let record = Registrum::from_hlist(original);
    let back = record.into_hlist();
    assert_eq!(back, original);
}

#[test]
fn test_registrum_extend_field() {
    let record = Registrum::new()
        .extend_field::<Name, _>("name", "Alice")
        .extend_field::<Age, _>("age", 30i32);

    assert_eq!(record.len(), 2);
}

#[test]
fn test_registrum_get() {
    let record = Registrum::from_hlist(hlist![field!(Name, "Alice"), field!(Age, 30i32)]);

    let name: &&str = record.get::<Name, &str, Here>();
    assert_eq!(*name, "Alice");

    let age: &i32 = record.get::<Age, i32, There<Here>>();
    assert_eq!(*age, 30);
}

#[test]
fn test_registrum_get_mut() {
    let mut record = Registrum::from_hlist(hlist![field!(Age, 30i32)]);

    {
        let age: &mut i32 = record.get_mut::<Age, i32, Here>();
        *age += 5;
    }

    let age: &i32 = record.get::<Age, i32, Here>();
    assert_eq!(*age, 35);
}

#[test]
fn test_registrum_restrict() {
    let record = Registrum::from_hlist(hlist![field!(Name, "Alice"), field!(Age, 30i32)]);

    let (name, rest): (&str, Registrum<_>) = record.restrict::<Name, Here>();
    assert_eq!(name, "Alice");
    assert_eq!(rest.len(), 1);
}

#[test]
fn test_registrum_merge() {
    let r1 = Registrum::from_hlist(hlist![field!(Name, "Alice")]);
    let r2 = Registrum::from_hlist(hlist![field!(Age, 30i32)]);

    let merged = r1.merge(r2);
    assert_eq!(merged.len(), 2);
}

#[test]
fn test_registrum_modify() {
    let record = Registrum::from_hlist(hlist![field!(Age, 30i32)]);
    let modified = record.modify::<Age, i32, Here, _>(|age| age + 1);

    let age: &i32 = modified.get::<Age, i32, Here>();
    assert_eq!(*age, 31);
}

#[test]
fn test_registrum_with() {
    let record = Registrum::from_hlist(hlist![field!(Age, 30i32)]);
    let result = record.with(|r| r.len());
    assert_eq!(result, 1);
}

#[test]
fn test_registrum_debug() {
    let record = Registrum::new();
    let debug = format!("{record:?}");
    assert!(debug.contains("Registrum"));
}

// =============================================================================
// ROW-POLYMORPHIC FUNCTION TESTS
// =============================================================================

fn get_age<R, I>(record: &Registrum<R>) -> i32
where
    R: HabetCampum<Age, i32, I>,
{
    *record.get::<Age, i32, I>()
}

#[test]
fn test_row_polymorphic_function() {
    // Function works with records that have Age field, regardless of other fields
    let person1 = Registrum::from_hlist(hlist![field!(Age, 25i32), field!(Name, "Alice")]);

    let person2 = Registrum::from_hlist(hlist![field!(Active, true), field!(Age, 30i32)]);

    assert_eq!(get_age::<_, Here>(&person1), 25);
    assert_eq!(get_age::<_, There<Here>>(&person2), 30);
}

// =============================================================================
// VARIATIO TESTS
// =============================================================================

type ResultRow = Coniunctio<Casus<Success, i32>, Coniunctio<Casus<Error, String>, Nihil>>;

#[test]
fn test_variatio_inject() {
    let v: Variatio<ResultRow> = inject::<Success, _, _, _>(42);
    assert!(v.is::<Success>());
    assert!(!v.is::<Error>());
}

#[test]
fn test_variatio_try_get() {
    let v: Variatio<ResultRow> = inject::<Success, _, _, _>(42i32);
    assert_eq!(v.try_get::<Success, i32>(), Some(&42));
    assert_eq!(v.try_get::<Error, String>(), None);
}

#[test]
fn test_variatio_on_matched() {
    let v: Variatio<ResultRow> = inject::<Success, _, _, _>(42i32);
    let result = v.on::<Success, i32, _, _, _>(|n| n * 2).otherwise(0);
    assert_eq!(result, 84);
}

#[test]
fn test_variatio_on_unmatched() {
    let v: Variatio<ResultRow> = inject::<Error, _, _, _>(String::from("oops"));
    let result = v.on::<Success, i32, _, _, _>(|n| n * 2).otherwise(-1);
    assert_eq!(result, -1);
}

#[test]
fn test_variatio_multiple_cases() {
    let v1: Variatio<ResultRow> = inject::<Success, _, _, _>(42i32);
    let v2: Variatio<ResultRow> = inject::<Error, _, _, _>(String::from("oops"));

    let r1 = v1
        .on::<Success, i32, _, _, _>(|n| format!("ok: {n}"))
        .on::<Error, String, _, _>(|e| format!("err: {e}"))
        .exhaust();

    let r2 = v2
        .on::<Success, i32, _, _, _>(|n| format!("ok: {n}"))
        .on::<Error, String, _, _>(|e| format!("err: {e}"))
        .exhaust();

    assert_eq!(r1, "ok: 42");
    assert_eq!(r2, "err: oops");
}

#[test]
fn test_variatio_otherwise_with() {
    let v: Variatio<ResultRow> = inject::<Error, _, _, _>(String::from("oops"));
    let result = v.on::<Success, i32, _, _, _>(|n| n).otherwise_with(|| 999);
    assert_eq!(result, 999);
}

#[test]
fn test_variatio_widen() {
    type NewCase = (Ln, Le, Lw);

    let v: Variatio<ResultRow> = inject::<Success, _, _, _>(42i32);
    let widened: Variatio<Coniunctio<Casus<NewCase, bool>, ResultRow>> = v.widen();

    assert!(widened.is::<Success>());
    assert!(!widened.is::<NewCase>());
}

#[test]
fn test_variatio_debug() {
    let v: Variatio<ResultRow> = inject::<Success, _, _, _>(42i32);
    let debug = format!("{v:?}");
    assert!(debug.contains("Variatio"));
}

// =============================================================================
// INTEGRATION TESTS
// =============================================================================

#[test]
fn test_record_variant_integration() {
    // Create a record
    let person = Registrum::from_hlist(hlist![field!(Name, "Alice"), field!(Age, 30i32)]);

    // Extract age and create a variant based on it
    let age: i32 = *person.get::<Age, i32, There<Here>>();

    let status: Variatio<ResultRow> = if age >= 18 {
        inject::<Success, _, _, _>(age)
    } else {
        inject::<Error, _, _, _>(String::from("underage"))
    };

    let message = status
        .on::<Success, i32, _, _, _>(|a| format!("Adult, age {a}"))
        .on::<Error, String, _, _>(|e| format!("Error: {e}"))
        .exhaust();

    assert_eq!(message, "Adult, age 30");
}

#[test]
fn test_complex_record_operations() {
    // Build record step by step using Registrum
    let record = Registrum::new()
        .extend_field::<Name, _>("name", "Bob")
        .extend_field::<Age, _>("age", 25i32)
        .extend_field::<Active, _>("active", true);

    // Modify age
    let older = record.modify::<Age, i32, There<Here>, _>(|a| a + 1);

    // Extract active status
    let (active, rest): (bool, _) = older.restrict::<Active, Here>();
    assert!(active);
    assert_eq!(rest.len(), 2);
}
