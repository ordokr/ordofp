use ordofp_macros::path_type;

fn main() {
    // Exercises the `path_type!` entry point's error branch (same helper as
    // `path!`, but a distinct proc macro that previously had zero coverage).
    let _: path_type!(module::field) = unimplemented!();
}
