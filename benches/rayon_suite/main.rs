//! Rayon-feature bench suite (one binary; modules were formerly standalone
//! criterion benches — grouped to cut the gate's link count).
mod cpu_rayon_non_indexed;
mod par_collect_rayon;

criterion::criterion_main!(cpu_rayon_non_indexed::benches, par_collect_rayon::benches);
