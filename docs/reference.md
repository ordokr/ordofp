# OrdoFP 0.1.0 Documentation

> *"Ordo est parium dispariumque rerum sua cuique loca tribuens dispositio."*
> — Order is the disposition assigning to each thing its proper place. (St. Augustine)

## Canonical Reference

> **Version:** 0.1.0 | **Toolchain:** stable Rust ≥ 1.97 (optional `nightly` feature for extra codegen) | **Edition:** 2024

OrdoFP provides a comprehensive suite of functional programming abstractions for Rust, designed for performance, correctness, and composability.

### Table of Contents

- [Core Types](#core-types)
- [Type Classes](#type-classes)
- [Optics](#optics)
- [Data Structures](#data-structures)
- [Monad Transformers](#monad-transformers)
- [Effects & Async](#effects--async)
- [Performance Primitives](#performance-primitives)
- [Macros & Syntax](#macros--syntax)
- [Cheatsheets](#cheatsheets)

---

## Core Types

### HList (Coniunctio)

Statically typed heterogeneous lists.

| Op | Signature | Example |
|----|-----------|---------|
| `hlist![]` | Create | `hlist![1, "hi", true]` |
| `.pop()` | `→ (H, Tail)` | `let (head, rest) = h.pop()` |
| `.pluck::<T>()` | `→ (T, Rest)` | `let (b, rest): (bool, _) = h.pluck()` |
| `.sculpt::<Target>()` | `→ (Target, Rest)` | `let (sub, _): (HList![i32], _) = h.sculpt()` |

**Scholastic names:** `Nihil` (empty), `Coniunctio` (cons).

### Disiunctio

Type-safe sum types (tagged unions).

| Op | Signature | Example |
|----|-----------|---------|
| `Disiunctio![]` | Type macro | `type C = Disiunctio!(i32, String)` |
| `::inject(v)` | `T → Disiunctio` | `C::inject(42i32)` |
| `.get::<T>()` | `→ Option<&T>` | `co.get::<i32>()` |
| `.fold(hlist![])` | `→ R` | `co.fold(hlist![f_int, f_str])` |

### Probatum

Accumulates errors using applicative combination (`liftN`).

```rust
let validation = Probatum::lift3(
    |name, age, email| Person { name, age, email },
    parse_name(s).into_probatum(),
    parse_age(s).into_probatum(),
    parse_email(s).into_probatum(),
);
```

### Aut (Either)

Disjunction with `Sinister` (Left) and `Dexter` (Right) variants.

---

## Type Classes

Based on GATs (Generalized Associated Types).

```
Functor → Apply → Applicatio → Monad
```

| Trait | Method | Impl for |
|-------|--------|----------|
| **Functor** | `.map(f)` | Option, Result, Vec |
| **Apply** | `.apply(f_wrapped)` | Option, Result, Vec |
| **Applicatio** | `::pure_target(v)` | Option, Result, Vec |
| **Monad** | `.flat_map(f)` | Option, Result, Vec |
| **Compositio** | `.combine(&other)` | String, Vec, Option, tuples |
| **Unitas** | `::empty()` | String, Vec, Option, tuples |

---

## Optics

Composable accessors for immutable data.

### Aspectus (Lens) — Product Focus

```rust
let name = aspectus(
    |p: &Person| p.name.clone(),
    |p, n| Person { name: n, ..p.clone() },
);
name.set(&person, "Bob".into());
```

### Divisio (Prism) — Sum Focus

```rust
let circle = divisio(
    |s: &Shape| match s { Shape::Circle(r) => Some(*r), _ => None },
    Shape::Circle,
);
circle.preview(&shape);
```

### Aequivalentia (Iso) — Bidirectional

Isomorphic transformation between types.

---

## Data Structures

### Persistent (PFDS)

Immutable structures with structural sharing. Safe for concurrency and "time travel".

- **Stack**: LIFO, O(1).
- **Queue**: FIFO, amortized O(1).
- **Deque**: Double-ended, O(1).
- **Seq**: Random access sequence, O(log n).
- **OrdMap**: AVL-based ordered map, O(log n).
- **OrdSet**: AVL-based ordered set, O(log n).

### NonEmpty

List guaranteed to have at least one element.

```rust
let nel = NonEmpty::new(1, vec![2, 3]);
assert_eq!(nel.head(), &1); // Safe!
```

### Zipper

Functional cursor into a list with O(1) focus operations.

```rust
let z = Zipper::new(3, vec![1, 2], vec![4, 5]); // [1, 2] <3> [4, 5]
let z = z.focus_next().unwrap();
```

---

## Monad Transformers

Stack effects cleanly.

| Type | Stack | Purpose |
|------|-------|---------|
| `OptionT<M>` | `M<Option<A>>` | Optionality |
| `EitherT<M,E>` | `M<Result<A,E>>` | Error handling |
| `ReaderT<R,M>` | `R → M<A>` | Dependency Injection |
| `StateT<S,M>` | `S → M<(A,S)>` | State |
| `Scriptor<W,A>` (Writer; alias `LogScriptor<A>` = `Scriptor<Vec<String>, A>`) | `(A, W)` | Accumulated output |
| `ContinuatioT<R,A>` | `(A → R) → R` | Continuations (plain continuation monad) |

---

## Effects & Async

### Flumen (Streams)

Async reactive streams with rich combinators.

```rust
let sum = Flumen::from_iter(1..=5)
    .fmap(|x| x * 2)
    .fold(0, |acc, x| acc + x)
    .await;
```

### Fibra (Fibers)

Lightweight structured concurrency.

```rust
let (a, b) = Fibra::par(
    async { compute_a().await },
    async { compute_b().await }
).await;
```

### Linear Types

Enforce single-use semantics for resources (`ordofp_core::linear`, feature
`linear`). Multiplicity-tracked types (`Qtt`, `FunctioLinearis`) live in the
sibling `ordofp_core::quantitative` module — see [guide.md](guide.md).

```rust
let res: Linearis<File> = Linearis::new(file);
res.consume(|f| f.close()); // Must be called exactly once
```

---

## Performance Primitives

- **TailRec**: Stack-safe recursion via trampolining.
- **Hints**: `likely`, `unlikely`, `hot_path`, `cold_path`.
- **Free Monads**: `Liber` (standard), `LiberEcclesia` (Church-encoded, O(n)).
- **ParFlumen**: Data-parallel execution (Rayon/GPU backends).

---

## Macros & Syntax

| Macro | Usage | Description |
|-------|-------|-------------|
| `hlist![...]` | `hlist![1, "a", true]` | Create heterogeneous list |
| `coniunctio_pat![...]` | `let coniunctio_pat![a, b] = h` | Pattern match HList |
| `HList![...]` | `type T = HList![i32, bool]` | Define HList type |
| `Disiunctio!(...)` | `type T = Disiunctio!(A, B)` | Define Disiunctio type |
| `mdo! { ... }` | `mdo! { x <- ma; mb }` | Monadic do-notation |
| `mdo_async! { ... }` | `mdo_async! { let x = await fut; ... }` | Async do-notation (`bind`/`await`/`pure`) |
| `pipe!(...)` | `pipe!(a, f, g)` | Forward pipe `g(f(a))` |
| `compose!(...)` | `compose!(g, f)` | Function composition `g(f(x))` |

---

## Cheatsheets

### Error Resolution

| Error | Fix |
|-------|-----|
| `Universalis not satisfied` | `#[derive(Universalis)]` |
| `expected HList, found tuple` | `.into()` |
| `convert_from not found` | Use `Universalis` not `NominataUniversalis`, or `transform_from` |

### Method Selection

| Goal | Use |
|------|-----|
| Struct → HList | `into_universalis(s)` |
| HList → Struct | `from_universalis(h)` |
| Struct → Struct | `convert_from` / `transform_from` |
| Extract type | `.pluck::<T>()` |
| Extract subset | `.sculpt::<Target>()` |

### Feature Flags

See **[FEATURE_FLAGS.md](FEATURE_FLAGS.md)** for the canonical matrix (descriptions, dependencies, defaults).

---
*See [guide.md](guide.md) for detailed tutorials.*
