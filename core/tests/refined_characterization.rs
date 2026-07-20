//! Characterization tests for `refined::common` — pins the full
//! `(predicate × type)` impl matrix ahead of its macro-isation.
//!
//! These tests capture *current* behavior, including quirks, so the
//! macro-generated impls can be proven drop-in equivalent:
//!
//! - exact `description()` strings per impl (the macro must reproduce them),
//! - float edge cases: `NaN` fails every ordering predicate but satisfies
//!   `NonNullus`; `-0.0` is non-negative and zero,
//! - `IntraFines<MIN, MAX>` on `usize` returns `false` unconditionally when
//!   `MIN < 0` (unlike the smaller unsigned types, which go through an `i64`
//!   cast and can pass), and its `usize as i64` cast wraps above `i64::MAX`
//!   (which still yields the mathematically correct `false`, since `MAX` can
//!   never exceed `i64::MAX`),
//! - `MaiorQuam` on `u64`/`usize` short-circuits `true` for negative
//!   thresholds and compares without an `i64` cast (so `u64::MAX` works);
//!   `MinorQuam` on `u32`/`usize` short-circuits `false` for negative
//!   thresholds,
//! - `Magnitudo*` on `String` measures **bytes**, not chars,
//! - impl-set asymmetries that the macro must *not* "helpfully" fill in:
//!   `MinorQuam` has no `u64` impl; `IntraFines` covers
//!   `{i8,i16,i32,i64,u8,u16,u32,usize}` only; `Par`/`Impar` cover
//!   `{i8,i16,i32,i64,u8,u16,u32,u64,usize}` only (no `i128`/`isize`).
//!   (Absences are compile-time facts and cannot be asserted here; this list
//!   is the reference.)

#![cfg(feature = "alloc")]

use ordofp_core::refined::{
    Falsum, Impar, IntraFines, MagnitudoExacta, MagnitudoMaxima, MagnitudoMinima, MaiorQuam,
    MinorQuam, Negativus, NonNegativus, NonNullus, NonVacuus, Par, Positivus, Praedicatum, Verum,
};

/// Pin one `(predicate, type)` impl: accepted values, rejected values, and
/// the exact `description()` string.
macro_rules! pin {
    ($name:ident, $pred:ty, $ty:ty, desc: $desc:literal,
     pass: [$($p:expr),* $(,)?], fail: [$($f:expr),* $(,)?]) => {
        #[test]
        fn $name() {
            $({
                let value: $ty = $p;
                assert!(
                    <$pred as Praedicatum<$ty>>::check(&value),
                    "{}<{}>::check({:?}) should hold",
                    stringify!($pred), stringify!($ty), value,
                );
            })*
            $({
                let value: $ty = $f;
                assert!(
                    !<$pred as Praedicatum<$ty>>::check(&value),
                    "{}<{}>::check({:?}) should NOT hold",
                    stringify!($pred), stringify!($ty), value,
                );
            })*
            assert_eq!(<$pred as Praedicatum<$ty>>::description(), $desc);
        }
    };
}

// ---------------------------------------------------------------------------
// Positivus (> 0): i8 i16 i32 i64 i128 isize f32 f64
// ---------------------------------------------------------------------------
pin!(positivus_i8, Positivus, i8, desc: "value must be positive (> 0)",
     pass: [1, i8::MAX], fail: [0, -1, i8::MIN]);
pin!(positivus_i16, Positivus, i16, desc: "value must be positive (> 0)",
     pass: [1, i16::MAX], fail: [0, -1, i16::MIN]);
pin!(positivus_i32, Positivus, i32, desc: "value must be positive (> 0)",
     pass: [1, i32::MAX], fail: [0, -1, i32::MIN]);
pin!(positivus_i64, Positivus, i64, desc: "value must be positive (> 0)",
     pass: [1, i64::MAX], fail: [0, -1, i64::MIN]);
pin!(positivus_i128, Positivus, i128, desc: "value must be positive (> 0)",
     pass: [1, i128::MAX], fail: [0, -1, i128::MIN]);
pin!(positivus_isize, Positivus, isize, desc: "value must be positive (> 0)",
     pass: [1, isize::MAX], fail: [0, -1, isize::MIN]);
pin!(positivus_f32, Positivus, f32, desc: "value must be positive (> 0)",
     pass: [1.0, f32::MIN_POSITIVE, f32::INFINITY],
     fail: [0.0, -0.0, -1.0, f32::NEG_INFINITY, f32::NAN]);
