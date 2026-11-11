# OrdoFP

```text
  ___               _            _____   ____
 / _ \   _ __    __| |   ___    |  ___| |  _ \
| | | | | '__|  / _` |  / _ \   | |_    | |_) |
| |_| | | |    | (_| | | (_) |  |  _|   |  __/
 \___/  |_|     \__,_|  \___/   |_|     |_|
```

[![crates.io](https://img.shields.io/crates/v/ordofp.svg)](https://crates.io/crates/ordofp)
[![docs.rs](https://img.shields.io/docsrs/ordofp)](https://docs.rs/ordofp)
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache--2.0-blue.svg)](LICENSE)
[![Rust: 1.97+](https://img.shields.io/badge/rust-1.97%2B-blue.svg)](https://www.rust-lang.org)

**OrdoFP is a functional programming library for Rust.** It gives you typed,
composable building blocks in one crate that builds on stable Rust. These
include type classes such as Functor and Monad, optics, heterogeneous lists,
validation that collects every error, persistent collections and monad
transformers.

## New here? Start with this

### What problem does it solve?

Rust has a strong type system, but it has no higher-kinded types. Because of
that, the classic functional toolkit is missing from the standard library.
That toolkit includes Functor, Applicative and Monad, optics, monad
transformers and generic programming over structs. Where these tools exist at
all, they are spread across many small crates that each do one thing.

OrdoFP brings that toolkit together in one coherent library. It is built on
generic associated types (GATs), so it works on stable Rust.

### Why does it exist?

Many software bugs come from parts of a program affecting each other in ways
nobody intended. OrdoFP helps you build programs from small pieces whose types
state what they take and what they return. The compiler checks those
statements before the program runs, and the pieces keep their guarantees when
you combine them. When the machine enforces the rules instead of developer
discipline, changing code becomes routine instead of risky.

### What do you get?

- **Core type classes:** Functor, Applicative (`Applicatio`), Monad,
  Semigroup (`Compositio`) and Monoid (`Unitas`), all based on GATs.
- **Data structures:** HLists (`Coniunctio`), sum types (`Disiunctio`),
  `NonEmpty`, `Zipper` and persistent collections.
- **Optics:** lenses, prisms, isos, traversals, affine traversals and indexed
  access, using a profunctor encoding.
- **Validation that collects errors:** `Probatum` gathers every error instead
  of stopping at the first one. Since 0.2.0, an invalid value always holds at
  least one error (`NonEmptyErrors<E>`).
- **Generic programming:** automatic conversion from structs to HLists.
- **Program architecture:** monad transformers, free monads, Church-encoded
  variants and stack-safe recursion schemes.
- **Opt-in extras:** async streams and structured concurrency, parallel
  execution and a row-typed effect system (`nexus`) that is still maturing.
- **Law checking:** the companion crate `ordofp_laws` tests the core type
  class instances against fifteen families of algebraic laws.
- **A small default build:** the default build has no runtime dependencies.
  serde, tokio, smol, rayon and wgpu are only added when you turn on a feature.

### Who is it for?

OrdoFP is for Rust developers who want Haskell-style or Scala-style functional
abstractions, checked at compile time, without switching to nightly Rust.

### What should you expect?

- **Maturity:** OrdoFP is a 0.x library. Most components have settled APIs and
  full test coverage (see [Component maturity](#component-maturity)), but the
  usual 0.x semver caveats apply. Read the [CHANGELOG](CHANGELOG.md) before you
  upgrade.
- **Latin names:** OrdoFP names its types with Latin terms from scholastic
  philosophy, such as `Disiunctio` for a sum type. Every Latin name also has a
  standard English alias in the [Vernacular API](core/src/vernacular/mod.rs),
  and the [Glossarium](docs/glossary.md) maps one to the other.

### Quick start

Add OrdoFP to your `Cargo.toml`:

```toml
[dependencies]
ordofp = "0.2"
```

Then try a small example:

```rust
use ordofp::hlist;
use ordofp::monoid::combine_all;
use ordofp::prelude::*;

fn main() {
    // Monadic chaining
    let result = Some(10)
        .flat_map(|x| if x > 5 { Some(x * 2) } else { None })
        .map(|x| x + 1);
    assert_eq!(result, Some(21));

    // Monoidal combination
    let v = vec![Some(1), Some(3)];
    assert_eq!(combine_all(&v), Some(4));

    // HList (heterogeneous list)
    let h = hlist![1, "hello", true];
    let (head, _tail) = h.pop();
    assert_eq!(head, 1);
}
```

To turn on optional features, such as linear types, async or parallelism, list
them in `Cargo.toml`:

```toml
[dependencies]
ordofp = { version = "0.2", features = ["derives", "linear", "async", "par", "rayon"] }
```

For a full walkthrough, read the [Developer Guide](docs/guide.md).

## Requirements

- **Rust:** stable Rust 1.97 or newer, edition 2024. The minimum supported
  Rust version (MSRV) is pinned for each release. A minor release may raise it
  so that it stays close to current stable Rust.
- **Nightly (optional):** the `nightly` cargo feature uses unstable Rust for
  better generated code, through branch prediction hints and `portable_simd`
  kernels. Behavior is the same with or without it.
- **Dependencies:** the default build has no runtime dependencies. It only
  uses the compile-time proc-macro stack (`syn` and `quote`). Heavier
  dependencies such as serde, tokio, smol, rayon and wgpu are added only by
  opt-in feature flags.

## Core features

- **Data structures:** HList (`Coniunctio`), `Disiunctio`, `NonEmpty`,
  `Zipper` and persistent collections (PFDS).
- **Type classes:** Functor, Applicatio, Monad, Compositio and Unitas, based on
  GATs.
- **Optics:** lenses (`Aspectus`), prisms (`Divisio`), isos (`Aequivalentia`),
  traversals (`Iteratio`), affine traversals (`IteratioAffinis`), indexed
  access (`Ad` and `Ix`) and a profunctor encoding.
- **Universalis programming:** automatic conversion from structs to HLists.
- **Effects and async:** `Flumen` streams, `Fibra` structured concurrency,
  row-typed effects (`nexus`) and monad transformers.
- **Program architecture:** free monads, Church-encoded variants, recursion
  schemes and a companion crate for law checking.
- **Performance:** stack-safe recursion, branch prediction hints and parallel
  execution (`ParFlumen`, `rayon` and `gpu-wgpu`).

## Feature flags

The default features are `std`, `derives`, `proc-macros` and `Probatum`. All
other features are opt-in and additive. They include `serde`, `async`, `tokio`,
`smol`, `fusion`, `linear`, `par`, `rayon`, `gpu-wgpu`, `transformers-cps` and
`nexus`, among others. See [docs/FEATURE_FLAGS.md](docs/FEATURE_FLAGS.md) for
the full matrix, with a description and the dependencies of each feature.

## Component maturity

OrdoFP is at **v0.2.0**. The components below have settled APIs and full test
coverage in this repository, but the usual 0.x semver caveats apply.

| Component | Maturity |
|-----------|----------|
| HList, Disiunctio | Mature |
| Type Classes | Mature |
| PFDS Collections | Mature |
| Optics | Mature |
| Async / Fibers | Mature |
| Linear Types | Mature (Affine discipline) |
| ParFlumen (Parallel) | Mature |

Two parts are not in the table. The row-typed effect system is an opt-in
feature (`nexus`). `ordofp_bayes` implements single-step SMC and single-site
trace MCMC. See the scope notes in [docs/bayes.md](docs/bayes.md).

Some opt-in APIs have known limits:

- In Nexus, the specialized state, reader and error computations run, but
  generic operations still include panic stubs.
- The parallel-named Vec traversal in the async layer runs sequentially.
- `ResAsync` does not call its stored release callback.

Read the [maturity boundaries](docs/FEATURE_FLAGS.md#async-and-nexus-maturity-boundaries)
before you choose these APIs.

## Documentation

- **[Developer Guide](docs/guide.md):** the main user manual and tutorial.
- **[Canonical Reference](docs/reference.md):** the complete API reference,
  with cheatsheets and tables.
- **[Vernacular API](core/src/vernacular/mod.rs):** standard English aliases
  for all Scholastic types and traits.
- **[Async Guide](docs/async.md):** async streams, fibers and monad
  transformers.
- **[Bayes Guide](docs/bayes.md):** probabilistic programming.
- **[Glossarium](docs/glossary.md):** the Scholastic Latin naming convention,
  with Latin and English name mappings.
- **[Core Principles and Axioms](docs/axioms.md):** design axioms drawn from
  audits and real refactoring work.
- **[Feature Flags Matrix](docs/FEATURE_FLAGS.md):** the canonical list of
  features and their dependencies.
- **[Lineage and Version Notes](docs/migration.md):** the frunk lineage and the
  version reset from 2.x to 0.1.0.

The map of every topic to its file, including feature flags, unsafe notes and
design decisions, is in **[docs/README.md](docs/README.md)**.

## The name

*Ordo* is Latin for *order*, meaning the right arrangement of parts. **FP**
stands for **functional programming**. The library brings to Rust the
discipline that Haskell pioneered and that the wider family of functional
languages (PureScript, Scala, OCaml and Koka) refined. Programs are built by
composing typed functions that obey laws, and each piece has its proper place.

> *"Ordo est parium dispariumque rerum sua cuique loca tribuens dispositio."*
>
> Order is the disposition assigning to each thing its proper place.
> (St. Augustine)

## Contributing

Contributions are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md) for the
prerequisites and workflow, and read the [Code of Conduct](CODE_OF_CONDUCT.md).

Verification is automated and deterministic. The merge bar, both locally and in
GitHub Actions CI, is one command:

```text
cargo run -p xtask -- all
```

It runs the format check, strict Clippy with `-D warnings`, the all-features
test suite, rustdoc, a stable MSRV check, `cargo-deny`, a wasm32 build and the
12-configuration feature matrix.

## License

OrdoFP is licensed under the Apache License 2.0. See [LICENSE](LICENSE) for
details.

Adapted third-party code is listed in
[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).

## Support the work

If OrdoFP saves you time or helps you build more reliable Rust software, please
consider [sponsoring my work](https://github.com/sponsors/ordokr). Your support
helps fund maintenance, reliability improvements, documentation and practical
examples.

Thank you for helping sustain independent development.

Tim V