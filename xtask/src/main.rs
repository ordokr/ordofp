//! Cross-platform verification driver for OrdoFP.
//!
//! This repository has no hosted CI by design: every verification dimension
//! is machine-owned *locally* by this crate, so any shell on any dev box runs
//! identical steps.
//!
//! ```text
//! cargo run -p xtask -- gate        # the canonical five-step gate
//! cargo run -p xtask -- stable      # stable-toolchain build + fallback-path tests
//! cargo run -p xtask -- deny        # advisories/licenses/bans (deny.toml)
//! cargo run -p xtask -- wasm        # core builds for wasm32-unknown-unknown
//! cargo run -p xtask -- all         # gate + stable + deny + wasm (pre-push)
//! cargo run -p xtask -- miri        # UB check on the unsafe-heavy modules
//! cargo run -p xtask -- fuzz-smoke  # 60s coverage-guided smoke per target
//! cargo run -p xtask -- perf-guard  # deterministic checksum/alloc regression guard
//! cargo run -p xtask -- pgo [feats] # PGO build of the e2e verdict binary
//! cargo run -p xtask -- semver      # API-stability diff vs the crates.io baseline
//! cargo run -p xtask -- deep        # miri + fuzz-smoke (weekly cadence)
//! ```
//!
//! With the `xtask` alias from `.cargo/config.toml.example`, `cargo xtask
//! <task>` works too.

use std::process::{Command, exit};

fn main() {
    let task = std::env::args().nth(1).unwrap_or_default();
    match task.as_str() {
        "gate" => gate(),
        "stable" => stable(),
        "deny" => deny(),
        "wasm" => wasm(),
        "miri" => miri(),
        "fuzz-smoke" => fuzz_smoke(),
        "perf-guard" => perf_guard(),
        "pgo" => pgo(),
        "semver" => semver(),
        "all" => {
            gate();
            stable();
            deny();
            wasm();
        }
        "deep" => {
            miri();
            fuzz_smoke();
        }
        _ => {
            eprintln!(
                "usage: cargo run -p xtask -- <gate|stable|deny|wasm|miri|fuzz-smoke|perf-guard|pgo|semver|all|deep>"
            );
            exit(2);
        }
    }
    println!("xtask {task}: GREEN");
}

/// The canonical five verification steps. Every commit is expected to pass
/// all five.
fn gate() {
    run("fmt (check)", &["fmt", "--all", "--", "--check"], None);
    run(
        "clippy -D warnings (all targets, all features)",
        &[
            "clippy",
            "--all-targets",
            "--all-features",
            "--",
            "-D",
            "warnings",
        ],
        None,
    );
    run(
        "tests (workspace, all features)",
        &["test", "--workspace", "--all-features"],
        None,
    );
    run(
        "docs (no warnings expected)",
        &["doc", "--no-deps", "--all-features"],
        None,
    );
    run(
        "build (all targets, all features)",
        &["build", "--all-targets", "--all-features"],
        None,
    );
}

/// The library's MSRV story: default features must build on stable Rust
/// (`rust-version` in the manifests). The gate's own steps run `--all-features`
/// on the pinned nightly, which enables `nightly` and skips the stable
/// fallback paths — so this step compiles the workspace on the installed
/// `stable` toolchain and runs the core unit tests that exercise those
/// fallbacks (`hints`, scalar `par::simd`).
fn stable() {
    run(
        "stable check (workspace, all targets, default features)",
        &["+stable", "check", "--workspace", "--all-targets"],
        Some("rustup toolchain install stable"),
    );
    run(
        "stable tests (ordofp_core lib, par fallback paths)",
        &[
            "+stable",
            "test",
            "-p",
            "ordofp_core",
            "--lib",
            "--features",
            "par,std",
        ],
        Some("rustup toolchain install stable"),
    );
}

/// API-stability guard: diffs every publishable crate's public API against
/// the latest release on crates.io and fails on any change that semver
/// forbids for the version bump being made. Part of the release procedure
/// (CONTRIBUTING.md §Releasing); it can only run once a baseline exists on
/// the registry, so it is not part of `all`.
fn semver() {
    run(
        "cargo-semver-checks (vs crates.io baseline)",
        &["semver-checks", "--workspace"],
        Some("cargo install cargo-semver-checks --locked"),
    );
}

fn deny() {
    run(
        "cargo-deny (advisories, licenses, bans, sources)",
        &["deny", "check"],
        Some("cargo install cargo-deny --locked"),
    );
}

fn wasm() {
    run(
        "wasm32 target check (ordofp_core)",
        &[
            "check",
            "-p",
            "ordofp_core",
            "--target",
            "wasm32-unknown-unknown",
        ],
        None,
    );
}

