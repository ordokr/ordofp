// Makes sure that the hlist macros in ordofp_core are reexported by ordofp
use ordofp::{HList, coniunctio_pat, hlist};

#[test]
fn use_ordofp_macros() {
    let h1 = hlist![1i32, 2u32];
    let h2 = hlist!["cool", ...h1];
    let coniunctio_pat![a, ...bs]: HList![&'static str, i32, ...HList![u32]] = h2;
    assert_eq!(a, "cool");
    assert_eq!(bs, h1);
}

/// Malformed derive/macro input must produce a pointed `syn::Error` compile
/// error (with a caret at the offending token), not a spanless "proc macro
/// panicked".
#[test]
fn ui_macro_errors() {
    let t = trybuild::TestCases::new();
    // derive(Universalis): structs only.
    t.compile_fail("tests/ui/universalis_enum.rs");
    t.compile_fail("tests/ui/universalis_union.rs");
    // derive(NominataUniversalis): structs and enums, not unions.
    t.compile_fail("tests/ui/nominata_universalis_union.rs");
    // path! / path_type!: field-access chains only.
    t.compile_fail("tests/ui/path_tuple_field.rs");
    t.compile_fail("tests/ui/path_module_sep.rs");
    t.compile_fail("tests/ui/path_invalid_expr.rs");
    t.compile_fail("tests/ui/path_type_module_sep.rs");
}
