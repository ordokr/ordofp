# Contributing to OrdoFP

Thank you for your interest in contributing to OrdoFP! We welcome contributions from the community.

## Prerequisites

- **Rust**: The library builds on stable Rust (current MSRV 1.97). MSRV is pinned per release and may be raised in minor releases. For *developing this repo*, rustup automatically picks up the pinned nightly from `rust-toolchain.toml` — the verification gate lints and tests with `--all-features`, which includes the nightly-only `nightly` acceleration feature. Keep a current `stable` toolchain installed too; the gate cross-checks the stable build.
- **Cargo**: Standard Rust build tool.

## Getting Started

1.  **Fork the repository** on GitHub.
2.  **Clone your fork**:
    ```bash
    git clone https://github.com/YOUR_USERNAME/ordofp.git
    cd ordofp
    ```
3.  **Create a branch** in your fork for your change.

## Development Workflow

### Running Tests

Run all tests including all features:

```bash
cargo test --all-features
```

Run tests for specific core features:

```bash
cargo test --features std,alloc,async
```

### Formatting and Linting

Ensure your code is formatted and lint-free before submitting:

```bash
cargo fmt
cargo clippy --all-features
```

## Documentation

- Use **Scholastic Latin** for new core type names (consult `docs/glossary.md`).
- Update `docs/` if you change public APIs.
- Add examples to the `examples/` directory for significant new features.

## Pull Requests

1.  Push your branch to GitHub.
2.  Open a Pull Request against the upstream `main` branch.
3.  Describe your changes clearly.
4.  Verification is local and owned by the `xtask` crate — there is no hosted CI, so a green local run is the merge bar. Before opening the PR run the five-step gate (fmt, clippy `-D warnings`, workspace tests, docs, full build) plus the stable-toolchain cross-check, `cargo-deny`, and the wasm32 check; run the deep pass (Miri + fuzz smoke) when touching `unsafe`:
    ```sh
    cargo run -p xtask -- all    # gate + stable + deny + wasm  (-- gate runs the five-step gate alone)
    cargo run -p xtask -- deep   # miri + fuzz smoke
    ```

## Releasing (maintainers)

Five crates publish to crates.io: `ordofp_macros`, `ordofp_core`, `ordofp`,
`ordofp_laws`, `ordofp_bayes` (`xtask` and `fuzz` are `publish = false`).
`ordofp` and `ordofp_laws` depend on each other at dev/test time, so the set
must be released together with workspace publishing (cargo ≥ 1.90), which
orders the graph and bridges the cycle automatically:

```sh
cargo run -p xtask -- all          # full local gate — the release bar
cargo run -p xtask -- semver      # no forbidden API breaks vs the published baseline
cargo package --workspace         # dry-run: builds every crate from its packaged tarball
cargo publish --workspace         # publishes in dependency order
```

(`xtask semver` needs a published baseline, so it is skipped for the very
first release and mandatory for every release after it.)

Bump `version` in every workspace `Cargo.toml` together — the internal
`path + version` dependencies pin exact sibling versions.

## Public maintenance loop (maintainers)

For ongoing public upkeep (dependency/toolchain drift, stability signaling, and
release hygiene), follow [`docs/maintenance.md`](docs/maintenance.md). Use the
maintenance issue template for tracked upkeep tasks.

## License

By contributing, you agree that your contributions will be licensed under the Apache License 2.0.
