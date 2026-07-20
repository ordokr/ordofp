use ordofp::hlist;

fn main() {
    println!("--- Example 01: HList Basics ---");

    // 1. Create an HList
    let h = hlist![1, "hello", true, 42.0];
    println!("Created HList: {h:?}");

    // 2. Extract head and tail
    let (head, tail) = h.pop();
    println!("Head: {head}, Tail: {tail:?}");
    assert_eq!(head, 1);

    // 3. Access by type (Type-based indexing)
    // We can pluck a value out if its type is unique in the list
    // Note: the original list is consumed and we get (value, remaining_list)
    let h2 = hlist![1, "hello", true, 42.0];
    let (s, _rest): (f64, _) = h2.pluck();
    println!("Plucked f64: {s}");
    assert_eq!(s, 42.0);

    // 4. Appending/Prepending
    let h3 = hlist![true];
    let h4 = h3.prepend("start");
    println!("Prepended: {h4:?}");
}
