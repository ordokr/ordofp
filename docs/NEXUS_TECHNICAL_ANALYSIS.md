# OrdoFP Nexus: Technical Analysis and Credible Roadmap

> A skeptical, delivery-focused revision of the Nexus effect system roadmap.

---

## 1. Scope & Architecture Decision

### Decision: (A) Pure Rust Stable Library

> **Status note:** shipped as designed — the crate builds on stable Rust
> (MSRV 1.97); the remaining unstable-Rust codegen paths (branch hints,
> `portable_simd`) live behind the opt-in `nightly` cargo feature with
> identical semantics.

**Justification:**

1. **Stable ecosystem integration** - Works with any Rust project without toolchain modifications
2. **Lower barrier to adoption** - No custom build steps, cargo subcommands, or compiler plugins
3. **Maintenance burden** - Proc macros are tractable; maintaining a custom compiler or IR interpreter is not for a small team
4. **6-12 month timeline** - Library-only approach is the only realistic option

**What is NOT possible under these constraints:**

| Feature | Status | Reason |
|---------|--------|--------|
| True compile-time specialization based on effect rows | LIMITED | Requires unstable `specialization` or manual dispatch |
| Automatic insertion of optimizations | NO | Library cannot inject code into user functions |
| Cross-function effect analysis | NO | Would require compiler integration |
| Assembly-level guarantees | BENCHMARK ONLY | Can measure but not enforce |
| "Zero-cost" as a compile-time proof | NO | Claims must be validated empirically |
| Region inference | NO | Would require borrow checker extensions |
| Automatic parallelization of user code | NO | User must opt-in via explicit combinators |

**What IS possible:**

- Type-level effect tracking via const generics and traits
- Compile-time effect row operations (union, intersection, subset checks)
- Explicit handler selection with inline hints
- Typed combinators that guide users toward optimizable patterns
- Benchmark infrastructure proving specific patterns match hand-written code
- Proc macros for ergonomic effect syntax (not semantic analysis)

---

## 2. Unified Effect Story

### Relationship to Existing Eff/Sem System

The codebase has an existing effect system in `src/effects/` with:
- `Eff<R, A>` - Effectful computation type
- `Sem` - Semantic handler approach
- Evidence-based effect handling (Koka/Polysemy inspired)
- Runtime effect tag validation

**Decision: Nexus is a NEW BACKEND REPRESENTATION, not a replacement.**

```
                    ┌─────────────────────────────────────┐
                    │         User-Facing API             │
                    │  (Eff<R, A>, do-notation macro)     │
                    └─────────────────────────────────────┘
                                    │
                    ┌───────────────┴───────────────┐
                    ▼                               ▼
          ┌─────────────────┐             ┌─────────────────┐
          │  Legacy Backend │             │  Nexus Backend  │
          │  (dynamic tags) │             │ (const Universalis) │
          └─────────────────┘             └─────────────────┘
                    │                               │
                    ▼                               ▼
          Runtime dispatch            Compile-time specialization
          (flexible, slower)          (restricted, faster)
```

### Migration Strategy

1. **Phase 0** (Current): Both systems coexist with separate namespaces (`effects::` vs `nexus::`)
2. **Phase 1**: Feature flag `nexus` enables new backend; `legacy` keeps old
   *(as shipped: no `legacy` feature was created — the existing `effects::` backend
   stays always-on and `nexus` is the only opt-in flag; see `docs/FEATURE_FLAGS.md`)*
3. **Phase 2**: Adapter layer allowing gradual migration:
   ```rust
   // Convert legacy Eff to Nexus Eff for hot paths
   fn optimize_hot_path<R: LegacyRow>(eff: legacy::Eff<R, A>) -> nexus::Eff<R::Nexus, A>
   ```
4. **Phase 3**: Document migration patterns; deprecate legacy in favor of Nexus for new code
5. **Phase 4** (6+ months): Consider removing legacy if adoption warrants

### Compatibility Layer

```rust
/// Trait implemented by legacy effect rows that have Nexus equivalents
pub trait NexusCompatible: LegacyEffectRow {
    type NexusRow: nexus::EffectRow;

    fn to_nexus<A>(eff: legacy::Eff<Self, A>) -> nexus::Eff<Self::NexusRow, A>;
    fn from_nexus<A>(eff: nexus::Eff<Self::NexusRow, A>) -> legacy::Eff<Self, A>;
}
```

