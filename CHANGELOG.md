# Changelog

All notable changes to OrdoFP are documented in this file. The format follows
[Keep a Changelog](https://keepachangelog.com/), and the project adheres to
[Semantic Versioning](https://semver.org/) with the usual 0.x caveats.

## [Unreleased]

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
- `nexus::optim::parallel` now uses a real optional Rayon backend for
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
