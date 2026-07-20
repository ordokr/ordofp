#![cfg(feature = "nexus")]

use ordofp_core::nexus::effects::writer::WriterComputation;

#[test]
#[should_panic(expected = "WriterComputation::Tell invariant violated")]
fn test_writer_ub_protection_run() {
    let comp = WriterComputation::<String, String>::Tell("log".to_string());
    let _ = comp.run();
}

#[test]
#[should_panic(expected = "WriterComputation::Tell invariant violated")]
fn test_writer_ub_protection_map() {
    let comp = WriterComputation::<String, String>::Tell("log".to_string());
    // Mapping should also trigger the check because map executes f(unit) immediately for Tell
    let _ = comp.map(|s| s.len());
}
