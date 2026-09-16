# Security Policy

## Supported Versions

Only the latest published 0.x release receives fixes.

## Reporting a Vulnerability

Please report vulnerabilities privately via
[GitHub security advisories](https://github.com/ordokr/ordofp/security/advisories/new)
rather than public issues. You should receive a response within a week.

Notes for triage:

- `unsafe` code is confined to a known set of modules: the arena, the refined
  and dependent wrappers, the nexus effects and optimizer, the FFI bedrock, the
  metrics, tracing, and specialization internals, the wgpu backend,
  `ordofp_bayes` inference, and one test-only `Waker::from_raw` in each of the
  `ordofp_laws` async law suites. `cargo run -p xtask -- deep` runs both dynamic
  checks over that surface: Miri interprets every `unsafe`-bearing module of
  `ordofp_core`, the whole of `ordofp_bayes`, and those two `ordofp_laws`
  modules — the one surface it cannot reach is the wgpu backend, whose paths
  need a real adapter — and a 60-second
  fuzz smoke runs the `algebraic_laws`, `nonempty_ops`, `pfds_ops`,
  `universalis_convert`, and `zipper_ops` targets. Those targets are oracle
  checks over safe API surface (pfds against `VecDeque`, round-trips, law
  properties), so UB detection rests on Miri, not on them.
- That coverage is itself enforced: `xtask miri-scope` and `xtask fuzz-scope`
  fail the per-commit gate if an `unsafe`-bearing module escapes every Miri
  filter, or if a fuzz target stops being registered, executed, or named here.
  The Miri scope is derived from `[workspace] members` rather than listed by
  hand, so a crate cannot join the workspace and stay unscanned; it covers
  library source and the fuzz targets, not the integration, bench, or example
  targets. The single whole-crate exemption is `xtask`, checked on every run to
  still be `publish = false` tooling that never enters a user's process.
- Documented invariants for the unsafe surfaces live in
  [docs/UNSAFE_NOTES.md](docs/UNSAFE_NOTES.md).