pin!(positivus_f64, Positivus, f64, desc: "value must be positive (> 0)",
     pass: [1.0, f64::MIN_POSITIVE, f64::INFINITY],
     fail: [0.0, -0.0, -1.0, f64::NEG_INFINITY, f64::NAN]);

// ---------------------------------------------------------------------------
// NonNegativus (>= 0): same type set as Positivus
// ---------------------------------------------------------------------------
pin!(non_negativus_i8, NonNegativus, i8, desc: "value must be non-negative (>= 0)",
     pass: [0, 1, i8::MAX], fail: [-1, i8::MIN]);
pin!(non_negativus_i16, NonNegativus, i16, desc: "value must be non-negative (>= 0)",
     pass: [0, 1, i16::MAX], fail: [-1, i16::MIN]);
pin!(non_negativus_i32, NonNegativus, i32, desc: "value must be non-negative (>= 0)",
     pass: [0, 1, i32::MAX], fail: [-1, i32::MIN]);
pin!(non_negativus_i64, NonNegativus, i64, desc: "value must be non-negative (>= 0)",
     pass: [0, 1, i64::MAX], fail: [-1, i64::MIN]);
pin!(non_negativus_i128, NonNegativus, i128, desc: "value must be non-negative (>= 0)",
     pass: [0, 1, i128::MAX], fail: [-1, i128::MIN]);
pin!(non_negativus_isize, NonNegativus, isize, desc: "value must be non-negative (>= 0)",
     pass: [0, 1, isize::MAX], fail: [-1, isize::MIN]);
pin!(non_negativus_f32, NonNegativus, f32, desc: "value must be non-negative (>= 0)",
     pass: [0.0, -0.0, 1.0, f32::INFINITY],
     fail: [-1.0, f32::NEG_INFINITY, f32::NAN]);
pin!(non_negativus_f64, NonNegativus, f64, desc: "value must be non-negative (>= 0)",
     pass: [0.0, -0.0, 1.0, f64::INFINITY],
     fail: [-1.0, f64::NEG_INFINITY, f64::NAN]);

// ---------------------------------------------------------------------------
// Negativus (< 0): same type set
// ---------------------------------------------------------------------------
pin!(negativus_i8, Negativus, i8, desc: "value must be negative (< 0)",
     pass: [-1, i8::MIN], fail: [0, 1, i8::MAX]);
pin!(negativus_i16, Negativus, i16, desc: "value must be negative (< 0)",
     pass: [-1, i16::MIN], fail: [0, 1, i16::MAX]);
pin!(negativus_i32, Negativus, i32, desc: "value must be negative (< 0)",
     pass: [-1, i32::MIN], fail: [0, 1, i32::MAX]);
pin!(negativus_i64, Negativus, i64, desc: "value must be negative (< 0)",
     pass: [-1, i64::MIN], fail: [0, 1, i64::MAX]);
pin!(negativus_i128, Negativus, i128, desc: "value must be negative (< 0)",
     pass: [-1, i128::MIN], fail: [0, 1, i128::MAX]);
pin!(negativus_isize, Negativus, isize, desc: "value must be negative (< 0)",
     pass: [-1, isize::MIN], fail: [0, 1, isize::MAX]);
pin!(negativus_f32, Negativus, f32, desc: "value must be negative (< 0)",
     pass: [-1.0, f32::NEG_INFINITY],
     fail: [0.0, -0.0, 1.0, f32::INFINITY, f32::NAN]);
pin!(negativus_f64, Negativus, f64, desc: "value must be negative (< 0)",
     pass: [-1.0, f64::NEG_INFINITY],
     fail: [0.0, -0.0, 1.0, f64::INFINITY, f64::NAN]);

// ---------------------------------------------------------------------------
// NonNullus (!= 0): all 12 integer types + f32/f64. NaN counts as non-zero.
// ---------------------------------------------------------------------------
pin!(non_nullus_i8, NonNullus, i8, desc: "value must be non-zero",
     pass: [1, -1, i8::MIN, i8::MAX], fail: [0]);
pin!(non_nullus_i16, NonNullus, i16, desc: "value must be non-zero",
     pass: [1, -1, i16::MIN, i16::MAX], fail: [0]);
pin!(non_nullus_i32, NonNullus, i32, desc: "value must be non-zero",
     pass: [1, -1, i32::MIN, i32::MAX], fail: [0]);
pin!(non_nullus_i64, NonNullus, i64, desc: "value must be non-zero",
     pass: [1, -1, i64::MIN, i64::MAX], fail: [0]);
pin!(non_nullus_i128, NonNullus, i128, desc: "value must be non-zero",
     pass: [1, -1, i128::MIN, i128::MAX], fail: [0]);