---

## 3. Phase-by-Phase Revision

### Phase 1: Measurably Efficient Effect Representations

**Renamed Objective:** Provide effect representations that benchmark within 10% of hand-written equivalents for common patterns.

**Concrete Acceptance Criteria:**
- [ ] `Eff<Pure, A>` unwraps to `A` with zero overhead (verified via `#[repr(transparent)]` + assembly inspection)
- [ ] `Eff<State<S>, A>` compiles to state-passing that matches hand-written `fn(S) -> (A, S)` within 5% on Criterion benchmarks
- [ ] `Eff<Reader<E>, A>` compiles to environment-passing matching `fn(&E) -> A` within 5%
- [ ] `Eff<Error<E>, A>` matches `Result<A, E>` performance exactly (should be isomorphic)
- [ ] No heap allocation for single-effect rows in release builds (verified via DHAT or similar)

**MVP:**
- `Eff<R, A>` with const-Universalis effect rows
- Handlers for State, Reader, Error, Writer
- Benchmark suite comparing to hand-written equivalents
- `#[inline(always)]` hints on hot paths

**Deferred:**
- IO effect (requires external world modeling)
- Effect combination beyond 2-3 effects
- Proc macro syntax sugar

**Key Risks:**
| Risk | Mitigation |
|------|------------|
| Const generics may not enable full specialization | Use enum dispatch as fallback; document when specialization applies |
| Boxing for complex effect combinations | Accept this for >3 effects; optimize common cases |
| Compile times for large effect rows | Limit row to 64 bits (64 effect types); use lazy trait evaluation |

---

### Phase 2: Typed Optimization Combinators

**Renamed Objective:** Provide explicit combinators that enable optimizations when users opt-in.

**Concrete Acceptance Criteria:**
- [ ] `par_map` over `Eff<Pure, _>` produces correct results
- [ ] `par_map` fails to compile if effect row is non-pure
- [ ] Memoization combinators cache correctly for idempotent effects
- [ ] Speculative execution combinators terminate correctly
- [ ] Each combinator has property-based tests

**MVP (COMPLETED):**
- `IsPure`, `IsIdempotent`, `IsTotal` marker traits
- `par_map`, `par_traverse`, `par_sequence` (type-safe parallelization)
- `memoize`, `Lazy`, `Thunk` (caching infrastructure)
- `speculative`, `race`, `first_success`, `fallback_chain`, `retry` (speculative execution)
- `fuse`, `compose_maps`, `filter_map_fused` (fusion combinators)

**Deferred:**
- Actual parallelization (requires rayon integration)
- Automatic memoization insertion (not possible without compiler support)
- Commutativity proofs beyond documentation

**Semantics Definitions:**

| Concept | Definition | Enforcement |
|---------|------------|-------------|
| Pure | Effect row = 0 (no effects) | Compile-time: `R::BITS == 0` |
| Idempotent | Same result on repeated execution | Type assertion: `R::BITS ⊆ {0, READER_BIT}` |
| Total | Always terminates | Type assertion: `R::BITS ∩ {ERROR_BIT, IO_BIT} = ∅` |
| Commutative | Order doesn't matter | Trait bound: `E1: EffectCommutes<E2>` |

**Key Risks:**
| Risk | Mitigation |
|------|------------|
| Users may not understand when to use combinators | Extensive documentation + examples |
| Property definitions may be wrong | Property-based testing; formal review |
| Performance regressions from abstraction | Benchmark every combinator against naive version |

---

### Phase 3: Handler Correctness Levels

**Renamed Objective:** Provide layered verification from documentation to optional formal proofs.

**Tier Structure:**

| Tier | What it means | How it's verified | User benefit |
|------|--------------|-------------------|--------------|
| 0 | Laws documented | Doc comments | Understanding |
| 1 | Laws tested | `#[cfg(test)]` property tests | Confidence |
| 2 | Runtime contracts | `debug_assertions` + checks | Debug-time errors |
| 3 | Static verification | External tool (Prusti, Kani) | Formal guarantee |
| 4 | Proof extraction | Coq/Lean | Academic publication |

**MVP:**
- Tier 0: All handlers have laws documented
- Tier 1: QuickCheck-style property tests for handler laws
- Tier 2: `VerifiedHandler` trait with runtime law checking

**Deferred:**
- Tier 3-4: Requires significant investment; track as research

