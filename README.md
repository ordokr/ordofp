# OrdoFP

[![crates.io](https://img.shields.io/crates/v/ordofp.svg)](https://crates.io/crates/ordofp)
[![docs.rs](https://img.shields.io/docsrs/ordofp)](https://docs.rs/ordofp)
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache--2.0-blue.svg)](LICENSE)
[![Rust: 1.97+](https://img.shields.io/badge/rust-1.97%2B-blue.svg)](https://www.rust-lang.org)

> **Toolchain**: Builds on **stable Rust** (MSRV **1.97**, Edition 2024). MSRV is pinned per release and may be raised in minor releases to stay near current stable Rust. The optional `nightly` cargo feature enables unstable-Rust acceleration — branch-prediction hints and `portable_simd` kernels — with identical semantics and better codegen.

> **Dependencies**: The default build has **zero runtime dependencies** — only the compile-time proc-macro stack (`syn`/`quote`). Everything heavier (serde, tokio/smol, rayon, wgpu) is strictly opt-in behind feature flags.

> **OrdoFP** - Functional programming toolbelt for Rust
>
> *"Ordo est parium dispariumque rerum sua cuique loca tribuens dispositio."*
> — Order is the disposition assigning to each thing its proper place. (St. Augustine)

OrdoFP provides powerful functional programming abstractions in Rust, using nomenclature rooted in Catholic scholastic philosophy.

**The name:** *Ordo* is Latin for *order* — the right arrangement of parts,
per the Augustine epigraph above — and **FP** stands for **functional
programming**. The library brings to Rust the discipline pioneered by Haskell
and refined across the functional-language family (PureScript, Scala, OCaml,
Koka): programs built by composing typed, law-abiding functions, ordered so
that each piece has its proper place.

## Why OrdoFP?

**In plain terms.** Most software defects come from parts of a program
affecting each other in ways nobody intended. OrdoFP is a toolkit for building
Rust programs out of small pieces that state, in their types, what they take
and what they produce — statements the compiler checks before the program
ever runs — and that combine into larger pieces which keep those guarantees. The
payoff is software that is easier to change without breaking: when the rules
are enforced by the machine instead of by developer discipline, refactoring
becomes routine instead of risky.

**In technical terms.** Rust has a strong type system but no higher-kinded
types, so the classic functional abstraction stack — Functor / Applicative /
Monad, optics, monad transformers, datatype-generic programming — is absent
from `std` and scattered across single-purpose crates where it exists at all.
OrdoFP provides that stack as one coherent library built on stable Rust's
GATs: HList-based generic programming, profunctor optics, monad transformers,
persistent collections, stack-safe recursion schemes, and opt-in
effect/async/parallel layers. Core typeclass instances are verified against
fifteen algebraic law families by a companion property-test crate
(`ordofp_laws`), and allocation and performance behavior are enforced by a
deterministic regression gate rather than promised in comments. Side-effect
tracking — making effects part of a function's stated interface — ships as
the opt-in row-typed `nexus` layer, still maturing (see the maturity notes
below).

**What you get:**

- **Fewer reachable states, fewer bugs** — encode invariants in types (HLists,
  linear types, validated values) so illegal states fail at compile time, not
  in production.
- **Refactor with confidence** — law-checked abstractions compose predictably;
  if it typechecks against a lawful interface, it behaves like the interface says.
- **Pay only for what you use** — zero runtime dependencies by default;
  serde, async runtimes, rayon, and GPU support are strictly opt-in features.
- **No nightly required** — builds on stable Rust (MSRV 1.97); the optional
  `nightly` feature adds codegen acceleration with identical semantics.

## Documentation

- **[Developer Guide](docs/guide.md)** - Main user manual and tutorial
- **[Canonical Reference](docs/reference.md)** - Complete API reference, cheatsheets, and tables
- **[Async Guide](docs/async.md)** - Async streams, fibers, and monad transformers
- **[Bayes Guide](docs/bayes.md)** - Probabilistic programming guide
- **[Glossarium](docs/glossary.md)** - Scholastic Latin naming convention
- **[Lineage & Version Notes](docs/migration.md)** - frunk lineage and the 2.x → 0.1.0 version reset

The full topic → file map (including feature flags, unsafe notes, and design decisions) is in **[docs/README.md](docs/README.md)**.

## Core Features

- **Data Structures**: HList (`Coniunctio`), Disiunctio (`Disiunctio`), `NonEmpty`, `Zipper`, Persistent Collections (PFDS)
- **Type Classes**: Functor, Applicatio, Monad, Compositio, Unitas (GAT-based)
- **Optics**: Lenses (`Aspectus`), Prisms (`Divisio`), Isos (`Aequivalentia`), Traversals (`Iteratio`), Affine (`IteratioAffinis`), indexed access (`Ad`/`Ix`), profunctor encoding
- **Universalis Programming**: Automatic struct-to-HList conversion
- **Effects & Async**: `Flumen` streams, `Fibra` structured concurrency, row-typed effects (`nexus`), Monad Transformers
- **Program Architecture**: Free monads, Church-encoded variants, recursion schemes, law-checking companion crate
- **Performance**: Stack-safe recursion, branch prediction hints, parallel execution (`ParFlumen`, `rayon`, `gpu-wgpu`)

## Feature Flags

Defaults: `std`, `derives`, `proc-macros`, `Probatum`. All other features (`serde`, `async`, `tokio`/`smol`, `fusion`, `linear`, `par`/`rayon`, `gpu-wgpu`, `transformers-cps`, `nexus`, …) are opt-in and additive — see **[docs/FEATURE_FLAGS.md](docs/FEATURE_FLAGS.md)** for the canonical matrix with descriptions and dependencies.

## Quick Start

Add to `Cargo.toml`:

```toml
[dependencies]
ordofp = "0.1"
```

For opt-in features (linear types, async, parallelism, etc.):

```toml
[dependencies]
ordofp = { version = "0.1", features = ["derives", "linear", "async", "par", "rayon"] }
```

### Simple Example

```rust
use ordofp::prelude::*;
use ordofp::monoid::combine_all;

fn main() {
    // Monadic chaining
    let result = Some(10)
        .flat_map(|x| if x > 5 { Some(x * 2) } else { None })
        .map(|x| x + 1);
    assert_eq!(result, Some(21));

    // Monoidal combination
    let v = vec![Some(1), Some(3)];
    assert_eq!(combine_all(&v), Some(4));

    // HList (Heterogeneous List)
    let h = hlist![1, "hello", true];
    let (head, tail) = h.pop();
    assert_eq!(head, 1);
}
```

See [docs/guide.md](docs/guide.md) for detailed usage examples.

## Component maturity

OrdoFP is at **v0.1.1**. The components below have settled APIs and full test coverage within this repository, but the usual 0.x semver caveats apply.

| Component | Maturity |
|-----------|----------|
| HList, Disiunctio | Mature |
| Type Classes | Mature |
| PFDS Collections | Mature |
| Optics | Mature |
| Async / Fibers | Mature |
| Linear Types | Mature |
| ParFlumen (Parallel) | Mature |

Not listed above: the row-typed effect system is an opt-in feature (`nexus`), and `ordofp_bayes` implements single-step SMC and single-site trace MCMC (see the scope notes in [docs/bayes.md](docs/bayes.md)).

## Contributing

Contributions are welcome — see [CONTRIBUTING.md](CONTRIBUTING.md) for prerequisites and workflow,
and the [Code of Conduct](CODE_OF_CONDUCT.md).

Verification is local and deterministic by design (no hosted CI): a green
`cargo run -p xtask -- all` — format, clippy `-D warnings`, all-features tests,
docs, stable cross-check, `cargo-deny`, wasm32 — is the merge bar.

## License

Apache License 2.0 - see [LICENSE](LICENSE) for details.

Adapted third-party code is inventoried in
[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