pin!(non_nullus_isize, NonNullus, isize, desc: "value must be non-zero",
     pass: [1, -1, isize::MIN, isize::MAX], fail: [0]);
pin!(non_nullus_u8, NonNullus, u8, desc: "value must be non-zero",
     pass: [1, u8::MAX], fail: [0]);
pin!(non_nullus_u16, NonNullus, u16, desc: "value must be non-zero",
     pass: [1, u16::MAX], fail: [0]);
pin!(non_nullus_u32, NonNullus, u32, desc: "value must be non-zero",
     pass: [1, u32::MAX], fail: [0]);
pin!(non_nullus_u64, NonNullus, u64, desc: "value must be non-zero",
     pass: [1, u64::MAX], fail: [0]);
pin!(non_nullus_u128, NonNullus, u128, desc: "value must be non-zero",
     pass: [1, u128::MAX], fail: [0]);
pin!(non_nullus_usize, NonNullus, usize, desc: "value must be non-zero",
     pass: [1, usize::MAX], fail: [0]);
pin!(non_nullus_f32, NonNullus, f32, desc: "value must be non-zero",
     pass: [1.0, -1.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY],
     fail: [0.0, -0.0]);
pin!(non_nullus_f64, NonNullus, f64, desc: "value must be non-zero",
     pass: [1.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY],
     fail: [0.0, -0.0]);

// ---------------------------------------------------------------------------
// NonVacuus: String, Vec<T>, [T; N], &[T] — note the four distinct
// description strings.
// ---------------------------------------------------------------------------
pin!(non_vacuus_string, NonVacuus, String, desc: "string must be non-empty",
     pass: [String::from("a")], fail: [String::new()]);
pin!(non_vacuus_vec, NonVacuus, Vec<i32>, desc: "collection must be non-empty",
     pass: [vec![1]], fail: [Vec::new()]);
pin!(non_vacuus_array, NonVacuus, [i32; 3], desc: "array must be non-empty",
     pass: [[1, 2, 3]], fail: []);
pin!(non_vacuus_array_zero, NonVacuus, [i32; 0], desc: "array must be non-empty",
     pass: [], fail: [[]]);

#[test]
fn non_vacuus_slice() {
    let full: &[i32] = &[1, 2];
    let empty: &[i32] = &[];
    assert!(<NonVacuus as Praedicatum<&[i32]>>::check(&full));
    assert!(!<NonVacuus as Praedicatum<&[i32]>>::check(&empty));
    assert_eq!(
        <NonVacuus as Praedicatum<&[i32]>>::description(),
        "slice must be non-empty"
    );
}

// ---------------------------------------------------------------------------
// Verum / Falsum: blanket impls
// ---------------------------------------------------------------------------
#[test]
fn verum_falsum_blanket() {
    assert!(<Verum as Praedicatum<i32>>::check(&0));
    assert!(<Verum as Praedicatum<String>>::check(&String::new()));
    assert_eq!(<Verum as Praedicatum<i32>>::description(), "always true");

    assert!(!<Falsum as Praedicatum<i32>>::check(&0));
    assert!(!<Falsum as Praedicatum<String>>::check(&String::new()));
    assert_eq!(<Falsum as Praedicatum<i32>>::description(), "always false");
}

// ---------------------------------------------------------------------------
// IntraFines<MIN, MAX>: i8 i16 i32 i64 u8 u16 u32 usize.
// Signed + small unsigned go through an `as i64` widening; `usize` has a
// MIN<0 short-circuit `false` and a wrapping `as i64` cast above i64::MAX.
// ---------------------------------------------------------------------------
pin!(intra_fines_i8, IntraFines<{ -5 }, 5>, i8, desc: "value must be in range",
     pass: [-5, 0, 5], fail: [-6, 6, i8::MIN, i8::MAX]);
pin!(intra_fines_i16, IntraFines<{ -5 }, 5>, i16, desc: "value must be in range",
     pass: [-5, 0, 5], fail: [-6, 6]);
pin!(intra_fines_i32, IntraFines<{ -5 }, 5>, i32, desc: "value must be in range",
     pass: [-5, 0, 5], fail: [-6, 6]);
pin!(intra_fines_i64, IntraFines<{ -5 }, 5>, i64, desc: "value must be in range",
     pass: [-5, 0, 5], fail: [-6, 6]);
pin!(intra_fines_i64_full_range, IntraFines<{ i64::MIN }, { i64::MAX }>, i64,
     desc: "value must be in range",
     pass: [i64::MIN, 0, i64::MAX], fail: []);
