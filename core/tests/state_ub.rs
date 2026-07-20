#![cfg(feature = "nexus")]

use ordofp_core::nexus::effects::state::StatefulComputation;

#[test]
#[should_panic(expected = "StatefulComputation::Get encountered in run_unit")]
fn test_get_in_run_unit() {
    // Construct Get with A=() (which is invalid as Get implies A=S)
    // Here S=i32, so A!=S.
    let comp: StatefulComputation<i32, ()> = StatefulComputation::Get;

    // This calls run_unit which hits unreachable_unchecked currently
    comp.run_unit(0);
}

#[test]
#[should_panic(expected = "StatefulComputation::Put encountered in run_get")]
fn test_put_in_run_get() {
    // Construct Put with A=i32 (which is invalid as Put implies A=())
    // Here S=i32, so A=S.
    let comp: StatefulComputation<i32, i32> = StatefulComputation::Put(42);

    // This calls run_get which hits unreachable_unchecked currently
    comp.run_get(0);
}

#[test]
#[should_panic(expected = "StatefulComputation::Modify encountered in run_get")]
fn test_modify_in_run_get() {
    // Construct Modify with A=i32 (which is invalid as Modify implies A=())
    let comp: StatefulComputation<i32, i32> = StatefulComputation::Modify(Box::new(|x| x + 1));

    // This calls run_get which hits unreachable_unchecked currently
    comp.run_get(0);
}
