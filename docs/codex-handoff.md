# Codex handoff: OrdoFP 0.1.4 maintenance tranche

## Assignment

Implement the compatible 0.1.4 maintenance tranche from
[next-release-plan.md](next-release-plan.md), beginning with the public Apply
combinator defects. Complete the work in coherent, reviewable commits and return
the implementation with verification evidence. The 0.2.0 sections are the
follow-on roadmap; their representation changes and experimental features have
independent acceptance gates.

This handoff was prepared on 2026-10-28 against
`01578e1e2782bc1ea78c141ea778931c4e0c0715`. Revalidate each source finding against the checkout before changing
code. The research findings are static-review conclusions, not compiled
reproductions or performance proof.

## Read first

- Any applicable `AGENTS.md` present in the checkout. None existed at the research baseline.
- [CONTRIBUTING.md](../CONTRIBUTING.md), [policies.md](policies.md),
  [axioms.md](axioms.md), [FEATURE_FLAGS.md](FEATURE_FLAGS.md), and
  [glossary.md](glossary.md).
- The full [next-release plan](next-release-plan.md), especially the source findings,
  0.1.4 scope, and acceptance gates.
- The actual [xtask command implementation](../xtask/src/main.rs).

The current tree has no hosted workflow files. CONTRIBUTING and xtask specify
local verification; policies and axioms contain contradictory CI descriptions.
Use the executable local gate and reconcile this documentation drift. There is
no Codex-specific execution convention or RFC directory at the baseline.

## Work order

### 1. Reproduce and repair Apply semantics

Inspect [applicative.rs](../core/src/typeclasses/applicative.rs) and the concrete
Option, Result, Vec, and Probatum instances. The inherited `ap_left` default
discards the right operand, and `ap_right` discards the left. Reproduce the
observable defects before fixing them.

Required behavior:

- Option and Result preserve failure from either operand according to the
  underlying applicative semantics.
- Vec preserves Cartesian multiplicity and established result ordering,
  including either empty operand.
- Probatum preserves error accumulation and left-to-right error ordering.
- Derived helpers agree with the underlying applicative operation.
- Non-Clone values compile wherever the existing method contract permits them.

Preserve public trait signatures for the patch release. Do not silently add
stricter implementation bounds. For Vec, the left-preserving operation can repeat
left elements according to the right vector's length without cloning right
elements. For the right-preserving operation, verify what the existing
whole-vector Clone bound permits and measure allocation effects. If a sound
general correction needs a breaking signature, repair compatible concrete
instances, document the remaining limitation, and reserve the signature change
for 0.2.0.

Keep existing property-testing infrastructure and add focused regression/law
coverage for these actual semantics.

### 2. Complete existing composition bridges

Add `modify_option`, `modify_result`, and `modify_validated` to
`ComposedAspectus` in [optics/lens.rs](../core/src/optics/lens.rs).
Preserve the current composition return type in 0.1.4.

Verify successful updates preserve unrelated structure; failed updates preserve
the original source. Show a composed, validated update in a runnable example.
Arbitrary-depth optic representation changes belong to 0.2.0.

Broaden inherent `Probatum::map/map2/map3` callbacks to `FnOnce` where each is
invoked at most once. Remove unnecessary `Seq::map` Clone constraints after
checking the implementation. Include representative consuming closures and
non-Clone values.

### 3. Correct contract and maturity documentation

Keep the working free `sequence_option` / `sequence_result` functions.
The similarly named trait defaults have the wrong shape for genuine sequencing.
Use compatible deprecation and precise replacement guidance where a corrected
signature would break callers.

Distinguish working specialized Nexus computations from generic panic/stub paths.
Proof-like assertions must either enforce the claimed predicate compatibly or
be described accurately as metadata. A trait implemented for both true and false
cases supplies no such proof.

Document the actual async behavior: the current parallel-traversal helper awaits
sequentially; `ResAsync` does not invoke its stored release callback; the bracket
and fiber-finalizer paths have cancellation limitations. Do not imply that
dropping a future executes asynchronous cleanup. Restrict implementation fixes
in this tranche to those that preserve the existing public contract; the owned
scope and bounded-concurrency redesigns are 0.2.0 work.

Reconcile the hosted-CI documentation discrepancy. Update the Unreleased
changelog with concrete changes and remaining limitations.

## Compatibility and architecture constraints

- Preserve zero third-party runtime dependencies for the default build, stable
  Rust support, no_std/alloc feature boundaries, and Latin/English API parity.
- Keep the four-element inline ErrorBuf storage. Nonempty invalid-state
  representation and serde changes belong to 0.2.0.
- Build affected downstream path consumers before public reexports or visibility
  changes. If the consumers are unavailable, report that gate as blocked.
- Reuse existing dependencies and modules. Retain
  `examples/inline_probe.rs` and follow the standing performance rejections.
- Keep release publication and version/tag changes separate from this
  implementation task. 0.1.3 is already recorded in the manifests and changelog;
  the planned maintenance target is 0.1.4.
- New Nexus transport, inspectable Plan, collection-backend replacement,
  serialized validation changes, and incremental-engine work are outside the
  first maintenance tranche.

## Verification and completion evidence

Record the starting commit, compiler versions, relevant features, and any
pre-existing failures. Use the pinned development toolchain and verify stable
compatibility; do not reinterpret the nightly development pin as a consumer
nightly requirement.

For each commit touching code/config/scripts, run the repository's canonical
gate, one Cargo process at a time:

```sh
cargo run -p xtask -- all
```

The executable command includes the core gate, stable check, cargo-deny, wasm,
and feature matrix. For unsafe changes, additionally run:

```sh
cargo run -p xtask -- deep
```

Use `perf-guard` for affected workload paths. Any optimization claim needs
paired Criterion/hyperfine A/B results in both execution orders on the same
machine, with allocation/clone behavior reported. A correctness repair still
needs honest regression evidence; do not label it a measured speedup without
measurement.

Release-readiness checks are separate from `all`:

```sh
cargo run -p xtask -- semver
cargo package --workspace
```

Report unavailable tools, missing downstream consumers, and pre-existing gate
failures precisely. Do not mark a gate passed if it was skipped. The initial
planning commit changes Markdown only; no Rust tests or benchmark results are
claimed for it.

Return:

1. A concise account of the defects fixed and resulting user-visible behavior.
2. The implementation commits and changed files.
3. Actual verification commands and results, plus remaining blockers.
4. Any compatibility or allocation tradeoff that needs reviewer attention.
5. The next ready work item from the roadmap.

## Follow-on roadmap

After the maintenance tranche is reviewed, advance through these independent
checkpoints from the full plan:

1. 0.2.0 core validation, checked domain reconstruction, ownership, and optics.
2. Bounded async execution and a resource-owning scope with explicit cancellation
   and cleanup contracts.
3. Consuming persistent updates, admitted on unique/shared snapshot correctness
   and measured workload results.
4. A functioning one-shot Nexus kernel with real and test interpreters,
   compile-fail coverage, and explicit residual requirements.
5. An optional first-order selective Plan preview whose analyzer executes no
   operation nodes and whose executor skips unselected branches.
6. Release-candidate consumer, feature-matrix, semver/migration, packaging, and
   performance evidence.

Nexus and Plan promotion must not block the otherwise releasable core. Typed
deltas, broader effect parallelism, and backend changes remain later experiments
until their own correctness and value gates are met.