/// Scope mirrors the manual practice recorded in `docs/UNSAFE_NOTES.md`:
/// the unsafe-heavy modules (arena allocator/pool).
fn miri() {
    run(
        "miri (arena unit tests)",
        &["miri", "test", "-p", "ordofp_core", "--lib", "--", "arena"],
        Some("rustup component add miri"),
    );
}

/// The libfuzzer targets link rustc's AddressSanitizer, whose runtime DLL
/// (`clang_rt.asan_dynamic-x86_64.dll`) ships with MSVC Build Tools, not the
/// Rust toolchain — without its directory on PATH every fuzz target dies at
/// startup with STATUS_DLL_NOT_FOUND (building with `-s none` is not an
/// option: sancov section symbols fail to link on COFF).
/// Hardcodes the VS-2022 directory shape; extend `roots` when a
/// new Visual Studio major lands.
#[cfg(windows)]
fn asan_dll_dir() -> Option<std::path::PathBuf> {
    let roots = [
        "C:\\Program Files (x86)\\Microsoft Visual Studio",
        "C:\\Program Files\\Microsoft Visual Studio",
    ];
    for root in roots {
        let Ok(years) = std::fs::read_dir(root) else {
            continue;
        };
        for year in years.flatten() {
            let Ok(editions) = std::fs::read_dir(year.path()) else {
                continue;
            };
            for edition in editions.flatten() {
                let msvc = edition.path().join("VC").join("Tools").join("MSVC");
                let Ok(versions) = std::fs::read_dir(&msvc) else {
                    continue;
                };
                for version in versions.flatten() {
                    let bin = version.path().join("bin").join("Hostx64").join("x64");
                    if bin.join("clang_rt.asan_dynamic-x86_64.dll").is_file() {
                        return Some(bin);
                    }
                }
            }
        }
    }
    None
}

/// 60s coverage-guided smoke per libfuzzer target; longer fuzz runs are
/// manual via `cargo fuzz run <target>`.
fn fuzz_smoke() {
    #[cfg(windows)]
    if let Some(dir) = asan_dll_dir() {
        let mut paths = vec![dir];
        paths.extend(std::env::split_paths(
            &std::env::var_os("PATH").unwrap_or_default(),
        ));
        let joined = std::env::join_paths(paths).expect("PATH entries contain no separator");
        // SAFETY: xtask is single-threaded; no other thread reads the
        // environment concurrently. Children (cargo fuzz) inherit the PATH.
        unsafe { std::env::set_var("PATH", joined) };
    }
    for target in [
        "universalis_convert",
        "zipper_ops",
        "nonempty_ops",
        "pfds_ops",
        "algebraic_laws",
    ] {
        run(
            &format!("fuzz smoke: {target} (60s)"),
            &["fuzz", "run", target, "--", "-max_total_time=60"],
            Some("cargo install cargo-fuzz --locked"),
        );
    }
}

