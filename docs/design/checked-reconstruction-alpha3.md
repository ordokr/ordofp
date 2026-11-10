# 0.2.0 alpha-3: checked domain reconstruction with composed optics

**Status:** Implementation PR, not yet a verified release gate.
**Independence:** Builds on `main` after merged PRs #21, #25, and #26.
PR #27 (typed locations) is still open, and is **not** needed here.

## Boundary

A lens is a generic getter/setter. It cannot know that a domain invariant
holds across two fields, and calling a lens setter directly on a refined type
does not constitute validation. A plain DTO, however, may safely represent
temporary invalid states while a user is editing it.

The minimal safe workflow is:

1. Start from a borrowed immutable domain value; explicitly derive a plain DTO.
2. Use normal or composed optics on the DTO only. Validate independently
   provided fields using `Probatum`, accumulating all such errors.
3. Call `.try_construct::<Domain>()` on the `Probatum<E, DTO>` result.
   This invokes the existing `Domain: TryFrom<DTO, Error = E>` **once** only
   if field validation succeeded; a domain error is returned as a single
   nonempty invalid result. Preexisting accumulated errors keep their order.
4. Publish or store the result only when `Valid(Domain)` is returned. On a
   rejected update, the original borrowed value remains unchanged.

```rust
// For Domain: TryFrom<Draft, Error = E>
let candidate: Probatum<E, Draft> =
    composed_lens.modify_validated(&draft, validate_field);
let domain: Probatum<E, Domain> = candidate.try_construct();
```

This is a thin, explicit boundary, **not** a new optic hierarchy or a
guarantee that arbitrary `TryFrom` implementations check their own
invariants. The domain's construction contract must be reviewed; its fields
should be private, and unchecked setters must not be exposed. Existing
`Refinatus::try_new` remains the preferred constructor for independent unary
predicates.

See [`checked_optic_update`](../../examples/checked_optic_update.rs)
for runnable code. In that example, a composed lens updates
`BatchDraft.window.start`. Invalid string input rejects before domain
construction; `start > end` fails the checked `TryFrom` conversion.
Neither error changes the source `CheckedBatch`.

## Tradeoffs and acceptance

- **No new runtime dependency or feature:** `try_construct` is a single
  method on the existing `Probatum` type behind its existing feature gate.
- **No global `Clone` bound:** the method consumes the DTO and its errors.
  A particular DTO snapshot may still require cloning its own fields,
  depending on the application's lens representation.
- **Accumulation vs dependent validation:** initial errors accumulate; the
  domain constructor is a deliberate fail-fast transition with at most one
  additional error. Do not claim that arbitrary cross-field errors accumulate.
- **State guarantees:** invalid input and invalid cross-field values never
  become published domain objects, and a failure leaves the borrowed source
  unchanged. The current composed lens still only composes two total lenses;
  arbitrary-depth optic representation is a separate design issue.
- **Validation evidence:** unit tests count checked-constructor invocations,
  exercise non-Clone errors, and check order preservation. The example tests
  valid, parse-failed, and invariant-failed nested updates. The full gate,
  exact MSRV, downstream builds, and allocation measurements must be reported
  from a Rust-equipped environment before merge or release.

## Verification commands

```sh
cargo fmt --all -- --check
cargo test -p ordofp_core --lib --features Probatum
cargo test -p ordofp_core --lib --no-default-features --features alloc,Probatum
cargo test --example checked_optic_update --features Probatum
cargo run --example checked_optic_update --features Probatum
cargo run -p xtask -- all
```

Do not merge or publish solely on static inspection.
