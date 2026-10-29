//! Validate a nested replacement using the English gateway.

use ordofp::vernacular::{Lens, Validated};

#[derive(Clone, Debug, PartialEq)]
struct Profile {
    name: String,
    active: bool,
}

#[derive(Debug, PartialEq)]
struct Account {
    profile: Profile,
    id: u32,
}

fn main() {
    let profile = Lens::new(
        |a: &Account| a.profile.clone(),
        |a: &Account, profile| Account { profile, id: a.id },
    );
    let name = Lens::new(
        |p: &Profile| p.name.clone(),
        |p: &Profile, name| Profile {
            name,
            active: p.active,
        },
    );
    let lens = profile.compose(&name);
    let source = Account {
        profile: Profile {
            name: "Alice".into(),
            active: true,
        },
        id: 7,
    };
    let validate = |name: String| {
        if name.trim().is_empty() {
            Validated::invalid_many(["name is required"])
        } else {
            Validated::Valid(name)
        }
    };
    let updated = lens.modify_validated(&source, |_| validate("Bob".into()));
    assert_eq!(
        updated,
        Validated::Valid(Account {
            profile: Profile {
                name: "Bob".into(),
                active: true
            },
            id: 7,
        })
    );
    assert_eq!(
        lens.modify_validated(&source, |_| validate(String::new())),
        Validated::invalid_many(["name is required"])
    );
    assert_eq!(source.profile.name, "Alice");
}
