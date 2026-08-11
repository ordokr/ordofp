---
name: Maintenance / SOTA upgrade
about: Dependency, toolchain, docs, security-hygiene, or release-process upkeep
labels: maintenance
---

**Maintenance area**
- [ ] Dependencies
- [ ] Rust toolchain / MSRV
- [ ] Docs and public-facing guidance
- [ ] Security posture
- [ ] Release hygiene
- [ ] Other

**What is drifting today?**

**Proposed change**

**Impact assessment**
- Public API impact:
- MSRV impact:
- Feature-flag impact:
- Release note needed:

**Acceptance bar**
- [ ] `cargo run -p xtask -- all`
- [ ] `cargo run -p xtask -- semver` (all releases after first publish)
- [ ] `cargo package --workspace`
