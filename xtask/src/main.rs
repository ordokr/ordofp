//! Cross-platform verification driver for `OrdoFP`.
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
//! cargo run -p xtask -- matrix      # key isolated/additive feature sets
//! cargo run -p xtask -- all         # gate + stable + deny + wasm (pre-push)
//! cargo run -p xtask -- miri        # UB check over every unsafe-bearing module
//! cargo run -p xtask -- miri-scope  # assert no library unsafe sits outside Miri's reach
//! cargo run -p xtask -- fuzz-smoke  # 60s coverage-guided smoke per target
//! cargo run -p xtask -- fuzz-scope  # assert every fuzz target is registered, run, and documented
//! cargo run -p xtask -- perf-guard  # deterministic checksum/alloc regression guard
//! cargo run -p xtask -- pgo [feats] # PGO build of the e2e verdict binary
//! cargo run -p xtask -- semver      # API-stability diff vs the crates.io baseline
//! cargo run -p xtask -- deep        # miri + fuzz-smoke (weekly cadence)
//! cargo run -p xtask -- maint       # local maintenance sweep (drift + all)
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
        "miri-scope" => assert_miri_scope_covers_unsafe(),
        "fuzz-scope" => assert_fuzz_scope_matches_targets(),
        "fuzz-smoke" => fuzz_smoke(),
        "perf-guard" => perf_guard(),
        "pgo" => pgo(),
        "semver" => semver(),
        "matrix" => matrix(),
        "all" => {
            gate();
            stable();
            deny();
            wasm();
            matrix();
        }
        "maint" => maint(),
        "deep" => {
            miri();
            fuzz_smoke();
        }
        _ => {
            eprintln!(
                "usage: cargo run -p xtask -- <gate|stable|deny|wasm|matrix|miri|miri-scope|fuzz-scope|fuzz-smoke|perf-guard|pgo|semver|all|maint|deep>"
            );
            exit(2);
        }
    }
    println!("xtask {task}: GREEN");
}

