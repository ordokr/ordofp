use ordofp::{HList, Universalis};

#[derive(Universalis, Debug, PartialEq)]
struct User {
    name: String,
    age: u32,
    is_admin: bool,
}

#[derive(Universalis, Debug, PartialEq)]
struct UserSummary {
    name: String,
    is_admin: bool,
}

fn main() {
    println!("--- Example 03: Sculpting (Partial Extraction) ---");

    let user = User {
        name: "Alice".to_string(),
        age: 28,
        is_admin: true,
    };

    println!("Full User: {user:?}");

    // Transform user into HList
    let user_hlist = ordofp::into_universalis(user);

    // Sculpt out the fields needed for UserSummary
    // The type annotation helps rustc figure out what we want
    let (summary_hlist, remainder): (HList![String, bool], _) = user_hlist.sculpt();

    let summary: UserSummary = ordofp::from_universalis(summary_hlist);

    println!("User Summary: {summary:?}");
    println!("Remainder: {remainder:?}");

    assert_eq!(summary.name, "Alice");
    assert!(summary.is_admin);
}
