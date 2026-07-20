//! Probatum-feature bench suite (one binary; modules were formerly standalone
//! criterion benches — grouped to cut the gate's link count).
mod grade_validation;
mod validated;

criterion::criterion_main!(grade_validation::grade_validation, validated::benches);