/// The canonical verification steps. Every commit is expected to pass them all.
///
/// The Miri and fuzz *scope* checks run here rather than only in `deep`: they
/// are filesystem scans costing milliseconds, so an `unsafe` construct landing
/// in a module no Miri filter reaches — or a fuzz target the smoke run never
/// executes — fails on the commit that introduces it instead of at the next
/// weekly `deep` run.
fn gate() {
    assert_miri_scope_covers_unsafe();
    assert_fuzz_scope_matches_targets();
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

/// Local-only maintenance sweep:
/// - dependency drift visibility (`cargo update --dry-run`)
/// - canonical verification gate (`all`)
/// - API compatibility check (`semver`)
fn maint() {
    run(
        "dependency drift (workspace dry-run)",
        &["update", "--workspace", "--dry-run"],
        None,
    );
    gate();
    stable();
    deny();
    wasm();
    semver();
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

/// Feature-matrix verification: builds critical isolated and additive feature
/// sets to guarantee feature orthogonality and prevent unflagged breakage.
fn matrix() {
    let combos: &[(&str, &[&str])] = &[
        (
            "workspace (no-default-features)",
            &["check", "--workspace", "--no-default-features"],
        ),
        (
            "ordofp_core (no-default-features + alloc)",
            &[
                "check",
                "-p",
                "ordofp_core",
                "--no-default-features",
                "--features",
                "alloc",
            ],
        ),
        (
            "ordofp (no-default-features + std,Probatum)",
            &[
                "check",
                "-p",
                "ordofp",
                "--no-default-features",
                "--features",
                "std,Probatum",
            ],
        ),
        (
            "ordofp (async, tokio)",
            &["check", "-p", "ordofp", "--features", "async,tokio"],
        ),
        (
            "ordofp (async, smol)",
            &["check", "-p", "ordofp", "--features", "async,smol"],
        ),
        (
            "ordofp (linear)",
            &["check", "-p", "ordofp", "--features", "linear"],
        ),
        (
            "ordofp (par, rayon)",
            &["check", "-p", "ordofp", "--features", "par,rayon"],
        ),
        (
            "ordofp (fusion)",
            &["check", "-p", "ordofp", "--features", "fusion"],
        ),
        (
            "ordofp (nexus)",
            &["check", "-p", "ordofp", "--features", "nexus"],
        ),
        (
            "ordofp (transformers-cps)",
            &["check", "-p", "ordofp", "--features", "transformers-cps"],
        ),
        (
            "ordofp (serde, alloc)",
            &["check", "-p", "ordofp", "--features", "serde,alloc"],
        ),
        (
            "ordofp (Probatum-smallvec)",
            &["check", "-p", "ordofp", "--features", "Probatum-smallvec"],
        ),
    ];

    for (name, args) in combos {
        run(&format!("matrix: {name}"), args, None);
    }
}

/// Test-name filters covering every `ordofp_core` module that contains an
/// `unsafe` construct. Miri interprets only the code a test actually executes,
/// so this list *is* the crate's UB-coverage surface.
///
/// `--all-features` is not optional here: `nexus::effects::region` and
/// `par::*` are feature-gated, so a default-feature run does not even compile
/// them, let alone interpret them.
/// Filters are *module paths*, not bare module names: `nexus` alone matches 291
/// tests where the `unsafe` lives in three submodules, so naming them precisely
/// takes the gate from 494 interpreted tests to 114 with identical coverage.
/// [`assert_miri_scope_covers_unsafe`] maps each back to a file path by
/// rewriting `::` as `/`.
///
/// The `ordofp_laws` equivalents live in [`MIRI_LAWS_FILTERS`]: they run
/// against a different package, so they cannot share this list.
const MIRI_CORE_FILTERS: &[&str] = &[
    "arena", // arena::{allocator, pool}
    "nexus::effects::region",
    "nexus::optim::commutativity",
    "nexus::optim::fusion",
    "specialization::hints",
    "ffi_bedrock", // unsafe extern blocks
    "metrics::registry",
    "dependent::refinement",
    "refined::wrapper",
    "refined::predicate",
    "tracing::collector",
];

/// Test-name filters covering the `ordofp_laws` modules that contain an
/// `unsafe` construct: the test-only `Waker::from_raw` in the hand-rolled
/// `block_on` each async law suite rolls for itself. Measured at ~7s of
/// interpretation for the 35 tests they match.
///
/// The crate's sync law suites hold no `unsafe` and stay out, for the same
/// reason [`MIRI_CORE_FILTERS`] is a list of module paths rather than one
/// whole-crate run. Separate list because these run against `-p ordofp_laws`.
const MIRI_LAWS_FILTERS: &[&str] = &["async_functor_laws", "async_monad_laws"];

/// `unsafe`-bearing paths whose coverage a name match in [`MIRI_CORE_FILTERS`]
/// or [`MIRI_LAWS_FILTERS`] cannot express — each with the invocation that
/// covers it, or the reason none can.
///
/// Kept explicit so an uncovered module is a reviewed line in the diff rather
/// than an invisible hole in the filter list.
const MIRI_PATH_NOTES: &[(&str, &str)] = &[
    (
        "ordofp_bayes/src/inference.rs",
        "covered by `miri test -p ordofp_bayes --lib`: the package is interpreted \
         whole rather than matched by a core filter name. Its three heavy, \
         unsafe-free statistical tests are `#[cfg_attr(miri, ignore)]`, which is \
         what keeps that run at ~9s.",
    ),
    (
        "core/src/par/backend/wgpu/",
        "NOT covered: the GPU backend carries no unit tests, and its unsafe paths \
         need a real wgpu adapter that Miri cannot provide.",
    ),
];

/// Workspace members whose source is deliberately outside the Miri scan, each
/// with its reason. Every entry is verified on every run — the directory must
/// hold a readable `Cargo.toml` that still says `publish = false` — so an
/// exemption cannot quietly become a claim about a crate that ships.
const MIRI_SCAN_EXEMPT: &[(&str, &str)] = &[(
    "xtask",
    "`publish = false` dev tooling that never runs in a user's process. Its one \
     `unsafe` construct is a single-threaded `env::set_var` in the fuzz PATH \
     setup, unreachable from the library.",
)];

/// UB check over the unsafe-bearing modules.
///
/// Previously this ran `-- arena` on default features, which reached 14 of the
/// crate's 109 `unsafe` constructs and none of `ordofp_bayes` or `ordofp_laws`.
/// The scope is now derived from where `unsafe` actually lives, and
/// [`assert_miri_scope_covers_unsafe`] fails the gate if the two drift apart.
fn miri() {
    assert_miri_scope_covers_unsafe();
    for filter in MIRI_CORE_FILTERS {
        run(
            &format!("miri (core::{filter})"),
            &[
                "miri",
                "test",
                "-p",
                "ordofp_core",
                "--lib",
                "--all-features",
                "--",
                filter,
            ],
            Some("rustup component add miri"),
        );
    }
    // Separate package: `-p ordofp_core` never reached it, yet `inference.rs`
    // holds the densest `unsafe` in the repo (27 constructs). The whole package
    // is interpreted now that its three statistically-heavy, `unsafe`-free tests
    // carry `#[cfg_attr(miri, ignore)]` — ~9s, where the unfiltered suite
    // previously ran past 35 minutes without finishing.
    run(
        "miri (ordofp_bayes)",
        &["miri", "test", "-p", "ordofp_bayes", "--lib"],
        Some("rustup component add miri"),
    );
    // Third package: `ordofp_laws` holds exactly two `unsafe` constructs, the
    // `Waker::from_raw` in the `#[cfg(test)]` `block_on` helper each async law
    // suite rolls for itself. `--all-features` because the modules are behind
    // `async` and do not even compile without it. ~7s for the 35 tests matched.
    for filter in MIRI_LAWS_FILTERS {
        run(
            &format!("miri (laws::{filter})"),
            &[
                "miri",
                "test",
                "-p",
                "ordofp_laws",
                "--lib",
                "--all-features",
                "--",
                filter,
            ],
            Some("rustup component add miri"),
        );
    }
}

/// Fail the gate when an `unsafe`-bearing file sits outside every Miri filter.
///
/// Without this, the filter list rots exactly as `-- arena` did: new `unsafe`
/// lands in a module no filter names, and the gate stays green while covering
/// less and less of the crate.
///
/// Scope is library source: the root package, every workspace member's `src/`
/// (derived, not listed — see [`miri_scan_roots`]), and the fuzz targets.
/// Integration, bench, and example targets sit outside it, because the
/// interpreted set is filtered `--lib` tests; whole-crate exemptions live in
/// [`MIRI_SCAN_EXEMPT`] and are printed on every run.
fn assert_miri_scope_covers_unsafe() {
    println!("==> miri scope covers every unsafe-bearing module");
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask always sits one level below the workspace root")
        .to_path_buf();

    let roots = miri_scan_roots(&root);
    for (crate_dir, reason) in MIRI_SCAN_EXEMPT {
        let manifest = std::fs::read_to_string(root.join(crate_dir).join("Cargo.toml"));
        if !manifest.is_ok_and(|text| text.contains("publish = false")) {
            eprintln!("FAILED: `{crate_dir}` is exempt from the Miri scan but does not read as a");
            eprintln!("        `publish = false` crate with a Cargo.toml. The exemption is for");
            eprintln!("        code unreachable from a user's process; anything else is a hole.");
            exit(1);
        }
        println!("    exempt: {crate_dir} — {reason}");
    }
    println!("    roots: {}", roots.join(", "));

    let mut uncovered = Vec::new();
    for dir in &roots {
        collect_unsafe_files(&root.join(dir), &root, &mut uncovered);
    }
    uncovered.retain(|p| {
        !MIRI_CORE_FILTERS
            .iter()
            .chain(MIRI_LAWS_FILTERS)
            .any(|f| p.contains(&f.replace("::", "/")))
            && !MIRI_PATH_NOTES
                .iter()
                .any(|(prefix, _)| p.starts_with(prefix))
    });

    if !uncovered.is_empty() {
        eprintln!("FAILED: these files contain `unsafe` but no Miri filter reaches them:");
        for path in &uncovered {
            eprintln!("  {path}");
        }
        eprintln!(
            "add a covering filter to MIRI_CORE_FILTERS or MIRI_LAWS_FILTERS, or a \
             justified entry to MIRI_PATH_NOTES, in xtask/src/main.rs"
        );
        exit(1);
    }
    for (path, note) in MIRI_PATH_NOTES {
        println!("    note: {path} — {note}");
    }
}

/// Library-source roots the Miri scan covers: the root package, every workspace
/// member's `src/`, and the fuzz targets.
///
/// Derived from `[workspace] members` rather than listed by hand, because the
/// hand list is exactly how `laws/src` went unscanned while this guard kept
/// reporting green — nothing compared the literal against the workspace it
/// claimed to cover. `fuzz/fuzz_targets` is the one path here that is not
/// derived; [`assert_fuzz_scope_matches_targets`] pins its contents, so it
/// cannot silently stop existing either.
fn miri_scan_roots(root: &std::path::Path) -> Vec<String> {
    let manifest = std::fs::read_to_string(root.join("Cargo.toml")).unwrap_or_default();
    let members = manifest
        .split_once("members = [")
        .and_then(|(_, rest)| rest.split_once(']'))
        .map(|(list, _)| list.split('"').skip(1).step_by(2).collect::<Vec<_>>())
        .unwrap_or_default();
    if members.is_empty() {
        eprintln!("FAILED: no `[workspace] members` found in Cargo.toml, so the scan would");
        eprintln!("        cover only the root package and call that complete");
        exit(1);
    }
    let mut roots: Vec<String> = members
        .into_iter()
        .filter(|member| !MIRI_SCAN_EXEMPT.iter().any(|(exempt, _)| exempt == member))
        .map(|member| format!("{member}/src"))
        .collect();
    roots.push("src".to_string());
    roots.push("fuzz/fuzz_targets".to_string());
    roots
}

fn collect_unsafe_files(dir: &std::path::Path, root: &std::path::Path, out: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_unsafe_files(&path, root, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            let Ok(src) = std::fs::read_to_string(&path) else {
                continue;
            };
            if src.lines().any(is_unsafe_construct) {
                let rel = path.strip_prefix(root).unwrap_or(&path);
                out.push(rel.to_string_lossy().replace('\\', "/"));
            }
        }
    }
}

/// True for a real `unsafe` construct, false for the word inside a `// SAFETY:`
/// note or a doc comment — the distinction that separates 109 real constructs
/// from the ~186 a plain word-grep reports.
fn is_unsafe_construct(line: &str) -> bool {
    let trimmed = line.trim_start();
    if trimmed.starts_with("//") || trimmed.starts_with('*') {
        return false;
    }
    [
        "unsafe {",
        "unsafe fn ",
        "unsafe impl ",
        "unsafe trait ",
        "unsafe extern ",
    ]
    .iter()
    .any(|kind| trimmed.contains(kind))
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

/// Fuzz targets and the surface each one exercises — the single source of truth
/// for [`fuzz_smoke`], the `[[bin]]` registrations in `fuzz/Cargo.toml`, and the
/// fuzz claim in `SECURITY.md`.
///
/// These are oracle checks over *safe* API surface: pfds diffs `Stack`/`Queue`
/// against `VecDeque`, zipper and Universalis check round-trips, `NonEmpty`
/// exercises the total-API contracts, and the law target drives the `QuickCheck`
/// properties. No target reaches an `unsafe`-bearing module — UB detection is
/// Miri's job (see [`MIRI_CORE_FILTERS`]), not the fuzzer's, and
/// [`assert_fuzz_scope_matches_targets`] prints that boundary on every run so a
/// future target over `unsafe` land is a reviewed decision rather than a silent
/// one.
const FUZZ_TARGETS: &[(&str, &str)] = &[
    ("algebraic_laws", "ordofp_laws law suites"),
    ("nonempty_ops", "ordofp_core::nonempty"),
    (
        "pfds_ops",
        "ordofp_core::pfds (Stack/Queue vs VecDeque oracle)",
    ),
    (
        "universalis_convert",
        "ordofp::{Universalis, from_universalis, into_universalis}",
    ),
    ("zipper_ops", "ordofp_core::zipper"),
];

/// Fail when the targets on disk, the set [`fuzz_smoke`] executes, the `[[bin]]`
/// registrations in `fuzz/Cargo.toml`, and the fuzz bullet in `SECURITY.md`
/// drift apart.
///
/// The cheap filesystem half of the pair whose other half is
/// [`assert_miri_scope_covers_unsafe`]. Rot here looks like a target added and
/// never run, a target renamed while the smoke list keeps the old name, or a
/// security doc still naming a surface nothing fuzzes.
fn assert_fuzz_scope_matches_targets() {
    println!("==> fuzz scope matches the targets on disk, in fuzz/Cargo.toml, and in SECURITY.md");
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask always sits one level below the workspace root")
        .to_path_buf();

    let mut on_disk = Vec::new();
    if let Ok(entries) = std::fs::read_dir(root.join("fuzz").join("fuzz_targets")) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "rs") {
                on_disk.push(
                    path.file_stem()
                        .expect("a *.rs path has a stem")
                        .to_string_lossy()
                        .into_owned(),
                );
            }
        }
    }
    on_disk.sort();

    let Ok(manifest) = std::fs::read_to_string(root.join("fuzz").join("Cargo.toml")) else {
        eprintln!("FAILED: fuzz/Cargo.toml is unreadable");
        exit(1);
    };
    let Ok(security) = std::fs::read_to_string(root.join("SECURITY.md")) else {
        eprintln!("FAILED: SECURITY.md is unreadable");
        exit(1);
    };

    let mut problems = Vec::new();
    for name in &on_disk {
        if !FUZZ_TARGETS
            .iter()
            .any(|(target, _)| *target == name.as_str())
        {
            problems.push(format!(
                "fuzz/fuzz_targets/{name}.rs exists but FUZZ_TARGETS never runs it"
            ));
        }
    }
    for &(name, _) in FUZZ_TARGETS {
        if !on_disk.iter().any(|target| target.as_str() == name) {
            problems.push(format!(
                "FUZZ_TARGETS names `{name}`, which has no fuzz/fuzz_targets/{name}.rs"
            ));
        }
        if !manifest.contains(&format!("name = \"{name}\"")) {
            problems.push(format!(
                "`{name}` is missing from the [[bin]] entries in fuzz/Cargo.toml"
            ));
        }
        if !security.contains(name) {
            problems.push(format!("SECURITY.md does not name the `{name}` target"));
        }
    }

    if !problems.is_empty() {
        eprintln!("FAILED: the fuzz scope has drifted:");
        for problem in &problems {
            eprintln!("  {problem}");
        }
        eprintln!(
            "keep FUZZ_TARGETS, fuzz/fuzz_targets/, fuzz/Cargo.toml, and the fuzz \
             bullet in SECURITY.md in agreement"
        );
        exit(1);
    }
    for &(name, surface) in FUZZ_TARGETS {
        println!("    {name}: {surface}");
    }
    println!("    boundary: fuzzing is oracle coverage over safe API surface; Miri owns UB");
}

