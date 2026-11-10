# OrdoFP 0.2.0 alpha 1: validation and checked-domain contract

**Status:** Original alpha-1 design record (November 6, 2026). The design and
nonempty-error implementation landed on main through [PR #25](https://github.com/ordokr/ordofp/pull/25)
and [PR #26](https://github.com/ordokr/ordofp/pull/26) on November 9.
Typed locations are proposed in [PR #27](https://github.com/ordokr/ordofp/pull/27);
their Rust and performance verification is still pending.
**Design baseline:** main at d39fc633 (2026-10-29). Maintenance PR #21 was
unmerged at the time of writing and has since merged.
**Implementation reference:** [checked-domain example](../../examples/checked_domain_alpha.rs).

## Problem and intended result

At the original design baseline, public `Probatum<E, A>::Invalid(ErrorBuf<E>)` could contain zero errors: the enum exposes the buffer, `invalid_many` collects an empty iterator, and serde accepts an empty invalid list. Consequently, `is_invalid()` does not imply at least one diagnostic. `Probatum::map2` and iterator validation already accumulate independent failures; they should be reused, not replaced with a second validation engine.

The first 0.2 milestone targets nonempty invalid results, located diagnostics,
and checked construction for cross-field invariants. The nonempty representation
has now merged; the example demonstrates the checked domain workflow and is
updated by PR #27 to use the new typed diagnostic envelope.

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

**Alpha-2 implementation:** `Locatus<E>` (English `Located<E>`) uses an owned
`Vec<SegmentumViae>` of `Field(String)` and `Index(usize)` segments. Its
consuming `at_field` / `at_index` prefix operations do not clone the caller's
`E` payload. The example now uses the actual type. This is not a measured
allocation optimization: benchmark the owned path against unlocated errors,
and consider a static-field optimization only if real workloads justify it.
Do not introduce a path interning subsystem without evidence.

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

The alpha-1 nonempty representation is now implemented on main. The alpha-2
location implementation is under review in PR #27; its added tests have not
been run in the authoring environment. Run
`cargo test -p ordofp_core --lib --features Probatum,serde`,
`cargo test -p ordofp_core --lib --features Probatum-smallvec,serde`,
`cargo test -p ordofp --test generic_suite --features serde,Probatum`,
`cargo test --example checked_domain_alpha --features Probatum`, and
`cargo run -p xtask -- all` on the pinned toolchain. Record actual results,
exact MSRV (Rust 1.97), downstream compatibility, and located/unlocated
allocation measurements before declaring the implementation ready. The
repository has no hosted CI; no green gate is implied by this RFC.