**Concrete Acceptance Criteria:**
- [ ] Every handler has docstrings stating its laws
- [ ] `proptest` suite covers handler laws for State, Reader, Error, Writer
- [ ] `VerifiedHandler<E>` trait exists with runtime checking
- [ ] Documentation explains what each tier guarantees

---

### Phase 4: Incremental Computation Effects

**Renamed Objective:** Provide dependency-tracking primitives for change propagation.

**MVP:**
- `Incremental` effect type with `read_input` and `memo` operations
- Simple invalidation-based recomputation
- Integration with existing memoization

**Deferred:**
- Differential dataflow integration
- Automatic incrementalization
- Cross-computation dependency graphs

**Acceptance Criteria:**
- [ ] Incremental effect compiles and runs
- [ ] Changing an input invalidates dependent cached values
- [ ] Benchmark shows recomputation avoidance

---

### Phase 5: Temporal and Session Types

**Renamed Objective:** Provide protocol-level effect types for ordered interactions.

**MVP:**
- `Session<Protocol>` effect type
- Send/Receive/Close operations
- Compile-time protocol state checking

**Deferred:**
- LTL modalities (Eventually, Always, Until)
- Async integration
- Temporal property verification

**Acceptance Criteria:**
- [ ] Session type example compiles with correct protocol
- [ ] Incorrect protocol usage fails at compile time
- [ ] Documentation explains session type encoding

---

### Phase 6: Probabilistic Effects

**Renamed Objective:** Provide PPL primitives with handler-selectable inference.

**MVP:**
- `Sample`, `Observe`, `Score` operations
- Simple MCMC handler
- Integration with existing random infrastructure

**Deferred:**
- SMC handler
- Variational inference
- Automatic differentiation integration
- GPU acceleration

**Acceptance Criteria:**
- [ ] Bayesian model compiles with probabilistic effects
- [ ] MCMC handler produces samples
- [ ] Results match reference implementation (Stan, PyMC)

---

### Phase 7: Computation Migration

**Renamed Objective:** Provide serializable computation checkpoints.

**Design Option 1: Async State Machine Serialization**

```rust
#[derive(Serialize, Deserialize)]
struct Checkpoint<A> {
    state: SerializedState,
    resume_point: ResumeMarker,
    _result: PhantomData<A>,
}

// Constraints:
// - No borrowed data (must be 'static or owned)
// - All state must be Send + Serialize
// - Resume points must be explicit yield points
```

**Limitations:**
- Cannot checkpoint arbitrary Rust code
- User must mark explicit checkpoint boundaries
- Borrowed data must be cloned or excluded

**Design Option 2: Defunctionalized Workflow IR**

```rust
// User writes workflow in restricted DSL
workflow! {
    let x = fetch_data(key).await;
    checkpoint!();
    let y = process(x).await;
    checkpoint!();
    save_result(y).await
}

// Compiles to:
enum WorkflowStep {
    FetchData { key: Key },
    Process { x: Data },
    SaveResult { y: Result },
}
```

**Limitations:**
- Restricted to workflow DSL
- Cannot use arbitrary Rust expressions
- Requires proc macro

**MVP:**
- Design Option 1 with explicit checkpoints
- State serialization via serde
- Resume from checkpoint

**Deferred:**
- Cross-machine migration
- Automatic checkpoint insertion
- Distributed handlers

---

### Phase 8: Effect-Scoped Regions

**Renamed Objective:** Provide region-based allocation via effect scopes (research).

**Reality Check:** True region inference requires compiler integration. What we can do:
- Explicit region markers via effect types
- Arena allocation tied to effect handler lifetime
- Documentation of region patterns

**MVP:**
- `Region<'r>` effect type
- `with_region` combinator allocating arena
- Examples showing region-based patterns

**Deferred (Likely Never):**
- Region inference
- Compiler integration
- Automatic region boundary detection

---

## 4. Disproof-Driven Risk Register

