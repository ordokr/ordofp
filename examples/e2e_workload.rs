//! End-to-end perf driver: a grade-processing batch modeled on a production
//! LMS application. This is the hyperfine/samply/PGO verdict binary.
//!
//! Build (consumer-matching features):
//!   cargo build --release --example `e2e_workload` --features Probatum-smallvec
//!
//! Modes:
//!   --mode steady       realistic mix, 3% invalid input (default)
//!   --mode error-heavy  50% invalid input (error-path stress)
//!   --mode startup      parse args + exit (process start-up floor)
//!
//! Prints a per-phase time table and a checksum that must be identical across
//! reps (in-run determinism assert) and across optimization commits
//! (behavior-preservation evidence).

#[path = "e2e/workload_core.rs"]
mod workload;

use std::time::Instant;

// Allocator lever. Steady-state runs ~55k
// allocs/rep; ntdll heap machinery was 14.9%/21.4% self in the baseline
// profiles. Build with `--features alloc-mimalloc` to swap the system heap.
#[cfg(feature = "alloc-mimalloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() {
    let (cfg, mode) = workload::parse_args();
    if mode == workload::Mode::Startup {
        println!("startup_ok");
        return;
    }

    let t0 = Instant::now();
    let data = workload::generate(&cfg);
    let mut st = workload::State::new(&cfg);
    let setup = t0.elapsed();

    let mut phase_total = [0u64; 5];
    let mut checksum: Option<u64> = None;
    let mut last = None;
    let t1 = Instant::now();
    for _ in 0..cfg.reps {
        let out = workload::run_rep(&data, &mut st);
        for (total, ns) in phase_total.iter_mut().zip(out.phase_ns) {
            *total += ns;
        }
        match checksum {
            None => checksum = Some(out.checksum),
            Some(c) => assert_eq!(c, out.checksum, "nondeterministic rep checksum"),
        }
        last = Some(out);
    }
    let work = t1.elapsed();

    let last = last.expect("reps must be >= 1");
    let grades = cfg.courses * cfg.students * cfg.assignments;
    println!(
        "mode={mode:?} courses={} students={} assignments={} reps={} error_pct={} seed={:#x}",
        cfg.courses, cfg.students, cfg.assignments, cfg.reps, cfg.error_pct, cfg.seed
    );
    println!("setup_ms={:.2}", setup.as_secs_f64() * 1e3);
    let work_ns: u64 = phase_total.iter().sum();
    for (name, ns) in workload::PHASES.iter().zip(phase_total) {
        println!(
            "phase={name} total_ms={:.2} share={:.1}% per_rep_us={:.1}",
            ns as f64 / 1e6,
            ns as f64 * 100.0 / work_ns as f64,
            ns as f64 / 1e3 / cfg.reps as f64
        );
    }
    println!(
        "work_ms={:.2} (timed_phases_ms={:.2})",
        work.as_secs_f64() * 1e3,
        work_ns as f64 / 1e6
    );
    println!(
        "grades_per_rep={grades} valid_grades={} errored_students={} errors={}",
        last.valid_grades, last.errored_students, last.errors
    );
    println!("checksum={:#018x}", checksum.expect("reps must be >= 1"));
}
