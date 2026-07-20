#![cfg(feature = "Probatum")]

use ordofp::Universalis;
use ordofp::prelude::*;

#[derive(Universalis, Debug, PartialEq, Eq)]
pub struct Person<'a> {
    pub first_name: &'a str,
    pub last_name: &'a str,
    pub age: usize,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Nope {
    NameNope,
    AgeNope,
}

fn get_name(ok: bool) -> Result<&'static str, Nope> {
    if ok { Ok("James") } else { Err(Nope::NameNope) }
}

fn get_age(ok: bool) -> Result<usize, Nope> {
    if ok { Ok(32) } else { Err(Nope::AgeNope) }
}

#[test]
fn probatum_success() {
    let name = get_name(true).into_probatum();
    let age = get_age(true).into_probatum();

    let person = name.map2(age, |first, age| Person {
        first_name: first,
        last_name: first,
        age,
    });

    assert!(person.is_valid());
    let built = person
        .into_result()
        .expect("Probatum person should be Ok after successful map2");
    assert_eq!(built.first_name, "James");
    assert_eq!(built.age, 32);
}

#[test]
fn probatum_accumulates_errors() {
    let name = get_name(false).into_probatum();
    let age = get_age(false).into_probatum();

    let result = name.map2(age, |_, _| unreachable!());
    assert!(result.is_invalid());
    let errs = result.into_result().unwrap_err();
    assert_eq!(errs.len(), 2);
}

#[test]
fn probatum_mixed_error() {
    let name = get_name(false).into_probatum();
    let age = get_age(true).into_probatum();

    let result = name.map2(age, |_, _| unreachable!());
    assert!(result.is_invalid());
    let errs = result.into_result().unwrap_err();
    assert_eq!(errs.as_slice(), &[Nope::NameNope]);
}
