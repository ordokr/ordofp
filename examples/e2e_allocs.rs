//! Allocation-count driver for the e2e workload.
//!
//! Runs the exact `e2e_workload` pipeline under a counting global allocator
//! (wrapping `System`) and reports allocation/byte counts for setup, the first
//! rep (buffer warm-up), and the steady-state per-rep average. NOT a wall-time
//! instrument — the counters add per-call overhead; use `e2e_workload` for time.
//!
//! Build:
//!   cargo build --release --example `e2e_allocs` --features Probatum-smallvec

#[path = "e2e/workload_core.rs"]
mod workload;

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};

static ALLOCS: AtomicU64 = AtomicU64::new(0);
static DEALLOCS: AtomicU64 = AtomicU64::new(0);
static REALLOCS: AtomicU64 = AtomicU64::new(0);
static BYTES: AtomicU64 = AtomicU64::new(0);
static CURRENT: AtomicU64 = AtomicU64::new(0);
static PEAK: AtomicU64 = AtomicU64::new(0);

fn record_alloc(size: usize) {
    ALLOCS.fetch_add(1, Relaxed);
    BYTES.fetch_add(size as u64, Relaxed);
    let now = CURRENT.fetch_add(size as u64, Relaxed) + size as u64;
    PEAK.fetch_max(now, Relaxed);
}

struct Counting;

// SAFETY: every method forwards verbatim to `System` with the caller's layout;
// the counters are side effects only and never touch the returned memory.
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: caller upholds GlobalAlloc's contract; forwarded unchanged.
        let p = unsafe { System.alloc(layout) };
        if !p.is_null() {
            record_alloc(layout.size());
        }
        p
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        // SAFETY: caller upholds GlobalAlloc's contract; forwarded unchanged.
        let p = unsafe { System.alloc_zeroed(layout) };
        if !p.is_null() {
            record_alloc(layout.size());
        }
        p
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        DEALLOCS.fetch_add(1, Relaxed);
        CURRENT.fetch_sub(layout.size() as u64, Relaxed);
        // SAFETY: ptr/layout come from a matching alloc on `System`.
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // SAFETY: ptr/layout come from a matching alloc on `System`; new_size
        // obeys the caller's contract.
        let p = unsafe { System.realloc(ptr, layout, new_size) };
        if !p.is_null() {
            REALLOCS.fetch_add(1, Relaxed);
            if new_size >= layout.size() {
                let grow = (new_size - layout.size()) as u64;
                BYTES.fetch_add(grow, Relaxed);
                let now = CURRENT.fetch_add(grow, Relaxed) + grow;
                PEAK.fetch_max(now, Relaxed);
            } else {
                CURRENT.fetch_sub((layout.size() - new_size) as u64, Relaxed);
            }
        }
        p
    }
}

#[global_allocator]
static GLOBAL: Counting = Counting;

#[derive(Clone, Copy)]
struct Snap {
    allocs: u64,
    deallocs: u64,
    reallocs: u64,
    bytes: u64,
}

fn snap() -> Snap {
    Snap {
        allocs: ALLOCS.load(Relaxed),
        deallocs: DEALLOCS.load(Relaxed),
        reallocs: REALLOCS.load(Relaxed),
        bytes: BYTES.load(Relaxed),
    }
}

fn report(label: &str, from: Snap, to: Snap, divisor: u64) {
    let d = divisor.max(1);
    println!(
        "{label}: allocs={} deallocs={} reallocs={} bytes={}",
        (to.allocs - from.allocs) / d,
        (to.deallocs - from.deallocs) / d,
        (to.reallocs - from.reallocs) / d,
        (to.bytes - from.bytes) / d,
    );
}

fn main() {
    let (cfg, mode) = workload::parse_args();
    if mode == workload::Mode::Startup {
        println!("startup_ok allocs={}", ALLOCS.load(Relaxed));
        return;
    }

    let s0 = snap();
    let data = workload::generate(&cfg);
    let mut st = workload::State::new(&cfg);
    let s_setup = snap();

    let first = workload::run_rep(&data, &mut st);
    let s_rep1 = snap();

    let mut checksum = first.checksum;
    for _ in 1..cfg.reps {
        let out = workload::run_rep(&data, &mut st);
        assert_eq!(checksum, out.checksum, "nondeterministic rep checksum");
        checksum = out.checksum;
    }
    let s_end = snap();

    println!(
        "mode={mode:?} courses={} students={} assignments={} reps={} error_pct={} seed={:#x}",
        cfg.courses, cfg.students, cfg.assignments, cfg.reps, cfg.error_pct, cfg.seed
    );
    report("setup_total", s0, s_setup, 1);
    report("rep1_total (buffer warm-up)", s_setup, s_rep1, 1);
    if cfg.reps > 1 {
        report("steady_per_rep", s_rep1, s_end, (cfg.reps - 1) as u64);
    }
    // Rep-1 phase shares under the counting allocator (sanity check that the
    // counter overhead lands where the allocations are, not a time verdict).
    let rep_ns: u64 = first.phase_ns.iter().sum();
    for (name, ns) in workload::PHASES.iter().zip(first.phase_ns) {
        println!(
            "phase={name} rep1_ms={:.2} share={:.1}%",
            ns as f64 / 1e6,
            ns as f64 * 100.0 / rep_ns as f64
        );
    }
    println!("peak_live_bytes={}", PEAK.load(Relaxed));
    println!(
        "valid_grades={} errored_students={} errors={}",
        first.valid_grades, first.errored_students, first.errors
    );
    println!("checksum={checksum:#018x}");
}
