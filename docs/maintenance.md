# Public maintenance playbook

This file captures the ongoing maintenance loop for keeping OrdoFP healthy as
a public crate and aligned with current Rust ecosystem practice.

## Cadence

- **Weekly**: local dependency/toolchain drift triage, changelog hygiene.
- **Monthly**: toolchain drift review (stable + pinned nightly), docs drift
  review (README/reference/feature matrix consistency).
- **Before each release**: full release bar from `CONTRIBUTING.md` plus
  semver and packaging checks.

Primary local sweep command:

```sh
cargo run -p xtask -- maint
```

## Dependency posture

- Keep non-default integrations (`tokio`, `smol`, `rayon`, `serde`, `wgpu`)
  current enough that feature users are not forced onto stale stacks.
- Prefer small, targeted local update passes; split large runtime or macro
  upgrades into dedicated changes when behavior risk is non-trivial.
- If a dependency bump changes behavior/perf, capture that in `CHANGELOG.md`.

## Toolchain posture

- Stable Rust is the user-facing baseline (`rust-version` in manifests).
- The pinned nightly in `rust-toolchain.toml` exists for reproducible repo
  development and should be refreshed regularly to avoid gate/tool drift.
- Any MSRV bump is a **minor** release item and must be called out in the
  changelog and release notes.
- CI policy is local-only via `xtask`; do not rely on GitHub Actions.

## Stability signaling

- Keep public docs explicit about maturity boundaries:
  - **Mature/stable within 0.x expectations**
  - **Experimental/off-by-default research surfaces**
- When graduating a feature from experimental status, record:
  1. default/feature-flag status change,
  2. law/perf/unsafe validation expectations,
  3. migration notes for downstream users.

## Security and supply-chain hygiene

- Keep `SECURITY.md` current and route vulnerability reports through GitHub
  advisories.
- Run `cargo-deny` as part of the local gate (`xtask -- all`) for license and
  advisory coverage.
- If a vulnerable integration is replaced/deprecated, keep a compatibility
  alias only when it is clearly documented (as with `async-std` -> `smol`).
