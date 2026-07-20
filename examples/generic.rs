use ordofp::{Universalis, coniunctio_pat, hlist};

#[derive(Universalis, Debug, PartialEq)]
struct Person<'a> {
    first_name: &'a str,
    last_name: &'a str,
    age: usize,
}

#[derive(Universalis, Debug, PartialEq)]
struct Person2<'a> {
    name_first: &'a str,
    name_last: &'a str,
    age_of_person: usize,
}

fn main() {
    let repr = hlist!("Joe", "Blow", 30);
    let person: Person = ordofp::from_universalis(repr);
    assert_eq!(
        person,
        Person {
            first_name: "Joe",
            last_name: "Blow",
            age: 30,
        }
    );
    println!("{}", person.first_name);

    let older_person = ordofp::map_repr(person, |repr| {
        let coniunctio_pat![first, last, age] = repr;
        hlist![first, last, age * 2]
    });
    assert_eq!(older_person.age, 60);

    let oldest_person = ordofp::map_inter(older_person, |p| Person2 {
        age_of_person: 90,
        ..p
    });
    assert_eq!(oldest_person.age, 90);

    // mapping over Universalis representation
    let peep = Person {
        first_name: "bo",
        last_name: "peep",
        age: 30,
    };
    let universalis = ordofp::into_universalis(peep);
    // mapping each one
    let _ = universalis.map(hlist![
        |first_name| println!("First name: {first_name}"),
        |last_name| println!("Last name: {last_name}"),
        |age| println!("age: {age}"),
    ]);
}
