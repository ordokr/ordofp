# Core Principles & Architectural Axioms of OrdoFP

These axioms and design principles were derived from adversarial audits, contrarian analysis, and empirical refactoring of the OrdoFP ecosystem. They govern all future design, implementation, and maintenance in this repository.

---

## Axiom 1: The Principle of Survivor Synergy (Cross-Subsystem Compounding)

> *"Abstractions multiply in value when their compositions require zero glue code."*

1. **Anti-Isolation Rule**: Core functional building blocks (`Validated`, `HList`, `Universalis`, `Optics`, `Monoid`, `PFDS`) must not exist as disconnected academic novelties.
2. **Native Composition Bridges**: Every core survivor subsystem must provide native combinators connecting with its sibling subsystems:
   - `HList::sequence_validated()` bridges `HList` with `Validated` (Probatum).
   - `.map(from_universalis)` bridges `HList` with `Universalis` for zero-boilerplate struct synthesis.
   - `Aspectus::modify_validated()` bridges `Optics` (Lenses) with `Validated` for deep invariant enforcement.
   - `IteratorValidateExt::validate_all()` bridges standard iterators and collections with `Validated`.
3. **Synergy Over Monolithic Sprawl**: Prefer deepening and polishing high-leverage connections between proven survivors over introducing speculative new subsystems.

---

## Axiom 2: The Principle of Vernacular Parity (Semantic Dual-Gateway)

> *"Scholastic precision informs the architecture; vernacular accessibility powers adoption."*

1. **Dual-Gateway Architecture**:
   - **Scholastic Latin** (`Probatum`, `Aspectus`, `Compositio`, `Unitas`, `Coniunctio`, `Universalis`) preserves philosophical precision, historical lineage, and eliminates naming collisions with ubiquitous domain identifiers.
   - **Vernacular English** (`Validated`, `Lens`, `Semigroup`, `Monoid`, `Cons`, `Either`) provides an immediate, zero-friction on-ramp for developers accustomed to conventional FP terminology.
2. **First-Class Parity**:
   - Vernacular names must not be second-class aliases or afterthought re-exports.
   - Core types must provide inherent methods (`.map()`, `.map_err()`, `.into_validated()`) so that vernacular workflows require zero trait acrobatics.
   - `ordofp::vernacular::*` and `ordofp::prelude::*` are fully maintained, documented, and tested.

---

## Axiom 3: The Principle of Total Feature Orthogonality

> *"Verification must test what users compile, not merely the omnibus union."*

1. **The Omnibus Trap**: Building only with `--all-features` masks serious defects, including:
   - Implicit standard library dependencies in nominal `no_std` crates.
   - Broken `no-default-features` configurations.
   - Unused variable, missing doc comment, or unreachable visibility warnings under isolated flags.
2. **Matrix Enforcement**:
   - Every crate must compile cleanly and pass verification under `--no-default-features`, minimal feature subsets (`alloc`, `std`), and isolated additive flags (`async`, `linear`, `par,rayon`, `nexus`).
   - The workspace feature matrix is continuously enforced via `cargo run -p xtask -- matrix` and wired directly into the root gate and CI.

---

## Axiom 4: The Principle of Honest Semantics & Anti-Drift Doctrine

> *"State what the compiler guarantees, not what the abstraction aspires to."*

1. **Semantic Honesty**:
   - Never claim compiler-enforced linear types when providing *advisory affine discipline* via Rust's move semantics. Always document escape hatches (`Clone`, `Copy`, `Deref`, `drop`).
   - Clearly delineate production-hardened primitives (e.g. `Probatum`, `HList`, `Aspectus`, `PFDS`) from experimental research modules (`nexus`, `distributed`).
2. **Anti-Drift Discipline**:
   - Documentation and code comments must never drift to speculative future versions (e.g. `2.0`, `3.0`) ahead of the canonical crate version (`0.1`).
   - Documentation must always match executable, verified examples.

---

## Axiom 5: The Principle of Dual-Layer Verification (Local Machine Ownership + Hosted CI)

> *"Local execution guarantees reproducible developer velocity; hosted CI guarantees transparent open-source trust."*

1. **Machine-Owned Local Gate**:
   - All verification steps (format, strict Clippy `-D warnings`, full test suite, doctests, rustdoc, stable MSRV check, `cargo-deny`, `wasm32`, and feature matrix) are encapsulated in the local `xtask` driver (`cargo run -p xtask -- all`).
   - Any developer on any machine can run the exact canonical gate without external orchestration dependencies.
2. **Transparent Hosted CI**:
   - Hosted CI (`.github/workflows/ci.yml`) executes the identical `xtask` targets on remote runners on every commit and PR.
   - Remote CI serves as a public seal of correctness and prevents platform-specific drift.
