# Standing policies

The decisions and doctrines in force that govern work on this repo. Each entry
is stated with enough context to stand alone; when a rule rests on a
measurement, the measurement is summarized inline.

## Verification & workflow

- **Local gate is canonical.** The five-step gate (fmt, clippy `-D warnings`,
  workspace tests, docs, full build) plus the stable-toolchain cross-check,
  `cargo-deny`, and the wasm32 check is owned by the `xtask` crate:
  `cargo run -p xtask -- all`; `-- deep` adds Miri (arena) and a
  60s fuzz smoke.
  **There is no hosted CI — xtask is the only CI.** A green local gate is the
  merge bar. (One check from the retired hosted workflow survives locally:
  `cargo check -p ordofp_core --no-default-features --features alloc`.)
- **Gate per commit** touching code/config/scripts; pure-`.md` commits ride
  the most recent green gate.
- **Never introduce an unnamed behavior change.** GPU tests self-skip when no
  device is present, never `#[ignore]`.

## Naming & API law

- **Scholastic Latin is project law** for new core public type names — consult
  [glossary.md](glossary.md); keep English aliases only where a sibling
  already has one.
- **Treat the public API as frozen by default**: no `pub use` changes or
  visibility reductions without building the downstream consumers that pin
  this crate by path.

## Performance doctrine

- **"A fast-but-wrong change is worse than a net lag."** Every downstream
  consumer must see a net gain; OrdoFP must never be a net performance lag.
- **Discipline**: verify against real workloads and build/test/bench, don't
  trust claims; one cargo process at a time (shared target-dir lock);
  warning-free build + green suite always, never commit red; **prove a
  regression-free gain with criterion A/B (`--baseline`) or revert**; one
  improvement per pass.
- **Paired A/B measurement protocol**: machine drift is real (+4.7% observed
  across sessions from antivirus churn alone), so competing builds are
  compared as separate exes **in one hyperfine session, both orders**;
  absolute ms across sessions are banner-only, deltas within a paired session
  are the evidence. `cargo run -p xtask -- perf-guard` is the deterministic
  regression gate (checksum + allocation-count identity).
- **ErrorBuf `[E;4]` constraint**: don't slim `Probatum`'s inline error buffer
  below the spill-free shape — `[E;2]` measurably spills on the error-heavy
  workload; re-measure before changing (also documented at
  `core/src/validated.rs`).
- **`examples/inline_probe.rs` is a keeper**: a deliberately committed
  codegen-regression probe proving cross-crate inlining of the generic pfds
  lookup without LTO; do not remove.

## Evaluated, NOT an improvement — do not retry

Each of these was measured and rejected; don't re-propose without new
evidence.

- `EffectContext::to_mask` O(k²) caching — wontfix; the type was later found
  orphaned and removed. Reopen only if a production consumer *and* a hot
  profile appear.
- `ord_map` union/intersection/difference `Vec::with_capacity` — measured
  noise-to-regression (+1.6% Union/10000); capacity bounds over-allocate for
  partial-overlap inputs. Reverted.
- `-C target-cpu=native` — measured, no net gain; dropped.
- `build-std` — measured +7.7% regression; dropped.
- `#[cold]`-outlining of cold error paths — measured, reverted.
- CpuRayon non-indexed paths for cheap-per-element work — net lag even at
  N=100k (overhead-bound); the `CpuScalar` default is correct. Consumers
  should **not** enable `rayon` for cheap-per-element pipelines.

## Consumer performance guidance

- In the reference LMS-grading workload, `rust_decimal` arithmetic is ≈55%+ of
  steady self-time — that is the consumer's floor, not OrdoFP's.
- Prefer the zero-clone `AspectusRef::compose` over the cloning `Aspectus`
  for read paths (~523× on nested reads, 0 allocs).
- Consumer binaries should mirror the release recipe: mimalloc one-liner,
  LTO/`panic="abort"` profile, and PGO (`cargo run -p xtask -- pgo`) — cross-crate
  inlining depends on the consumer's own profile.

## Structural & style standing rejections

- **No `foo.rs`+`foo/` migration** — the codebase is uniformly `mod.rs`
  (zero mixed pairs); wholesale migration is churn and blame pollution.
- **No `[lints]` deny table** — a *deny* table would export gate strictness
  into consumers' path-builds and turn future-nightly warnings into consumer
  build breaks. (The warn-level `[workspace.lints.clippy]` table with
  documented allows is compatible with this: warn does not break consumer
  builds; `-D warnings` stays a gate-time flag.)
- **No rustfmt.toml / clippy.toml** — pinned-toolchain defaults; empty config
  is noise.
- **`Cargo.lock` and `.cargo/config.toml` stay gitignored** (library
  convention; machine-specific MSVC paths — `.example` is tracked).

## Feature gating

`dependent`, `quantitative`, `rows`, `distributed`, `supervision`, and `ffi`
are off-by-default; `async` still implies `quantitative`. Consumers restore
access by enabling the matching feature. Canonical matrix:
[FEATURE_FLAGS.md](FEATURE_FLAGS.md).