pin!(intra_fines_u8, IntraFines<0, 10>, u8, desc: "value must be in range",
     pass: [0, 10], fail: [11, u8::MAX]);
// Negative MIN is fine for u8..u32: the value is widened to i64 first.
pin!(intra_fines_u8_negative_min, IntraFines<{ -5 }, 5>, u8, desc: "value must be in range",
     pass: [0, 5], fail: [6]);
pin!(intra_fines_u16, IntraFines<0, 10>, u16, desc: "value must be in range",
     pass: [0, 10], fail: [11]);
pin!(intra_fines_u32, IntraFines<0, 10>, u32, desc: "value must be in range",
     pass: [0, 10], fail: [11]);
// usize quirk #1: MIN < 0 rejects EVERYTHING (no widening like u8..u32).
pin!(intra_fines_usize_negative_min_rejects_all, IntraFines<{ -5 }, 5>, usize,
     desc: "value must be in range",
     pass: [], fail: [0, 3, 5]);
// usize quirk #2: `*v as i64` wraps above i64::MAX; the result is still
// (luckily) correct because MAX <= i64::MAX always.
pin!(intra_fines_usize, IntraFines<0, { i64::MAX }>, usize,
     desc: "value must be in range",
     pass: [0, 12_345], fail: [usize::MAX]);

// ---------------------------------------------------------------------------
// MaiorQuam<THRESHOLD> (>): i32 i64 u32 u64 usize.
// u64/usize: `true` short-circuit for THRESHOLD<0, threshold cast to the
// unsigned type (no i64 widening — u64::MAX compares correctly).
// ---------------------------------------------------------------------------
pin!(maior_quam_i32, MaiorQuam<10>, i32, desc: "value must be greater than threshold",
     pass: [11, i32::MAX], fail: [10, 9, -1]);
pin!(maior_quam_i32_negative, MaiorQuam<{ -10 }>, i32, desc: "value must be greater than threshold",
     pass: [-9, 0], fail: [-10, -11]);
pin!(maior_quam_i64, MaiorQuam<10>, i64, desc: "value must be greater than threshold",
     pass: [11, i64::MAX], fail: [10, 9]);
pin!(maior_quam_u32, MaiorQuam<10>, u32, desc: "value must be greater than threshold",
     pass: [11, u32::MAX], fail: [10, 0]);
pin!(maior_quam_u32_negative, MaiorQuam<{ -1 }>, u32, desc: "value must be greater than threshold",
     pass: [0, 1], fail: []);
pin!(maior_quam_u64, MaiorQuam<10>, u64, desc: "value must be greater than threshold",
     pass: [11, u64::MAX], fail: [10, 0]);
pin!(maior_quam_u64_negative, MaiorQuam<{ -1 }>, u64, desc: "value must be greater than threshold",
     pass: [0, u64::MAX], fail: []);
pin!(maior_quam_usize, MaiorQuam<10>, usize, desc: "value must be greater than threshold",
     pass: [11, usize::MAX], fail: [10, 0]);
pin!(maior_quam_usize_negative, MaiorQuam<{ -1 }>, usize, desc: "value must be greater than threshold",
     pass: [0], fail: []);

// ---------------------------------------------------------------------------
// MinorQuam<THRESHOLD> (<): i32 i64 u32 usize — NO u64 impl (pinned absence).
// u32/usize: `false` short-circuit for THRESHOLD<0.
// ---------------------------------------------------------------------------
pin!(minor_quam_i32, MinorQuam<10>, i32, desc: "value must be less than threshold",
     pass: [9, 0, -1, i32::MIN], fail: [10, 11]);
pin!(minor_quam_i64, MinorQuam<10>, i64, desc: "value must be less than threshold",
     pass: [9, i64::MIN], fail: [10, 11]);
pin!(minor_quam_i64_negative, MinorQuam<{ -10 }>, i64, desc: "value must be less than threshold",
     pass: [-11], fail: [-10, -9]);
pin!(minor_quam_u32, MinorQuam<10>, u32, desc: "value must be less than threshold",
     pass: [0, 9], fail: [10, 11, u32::MAX]);
pin!(minor_quam_u32_negative, MinorQuam<{ -1 }>, u32, desc: "value must be less than threshold",
     pass: [], fail: [0, 1]);
pin!(minor_quam_usize, MinorQuam<10>, usize, desc: "value must be less than threshold",
     pass: [0, 9], fail: [10, usize::MAX]);
pin!(minor_quam_usize_negative, MinorQuam<{ -1 }>, usize, desc: "value must be less than threshold",
     pass: [], fail: [0]);

