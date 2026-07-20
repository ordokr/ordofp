#![cfg(feature = "par")]

use ordofp::par::{ParFlumen, backend::CpuScalar};

#[test]
fn par_flumen_map_collect_matches_scalar() {
    let data: Vec<i32> = (0..10_000).collect();

    let par = ParFlumen::from_slice(&data).map(|x| x + 1).map(|x| x * 2);

    let got = par.collect_vec(&CpuScalar);

    let expected: Vec<i32> = data.into_iter().map(|x| (x + 1) * 2).collect();

    assert_eq!(got, expected);
}

#[test]
fn par_flumen_scan_matches_scalar() {
    let data: Vec<i32> = (1..=1000).collect();

    let par = ParFlumen::from_slice(&data).scan(0i32, |acc, x| acc + x);

    let got = par.collect_vec(&CpuScalar);

    let mut expected = Vec::with_capacity(data.len());
    let mut acc = 0i32;
    for x in data {
        acc += x;
        expected.push(acc);
    }

    assert_eq!(got, expected);
}

#[test]
fn par_flumen_reduce_matches_scalar() {
    let data: Vec<i32> = (0..10_000).collect();

    let par = ParFlumen::from_slice(&data).map(|x| x + 1);

    let got = par.reduce(&CpuScalar, |a, b| a + b).unwrap_or(0);

    let expected: i32 = data.into_iter().map(|x| x + 1).sum();

    assert_eq!(got, expected);
}

#[cfg(feature = "rayon")]
#[test]
fn par_flumen_rayon_backend_equivalence() {
    use ordofp::par::backend::CpuRayon;

    let data: Vec<i32> = (0..50_000).collect();

    let par = ParFlumen::from_slice(&data).map(|x| x + 1).map(|x| x * 2);

    let scalar = par.collect_vec(&CpuScalar);
    let rayon = par.collect_vec(&CpuRayon { min_len: 128 });

    assert_eq!(scalar, rayon);
}
