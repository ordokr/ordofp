use ordofp::Universalis;

#[derive(Universalis, Debug, PartialEq)]
struct ApiUser {
    first_name: String,
    last_name: String,
    age: u32,
}

#[derive(Universalis, Debug, PartialEq)]
struct DbUser {
    name: String,    // Different name, but first position
    surname: String, // Different name, but second position
    age: u32,
}

fn main() {
    println!("--- Example 02: Universalis Conversion ---");

    let db_user = DbUser {
        name: "John".to_string(),
        surname: "Doe".to_string(),
        age: 30,
    };

    println!("Original DbUser: {db_user:?}");

    // Convert DbUser -> ApiUser
    // Since both have the same "shape" (String, String, u32), we can convert
    let api_user: ApiUser = ordofp::convert_from(db_user);

    println!("Converted ApiUser: {api_user:?}");

    assert_eq!(api_user.first_name, "John");
    assert_eq!(api_user.last_name, "Doe");
    assert_eq!(api_user.age, 30);

    // We can also go via HList manually
    let hlist = ordofp::into_universalis(api_user);
    println!("As HList: {hlist:?}");
}
