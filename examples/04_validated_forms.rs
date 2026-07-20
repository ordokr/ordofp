use ordofp::Probatum;
use ordofp::prelude::*;

#[derive(Debug, PartialEq)]
struct Person {
    name: String,
    age: u32,
}

#[derive(Debug)]
enum ValidationError {
    NameTooShort,
    Underage,
}

fn validate_name(name: &str) -> Result<String, ValidationError> {
    if name.len() < 2 {
        Err(ValidationError::NameTooShort)
    } else {
        Ok(name.to_string())
    }
}

fn validate_age(age: u32) -> Result<u32, ValidationError> {
    if age < 18 {
        Err(ValidationError::Underage)
    } else {
        Ok(age)
    }
}

fn main() {
    println!("--- Example 04: Probatum (Accumulating Errors) ---");

    // Case 1: Success
    let v_name = validate_name("Bob").into_probatum();
    let v_age = validate_age(20).into_probatum();

    let result = Probatum::lift2(|name, age| Person { name, age }, v_name, v_age);

    println!("Success Case: {result:?}");
    assert!(result.is_valid());

    // Case 2: Multiple Errors
    let v_name_fail = validate_name("A").into_probatum(); // Too short
    let v_age_fail = validate_age(15).into_probatum(); // Underage

    let fail_result: Probatum<ValidationError, Person> =
        Probatum::lift2(|name, age| Person { name, age }, v_name_fail, v_age_fail);

    println!("Failure Case: {fail_result:?}");

    if let Probatum::Invalid(errors) = fail_result {
        println!("Errors found: {}", errors.len());
        assert_eq!(errors.len(), 2); // We got BOTH errors, not just the first one
    } else {
        panic!("Should have failed");
    }
}
