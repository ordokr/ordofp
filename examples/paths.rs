use ordofp::NominataUniversalis;
use ordofp_core::path::ViaTraversor;
use ordofp_macros::{path, path_type};

#[derive(NominataUniversalis)]
struct Dog<'a> {
    name: &'a str,
    dimensions: Dimensions,
}

#[derive(NominataUniversalis)]
struct Cat<'a> {
    name: &'a str,
    dimensions: Dimensions,
}

#[derive(NominataUniversalis)]
struct Dimensions {
    height: usize,
    width: usize,
    unit: SizeUnit,
}

#[derive(Debug)]
enum SizeUnit {
    Cm,
    Inch,
}

fn main() {
    let dog = Dog {
        name: "Joe",
        dimensions: Dimensions {
            height: 10,
            width: 5,
            unit: SizeUnit::Inch,
        },
    };

    let cat = Cat {
        name: "Schmoe",
        dimensions: Dimensions {
            height: 7,
            width: 3,
            unit: SizeUnit::Cm,
        },
    };

    // Prints height as long as `A` has the right "shape" (e.g.
    // has `dimensions.height: usize` and `dimension.unit: SizeUnit)
    fn print_height<'a, A, HeightIdx, UnitIdx>(obj: &'a A)
    where
        &'a A: ViaTraversor<path_type!(dimensions.height), HeightIdx, TargetValue = &'a usize>
            + ViaTraversor<path_type!(dimensions.unit), UnitIdx, TargetValue = &'a SizeUnit>,
    {
        println!(
            "Height [{} {:?}]",
            path!(dimensions.height).get(obj),
            path!(dimensions.unit).get(obj)
        );
    }

    print_height(&dog);
    print_height(&cat);
}