| # | Risk | What would prove this wrong? | Mitigation / Alternate Path |
|---|------|------------------------------|---------------------------|
| 1 | **Compile-time blowup** from const Universalis effect rows | Compile times >30s for reasonable programs | Limit effect row to 64 bits; lazy trait evaluation; profile and optimize |
| 2 | **Trait coherence complexity** - orphan rules prevent user effect types | Users cannot define new effects without forking | Use sealed trait pattern; provide `custom_effect!` macro with reserved bits |
| 3 | **Macro hygiene issues** - `do!` macro conflicts with user code | Macro produces unhygienic identifiers; shadowing bugs | Use `tt` muncher with explicit scoping; extensive macro tests |
| 4 | **Debugging/stack traces** - effect handlers produce unreadable traces | Stack traces show only handler internals | Add `#[track_caller]`; create debug formatters; document debugging patterns |
| 5 | **Ecosystem adoption** - no one uses it | <100 downloads/month after 6 months | Focus on solving real problems; integrate with popular crates; write tutorials |
| 6 | **Soundness hole** - effect system can be bypassed unsafely | `safe` code can violate effect guarantees | Minimize unsafe; fuzz testing; security audit before 1.0 |
| 7 | **Performance regression** for complex effect combinations | >3 effects in row performs 10x worse than hand-written | Accept this as documented limitation; optimize common 2-effect combinations |
| 8 | **API instability** - breaking changes every release | Users stop upgrading | Semantic versioning; deprecation warnings; migration guides |
| 9 | **Async integration** complexity | Cannot compose with tokio/async-std | Provide async adapters; accept some effects are sync-only |
| 10 | **Documentation debt** - too complex to understand | GitHub issues all about "how do I..." | Invest in docs, examples, tutorials; hire technical writer |
| 11 | **Competitor emerges** - better Rust effect library appears | Another crate gets more adoption | Focus on differentiators; collaborate vs compete |
| 12 | **Rust language changes** - breaking changes to const generics | Nexus fails to compile on new Rust | Track nightly; participate in RFC process; have fallback impl |

---

## 5. Reframing Controversial Areas

### A) Phase 2 Optimization

**Library-Only Reframe:**

The optimization module provides **typed combinators that users explicitly invoke**. There is no automatic optimization. The type system enforces preconditions.

```rust
// User explicitly opts into parallel execution
// Type system ensures this is safe (Pure effect)
let results = par_map(&items, |x| pure(x * 2));

// This would NOT compile:
// let results = par_map(&items, |x| state_modify(|s| s + x));
// Error: Row<STATE_BIT> does not implement ParallelSafe
```

**Semantic Definitions:**

| Property | Type-Level Definition | Runtime Meaning |
|----------|----------------------|-----------------|
| Pure | `R::BITS == 0` | No observable side effects |
| Idempotent | `R::BITS & !READER_BIT == 0` | f(x) = f(f(x)) for same environment |
| Total | `R::BITS & (ERROR_BIT \| IO_BIT) == 0` | Always terminates with value |
| Commutative | `E1: EffectCommutes<E2>` | Order of execution doesn't affect result |

**Enforcement:**
- **Safe by default**: Marker traits only implemented for known-safe combinations
- **No unsafe overrides**: Users cannot bypass checks without modifying library
- **Clear errors**: Compile errors explain why optimization is refused

---

### B) Phase 7 Migration

**Design 1: Async State Machine Serialization**

```rust
/// Checkpoint a running computation
pub trait Checkpointable: Sized {
    type State: Serialize + DeserializeOwned + Send + 'static;

    fn checkpoint(&self) -> Self::State;
    fn resume(state: Self::State) -> Self;
}

/// Constraints that must be satisfied
/// - All captured state must be 'static + Send + Serialize
/// - No borrowed references
/// - Explicit checkpoint boundaries
```

**MVP Features:**
- `#[derive(Checkpointable)]` for simple state machines
- Manual `checkpoint()` calls at yield points
- Serde-based serialization

**Limitations:**
- Cannot checkpoint arbitrary closures
- Borrows must be removed before checkpoint
- User responsible for checkpoint placement

**Design 2: Defunctionalized Workflow IR**

```rust
/// Workflow definition macro
macro_rules! workflow {
    ($($body:tt)*) => { /* generates IR */ }
}

/// IR representation
#[derive(Serialize, Deserialize)]
enum WorkflowIR<S> {
    Step { action: Action, next: Box<Self> },
    Checkpoint { state: S, next: Box<Self> },
    Done,
}
```

**MVP Features:**
- `workflow!` macro for simple sequential workflows
- Automatic checkpoint at each step boundary
- Serializable IR representation

**Limitations:**
- Restricted syntax (no arbitrary Rust)
- Must use workflow-specific primitives
- Limited control flow

---

### C) Phase 3 Verification