// ---------------------------------------------------------------------------
// Magnitudo* on String measures BYTES (not chars): "héllo" is 6 bytes.
// ---------------------------------------------------------------------------
pin!(magnitudo_exacta_string, MagnitudoExacta<5>, String,
     desc: "string must have exact length",
     pass: [String::from("hello")],
     fail: [String::from("hi"), String::from("héllo"), String::new()]);
pin!(magnitudo_exacta_string_bytes, MagnitudoExacta<6>, String,
     desc: "string must have exact length",
     pass: [String::from("héllo")], fail: [String::from("hello")]);
pin!(magnitudo_exacta_vec, MagnitudoExacta<3>, Vec<i32>,
     desc: "collection must have exact size",
     pass: [vec![1, 2, 3]], fail: [vec![1, 2], Vec::new()]);
pin!(magnitudo_minima_string, MagnitudoMinima<3>, String,
     desc: "string must have minimum length",
     pass: [String::from("abc"), String::from("abcd")],
     fail: [String::from("ab"), String::new()]);
pin!(magnitudo_minima_vec, MagnitudoMinima<2>, Vec<i32>,
     desc: "collection must have minimum size",
     pass: [vec![1, 2], vec![1, 2, 3]], fail: [vec![1], Vec::new()]);
pin!(magnitudo_maxima_string, MagnitudoMaxima<5>, String,
     desc: "string must have maximum length",
     pass: [String::from("hello"), String::new()],
     fail: [String::from("hello!")]);
pin!(magnitudo_maxima_vec, MagnitudoMaxima<2>, Vec<i32>,
     desc: "collection must have maximum size",
     pass: [Vec::new(), vec![1, 2]], fail: [vec![1, 2, 3]]);

// ---------------------------------------------------------------------------
// Par (even) / Impar (odd): i8 i16 i32 i64 u8 u16 u32 u64 usize
// (no i128/isize — pinned absence). Every unsigned MAX is odd.
// ---------------------------------------------------------------------------
pin!(par_i8, Par, i8, desc: "value must be even",
     pass: [0, 2, -2, i8::MIN], fail: [1, -1, i8::MAX]);
pin!(par_i16, Par, i16, desc: "value must be even",
     pass: [0, 2, -2, i16::MIN], fail: [1, -1, i16::MAX]);
pin!(par_i32, Par, i32, desc: "value must be even",
     pass: [0, 2, -2, i32::MIN], fail: [1, -1, i32::MAX]);
pin!(par_i64, Par, i64, desc: "value must be even",
     pass: [0, 2, -2, i64::MIN], fail: [1, -1, i64::MAX]);
pin!(par_u8, Par, u8, desc: "value must be even",
     pass: [0, 2, 254], fail: [1, u8::MAX]);
pin!(par_u16, Par, u16, desc: "value must be even",
     pass: [0, 2], fail: [1, u16::MAX]);
pin!(par_u32, Par, u32, desc: "value must be even",
     pass: [0, 2], fail: [1, u32::MAX]);
pin!(par_u64, Par, u64, desc: "value must be even",
     pass: [0, 2], fail: [1, u64::MAX]);
pin!(par_usize, Par, usize, desc: "value must be even",
     pass: [0, 2], fail: [1, usize::MAX]);

pin!(impar_i8, Impar, i8, desc: "value must be odd",
     pass: [1, -1, i8::MAX], fail: [0, 2, -2, i8::MIN]);
pin!(impar_i16, Impar, i16, desc: "value must be odd",
     pass: [1, -1, i16::MAX], fail: [0, 2, -2, i16::MIN]);
pin!(impar_i32, Impar, i32, desc: "value must be odd",
     pass: [1, -1, i32::MAX], fail: [0, 2, -2, i32::MIN]);
pin!(impar_i64, Impar, i64, desc: "value must be odd",
     pass: [1, -1, i64::MAX], fail: [0, 2, -2, i64::MIN]);
pin!(impar_u8, Impar, u8, desc: "value must be odd",
     pass: [1, u8::MAX], fail: [0, 2]);
pin!(impar_u16, Impar, u16, desc: "value must be odd",
     pass: [1, u16::MAX], fail: [0, 2]);
pin!(impar_u32, Impar, u32, desc: "value must be odd",
     pass: [1, u32::MAX], fail: [0, 2]);
pin!(impar_u64, Impar, u64, desc: "value must be odd",
     pass: [1, u64::MAX], fail: [0, 2]);
pin!(impar_usize, Impar, usize, desc: "value must be odd",
     pass: [1, usize::MAX], fail: [0, 2]);
