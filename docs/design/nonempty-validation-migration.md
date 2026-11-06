# 0.2.0 alpha: nonempty validation error migration

This implementation is stacked on [validation-domains-alpha1.md](validation-domains-alpha1.md)
and intentionally introduces a breaking `Probatum` representation before the
0.2.0 version bump. **Do not merge into a 0.1.x release.**

## Changes

- `Probatum::Invalid` carries `NonEmptyErrors<E>`, a private-backed,
  `repr(transparent)` wrapper around the existing Vec or 4-inline SmallVec
  error store. An empty invalid collection is not safe to construct.
- `Probatum::invalid(error)` remains infallible.
- `Probatum::invalid_many(iter)` returns
  `Result<Probatum<E, A>, EmptyErrors>`, with `Err(EmptyErrors)` for no
  elements. This is a deliberate source break: callers must handle or justify
  the nonempty precondition.
- `Probatum::into_result()` now returns
  `Result<A, NonEmptyErrors<E>>`; `.errors()` still returns `Option<&[E]>`.
  This preserves the zero-copy owned error representation and makes the
  nonemptiness guarantee visible at the result boundary.
- `NonEmptyErrors::one`, `try_from_iter`, `as_slice`, `concat`,
  `map`, and consuming `IntoIterator` avoid unsafe or unchecked
  construction. The English gateway reexports the same types.
- Serde retains the prior `{"Invalid":["a","b"]}` wire format and order,
  but rejects `{"Invalid":[]}`. The `Valid` wire form is unchanged.

## Downstream source migration

| Former usage | New usage |
|---|---|
| `Probatum::Invalid(vec![])` | No replacement: identify missing diagnostic or return an ordinary success from the caller's domain logic |
| `Probatum::Invalid(vec![error])` | `Probatum::invalid(error)` |
| `Probatum::invalid_many(errors)` | `Probatum::invalid_many(errors)?` when propagating `EmptyErrors`, or `.expect("known nonempty")` when proven |
| `Probatum::Invalid(errors.collect())` | `Probatum::invalid_many(errors)` plus explicit empty handling |
| `let err: Vec<E> = validation.into_result().unwrap_err()` | `let err: NonEmptyErrors<E> = ...`; use `err.as_slice()` or `err.into_iter()` |
| `match Probatum::Invalid(errors)` | Pattern still works; the payload is `NonEmptyErrors<E>`, not Vec |

No compatibility shim silently manufactures an error, maps empty-invalid to
valid, or hides a panic. `ErrorBuf` stays private.

## Gates (must be executed before ready-for-review)

```sh
cargo test -p ordofp_core --lib --features Probatum
cargo test -p ordofp --test generic_suite --features serde,Probatum
cargo test -p ordofp --test algebra_suite --features Probatum
cargo test -p ordofp_core --lib --no-default-features --features alloc,Probatum
cargo test -p ordofp_core --lib --features Probatum,Probatum-smallvec,serde
cargo test --example checked_domain_alpha --features Probatum
cargo run -p xtask -- all
cargo run -p xtask -- perf-guard
cargo run -p xtask -- semver
cargo package --workspace
```

The semver check is expected to flag intended API breaks relative to 0.1.x
until the planned **0.2.0** version-bump/migration review. Perf baselines must
be compared to equivalent environments and never overwritten merely to turn a
failed gate green. Check both default Vec and the production SmallVec
configurations and compile real downstream consumers before integration.

The authoring environment cannot execute Cargo or install the pinned
toolchain. These checks remain **unverified**, not green.
