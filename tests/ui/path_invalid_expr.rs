use ordofp_macros::path;

fn main() {
    // Not a field-access chain at all: must produce the pointed
    // "Invalid path expression" error, not a proc-macro panic.
    let _ = path!(1 + 2);
}
