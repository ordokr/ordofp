# Security Policy

## Supported Versions

Only the latest published 0.x release receives fixes.

## Reporting a Vulnerability

Please report vulnerabilities privately via
[GitHub security advisories](https://github.com/ordokr/ordofp/security/advisories/new)
rather than public issues. You should receive a response within a week.

Notes for triage:

- `unsafe` code is concentrated in the arena, pfds, and async internals;
  `cargo run -p xtask -- deep` runs Miri and a fuzz smoke over those surfaces.
- Documented invariants for the unsafe surfaces live in
  [docs/UNSAFE_NOTES.md](docs/UNSAFE_NOTES.md).