**Tier 0: Laws as Documentation**

```rust
/// # Laws
///
/// 1. Pure law: `handle(pure(x)) = pure(x)`
/// 2. Bind law: `handle(m.and_then(f)) = handle(m).and_then(|x| handle(f(x)))`
/// 3. Specific: `run_state(get(), s) = (s, s)`
impl<S> Handler<State<S>> for StateHandler<S> { ... }
```

**What "verified" means:** Documentation exists. No guarantee it's correct.

**Tier 1: Runtime Property Tests**

```rust
#[cfg(test)]
mod handler_laws {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn state_pure_law(x: i32, s: i32) {
            let handled = run_state(pure(x), s);
            prop_assert_eq!(handled, (x, s));
        }
    }
}
```

**What "verified" means:** Laws pass property-based tests. May still fail on edge cases.

**Tier 2: Debug Runtime Contracts**

```rust
pub trait VerifiedHandler<E: Effect>: Handler<E> {
    fn verify_pure_law(&self) -> bool;
    fn verify_bind_law(&self) -> bool;
}

#[cfg(debug_assertions)]
fn checked_handle<H: VerifiedHandler<E>, E, R, A>(handler: H, eff: Eff<R, A>) -> ... {
    debug_assert!(handler.verify_pure_law(), "Handler violates pure law");
    handler.handle(eff)
}
```

**What "verified" means:** Laws checked at runtime in debug builds. Production still trusts implementation.

**Tier 3: Static Verification (Future)**

```rust
// Using Prusti or Kani for verification
#[requires(true)]
#[ensures(result == (value, state))]
fn run_state_pure<S, A>(value: A, state: S) -> (A, S) {
    (value, state)
}
```

**What "verified" means:** Static analyzer proves property. Limited to what tool can express.

**Tier 4: Proof Extraction (Research)**

```coq
(* Coq theorem *)
Theorem state_handler_pure_law : forall (A S : Type) (x : A) (s : S),
  run_state (pure x) s = (x, s).
Proof. reflexivity. Qed.
```

**What "verified" means:** Mathematical proof in theorem prover. Maximum assurance but high cost.

---

## 6. Updated Roadmap

| Phase | Goal | MVP Deliverable | Tooling | Exit Criteria | Drop Criteria |
|-------|------|-----------------|---------|---------------|---------------|
| **1** | Efficient effect representations | Eff type + handlers for State/Reader/Error/Writer | None | Benchmarks within 10% of hand-written | >20% overhead after optimization attempts |
| **2** | Typed optimization combinators | par_*, memoize, speculative, fusion | None | Type-safe combinators compile; tests pass | Users find API unusable |
| **3** | Handler correctness tiers | Tier 0-2 verification | proptest | Laws documented + tested | No user demand for verification |
| **4** | Incremental computation | Incremental effect + invalidation | None | Demo showing recomputation avoidance | Cannot express useful incrementality |
| **5** | Session types | Session effect + protocol checking | Macros | Compile-time protocol enforcement | Type-level encoding too complex |
| **6** | Probabilistic effects | Sample/Observe + MCMC handler | None | Samples match reference implementation | Cannot integrate with existing PPL ecosystem |
| **7** | Computation migration | Explicit checkpoints + serialization | derive macro | Checkpoint/resume works across process | Serde limitations too restrictive |
| **8** | Effect regions | Region markers + arena integration | None | Memory savings demonstrated | No way to make ergonomic without compiler |

### Timeline (6-12 months)

| Month | Milestone |
|-------|-----------|
| 1-2 | Phase 1 complete (done) + Phase 2 complete (done) |
| 3-4 | Phase 3 Tier 0-1 + Phase 4 MVP |
| 5-6 | Phase 5 MVP + Phase 6 MVP |
| 7-9 | Phase 3 Tier 2 + Phase 7 MVP |
| 10-12 | Phase 8 exploration + documentation + 1.0 prep |

---

## Conclusion

This revision replaces aspirational claims with measurable goals, explicit constraints, and clear drop criteria. The core insight: **we're building a library, not a compiler**. This means:

1. Optimizations require user opt-in via explicit combinators
2. Verification is tiered from documentation to optional formal proofs
3. Migration requires explicit checkpoints, not magic
4. Region inference is out of scope without compiler integration

The resulting system will be less magical but more honest, more maintainable, and actually shippable within the given timeline.
