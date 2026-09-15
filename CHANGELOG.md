# Changelog

All notable changes to OrdoFP are documented in this file. The format follows
[Keep a Changelog](https://keepachangelog.com/), and the project adheres to
[Semantic Versioning](https://semver.org/) with the usual 0.x caveats.

## [Unreleased]

## [0.1.3] - 2026-09-15

Patch release of `ordofp_core`, `ordofp` and `ordofp_bayes`. `ordofp_macros` and
`ordofp_laws` are unchanged and stay at 0.1.2.

### Added

- HList sequencing: `HListSequenceOption`, `HListSequenceResult` and
  `HListSequenceValidated` turn an HList of `Option` / `Result` / `Probatum`
  into the effect over an HList, with a `Nihil` base case and a `Coniunctio`
  inductive step.
- Fallible lens modifiers: `Lens::modify_option`, `modify_result` and
  `modify_validated` thread a failing modifier through a lens without
  discarding the effect.
- `Probatum::map` and `Probatum::map_err`, an `IntoValidated` conversion from
  `Result`, an `IteratorValidateExt` iterator extension, and a
  `Validated<E, A>` alias. The facade re-exports all of these.
- `xtask matrix`, which builds the isolated and additive feature sets that
  guard feature orthogonality, and joins `xtask all`.

### Changed

- `xtask miri` now interprets every `unsafe`-bearing module instead of only
  `arena` on default features. The previous scope reached 14 of the crate's 109
  `unsafe` constructs: the test-name filter excluded most modules, the missing
  `--all-features` meant `nexus::effects::region` and `par::*` were never
  compiled (`cargo test -- region --list` returns 0 tests on default features
  and 19 with `--all-features`), and `-p ordofp_core` excluded `ordofp_bayes`
  entirely — whose `inference.rs` holds the densest `unsafe` in the repo.
  Filters are module paths rather than bare module names, which keeps the run
  at 114 interpreted tests instead of 494 for identical coverage.
- New `xtask miri-scope` fails when a file containing `unsafe` sits outside
  every Miri filter, so the gate's scope can no longer drift away from the
  code the way `-- arena` did. It is a filesystem scan costing milliseconds, so
  it runs in `gate` (per-commit) while the interpretation stays in `deep`
  (weekly). Paths whose coverage a name match cannot express — `ordofp_bayes`,
  and the wgpu backend Miri cannot reach at all — are listed explicitly with
  their reason instead of being silently absent.

### Fixed

- `cargo clippy -D warnings` passes from a cold cache again. 42 lint failures had
  accumulated behind incremental-build caching, so `xtask gate` reported green
  without recompiling the affected targets: 11 integration tests lacked the
  crate-level docs `missing_docs` requires, `test_structs` and
  `laws::fixpoint_laws` exposed `pub` items from private modules, and the
  `ffi_bedrock` test module tripped `unreachable_pub` on the `pub` items its
  `#[macro_export]` macros emit for downstream expansion.
- `xtask semver` no longer fails on a false positive. `Continuatio<A, B, M>` is a
  typestate whose `Semel`/`Affinis` impls take `resume(self)` while `Pluries`
  takes `resume(&self)`; cargo-semver-checks 0.50 collapses the three inherent
  impls and reports a receiver change. It fires against a byte-identical
  baseline — reproducible by running it at the 0.1.2 release commit against
  0.1.2 itself — so the lint is allowed in `core/Cargo.toml` with that rationale.
- The full `ordofp_bayes` suite runs under Miri in ~9s instead of exceeding 35
  minutes. Its three heavy statistical tests (100k-draw moment checks, a
  3000-step MCMC chain) are `#[cfg_attr(miri, ignore)]`: they exercise no
  `unsafe`, and shrinking their samples instead would widen estimator noise past
  the tolerances they exist to assert. Native runs are unchanged.
- `Arena::alloc_layout` and `SyncArena::alloc_layout` now use `checked_add` when
  computing the bump-pointer end. The preceding `new_ptr <= end` guard could not
  rule out wrap-around as its `SAFETY` comment claimed — a wrapped sum is small
  enough to pass the guard while pointing outside the chunk. Reachable on 32-bit
  targets; the `SAFETY` comments have been corrected.
- `Simd4f32`/`Simd8f32` `store`, and `Simd8f32::load`, now document that they
  panic on a short slice, matching `Simd4f32::load`. Behaviour is unchanged;
  `store_partial`/`load_partial` remain the non-panicking forms.

### Removed

- The six hand-written `unsafe impl Send`/`Sync` blocks on the internal spin-lock
  guards in `metrics::registry` and `tracing::collector`. Each guard is
  `{ lock: &'a Lock<T>, _marker: PhantomData<T> }`, which auto-derives to exactly
  `T: Send + Sync`; the manual impls only loosened that bound.

## [0.1.2] - 2026-08-28

### Changed

- `Probatum::collect` now unswitches the valid/invalid loops. The all-valid
  path no longer branches on error state per item. The first `Invalid` drops
  the collected values immediately, then a second loop accumulates remaining
  errors. Sequence error-accumulation semantics are unchanged.

## [0.1.1] - 2026-08-11

### Added

- Public maintenance playbook at `docs/maintenance.md` documenting cadence for
  dependency/toolchain drift checks, stability signaling, and release hygiene.
- Maintenance-focused issue template (`.github/ISSUE_TEMPLATE/maintenance.md`)
  for dependency/toolchain/security/release upkeep work.
- `xtask maint` local maintenance sweep command (dependency drift dry-run +
  local gate + semver check).

### Changed

- Documentation index now links the maintenance playbook from `docs/README.md`.
- `CONTRIBUTING.md` now references the maintenance playbook for maintainers.
- Maintenance playbook now explicitly codifies local-only CI via `xtask` (no
  GitHub Actions).
- Parallel effect execution engine (`ordofp_core::nexus::optim::parallel`) now uses a real optional Rayon backend for
  `par_map`, `par_map_with`, `par_traverse`, `par_traverse_with`,
  `par_fold`, `par_chunks`, and `ParallelBuilder::map`; non-`rayon` builds
  keep sequential behavior.
- Feature-flag docs now include public maturity labels (Mature / Advanced
  opt-in / Experimental).
- Toolchain/MSRV wording is now explicit: MSRV is pinned per release and may
  be raised in minor releases.

## [0.1.0] - Initial public release

First release on crates.io. OrdoFP began as a fork of
[frunk](https://github.com/lloydmeta/frunk) (see `THIRD_PARTY_NOTICES.md` for
lineage and attribution) and was developed privately before this release;
`docs/migration.md` explains the internal 2.x → 0.1.0 version reset.

### Workspace

- `ordofp` — facade crate re-exporting the default surface (`std`, `derives`,
  `proc-macros`, `Probatum`).
- `ordofp_core` — the library proper: data structures, type classes, optics,
  effects, async, parallel execution.
- `ordofp_macros` — derive (`Universalis`, `NominataUniversalis`) and
  procedural (`path!`, `path_type!`) macro support.
- `ordofp_laws` — property-based law checking (Functor/Monad/algebraic laws)
  for OrdoFP instances and your own.
- `ordofp_bayes` — standalone probabilistic-programming crate (SMC,
  Metropolis-Hastings, importance sampling).

### Highlights

- **Data structures**: HList (`Coniunctio`), coproducts (`Disiunctio`),
  `NonEmpty`, `Zipper`, persistent collections (`pfds`).
- **Type classes**: GAT-based Functor / Applicatio / Monad / Compositio /
  Unitas hierarchy, Semigroup/Monoid, Traversable.
- **Optics**: lenses, prisms, isos, traversals, affine + indexed optics,
  profunctor encoding, zero-clone `AspectusRef` composition.
- **Generic programming**: `Universalis` / `NominataUniversalis`
  struct↔HList conversion, sculpting, `Transfigurator`.
- **Effects & async**: `Flumen` streams with optional fusion, `Fibra`
  structured concurrency, row-typed effects (`nexus`), CPS monad
  transformers, tokio/smol runtime integrations.
- **Parallelism**: `ParFlumen` with scalar/rayon/wgpu backends (feature
  gated), SIMD helpers.
- **Validation**: `Probatum` accumulating validation with a spill-free inline
  error buffer.

All non-default functionality is feature-gated and additive; see
`docs/FEATURE_FLAGS.md` for the canonical matrix.

Builds on **stable Rust** (MSRV 1.97). The opt-in `nightly` feature enables
unstable-Rust acceleration (branch-prediction hints, `portable_simd`-backed
f32 kernels) with identical semantics.

Dependency posture: the default build has zero runtime dependencies (only
the compile-time proc-macro stack); serde/tokio/smol/rayon/wgpu integrations
are strictly opt-in. `ordofp_bayes` depends only on `rand` (Normal and
Exp(1) sampling are implemented in-crate), and `ordofp_laws` pulls
`quickcheck` without its logging stack.

## Versioning Policy

- **MAJOR**: Breaking API changes
- **MINOR**: New features, MSRV bumps, deprecations
- **PATCH**: Bug fixes, documentation improvements

## Toolchain Policy

OrdoFP builds on stable Rust. The MSRV (`rust-version` in each manifest) is
pinned per release and may be bumped in minor releases to stay near current
stable Rust. The optional `nightly` cargo feature enables unstable-Rust
acceleration (branch-prediction hints, `portable_simd` kernels) with
identical semantics; the pinned nightly in `rust-toolchain.toml` is the
repo's development toolchain, not a user requirement.
