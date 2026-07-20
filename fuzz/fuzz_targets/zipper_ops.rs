#![no_main]
use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use ordofp_core::zipper::Zipper;

#[derive(Arbitrary, Debug)]
enum Op {
    FocusNext,
    FocusPrev,
    FocusFirst,
    FocusLast,
    InsertLeft(u8),
    InsertRight(u8),
    Delete,
    Update(u8),
    SwapLeft,
    SwapRight,
}

fuzz_target!(|data: (Vec<u8>, Vec<Op>)| {
    let (initial_vec, ops) = data;
    if initial_vec.is_empty() {
        return;
    }

    let mut zipper = Zipper::from_vec(initial_vec.clone()).unwrap();
    let mut model_vec = initial_vec;
    let mut focus_idx = 0;

    for op in ops {
        match op {
            Op::FocusNext => {
                if let Some(z) = zipper.clone().focus_next() {
                    zipper = z;
                    focus_idx += 1;
                } else {
                    // Model check: confirm we are at end
                    assert_eq!(focus_idx, model_vec.len() - 1);
                }
            }
            Op::FocusPrev => {
                if let Some(z) = zipper.clone().focus_prev() {
                    zipper = z;
                    focus_idx -= 1;
                } else {
                    assert_eq!(focus_idx, 0);
                }
            }
            Op::FocusFirst => {
                zipper = zipper.focus_first();
                focus_idx = 0;
            }
            Op::FocusLast => {
                zipper = zipper.focus_last();
                focus_idx = model_vec.len() - 1;
            }
            Op::InsertLeft(val) => {
                zipper = zipper.insert_left(val);
                model_vec.insert(focus_idx, val);
                focus_idx += 1; // Focus stays on original element, which shifted right
            }
            Op::InsertRight(val) => {
                zipper = zipper.insert_right(val);
                model_vec.insert(focus_idx + 1, val);
            }
            Op::Delete => {
                if let Some((val, z)) = zipper.clone().delete() {
                    assert_eq!(val, model_vec[focus_idx]);
                    model_vec.remove(focus_idx);
                    zipper = z;
                    // Focus moves right unless it was last element, then left
                    if focus_idx == model_vec.len() {
                        focus_idx -= 1;
                    }
                } else {
                    assert_eq!(model_vec.len(), 1);
                }
            }
            Op::Update(val) => {
                zipper = zipper.update(|x| x.wrapping_add(val));
                model_vec[focus_idx] = model_vec[focus_idx].wrapping_add(val);
            }
            Op::SwapLeft => {
                if let Some(z) = zipper.clone().swap_left() {
                    model_vec.swap(focus_idx, focus_idx - 1);
                    zipper = z;
                    // Index stays same
                } else {
                    assert_eq!(focus_idx, 0);
                }
            }
            Op::SwapRight => {
                if let Some(z) = zipper.clone().swap_right() {
                    model_vec.swap(focus_idx, focus_idx + 1);
                    zipper = z;
                    // Index stays same
                } else {
                    assert_eq!(focus_idx, model_vec.len() - 1);
                }
            }
        }

        // Invariant checks
        assert_eq!(zipper.clone().to_vec(), model_vec);
        assert_eq!(zipper.len(), model_vec.len());
        if !model_vec.is_empty() {
            let current_focus = zipper.focus();
            assert_eq!(current_focus, &model_vec[focus_idx]);
        }
    }
});
