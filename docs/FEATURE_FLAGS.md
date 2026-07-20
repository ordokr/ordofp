# Feature Flags Matrix

Canonical matrix of all OrdoFP feature flags, their dependencies, and defaults
(source of truth: `[features]` in the root `Cargo.toml`). All features are
additive — any combination is valid.

| Feature | Description | Pulls in | Default |
|---------|-------------|----------|---------|
| `std` | Standard library support | `alloc` | ✅ |
| `alloc` | Heap allocation support | — | ✅ (via `std`) |
| `derives` | Custom derives (`Universalis`, `NominataUniversalis`) | — | ✅ |
| `proc-macros` | Additional proc-macro tooling | — | ✅ |
| `Probatum` | Validation API surface | — | ✅ |
| `Probatum-smallvec` | `Probatum` invalid storage via `smallvec` | `Probatum` | ❌ |
| `nightly` | Unstable-Rust acceleration (branch-prediction hints; `portable_simd` f32 kernels with `par`). Requires a nightly toolchain; identical semantics, better codegen. | — | ❌ |
| `serde` | Serde serialization support | `alloc` | ❌ |
| `async` | Core async traits and `Futurus` | `std` | ❌ |
| `tokio` | Tokio runtime integration | `async` | ❌ |
| `smol` | smol runtime integration (successor to the discontinued async-std) | `async` | ❌ |
| `async-std` | Back-compat alias for the discontinued async-std runtime (RUSTSEC-2025-0052); selects `smol`. Never gated on directly. | `smol` | ❌ |
| `fusion` | Stream fusion (`FlumenFusus`) | `async` | ❌ |
| `linear` | Linear types and resource management (OrdoFP 0.1.0) | `alloc` | ❌ |
| `par` | ParFlumen data-parallel IR | `alloc` | ❌ |
| `rayon` | Rayon CPU backend | `par` | ❌ |
| `gpu-wgpu` | wgpu GPU backend | `par` | ❌ |
| `gpu-buffer-pool` | Buffer reuse optimization | `par`, `gpu-wgpu` | ❌ |
| `transformers-cps` | CPS (Church-encoded) transformers | `alloc` | ❌ |
| `nexus` | Row-typed effects system | `alloc` | ❌ |
| `alloc-mimalloc` | Opt-in mimalloc for this repo's own perf-driver binaries (`e2e_workload`); the library never sets a global allocator. | — | ❌ |
| `dependent` | Dependent-type experiments. Off by default; no downstream consumers. | — | ❌ |
| `quantitative` | Multiplicity markers. Enabled transitively by `async` (effects subsystem uses it). | — | ❌ |
| `rows` | Row-polymorphism experiments. Off by default. | — | ❌ |
| `distributed` | Distributed-effect scaffolding. Off by default. | — | ❌ |
| `supervision` | Supervision-tree scaffolding. Off by default. | — | ❌ |
| `ffi` | FFI wrapper toolkit (`ffi_bedrock`). Off by default; no callers. | — | ❌ |

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
