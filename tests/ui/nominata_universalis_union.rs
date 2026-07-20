use ordofp::NominataUniversalis;

#[derive(NominataUniversalis)]
union U {
    a: u32,
    b: f32,
}

fn main() {}
