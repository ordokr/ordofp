# Feature Flags Matrix

Canonical matrix of all OrdoFP feature flags, their dependencies, and defaults
(source of truth: `[features]` in the root `Cargo.toml`). All features are
additive — any combination is valid.

| Feature | Description | Pulls in | Default | Maturity |
|---------|-------------|----------|---------|----------|
| `std` | Standard library support | `alloc` | ✅ | Mature |
| `alloc` | Heap allocation support | — | ✅ (via `std`) | Mature |
| `derives` | Custom derives (`Universalis`, `NominataUniversalis`) | — | ✅ | Mature |
| `proc-macros` | Additional proc-macro tooling | — | ✅ | Mature |
| `Probatum` | Validation API surface | — | ✅ | Mature |
| `Probatum-smallvec` | `Probatum` invalid storage via `smallvec` | `Probatum` | ❌ | Mature |
| `nightly` | Unstable-Rust acceleration (branch-prediction hints; `portable_simd` f32 kernels with `par`). Requires a nightly toolchain; identical semantics, better codegen. | — | ❌ | Advanced (opt-in) |
| `serde` | Serde serialization support | `alloc` | ❌ | Mature |
| `async` | Core async traits and `Futurus` | `std` | ❌ | Mature |
| `tokio` | Tokio runtime integration | `async` | ❌ | Mature |
| `smol` | smol runtime integration (successor to the discontinued async-std) | `async` | ❌ | Mature |
| `async-std` | Back-compat alias for the discontinued async-std runtime (RUSTSEC-2025-0052); selects `smol`. Never gated on directly. | `smol` | ❌ | Compatibility |
| `fusion` | Stream fusion (`FlumenFusus`) | `async` | ❌ | Mature |
| `linear` | Linear types and resource management | `alloc` | ❌ | Mature |
| `par` | ParFlumen data-parallel IR | `alloc` | ❌ | Mature |
| `rayon` | Rayon CPU backend | `par` | ❌ | Mature |
| `gpu-wgpu` | wgpu GPU backend | `par` | ❌ | Advanced (opt-in) |
| `gpu-buffer-pool` | Buffer reuse optimization | `par`, `gpu-wgpu` | ❌ | Advanced (opt-in) |
| `transformers-cps` | CPS (Church-encoded) transformers | `alloc` | ❌ | Mature |
| `nexus` | Row-typed effects system | `alloc` | ❌ | Experimental |
| `alloc-mimalloc` | Opt-in mimalloc for this repo's own perf-driver binaries (`e2e_workload`); the library never sets a global allocator. | — | ❌ | Maintainer/perf tooling |
| `dependent` | Dependent-type experiments. Off by default; no downstream consumers. | — | ❌ | Experimental |
| `quantitative` | Multiplicity markers. Enabled transitively by `async` (effects subsystem uses it). | — | ❌ | Experimental |
| `rows` | Row-polymorphism experiments. Off by default. | — | ❌ | Experimental |
| `distributed` | Distributed-effect scaffolding. Off by default. | — | ❌ | Experimental |
| `supervision` | Supervision-tree scaffolding. Off by default. | — | ❌ | Experimental |
| `ffi` | FFI wrapper toolkit (`ffi_bedrock`). Off by default; no callers. | — | ❌ | Experimental |

Maturity labels are public guidance:
- **Mature**: production-ready within normal 0.x semver caveats.
- **Advanced (opt-in)**: stable APIs with heavier platform/perf constraints.
- **Experimental**: roadmap/research surface, off-by-default.

## Recommended Feature Sets

```toml
# Minimal
features = ["alloc"]

# Standard
features = ["std", "alloc", "serde"]

# Async
features = ["std", "alloc", "async", "tokio", "fusion"]

# Parallel computing
features = ["std", "alloc", "par", "rayon", "gpu-wgpu"]

# Full feature set
features = [
    "std", "alloc", "serde",
    "async", "tokio", "fusion",
    "linear",
    "par", "rayon", "gpu-wgpu", "gpu-buffer-pool",
    "transformers-cps", "nexus",
    "alloc-mimalloc",
    "dependent", "quantitative", "rows", "distributed", "supervision", "ffi",
]
```
