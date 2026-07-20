# OrdoFP Developer Guide

> *"Nomina sunt consequentia rerum."*
> — Names are the consequence of things. (Justinian)

OrdoFP is a functional programming library for Rust that brings Haskell-level abstractions with a distinctive Scholastic Latin naming convention.

**Version:** 0.1.0 | **Toolchain:** stable Rust ≥ 1.97 (optional `nightly` feature for extra codegen) | **Edition:** 2024

---

## Table of Contents

1. [Getting Started](#getting-started)
2. [Why Use OrdoFP](#why-use-ordofp)
3. [Core Concepts](#core-concepts)
4. [Type Classes](#type-classes)
5. [Data Structures](#data-structures)
6. [Monad Transformers](#monad-transformers)
7. [Optics](#optics)
8. [Async Programming](#async-programming)
9. [Effects System](#effects-system)
10. [Recursion Schemes](#recursion-schemes)
11. [Linear Types](#linear-types)
12. [Free Monads](#free-monads)
13. [Category Theory](#category-theory)
14. [Common Patterns](#common-patterns)
15. [Quick Reference](#quick-reference)

---

## Getting Started

### Installation

```toml
[dependencies]
ordofp = "0.1"
# For specific features:
# ordofp = { version = "0.1", features = ["async", "linear", "par", "serde"] }
```

### Feature Flags

See **[FEATURE_FLAGS.md](FEATURE_FLAGS.md)** for the canonical matrix (descriptions, dependencies, defaults).

### Basic Usage

```rust
use ordofp::prelude::*;
use ordofp_core::{hlist, HList, mdo, compose, pipe};

// HList - heterogeneous list
let h = hlist![1, "hello", true];

// Probatum - accumulating errors
let result = parse_name(input).into_probatum()
           + parse_age(input)
           + parse_email(input);

// Monadic chaining
let x = Some(10)
    .flat_map(|n| safe_div(n, 2))
    .map(|n| n * 3);

// Do-notation
let result = mdo! {
    x <- Some(10);
    y <- Some(5);
    Some(x + y)
};
```

---

## Why Use OrdoFP

### Practical Benefits by Feature

| Feature | Problem It Solves | Real-World Benefit |
|---------|-------------------|-------------------|
| **HList** | Losing type info with `Vec<Box<dyn Any>>` | Store mixed types with full compile-time safety |
| **Disiunctio** | Runtime type checks for "one of many" | Compiler forces handling all cases |
| **Universalis** | Boilerplate for struct conversions | Auto-convert between structs, serialize Universalisally |
| **Probatum** | `Result` fails on first error | Collect ALL validation errors at once |
| **Optics** | Verbose nested struct updates | One-liner deep updates without mutation |
| **PFDS** | Defensive cloning, lock contention | Cheap copies, safe concurrency, undo/redo |
| **Transformers** | Callback hell with stacked effects | Clean composition of Reader+State+Error+Async |
| **Free Monads** | Untestable side effects | Describe programs as data, interpret differently |
| **Algebraic Effects** | Hard-coded dependencies | Swap implementations without code changes |
| **Linear Types** | Resource leaks, use-after-free | Compiler guarantees exactly-once usage |
| **Recursion Schemes** | Stack overflow, repeated patterns | Safe recursion, automatic optimizations |

For a problem → feature decision table, see the [Quick Reference](#quick-reference) at the end of this guide.

---

## Core Concepts

### HList (Coniunctio)

Heterogeneous lists that preserve type information at compile time.

**Practical Benefit**: Store different types in one list without losing type information. Unlike `Vec<Box<dyn Any>>`, all types are known at compile time.

**Use Cases**: Configuration objects, database rows, API responses with mixed types, Universalis programming.

**Type:** `HList![T1, T2, T3, ...]`

**Operations:**

| Operation | Signature | Description |
|-----------|-----------|-------------|
| `hlist![]` | Create | `hlist![1, "hi", true]` |
| `.pop()` | `-> (H, Tail)` | Extract head and tail |
| `.pluck::<T>()` | `-> (T, Rest)` | Extract by type |
| `.sculpt::<Target>()` | `-> (Target, Rest)` | Extract subset |
| `.map(fns)` | `-> HList` | Map with per-element functions |
| `.foldl(fns, init)` | `-> R` | Left fold |
| `.foldr(fns, init)` | `-> R` | Right fold |
| `.to_ref()` | `-> HList of refs` | Borrow all elements |
| `.into_reverse()` | `-> HList` | Reverse the list |

**Example:**

```rust
use ordofp_core::{hlist, HList};

let h: HList![i32, &str, bool] = hlist![42, "hello", true];

// Destructure
let (head, tail) = h.pop();  // head: 42, tail: HList![&str, bool]

// Pluck by type
let (boolean, rest): (bool, _) = h.pluck();  // boolean: true

// Sculpt subset
let (subset, remainder): (HList![i32, bool], _) = h.sculpt();

// Map with per-element functions
let mapped = h.to_ref().map(hlist![
    |&n| n * 2,
    |&s| s.len(),
    |&b| !b
]);  // hlist![84, 5, false]
```

### Disiunctio (Disiunctio)

Type-safe sum types (tagged unions).

**Practical Benefit**: Handle "one of these types" without runtime type checks. The compiler forces you to handle all cases, eliminating missed variant bugs.

**Use Cases**: API responses (success/error/redirect), state machines, event systems, protocol messages.

**Type:** `Disiunctio!(T1, T2, T3, ...)`

**Operations:**

| Operation | Description |
|-----------|-------------|
| `Disiunctio::inject(v)` | Inject value into Disiunctio |
| `.get::<T>()` | Get value if it matches type |
| `.fold(hlist![...])` | Pattern match via fold |

**Example:**

```rust
use ordofp_core::{hlist, Disiunctio};

type Response = Disiunctio!(Success, NotFound, ServerError);

let resp: Response = Disiunctio::inject(Success { data: "ok" });

// Pattern match via fold
let message = resp.fold(hlist![
    |s: Success| format!("Success: {}", s.data),
    |_: NotFound| "Not found".to_string(),
    |e: ServerError| format!("Error: {}", e.code),
]);
```

### Universalis Derivation

Automatic conversion between structs and HLists.

**Practical Benefit**: Write Universalis code that works on any struct shape. Convert between compatible structs automatically without manual field mapping.

**Use Cases**: Serialization, ORM mapping, form handling, API adapters, struct migrations.

**Derive:** `#[derive(Universalis)]`

**Functions:**

| Function | Description |
|----------|-------------|
| `ordofp::into_universalis(s)` | Struct -> HList |
| `ordofp::from_universalis(h)` | HList -> Struct |
| `ordofp::convert_from(s)` | Struct -> Struct (same fields) |
| `ordofp::transform_from(s)` | Struct -> Struct (with transformation) |

**Example:**

```rust
use ordofp::Universalis;

#[derive(Universalis, Debug)]
struct Person { name: String, age: u32 }

#[derive(Universalis, Debug)]
struct Employee { name: String, age: u32, department: String }

// Convert struct to HList
let person = Person { name: "Alice".into(), age: 30 };
let h: HList![String, u32] = ordofp::into_universalis(person);

// Convert HList to struct
let new_person: Person = ordofp::from_universalis(h);

// Convert between compatible structs
let employee = Employee { name: "Bob".into(), age: 25, department: "Eng".into() };
let person: Person = ordofp::convert_from(employee);
```

### Probatum

Accumulate multiple errors instead of short-circuiting.

**Practical Benefit**: Instead of "name invalid" → fix → "email invalid" → fix → "age invalid", get ALL errors at once: `["name invalid", "email invalid", "age invalid"]`. Users fix everything in one pass.

**Use Cases**: Form validation, config file parsing, batch operations, data import.

**Type:** `Probatum<T, E>`

**Operators:**

| Operator | Description |
|----------|-------------|
| `.into_probatum()` | Convert `Result` to `Probatum` |
| `Probatum::lift2(f, a, b)` | Applicative combine of two validations |
| `Probatum::collect(iter)` | Sequence/collect many validations |
| `.into_result()` | Convert back to `Result` |
| `.map(f)` | Map over success value |

**Example:**

```rust
use ordofp::Probatum;

fn parse_name(s: &str) -> Result<String, String> {
    if s.is_empty() { Err("Name empty".into()) }
    else { Ok(s.to_string()) }
}

fn parse_age(s: &str) -> Result<u32, String> {
    s.parse().map_err(|_| "Invalid age".into())
}

fn parse_email(s: &str) -> Result<String, String> {
    if s.contains('@') { Ok(s.to_string()) }
    else { Err("Invalid email".into()) }
}

let validation: Probatum<String, _> = Probatum::lift3(
    |name, age, email| Person { name, age, email },
    parse_name(name_input).into_probatum(),
    parse_age(age_input).into_probatum(),
    parse_email(email_input).into_probatum(),
);

match validation.into_result() {
    Ok(person) => println!("Valid person: {:?}", person),
    Err(errors) => {
        for e in errors { eprintln!("Error: {}", e); }
    }
}
```

### Aut (Either)

Disjunction type with Scholastic naming.

**Type:** `Aut<L, R>`

**Variants:**
- `Aut::Sinister(L)` - Left value
- `Aut::Dexter(R)` - Right value

**Operations:**

| Operation | Description |
|-----------|-------------|
| `Aut::sinister(v)` | Create left |
| `Aut::dexter(v)` | Create right |
| `.is_sinister()` | Check if left |
| `.is_dexter()` | Check if right |
| `.map_sinister(f)` | Map left value |
| `.map_dexter(f)` | Map right value |
| `.bimap(f, g)` | Map both sides |
| `.fold(f, g)` | Eliminate to single type |

**Example:**

```rust
use ordofp_core::datatypes::Aut;

let left: Aut<i32, String> = Aut::sinister(42);
let right: Aut<i32, String> = Aut::dexter("hello".into());

// Pattern matching
match left {
    Aut::Sinister(n) => println!("Left: {}", n),
    Aut::Dexter(s) => println!("Right: {}", s),
}

// Mapping
let mapped = right.map_dexter(|s| s.len());  // Aut::Dexter(5)

// Fold
let result = left.fold(|n| n.to_string(), |s| s);  // "42"
```

---

## Type Classes

### Type Class Hierarchy

```
Functor -> Apply -> Applicatio -> Monad
    |
    v
Foldable -> Traversable
    |
    v
Alternative (MonadPlus)
```

(`Applicatio` is the canonical Latin name for Applicative — see
[glossary.md](glossary.md) for the full naming convention. `Compositio`
(Semigroup) and `Unitas` (Monoid) sit alongside this hierarchy; see
[reference.md](reference.md).)

**Practical Benefit**: Write code once, use with Option, Result, Vec, Future, and any custom type. The same `.map()`, `.flat_map()`, `.traverse()` work everywhere.

### Functor

Transform values inside a context without unwrapping/rewrapping.

**Practical Benefit**: One method works across all container types. No need for separate `option.map()` vs `result.map()` vs `vec.iter().map()` patterns.

**Method:** `.map(f)` or `.fmap(f)`

```rust
let doubled = Some(21).map(|x| x * 2);  // Some(42)
let parsed: Result<i32, _> = Ok(21).map(|x| x * 2);  // Ok(42)
```

### Apply

Apply wrapped functions to wrapped values.

**Practical Benefit**: Combine independent computations. Fetch user AND posts in parallel, combine when both done.

**Methods:**
- `.apply(wrapped_f)` - Apply wrapped function
- `::map2(fa, fb, f)` - Combine two values

```rust
let f: Option<fn(i32) -> i32> = Some(|x| x * 2);
let result = Some(21).apply(f);  // Some(42)

let sum = Option::map2(Some(10), Some(32), |a, b| a + b);  // Some(42)
```

### Applicatio (Applicative)

Lift values into context.

**Methods:**
- `::pure(v)` or `::pure_target(v)` - Lift value

```rust
let wrapped = Option::pure(42);  // Some(42)
```

### Monad

Sequential composition with context.

**Practical Benefit**: Chain operations where each step depends on the previous. If any step fails, the whole chain short-circuits cleanly.

**Methods:**
- `.flat_map(f)` or `.bind(f)` - Monadic bind

```rust
let result = Some(10)
    .flat_map(|x| Some(x * 2))
    .flat_map(|x| if x > 15 { Some(x) } else { None });
```

**Do-notation with `mdo!`:**

```rust
use ordofp_core::mdo;

let result = mdo! {
    x <- Some(10);
    y <- Some(x * 2);
    z <- if y > 15 { Some(y) } else { None };
    Some(z + 1)
};
```

### Monad Laws

| Law | Expression |
|-----|------------|
| Left Identity | `pure(a).flat_map(f) == f(a)` |
| Right Identity | `m.flat_map(pure) == m` |
| Associativity | `m.flat_map(f).flat_map(g) == m.flat_map(\|x\| f(x).flat_map(g))` |

### Foldable

Reduce structures to a single value.

**Methods:**
- `.fold_left(init, f)` - Left fold
- `.fold_right(init, f)` - Right fold
- `.fold_map(f)` - Fold with monoid

```rust
use ordofp_core::foldable::Foldable;

let list = vec![1, 2, 3, 4, 5];
let sum = list.fold_left(0, |acc, x| acc + x);  // 15
```

### Traversable

Traverse structures with effects.

**Practical Benefit**: Flip container nesting. Turn `Vec<Option<T>>` into `Option<Vec<T>>`. Perfect for "all must succeed" batch operations.

**Methods:**
- `.traverse(f)` - Apply effectful function to each element
- `.sequence()` - Flip nested structures

```rust
use ordofp_core::traversable::Traversable;

let strings = vec!["1", "2", "3"];
let parsed: Option<Vec<i32>> = strings.traverse(|s| s.parse().ok());
// Some(vec![1, 2, 3])

let options = vec![Some(1), Some(2), Some(3)];
let sequenced: Option<Vec<i32>> = options.sequence();
// Some(vec![1, 2, 3])
```

### Alternative

Choice and failure handling.

**Practical Benefit**: Try fallbacks automatically. `cache.get(key).alt(|| db.get(key)).alt(|| default())` - clean fallback chains.

**Methods:**
- `::empty()` - Identity for choice
- `.or_else(f)` - Try alternative on failure

```rust
use ordofp_core::alternative::Alternative;

let empty: Option<i32> = Alternative::empty();  // None
let result = None.or_else(|| Some(42));  // Some(42)
```

---

## Data Structures

### Persistent Functional Data Structures (PFDS)

All structures are immutable with structural sharing.

**Practical Benefit**: Copies are nearly free (they share structure). Safe for concurrency without locks. Previous versions remain valid - perfect for undo/redo and time-travel debugging.

```rust
let map1 = OrdMap::new().insert("a", 1);
let map2 = map1.insert("b", 2);  // map1 unchanged, shares memory with map2
// Both map1 and map2 are valid and usable
```

**Module:** `ordofp_core::pfds`

| Structure | Type | Description |
|-----------|------|-------------|
| `Stack<T>` | LIFO | Push/pop from top |
| `Queue<T>` | FIFO | Enqueue back, dequeue front |
| `Deque<T>` | Double-ended | Push/pop from both ends |
| `Seq<T>` | Balanced tree | O(log n) random access sequence |
| `OrdMap<K, V>` | Balanced tree (AVL) | Key-value map |
| `OrdSet<T>` | Balanced tree (AVL) | Unique values |

**Stack Example:**

```rust
use ordofp_core::pfds::Stack;

let stack = Stack::empty()
    .push(1).push(2).push(3);
let (top, rest) = stack.pop().unwrap();  // top=3
```

**Queue Example:**

```rust
use ordofp_core::pfds::Queue;

let queue = Queue::empty()
    .enqueue(1).enqueue(2).enqueue(3);
let (front, rest) = queue.dequeue().unwrap();  // front=1
```

**OrdMap Example:**

```rust
use ordofp_core::pfds::OrdMap;

let map = OrdMap::empty()
    .insert("a", 1)
    .insert("b", 2);
let value = map.get(&"a");  // Some(&1)
```

### NonEmpty

Lists guaranteed to have at least one element.

**Type:** `NonEmpty<T>`

```rust
use ordofp_core::nonempty::NonEmpty;

let ne = NonEmpty::new(1, vec![2, 3, 4]);

let first = ne.head();      // &1 — a reference (no Option needed)
let min = ne.minimum();     // 1
let max = ne.maximum();     // 4
```

---

## Monad Transformers

Stack effects with monad transformers.

**Practical Benefit**: Combine multiple effects cleanly. Instead of `Result<Option<Result<T, E1>>, E2>` nightmare, use composable layers. Access config AND state AND handle errors in one clean computation.

```rust
// Without transformers: deeply nested, hard to compose
fn process() -> Result<Option<Result<Data, DbError>>, ConfigError> { ... }

// With transformers: flat, composable
fn process() -> ReaderT<Config, EitherT<Option, Error, Data>> { ... }
```

**Module:** `ordofp_core::transformers`

| Transformer | Purpose |
|-------------|---------|
| `OptionT<M, A>` | Optional values in M |
| `EitherT<M, E, A>` | Errors in M |
| `ReaderT<R, M, A>` | Dependency injection |
| `StateT<S, M, A>` | Stateful computation |
| `Scriptor<W, A>` (Writer; alias `LogScriptor<A>`) | Logging/accumulation |
| `ContinuatioT<R, A>` (plain continuation monad) | Continuations |

### ReaderT

Dependency injection via environment.

```rust
use ordofp_core::transformers::ReaderT;

struct Config { db_url: String, timeout: u64 }

fn get_user(id: u64) -> ReaderT<Config, Option<User>> {
    ReaderT::new(|config: &Config| {
        // Use config.db_url to fetch user
        Some(User { id, name: "Alice".into() })
    })
}

// Run with environment
let config = Config { db_url: "...".into(), timeout: 30 };
let user = get_user(1).run(&config);
```

### StateT

Stateful computations.

```rust
use ordofp_core::transformers::StateT;

fn increment() -> StateT<i32, Option<()>> {
    StateT::modify(|s| s + 1)
}

let computation = increment()
    .flat_map(|_| increment())
    .flat_map(|_| StateT::get());

let (result, final_state) = computation.run(0).unwrap();
// result = 2, final_state = 2
```

### Scriptor (Writer)

Logging/accumulation alongside computation. (`W` must implement `Unitas`/Monoid.)

```rust
use ordofp_core::transformers::Scriptor;

fn log_op(msg: &str, value: i32) -> Scriptor<Vec<String>, i32> {
    Scriptor::new(vec![msg.to_string()], value)
}

let computation = log_op("Starting", 10)
    .flat_map(|x| log_op("Doubling", x * 2));

let (logs, result) = computation.run();
// result = 20, logs = ["Starting", "Doubling"]
```

---

## Optics

Composable getters/setters for nested data.

**Practical Benefit**: Update deeply nested immutable data in one line. No more verbose `User { address: Address { city: new_city, ..user.address }, ..user }` patterns.

```rust
// Without optics: verbose, error-prone
let updated = User {
    address: Address {
        city: "New York".to_string(),
        ..user.address
    },
    ..user
};

// With optics: one line, composable
let updated = user_address_city_lens.set(&user, "New York".to_string());
```

**Module:** `ordofp_core::optics`

| Optic | Latin Name | Focus | Can Fail |
|-------|------------|-------|----------|
| Lens | Aspectus | Single field | No |
| Prism | Divisio | Variant | Yes |
| Iso | Aequivalentia | Bidirectional | No |
| Traversal | Iteratio | Multiple | N/A |
| Affine | IteratioAffinis | 0 or 1 | Yes |

### Lens (Aspectus)

Focus on a single field.

```rust
use ordofp_core::optics::{aspectus, Aspectus};

#[derive(Clone)]
struct Person { name: String, address: Address }

#[derive(Clone)]
struct Address { city: String }

let name_lens = aspectus(
    |p: &Person| p.name.clone(),
    |p, name| Person { name, ..p.clone() }
);

let city_lens = aspectus(
    |a: &Address| a.city.clone(),
    |a, city| Address { city, ..a.clone() }
);

// Compose lenses
let address_lens = aspectus(
    |p: &Person| p.address.clone(),
    |p, address| Person { address, ..p.clone() }
);
let person_city = address_lens.compose(city_lens);

// Use
let city = person_city.get(&person);           // Get
let updated = person_city.set(&person, "LA");  // Set
let modified = person_city.modify(&person, |c| c.to_uppercase());  // Modify
```

### Prism (Divisio)

Focus on one variant of a sum type.

```rust
use ordofp_core::optics::{divisio, Divisio};

enum Shape {
    Circle { radius: f64 },
    Rectangle { width: f64, height: f64 },
}

let circle_prism = divisio(
    |s: &Shape| match s {
        Shape::Circle { radius } => Some(*radius),
        _ => None,
    },
    |radius| Shape::Circle { radius }
);

let radius: Option<f64> = circle_prism.preview(&shape);  // May fail
let new_circle = circle_prism.review(10.0);              // Always succeeds
```

### Iso (Aequivalentia)

Bidirectional transformation.

```rust
use ordofp_core::optics::{aequivalentia, Aequivalentia};

let celsius_fahrenheit = aequivalentia(
    |c: &f64| c * 9.0 / 5.0 + 32.0,
    |f: &f64| (f - 32.0) * 5.0 / 9.0
);

let fahrenheit = celsius_fahrenheit.forward(&100.0);   // 212.0
let celsius = celsius_fahrenheit.backward(&212.0);     // 100.0
```

### Traversal (Iteratio)

Focus on multiple elements.

```rust
use ordofp_core::optics::{iteratio_each, Iteratio};

let list_traversal = iteratio_each::<Vec<i32>, i32>();

let numbers = vec![1, 2, 3, 4, 5];
let all: Vec<i32> = list_traversal.get_all(&numbers);
let doubled = list_traversal.modify_all(&numbers, |x| x * 2);
```

---

## Async Programming

**Practical Benefit**: Functional async with composable operators. Chain, combine, and transform async operations with the same patterns used for sync code.

> Canonical async documentation (full worked examples, async type classes,
> `pipe_async!`/`compose_async!`/`chain_async!`, runtime integration):
> **[async.md](async.md)**. This section is the overview.

### Futurus (Async Future Wrapper)

**Module:** `ordofp_core::async_core`

```rust
use ordofp_core::async_core::Futurus;

// Create
let f = Futurus::purus(42);

// Transform
let result = Futurus::purus(10)
    .fmap(|x| x * 2)
    .flat_map(|x| Futurus::purus(x + 1))
    .await;  // 21

// Combine
let sum = Futurus::map2(
    Futurus::purus(10),
    Futurus::purus(32),
    |a, b| a + b
).await;  // 42
```

### Flumen (Async Stream)

**Practical Benefit**: Process streaming data with functional operators. Filter, transform, take, and fold streams reactively.

```rust
use ordofp_core::async_core::Flumen;

let stream = Flumen::from_iter(vec![1, 2, 3, 4, 5]);

let results: Vec<i32> = stream
    .fmap(|x| x * 2)
    .filter(|x| *x > 4)
    .take(3)
    .collect_vec()
    .await;

let sum = Flumen::from_iter(1..=10)
    .fold(0, |acc, x| acc + x)
    .await;  // 55
```

### Stateful Stream Operations

Introduced in 2.0: `scan` (running accumulation — unlike `fold`, yields each
intermediate state), `scan_with` (state type may differ from output; return
`None` to terminate early), and `chunks` (fixed-size batches). Examples:
[async.md → Stateful Stream Operations](async.md#stateful-stream-operations).

### Async Monad Transformers

**Module:** `ordofp_core::transformers::async_transforms`

| Transformer | Purpose |
|-------------|---------|
| `LectorAsync<E, A>` | Async reader |
| `StatusAsync<S, A>` | Async state |
| `OptionTAsync<A>` | Async option |
| `EitherTAsync<E, A>` | Async either |
| `ScriptorAsync<W, A>` | Async writer |

### Async Do-Notation

```rust
use ordofp_core::mdo_async;

async fn load(id: UserId) -> Combined {
    mdo_async! {
        let user = await fetch_user(id);
        let profile = await fetch_profile(user.id);
        let settings = await fetch_settings(user.id);
        combine(user, profile, settings)
    }
}
```

### Runtime Abstraction

Runtime-agnostic spawning via the `RuntimeGenerare` trait (`TokioRuntime`,
`SmolRuntime`; handles are `JoinManubrium`) — see
[async.md → Runtime Integration](async.md#runtime-integration).

---

## Effects System

### Fiber-Based Concurrency

**Practical Benefit**: Lightweight concurrency - run millions of tasks without thread overhead. Cooperative scheduling with structured concurrency guarantees.

**Module:** `ordofp_core::async_core::fibra`

```rust
use ordofp_core::async_core::fibra::Fibra;

// Spawn fibers
let fiber1 = Fibra::spawn(async { task1() });
let fiber2 = Fibra::spawn(async { task2() });

// Parallel execution
let (r1, r2) = Fibra::par(fiber1, fiber2).await;

// Race (first to complete wins)
let winner = Fibra::race(fiber1, fiber2).await;

// Structured concurrency
let results = Fibra::zip_par(vec![f1, f2, f3]).await;
```

### Resource Management

**Practical Benefit**: Guaranteed cleanup even on errors or panics. File handles, connections, and locks are always released - no more resource leaks.

**Module:** `ordofp_core::async_core::res`

> **Note:** this async-oriented `Res` (behind the `async` feature) is distinct
> from the synchronous `ordofp_core::linear::Res` (behind `linear`), which
> offers `Res::bracket` — see [migration.md](migration.md). Same name, two
> different types with different APIs.

```rust
use ordofp_core::async_core::res::{Res, ResAsync};

// Synchronous resources
let file_resource = Res::new(
    || File::open("data.txt"),   // acquire
    |f| f.close()                 // release
);

let data = file_resource.use_resource(|file| {
    file.read_to_string()
})?;  // File automatically closed

// Async resources
let db_pool = ResAsync::new(
    || async { Pool::connect(&url).await },
    |pool| async { pool.close().await }
);

let result = db_pool.use_resource(|pool| async {
    pool.query("SELECT * FROM users").await
}).await?;
```

### Algebraic Effects

**Practical Benefit**: Inject different behaviors without changing code. Same program, different handlers for production, testing, and debugging.

```rust
// Same code, different handlers:
run_with_real_db(program)   // Production
run_with_mock_db(program)   // Testing
run_with_logged_db(program) // Debugging
```

**Module:** `ordofp_core::effects`

```rust
use ordofp_core::effects::{Eff, Effectus};

// Define effect
struct ConsoleEffect;
impl Effectus for ConsoleEffect {
    type Operation = ConsoleOp;
}

enum ConsoleOp {
    Print(String),
    ReadLine,
}

// Use effect
fn greet() -> Eff<ConsoleEffect, ()> {
    Eff::perform(ConsoleOp::Print("Hello!".into()))
        .flat_map(|_| Eff::perform(ConsoleOp::ReadLine))
        .flat_map(|name| Eff::perform(ConsoleOp::Print(format!("Hi, {}!", name))))
}

// Handle effect
let result = greet().run_with(|op| match op {
    ConsoleOp::Print(s) => { println!("{}", s); Ok(()) }
    ConsoleOp::ReadLine => { /* read input */ Ok(input) }
});
```

---

## Recursion Schemes

Principled recursion over data structures.

**Practical Benefit**: Write recursive algorithms once, get optimizations free. Avoid stack overflow, enable fusion, and express complex tree operations clearly.

```rust
// Histomorphism: access all previously computed values (dynamic programming)
// Fibonacci in O(n) without manual memoization
let fib = histo(n, |layer| match layer {
    NatF::ZeroF => 0,
    NatF::SuccF(cofree) => /* access history */ ...
});
```

**Module:** `ordofp_core::recursion`

| Morphism | Description | Signature |
|----------|-------------|-----------|
| `cata` | Catamorphism (fold) | `F<A> -> A` |
| `ana` | Anamorphism (unfold) | `A -> F<A>` |
| `hylo` | Hylomorphism | `ana` then `cata` |
| `para` | Paramorphism | fold with access to original |
| `apo` | Apomorphism | unfold with early termination |
| `histo` | Histomorphism | fold with history |
| `futu` | Futumorphism | unfold producing layers |
| `zygo` | Zygomorphism | fold with auxiliary |
| `chrono` | Chronomorphism | time-traveling refold (`futu` then `histo`) |
| `dyna` | Dynamorphism | course-of-values refold (`ana` then `histo`) |
| `mhylo` | Monadic hylomorphism | `hylo` with effects in a monad |

(Etymologies and the base-functor inventory — `NatF`, `ListF`, `TreeF`,
`MaybeF`, `ExprF`, `RoseF`, `Cofree` — live in [glossary.md](glossary.md).)

### Basic Example

```rust
use ordofp_core::recursion::{cata, ana, hylo};

// Catamorphism (fold)
let sum = cata(&nat_value, |layer| match layer {
    NatF::ZeroF => 0,
    NatF::SuccF(n) => n + 1,
});

// Anamorphism (unfold)
let nat = ana(5, |n| {
    if n == 0 { NatF::ZeroF }
    else { NatF::SuccF(n - 1) }
});

// Hylomorphism (unfold then fold, no intermediate)
let factorial = hylo(
    5,
    |layer| match layer {  // algebra
        NatF::ZeroF => 1,
        NatF::SuccF((n, acc)) => n * acc,
    },
    |n| {  // coalgebra
        if n == 0 { NatF::ZeroF }
        else { NatF::SuccF((n, n - 1)) }
    }
);
```

---

## Linear Types

Quantitative Type Theory for resource safety.

**Practical Benefit**: Compiler-enforced resource usage. Use exactly once, can't forget, can't duplicate. Perfect for file handles, network connections, cryptographic keys, session tokens.

```rust
let handle: Qtt<FileHandle, Semel> = open_file();
// MUST use handle exactly once:
// - Compiler error if you forget to use it
// - Compiler error if you try to use it twice
handle.consume();  // ✓ Used exactly once
```

**Module:** `ordofp_core::quantitative`

### Multiplicities

| Type | Latin | Meaning |
|------|-------|---------|
| `Semel` | *semel* = once | Use exactly once |
| `Omega` | ω | Use any number of times |
| `Nihil` | *nihil* = nothing | Compile-time only |

### Qtt (Quantitative Type)

```rust
use ordofp_core::quantitative::{Qtt, Semel, Omega};

// Linear (use exactly once)
let linear: Qtt<FileHandle, Semel> = Qtt::new(open_file("data.txt"));
let data = linear.consume();  // Must consume

// Unrestricted (use any times)
let unrestricted: Qtt<i32, Omega> = Qtt::new(42);
let a = unrestricted.borrow();
let b = unrestricted.clone();
```

### Linear Functions

```rust
use ordofp_core::quantitative::{FunctioLinearis, linear_compose};

// Linear function (lollipop ⊸)
let f: FunctioLinearis<Resource, Output> =
    FunctioLinearis::new(|r| process(r));

let result = f.apply(resource);

// Compose
let pipeline = linear_compose(step1, step2);
```

### Linear Pairs

```rust
use ordofp_core::quantitative::{ParLinearis, WithLinearis};

// Tensor (⊗) - both must be used
let tensor: ParLinearis<A, B> = ParLinearis::new(a, b);
let (a, b) = tensor.split();

// With (&) - choose one
let with: WithLinearis<A, B> = WithLinearis::new(a, b);
let chosen = with.choose_fst();  // OR .choose_snd()
```

---

## Free Monads

Build interpreters and DSLs.

**Practical Benefit**: Describe computations as data, interpret later. Same program can run in production (real DB), testing (in-memory), or debugging (just print what would happen).

```rust
// Build a program description (no side effects yet):
let program = get_user(1).flat_map(|u| save_log(u.name));

// Run differently based on context:
run_production(program)  // Real database
run_test(program)        // In-memory mock
run_dry(program)         // Print operations, don't execute
```

**Module:** `ordofp_core::free`

### Liber (Free Monad)

Standard Free monad - O(1) for right-associated binds, O(n²) for left-associated.

```rust
use ordofp_core::free::{Liber, liftF};

// Define DSL
enum ConsoleF<Next> {
    Print(String, Next),
    Read(Box<dyn FnOnce(String) -> Next>),
}

type Console<A> = Liber<ConsoleFWitness, A>;

// Smart constructors
fn print(s: &str) -> Console<()> {
    liftF(ConsoleF::Print(s.to_string(), ()))
}

// Build program
let program = print("Hello")
    .flat_map(|_| read_line())
    .flat_map(|name| print(&format!("Hi, {}!", name)));

// Interpret
fn interpret<A>(program: Console<A>) -> A {
    match program {
        Liber::Purus(a) => a,
        Liber::Impurus(ConsoleF::Print(s, next)) => {
            println!("{}", s);
            interpret(next)
        }
        // ...
    }
}
```

### LiberEcclesia (Church-Encoded Free Monad) - 2.0

**O(n) for both left- and right-associated binds.** Uses Church encoding with a continuation stack.

**Why it matters**: Left-associated binds are common in imperative-style code:
```rust
// This pattern is O(n²) with Liber, O(n) with LiberEcclesia
let mut program = LiberEcclesia::purus(0);
for i in 0..1000 {
    program = program.flat_map(move |x| LiberEcclesia::purus(x + i));
}
```

**Memory Coalescing**: Small chains (≤4 continuations) use inline storage via `AcervusParvus`.

```rust
use ordofp_core::free::LiberEcclesia;

// Small chain - inline storage, no heap
let small = LiberEcclesia::purus(0)
    .flat_map(|x| LiberEcclesia::purus(x + 1))
    .flat_map(|x| LiberEcclesia::purus(x + 2));

// Large chain - spills to heap, still O(n)
let large = (0..100).fold(
    LiberEcclesia::purus(0),
    |acc, i| acc.flat_map(move |x| LiberEcclesia::purus(x + i))
);

// Extract pure value
assert_eq!(large.extract_pure(), Some(4950));
```

### Codensity Transform - 2.0

Guaranteed O(1) binds via continuation-passing style.

**CodOption** - O(1) Option binds:
```rust
use ordofp_core::free::CodOption;

let result = CodOption::purus(42)
    .flat_map(|x| CodOption::purus(x + 1))
    .flat_map(|x| CodOption::purus(x * 2))
    .lower();  // Convert back to Option

assert_eq!(result, Some(86));
```

**CodResult** - O(1) Result binds:
```rust
use ordofp_core::free::CodResult;

let result = CodResult::ok(42)
    .flat_map(|x| CodResult::ok(x + 1))
    .map_err(|e: &str| e.len())
    .lower();

assert_eq!(result, Ok(43));
```

### Performance Comparison

| Implementation | Left-Associated | Right-Associated | Memory (small) |
|---------------|-----------------|------------------|----------------|
| Liber | O(n²) | O(n) | Heap |
| LiberEcclesia | O(n) | O(n) | Inline |
| CodOption/CodResult | O(1) per bind | O(1) per bind | CPS |

### Liberior (Freer Monad)

More efficient, no Functor constraint required.

```rust
use ordofp_core::free::{Liberior, mitto_liberior};

type Db<A> = Liberior<DbOp, A>;

fn query(sql: &str) -> Db<Vec<Row>> {
    mitto_liberior(DbOp::Query(sql.to_string()))
        .map(|result| parse_rows(result))
}
```

---

## Category Theory

Advanced abstractions for composition.

**Practical Benefit**: Build complex pipelines with branching, merging, and looping. Transform both inputs and outputs of functions with profunctors.

**Module:** `ordofp_core::category`

### Arrow (Sagitta)

**Practical Benefit**: Compose pipelines with more structure than functions. Split input to multiple processors, merge results, add feedback loops.

```rust
use ordofp_core::category::fn_arrows::*;

let double: BoxedFn<i32, i32> = arr(|x| x * 2);
let add_one: BoxedFn<i32, i32> = arr(|x| x + 1);

// Compose
let pipeline = compose(add_one, double);  // double then add_one

// Parallel (split)
let both = split(double, add_one);
let (a, b) = both((5, 10));  // (10, 11)

// First/Second
let first_doubled = first::<i32, i32, &str>(double);
let result = first_doubled((5, "hello"));  // (10, "hello")
```

### ArrowChoice (SagittaElectio)

```rust
use ordofp_core::category::fn_arrows::*;
use ordofp_core::datatypes::Aut;

// Choice operations on Aut (Either)
let on_left = sinister::<i32, i32, String>(double);
let result = on_left(Aut::sinister(5));  // Aut::Sinister(10)

// Fanin (merge)
let merged = confluo(
    arr(|n: i32| n.to_string()),
    arr(|s: String| s)
);
```

### Kan Extensions

**Practical Benefit**: Performance optimizations. Coyoneda fuses multiple maps into one pass. Codensity fixes left-associated bind performance issues.

```rust
use ordofp_core::category::kan::{Coyoneda, Codensitas};

// Coyoneda - free functor (deferred map fusion)
let coyoneda = Coyoneda::lift(vec![1, 2, 3]);
let mapped = coyoneda
    .map(|x| x * 2)
    .map(|x| x + 1);
// All maps fused when lowered

// Codensity - performance for left-associated binds
let codensity = Codensitas::purus(42);
```

---

## Common Patterns

### Railway-Oriented Programming

```rust
fn process_order(order: RawOrder) -> Result<ProcessedOrder, OrderError> {
    validate(order)
        .flat_map(check_inventory)
        .flat_map(calculate_total)
        .flat_map(apply_discounts)
        .map(finalize)
}
```

### Dependency Injection with ReaderT

```rust
struct AppEnv { db: DbPool, cache: Cache, config: Config }
type App<A> = ReaderT<AppEnv, Result<A, AppError>>;

fn get_user(id: UserId) -> App<User> {
    ReaderT::ask().flat_map(|env| {
        ReaderT::lift(env.db.find_user(id))
    })
}
```

### Error Accumulation with Probatum

```rust
fn validate_form(form: &Form) -> Probatum<ValidForm, Vec<String>> {
    validate_name(&form.name).into_probatum()
        + validate_email(&form.email)
        + validate_password(&form.password)
}
```

### Lens-Based Updates

```rust
let updated = user_address_lens
    .compose(address_city_lens)
    .set(&user, "New York".into());
```

### Composable Pipelines

```rust
use ordofp_core::{pipe, compose};

// Left-to-right
let process = pipe!(parse, validate, transform, format);

// Right-to-left
let process = compose!(format, transform, validate, parse);
```

---

## Quick Reference

For a complete API reference, tables, and syntax cheatsheets, see the **[Canonical Reference](reference.md)**.

### Feature Decision Guide

| Problem | Solution |
|---------|----------|
| Form validation (collect all errors) | `Probatum` |
| Nested struct updates / immutable data | Optics (`Aspectus`) |
| Guaranteed resource cleanup | `Res` / `ResAsync` |
| Testable side effects (e.g. database calls) | Free monads / Algebraic effects |
| Safe concurrent state / map access | PFDS (`OrdMap`, `OrdSet`) |
| Stacked effects (config + state + errors) | Monad transformers |
| Use-once resources | Linear types (`Qtt<T, Semel>`) |
| Tree/recursive data | Recursion schemes |
| Streaming data | `Flumen` |
| Lightweight concurrency (millions of tasks) | `Fibra` |

---

## Naming Convention Reference

OrdoFP uses Scholastic Latin naming throughout, for a consistent, expressive API. See **[glossary.md](glossary.md)** for the complete reference.
