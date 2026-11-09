# OrdoFP 0.2.0 alpha 1: validation and checked-domain contract

**Status:** Proposed, with an executable application-level prototype; not a shipped core API.
**Base:** main at d39fc633 (2026-10-29). **Related:** [0.1.4 maintenance PR #21](https://github.com/ordokr/ordofp/pull/21), still unmerged when this decision was drafted.
**Implementation reference:** [checked-domain example](../../examples/checked_domain_alpha.rs).

## Problem and intended result

The current public `Probatum<E, A>::Invalid(ErrorBuf<E>)` can contain zero errors: the enum exposes the buffer, `invalid_many` collects an empty iterator, and serde accepts an empty invalid list. Consequently, `is_invalid()` does not imply at least one diagnostic. `Probatum::map2` and iterator validation already accumulate independent failures; they should be reused, not replaced with a second validation engine.

The first 0.2 milestone should make invalid values nonempty by construction, carry caller-defined errors with useful locations, and require a checked constructor for cross-field invariants. The runnable example proves the *workflow* against current main without claiming that these new representation guarantees already ship.

### End-to-end contract

1. Parse and validate independent input fields with accumulating `Probatum`, preserving field order and caller-specific error payloads.
2. When all fields succeed, form a plain parsed DTO and call `TryFrom<ParsedDto>` on an invariant-carrying domain type. Never silently map raw parsed fields directly into a public domain struct.
3. If any field failed, do not invoke the cross-field constructor; preserve all field errors. If fields passed but a cross-field rule failed, return the constructor's domain error.
4. Update an established domain value only through a checked method or another explicitly validated reconstruction boundary. An optic that exposes an unchecked setter must not be advertised as invariant-preserving.

## Decision A — nonempty invalid values (breaking 0.2.0 change)

Use a private-internals wrapper around the *existing* error buffer, conceptually `NonEmptyErrors<E>(ErrorBuf<E>)`. `Probatum<E,A>::Invalid` remains pattern-matchable but its payload is not freely constructible from an empty buffer. The English gateway can alias the new wrapper; assign its public scholastic name using the repository glossary before stabilizing. Do not introduce a parallel public validation enum.

Planned constructor/API behavior:

| API | 0.2.0 rule |
|---|---|
| `invalid(error)` | Infallible, exactly one initial error |
| `invalid_many(iter)` | Change to `Result<Probatum<E,A>, EmptyErrors>`; never panic on empty input or interpret it as valid |
| `NonEmptyErrors::one(error)` | Infallible constructor; internal buffer contains one error |
| `NonEmptyErrors::try_from_iter(iter)` | Fallible; returns `Err(EmptyErrors)` if empty |
| append / merge / map errors | Preserve order and nonemptiness, including 1+1 and 1+N cases |
| `errors()` / `into_result()` | Preserve read access and error order; update the owned error return type where required |

`NonEmpty` already exists in `core/src/nonempty.rs`, but replacing the buffer with a head-plus-tail list is **not** automatically acceptable: the `Probatum-smallvec` variant deliberately stores four elements inline. Prototype both layout options and retain a wrapper over `ErrorBuf` unless measured evidence justifies otherwise. Do not expose `Default`, an unchecked `From<Vec<E>>`, or a safe constructor that can form an empty error list.

**Serialization:** retain the current externally tagged `Valid`/`Invalid` JSON variant shape and flat error-array payload when feasible. Implement a validating deserializer for `Invalid`: `{"Invalid":[]}` must be rejected; nonempty lists must round-trip in order. Do not derive `Deserialize` on a buffer wrapper if the derive can bypass nonemptiness. Round-trip tests must cover both Vec-backed and SmallVec-backed feature sets. A wire break beyond rejecting the previously accepted invalid empty state needs its own migration note.

**Migration:** 0.1.x callers can currently construct `Invalid(vec![])` or call infallible `invalid_many`. The 0.2 change makes these compile-time or explicit result-handling failures. Document a codemod-style inventory of direct variant constructions, `.invalid_many(...)`, `.into_result()`, `match Invalid(...)`, and serde fixtures across the workspace and downstream consumers. Do not paper over an empty iterable with a fabricated error or a hidden panic.

## Decision B — located diagnostics

Target a generic `Located<E>` envelope so the payload remains typed, not coerced into a string. A path is ordered outermost to innermost, for example `Field("rows"), Index(4), Field("amount")`. A root/cross-field issue may have an empty path or a stable named scope. Field validation order and diagnostic path order are distinct and must not be conflated.

**Prototype boundary:** the example uses a lightweight `&'static str` field label to demonstrate attribution, not a new public location type. For the real public type, start with a simple owned sequence of `Field(String)` and `Index(usize)` segments so deserialization is straightforward and no borrowed lifetime leaks into wire data. Benchmark that against an optional static-field optimization before freezing the representation. Do not build a complex path interning system without measured need. Prefixing locations must not clone the caller's `E` payload.

Provide field/index prefix helpers and typed accessors; keep `map_err` consuming. Do not require `E: Clone` or `E: Display` for the core type. Serde support is feature-gated, with unambiguous nested paths and lossless index/field distinction.

## Decision C — checked reconstruction rather than unchecked generic conversion

`ParsedDto` is a transport shape, not proof of a domain invariant. The production domain type has private fields and a checked `TryFrom<ParsedDto>` boundary. The importer example tests an ordered integer window, where each endpoint parses independently but `start <= end` is only checked *after* both parse. Its borrowed `with_start` method validates a new window and leaves the original untouched on failure; it does not require the domain type to implement `Clone`.

Existing `Refinatus::try_new` and its checked `map`/`replace` methods should be reused where a unary predicate expresses an invariant. A multi-field invariant may need a domain-specific `TryFrom`; do not reimplement refinement derives or promise that arbitrary generic reconstruction enforces hidden invariants. Do not treat the unsafe `new_unchecked` constructor as part of a safe workflow.

## Follow-on PR slices and stop/go gates

| Slice | Smallest reviewable change | Must pass before acceptance |
|---|---|---|
| 0 — this PR | RFC, reproducible two-stage importer and tests, documentation navigation | Static review; executable proof needs the actual Rust toolchain |
| 1 — error representation | Private nonempty buffer, fallible multi-error constructor, serde guard, migration of internal pattern matches | Empty construction/serde rejected; single and multi-error order; `no_std + alloc`; both error buffer configurations; size/allocation measurements |
| 2 — locations | Generic `Located<E>` plus path segments and serde tests | Stable nested field/index round trips, no mandatory error cloning, ordered aggregation, overhead vs unlocated baseline |
| 3 — invariant-aware updates | Checked reconstruction adapters for domain types; composed optic integration only after PR #21 settles | Failed update keeps original, constructor called exactly once only after parse success, source untouched on any failure |
| 4 — release verification | Migrate facade, laws, examples and external consumers | Rust 1.97 MSRV, local xtask all, semver/migration review, package dry-run, downstream and paired perf gates |

The release does not depend on async scopes, Nexus handlers, a Plan preview, or a persistent collection rewrite. Keep those on independently gated milestones.

## Focused acceptance cases

- No safe API or serde input can produce `Invalid` with zero errors. Empty `invalid_many` returns a typed error.
- Both one and multiple failures preserve left-to-right accumulation and original payloads, including a deliberately non-`Clone` error.
- A nested path distinguishes named fields from indexed positions and survives serialization.
- Two independent bad endpoints produce two located parse errors; a successful parse followed by an inverted range yields only one domain error.
- The checked constructor is never called when parsing failed; valid ranges can be constructed and updated; rejected updates leave the original unchanged.
- `Invalid` layouts and allocations are measured separately for unlocated and located errors under Vec and SmallVec, without silently modifying the stored perf reference.
- The actual default dependency graph, feature matrix, and required release toolchain remain unchanged until an explicit reviewed decision.

## Verification status of this draft

This is a design PR, not a claim that 0.2.0's representation has been implemented. The prototype targets the public `Validated` alias and existing `map2` / `IntoValidated` methods on current main. Run `cargo test --example checked_domain_alpha --features Probatum`, `cargo run --example checked_domain_alpha --features Probatum`, then `cargo run -p xtask -- all` in a provisioned Rust environment. Record the actual results. Neither a hosted CI check nor a green local gate is implied by the presence of this RFC.