/// Deterministic perf/behavior regression guard for the e2e verdict workload.
///
/// Asserts, for both workload modes:
///   1. checksums — behavior must be bit-identical;
///   2. steady-state allocation counts/bytes per rep — exact, noise-free
///      counters (the Windows-viable equivalent of an iai-callgrind gate).
///
/// Wall-time is deliberately NOT gated here: cross-session drift is real
/// (+4.7% observed from antivirus churn alone) — use paired hyperfine
/// sessions for time verdicts. An intentional behavior change must
/// re-record the reference values below with a fresh measurement.
fn perf_guard() {
    struct Case {
        mode: &'static str,
        args: &'static [&'static str],
        checksum: &'static str,
        allocs: u64,
        bytes: u64,
    }
    // Reference values recorded at ErrorBuf=[E;4].
    let expected = [
        Case {
            mode: "steady",
            args: &["--reps", "10"],
            checksum: "0x2018fd5f861c282f",
            allocs: 54639,
            bytes: 1_799_437,
        },
        Case {
            mode: "error-heavy",
            args: &["--mode", "error-heavy", "--reps", "10"],
            checksum: "0x7ddbc91595b27746",
            allocs: 122_908,
            bytes: 8_878_369,
        },
    ];

    run(
        "build e2e_allocs (release, Probatum-smallvec)",
        &[
            "build",
            "--release",
            "--example",
            "e2e_allocs",
            "--features",
            "Probatum-smallvec",
        ],
        None,
    );

    let exe = format!(
        "target/release/examples/e2e_allocs{}",
        std::env::consts::EXE_SUFFIX
    );
    // Extracts the digits following `key` in `out` (e.g. "allocs=54639").
    fn field(out: &str, key: &str) -> Option<u64> {
        let start = out.find(key)? + key.len();
        let digits: String = out[start..]
            .chars()
            .take_while(char::is_ascii_digit)
            .collect();
        digits.parse().ok()
    }

    let mut failed = false;
    for case in &expected {
        let output = Command::new(&exe).args(case.args).output();
        let Ok(output) = output else {
            eprintln!("FAIL [{}]: could not run {exe}", case.mode);
            exit(1);
        };
        let out = String::from_utf8_lossy(&output.stdout);

        let checksum = out.find("checksum=").map_or_else(
            || "<missing>".to_string(),
            |i| {
                out[i + "checksum=".len()..]
                    .chars()
                    .take_while(|c| c.is_ascii_hexdigit() || *c == 'x')
                    .collect::<String>()
            },
        );
        let steady = out
            .find("steady_per_rep:")
            .map(|i| &out[i..])
            .unwrap_or_default();
        let allocs = field(steady, "allocs=");
        let bytes = field(steady, "bytes=");

        let mut ok = true;
        if checksum != case.checksum {
            eprintln!(
                "FAIL [{}]: checksum {checksum} != expected {} (BEHAVIOR CHANGED)",
                case.mode, case.checksum
            );
            ok = false;
        }
        if allocs != Some(case.allocs) {
            eprintln!(
                "FAIL [{}]: steady allocs/rep {allocs:?} != expected {}",
                case.mode, case.allocs
            );
            ok = false;
        }
        if bytes != Some(case.bytes) {
            eprintln!(
                "FAIL [{}]: steady bytes/rep {bytes:?} != expected {}",
                case.mode, case.bytes
            );
            ok = false;
        }
        if ok {
            println!(
                "PASS [{}]: checksum {checksum}, allocs/rep {}, bytes/rep {}",
                case.mode, case.allocs, case.bytes
            );
        } else {
            failed = true;
        }
    }
    if failed {
        eprintln!("PERF GUARD FAILED");
        exit(1);
    }
    println!("PERF GUARD GREEN: behavior + allocation counts unchanged.");
}

/// Reproducible PGO build of the e2e verdict binary.
/// Two-phase: instrument -> train on all three workload modes -> optimize.
/// Measured: -28% steady / -21% error-heavy on top of mimalloc.
///
/// Optional second CLI arg overrides the feature set; the default is the
/// max-performance configuration (this runs on the repo's pinned nightly,
/// so `nightly` is included).
fn pgo() {
    let features = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "Probatum-smallvec,alloc-mimalloc,nightly".to_string());

    run(
        "PGO phase 1: instrumented build",
        &[
            "pgo",
            "build",
            "--",
            "--example",
            "e2e_workload",
            "--features",
            &features,
        ],
        Some("cargo install cargo-pgo --locked (plus the llvm-tools-preview rustup component)"),
    );

    // cargo-pgo builds into the explicit host-triple target dir.
    let host = {
        let out = Command::new("rustc")
            .args(["-vV"])
            .output()
            .expect("rustc -vV failed");
        let out = String::from_utf8_lossy(&out.stdout).to_string();
        out.lines()
            .find_map(|l| l.strip_prefix("host: ").map(str::to_string))
            .expect("rustc -vV printed no host line")
    };
    let exe = format!(
        "target/{host}/release/examples/e2e_workload{}",
        std::env::consts::EXE_SUFFIX
    );

    println!("==> PGO phase 2: training (steady, error-heavy, startup)");
    for args in [
        &[][..],
        &["--mode", "error-heavy"][..],
        &["--mode", "startup"][..],
    ] {
        let status = Command::new(&exe).args(args).status();
        if !status.is_ok_and(|s| s.success()) {
            eprintln!("FAILED at: PGO training run {exe} {args:?}");
            exit(1);
        }
    }

    run(
        "PGO phase 3: optimized build",
        &[
            "pgo",
            "optimize",
            "build",
            "--",
            "--example",
            "e2e_workload",
            "--features",
            &features,
        ],
        None,
    );
    println!("PGO-optimized binary: {exe}");
}

fn run(desc: &str, args: &[&str], install_hint: Option<&str>) {
    println!("==> {desc}");
    let status = Command::new("cargo").args(args).status();
    let ok = match status {
        Ok(s) if s.success() => true,
        Ok(s) => {
            eprintln!("FAILED at: {desc}");
            if let Some(hint) = install_hint {
                eprintln!("(if the subcommand is missing: {hint})");
            }
            exit(s.code().unwrap_or(1));
        }
        Err(e) => {
            eprintln!("failed to spawn cargo for {desc}: {e}");
            false
        }
    };
    if !ok {
        exit(1);
    }
}
