#![no_main]
use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use ordofp_core::nonempty::NonEmpty;

#[derive(Arbitrary, Debug)]
enum Op {
    Push(u8),
    Prepend(u8),
    MapAdd(u8),
    Reverse,
    Concat(Vec<u8>),
    FilterMod2, // Keep evens
}

fuzz_target!(|data: (Vec<u8>, Vec<Op>)| {
    let (initial_vec, ops) = data;
    if initial_vec.is_empty() {
        return;
    }

    let mut nel = NonEmpty::from_vec(initial_vec.clone())
        .expect("initial_vec is non-empty — early-return guard above ensures this");
    let mut model_vec = initial_vec;

    for op in ops {
        match op {
            Op::Push(val) => {
                nel = nel.push(val);
                model_vec.push(val);
            }
            Op::Prepend(val) => {
                nel = nel.prepend(val);
                model_vec.insert(0, val);
            }
            Op::MapAdd(val) => {
                nel = nel.map(|x| x.wrapping_add(val));
                for x in &mut model_vec {
                    *x = x.wrapping_add(val);
                }
            }
            Op::Reverse => {
                nel = nel.reverse();
                model_vec.reverse();
            }
            Op::Concat(vec) => {
                if let Some(other) = NonEmpty::from_vec(vec.clone()) {
                    nel = nel.concat(other);
                    model_vec.extend(vec);
                }
            }
            Op::FilterMod2 => {
                let filtered_opt = nel.clone().filter(|x| x % 2 == 0);
                model_vec.retain(|x| x % 2 == 0);

                if model_vec.is_empty() {
                    assert!(filtered_opt.is_none());
                    // Since model is empty, we can't continue operations on NonEmpty
                    break;
                } else {
                    if let Some(filtered) = filtered_opt {
                        nel = filtered;
                    } else {
                        panic!("Model not empty but NonEmpty filter returned None");
                    }
                }
            }
        }

        // Invariants
        assert_eq!(nel.clone().to_vec(), model_vec);
        assert_eq!(nel.len(), model_vec.len());
        assert_eq!(nel.head(), &model_vec[0]);
        assert_eq!(nel.last(), model_vec.last().expect("model_vec is non-empty: FilterMod2 breaks out of the loop when it drains the vec, so all other paths leave at least one element"));
    }
});