/// 60s coverage-guided smoke per libfuzzer target; longer fuzz runs are
/// manual via `cargo fuzz run <target>`.
fn fuzz_smoke() {
    assert_fuzz_scope_matches_targets();
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
    for &(target, _) in FUZZ_TARGETS {
        run(
            &format!("fuzz smoke: {target} (60s)"),
            &["fuzz", "run", target, "--", "-max_total_time=60"],
            Some("cargo install cargo-fuzz --locked"),
        );
    }
}

/// One `perf_guard` reference case: expected checksum and allocation counters.
struct PerfCase {
    mode: &'static str,
    args: &'static [&'static str],
    checksum: &'static str,
    allocs: u64,
    bytes: u64,
}

/// Extracts the digits following `key` in `out` (e.g. "allocs=54639").
#[must_use]
fn perf_field(out: &str, key: &str) -> Option<u64> {
    let start = out.find(key)? + key.len();
    let digits: String = out[start..]
        .chars()
        .take_while(char::is_ascii_digit)
        .collect();
    digits.parse().ok()
}

/// Runs one [`PerfCase`] against the built `e2e_allocs` binary; returns whether
/// the checksum and allocation counters all match their references.
#[must_use]
fn run_perf_case(exe: &str, case: &PerfCase) -> bool {
    let output = Command::new(exe).args(case.args).output();
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
    let allocs = perf_field(steady, "allocs=");
    let bytes = perf_field(steady, "bytes=");

    let checksum_ok = checksum == case.checksum;
    if !checksum_ok {
        eprintln!(
            "FAIL [{}]: checksum {checksum} != expected {} (BEHAVIOR CHANGED)",
            case.mode, case.checksum
        );
    }
    let allocs_ok = allocs == Some(case.allocs);
    if !allocs_ok {
        eprintln!(
            "FAIL [{}]: steady allocs/rep {allocs:?} != expected {}",
            case.mode, case.allocs
        );
    }
    let bytes_ok = bytes == Some(case.bytes);
    if !bytes_ok {
        eprintln!(
            "FAIL [{}]: steady bytes/rep {bytes:?} != expected {}",
            case.mode, case.bytes
        );
    }
    let ok = checksum_ok && allocs_ok && bytes_ok;
    if ok {
        println!(
            "PASS [{}]: checksum {checksum}, allocs/rep {}, bytes/rep {}",
            case.mode, case.allocs, case.bytes
        );
    }
    ok
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
    // Reference values recorded at ErrorBuf=[E;4].
    let expected = [
        PerfCase {
            mode: "steady",
            args: &["--reps", "10"],
            checksum: "0x2018fd5f861c282f",
            allocs: 54639,
            bytes: 1_799_437,
        },
        PerfCase {
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
    let mut failed = false;
    for case in &expected {
        if !run_perf_case(&exe, case) {
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
