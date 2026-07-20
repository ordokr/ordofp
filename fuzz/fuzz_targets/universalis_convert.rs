#![no_main]
use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use ordofp::{Universalis, from_universalis, into_universalis};

#[derive(Universalis, Debug, PartialEq, Arbitrary, Clone)]
struct FuzzStruct {
    a: i32,
    b: u8,
    c: bool,
    d: String,
}

fuzz_target!(|data: FuzzStruct| {
    // Round trip test
    let h = into_universalis(data.clone());
    let s2: FuzzStruct = from_universalis(h);
    assert_eq!(data, s2);
});
