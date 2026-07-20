# Glossarium OrdoFP

> *"Nomina sunt consequentia rerum."*
> — Names are the consequence of things. (Justinian, *Digest* 1.4.1)

This glossary maps OrdoFP's nomenclature to its roots in Catholic scholastic philosophy, drawing from Latin, Greek, Aristotle, St. Thomas Aquinas, Cicero, Sacred Scripture, and the classical liberal arts of the Trivium and Quadrivium.

> **Era labels:** headings tagged "v2.0" or "OrdoFP 4.0 Phase N" are the pre-reset
> development-era names under which those subsystems shipped (the CHANGELOG records
> the 4.0 program). The project reset its versioning to 0.1.0 on 2026-04-25 — see
> the version-reset note in `migration.md`. The labels are kept as historical
> provenance; all listed names describe the current codebase.

---

## Canonical Naming Guide

| Canonical Name | Canonical Alias | Etymology & Reference |
|----------------|-----------------|----------------------|
| `Nihil` | `Nihil` | Latin: "nothing" — the empty case, *privatio* |
| `Coniunctio` | `Coniunctio` | Latin: "joining together" — Aristotle's σύνθεσις |
| `Disiunctio` | `Disiunctio` | Latin: "disjunction" — scholastic logic |
| `Absurdum` | `Absurdum` | Latin: "the absurd" — *ex absurdo quodlibet* |
| `Sinister` | `Sinister` | Latin: "left" — sinister in heraldic tradition |
| `Dexter` | `Dexter` | Latin: "right" — dexter, the favored side |
| `Semigroup` | `Compositio` | Latin: "composition" — Aquinas on *compositio et divisio* |
| `Monoid` | `Unitas` | Latin: "unity" — *unum* as transcendental (ST I, q.11) |
| `Applicative` | `Applicatio` | Latin: "application" — *applicatio formae ad materiam* |
| `Sum` | `Aggregatio` | Latin: "aggregation" — collecting into one |
| `Product` | `Multiplicatio` | Latin: "multiplication" — repeated unity |
| `All` | `Omnis` | Latin: "all/every" — universal quantifier |
| `Any` | `Aliquid` | Latin: "something" — existential, *aliquid* as transcendental |
| `First` | `Primus` | Latin: "first" — *primum movens* |
| `Last` | `Ultimus` | Latin: "last/ultimate" — *finis ultimus* |
| `Endo` | `Reflexio` | Latin: "bending back" — self-referential morphism |
| `Lens` | `Aspectus` | Latin: "view/sight" — *aspectus mentis* |
| `Prism` | `Divisio` | Latin: "division" — division of cases |
| `Iso` | `Aequivalentia` | Latin: "equivalence" — mutual convertibility |
| `Traversal` | `Iteratio` | Latin: "iteration" — *actus repetendi* |
| `Identity` | `Identitas` | Latin: "sameness" — *idem*, principle of identity |
| `Store` | `Thesaurus` | Latin/Greek: "treasury" — θησαυρός (Mt 13:52) |
| `Env` | `Contextus` | Latin: "woven together" — surrounding context |
| `Traced` | `Vestigium` | Latin: "trace/footprint" — *vestigia Dei* |
| `Lazy` | `Pigritia` | Latin: "laziness" — deferred evaluation |
| `Void/Never` | `Absurdum` | Latin: "the absurd" — *ex falso quodlibet* |
| `Unit` | `Unitas` | Latin: "unity" — the terminal object |
| `PhantomData` | `Phantasma` | Latin: "apparition" — type-level marker |

---

## I. Trivium — The Three Ways of Language

The Trivium governs *ratio* (reason) through language: Grammar (structure), Logic (validity), and Rhetoric (expression).

### Grammatica — Types and Syntax

> *"Grammatica est scientia recte loquendi."*
> — Grammar is the science of speaking correctly. (Isidore of Seville)

These types form the syntactic building blocks of OrdoFP:

| Type | Purpose | Scholastic Parallel |
|------|---------|---------------------|
| `Nihil` | Empty heterogeneous list | *Privatio* — the absence of form |
| `Coniunctio` | HList cons cell | *Compositum* — joining of parts |
| `Disiunctio` | Sum type (either/or) | *Disiunctio* — exclusive alternation |
| `Absurdum` | Void/never type (uninhabited) | *Ex falso quodlibet* — from falsity, anything |
| `Unitas` | Unit type wrapper | *Unum* — the terminal object |
| `Field<Name, T>` | Labelled field | *Nomen proprium* — proper naming |
| `Pigritia` | Lazy computation | *Pigritia* — virtuous delay, deferral of action |
| `Phantasma` | Zero-sized type marker | *Phantasma* — apparition, type without substance |

### Dialectica — Traits and Logic

> *"Dialectica est ars artium, scientia scientiarum."*
> — Dialectic is the art of arts, the science of sciences. (Boethius)

Traits express logical relationships and operations:

| Trait | Method(s) | Scholastic Parallel |
|-------|-----------|---------------------|
| `Compositio` | `.combine()` | *Compositio* — joining judgments (Aquinas, *De Veritate*) |
| `Unitas` | `::empty()` | *Unum* — transcendental unity (ST I, q.11, a.1) |
| `Functor` | `.map()` | *Forma* — preserving structure under transformation |
| `Applicatio` | `.apply()` | *Applicatio* — applying form to matter |
| `Monad` | `.flat_map()` | *Vinculum* — the chain of being (Leibniz) |

### Rhetorica — Patterns and Idioms

> *"Rem tene, verba sequentur."*
> — Grasp the matter, the words will follow. (Cato the Elder)

Common patterns for eloquent functional programming:

| Pattern | Description | Example |
|---------|-------------|---------|
| Railway | Chain operations that may fail | `parse().flat_map(validate).map(finalize)` |
| Accumulation | Gather multiple validations | `Probatum` with `+` operator |
| Sculpting | Reshape data by type | `h.sculpt::<Target>()` |

---

## II. Quadrivium — The Four Ways of Number

The Quadrivium governs *intellectus* (understanding) through quantity: Arithmetic (number), Geometry (space), Music (harmony), and Astronomy (motion).

### Arithmetica — Algebraic Structures

> *"Numerus est multitudo ex unitatibus constituta."*
> — Number is a multitude constituted from unities. (Euclid, via Boethius)

| Type | Purpose | Scholastic Parallel |
|------|---------|---------------------|
| `Compositio` | Associative binary operation | *Synthesis* — Aristotle's combination |
| `Unitas` | Compositio with identity | *Unum* — the One from which many proceed |
| `Aggregatio` | Additive wrapper | *Additio* — arithmetic addition |
| `Multiplicatio` | Multiplicative wrapper | *Multiplicatio* — repeated folding |
| `Omnis` | Boolean conjunction | *Omnis* — "all" in syllogistic (Barbara) |
| `Aliquid` | Boolean disjunction | *Aliquid* — "something" exists |

### Geometria — Optics and Structure

> *"Sine geometria nemo ingrediatur."*
> — Let no one enter without geometry. (Plato's Academy)

Optics provide *aspectus* (sight) into immutable structures:

| Type | Purpose | Scholastic Parallel |
|------|---------|---------------------|
| `Aspectus` | Focus on product fields | *Aspectus* — the mind's gaze upon particulars |
| `Divisio` | Focus on sum variants | *Divisio* — distinguishing cases |
| `Aequivalentia` | Bidirectional isomorphism | *Convertibilitas* — mutual entailment |
| `Iteratio` | Focus on multiple elements | *Iteratio* — the act of repeating |

#### Optica Nova — Enhanced Optics (v0.1.0)

> *"Per profundum ad veritatem."*
> — Through depth to truth.

Advanced optics for principled composition and indexed access:

| Type | Purpose | Scholastic Parallel |
|------|---------|---------------------|
| `IteratioAffinis` | Focus on 0 or 1 elements | *Affinis* — neighboring, optional access |
| `Profunctor` | Bifunctor contravariant in first arg | *Profunctor* — before the performer |
| `Fortis` | Strong profunctor (lens-like) | *Fortis* — strong, passing through pairs |
| `Electio` | Choice profunctor (prism-like) | *Electio* — choice, passing through sums |
| `Ad` | Indexed container access | *Ad* — at, access by index |
| `AdRemovere` | Removal by index | *Removere* — to move back |
| `AdInserere` | Insertion by index | *Inserere* — to put in |
| `AspectusAd` | At-optic for specific index | *Aspectus Ad* — gaze at index |
| `AspectusProfunctor` | Polymorphic lens | *Aspectus Polymorphicus* — type-changing lens |
| `DivisioProfunctor` | Polymorphic prism | *Divisio Polymorphica* — type-changing prism |

### Musica — Traversable and Harmony

> *"Musica est exercitium arithmeticae occultum nescientis se numerare animi."*
> — Music is a hidden arithmetic exercise of the soul. (Leibniz)

Traversable structures exhibit *harmonia* — coordinated effectful iteration:

| Type/Trait | Purpose | Scholastic Parallel |
|------------|---------|---------------------|
| `Traversable` | Effectful mapping | *Harmonia* — ordered movement through parts |
| `Foldable` | Reduction to unity | *Reductio* — leading back to principle |
| `NonEmpty` | Guaranteed substance | *Ens* — that which exists (not nothing) |

### Astronomia — Transformers and Higher Abstraction

> *"Caeli enarrant gloriam Dei."*
> — The heavens declare the glory of God. (Psalm 19:1)

Monad transformers lift computations to higher spheres:

| Type | Purpose | Scholastic Parallel |
|------|---------|---------------------|
| `OptionT` | Optionality over any monad | *Potentia* — possibility |
| `EitherT` | Error handling elevated | *Defectibilitas* — capacity for failure |
| `ReaderT` | Environment access | *Contextus* — the surrounding whole |
| `StateT` | Stateful computation | *Motus* — change of state (Aristotle) |
| `Scriptor` | Accumulated output (Writer) | *Scriptor* — the scribe who records |
| `ContinuatioT` | Continuation-passing | *Continuatio* — unbroken succession |

### Futura — Async Computation (v2.0)

> *"Nunc fluens facit tempus, nunc stans facit aeternitatem."*
> — The flowing now makes time, the standing now makes eternity. (Boethius)

Async types bring temporal and concurrent computation to the functional paradigm:

| Type | Purpose | Scholastic Parallel |
|------|---------|---------------------|
| `Futurus` | Async computation wrapper | *Futurus* — that which is to come, the potential act |
| `Flumen` | Async stream | *Flumen* — river, continuous flow (Heraclitus) |
| `LectorAsync` | Async environment reader | *Lector* — one who reads, the interpreter |
| `StatusAsync` | Async state transformer | *Status* — standing/condition, mutable state |
| `OptionTAsync` | Async optionality | *Potentia Futura* — future possibility |
| `EitherTAsync` | Async error handling | *Defectibilitas Futura* — future capacity for failure |
| `ScriptorAsync` | Async logging/accumulation | *Scriptor* — one who writes, the recorder |

#### Async Type Classes

| Trait | Method(s) | Scholastic Parallel |
|-------|-----------|---------------------|
| `FunctorAsync` | `.fmap_async()` | *Forma in Tempore* — structure-preserving in time |
| `ApplicatioAsync` | `.apply_async()`, `.map2_async()` | *Applicatio Futura* — future application of form |
| `MonadAsync` | `.flat_map_async()` | *Vinculum Temporis* — the chain across time |
| `TraversableAsync` | `.traverse_async()` | *Harmonia Futura* — coordinated future iteration |

#### Runtime Abstraction

| Type | Purpose | Scholastic Parallel |
|------|---------|---------------------|
| `RuntimeGenerare` | Task spawning trait | *Generare* — to beget, bring forth action |
| `JoinManubrium` | Task join handle | *Manubrium* — handle, that by which we grasp |
| `TokioRuntime` | Tokio executor | *Executor* — one who carries out |
| `SmolRuntime` | smol executor (async-std was discontinued, RUSTSEC-2025-0052; the `async-std` cargo feature is an alias for `smol`) | *Executor Alternus* — alternative executor |

#### Effect System

| Type | Purpose | Scholastic Parallel |
|------|---------|---------------------|
| `Effectus` | Effect marker trait | *Effectus* — that which is brought about |
| `EffectusHandler` | Effect handler trait | *Tractator* — one who handles |
| `IoEffectus` | I/O effect marker | *Effectus Externus* — external effect |
| `StatusEffectus` | State effect marker | *Effectus Mutationis* — effect of change |
| `ErrorEffectus` | Error effect marker | *Effectus Defectus* — effect of deficiency |
| `PurusEffectus` | Pure effect marker | *Effectus Purus* — effect without side-effect |

#### Fibra — Fiber-Based Concurrency (v0.1.0)

> *"Motus est actus entis in potentia."*
> — Motion is the actuality of that which exists potentially. (Aristotle)

Lightweight fibers for structured concurrency:

| Type | Purpose | Scholastic Parallel |
|------|---------|---------------------|
| `Fibra` | Lightweight fiber | *Fibra* — fiber, thread of execution |
| `FibraManubrium` | Fiber handle | *Manubrium* — handle, grip |
| `FibraId` | Fiber identifier | *Identitas* — unique identity |
| `FibraStatus` | Fiber state | *Status* — standing, condition |
| `FibraExitus` | Fiber exit result | *Exitus* — outcome, departure |
| `Praefectus` | Fiber supervisor | *Praefectus* — overseer, commander |
| `StrategiaRestart` | Restart strategy | *Strategia* — generalship, plan |
| `InfansSpecificatio` | Child specification | *Infans* — child, offspring |
| `StrategiaMora` | Backoff strategy | *Mora* — delay |

#### Res Safetica — Resource Management (v0.1.0)

> *"Amplexus est tutela."*
> — Embrace is protection.

Safe resource acquisition and release:

| Type | Purpose | Scholastic Parallel |
|------|---------|---------------------|
| `Res` | Synchronous resource | *Res* — thing, resource |
| `ResAsync` | Async resource | *Res Futura* — future resource |
| `Piscina` | Resource pool | *Piscina* — pool, reservoir |
| `amplexus` | Bracket pattern | *Amplexus* — embrace, safe acquisition |
| `finaliter` | Finalizer pattern | *Finaliter* — finally, guaranteed cleanup |

#### Typus Dependens — Dependent Types Foundation (v0.1.0)

> *"Nihil est in intellectu quod non prius fuerit in sensu."*
> — Nothing is in the intellect that was not first in the senses.

Type-level programming foundations:

| Type | Purpose | Scholastic Parallel |
|------|---------|---------------------|
| `Zero` | Type-level zero | *Nihil* — nothing |
| `Succ` | Type-level successor | *Successio* — succession |
| `Naturalis` | Natural number trait | *Naturalis* — natural |
| `Vectum` | Length-indexed vector | *Vectum* — that which carries |
| `Testimonium` | Proof witness | *Testimonium* — testimony, proof |
| `Aequalitas` | Type equality proof | *Aequalitas* — equality |
| `Refined` | Refinement type | *Refinatus* — refined, constrained |

---

## III. Sacra Doctrina — Higher Abstractions

> *"Sacra doctrina est scientia, quia procedit ex principiis notis lumine superioris scientiae."*
> — Sacred doctrine is a science because it proceeds from principles known by the light of a higher science. (ST I, q.1, a.2)

### Comonad — Contextual Computation

The dual of Monad, representing values *in context*:

| Type | Purpose | Scholastic Parallel |
|------|---------|---------------------|
| `Comonad` | Contextual extraction | *Participatio* — sharing in a greater whole |
| `Identitas` | Identity comonad | *Idem* — the self-same |
| `Thesaurus` | Store comonad | *Thesaurus* — storehouse (Mt 13:52: "like a householder bringing out treasures") |
| `Contextus` | Environment comonad | *Circumstantia* — that which stands around |
| `Vestigium` | Traced comonad | *Vestigium Trinitatis* — traces of the divine in creation |

### Alternative — Choice and Failure

> *"Oportet eligentem unum alteri praeferre."*
> — One who chooses must prefer one thing to another. (Aquinas)

| Method | Purpose | Scholastic Parallel |
|--------|---------|---------------------|
| `.alt()` | Alternative choice | *Electio* — choosing between alternatives |
| `::empty()` | Failure/impossibility | *Nihil* — the null case |
| `.guard()` | Conditional continuation | *Conditio* — prerequisite condition |

### Bifunctor — Two-Parameter Transformation

> *"Duo sunt in quolibet composito."*
> — There are two in every composite. (Aquinas on matter/form)

| Method | Purpose | Scholastic Parallel |
|--------|---------|---------------------|
| `.bimap()` | Transform both | *Transmutatio* — change in both respects |
| `.map_left()` | Transform first | *Materia* — the receptive principle |
| `.map_right()` | Transform second | *Forma* — the determining principle |

### Recursio — Recursion Schemes (OrdoFP 4.0)

> *"Per varios casus, per tot discrimina rerum."*
> — Through various chances, through so many changes of things. (Virgil, *Aeneid* I.204)

Recursion schemes provide structured, principled recursion over data structures.
The Greek morphism names describe the direction and nature of transformation:

| Morphism | Greek Etymology | Purpose |
|----------|-----------------|---------|
| `cata` | κατά (down) + μορφή (form) | Fold — tear down structure |
| `ana` | ἀνά (up) + μορφή (form) | Unfold — build up structure |
| `hylo` | ὕλη (matter) + μορφή (form) | Refold — matter taking form |
| `para` | παρά (beside) + μορφή (form) | Fold with subtree access |
| `apo` | ἀπό (away) + μορφή (form) | Unfold with early termination |
| `histo` | ἱστορία (history) + μορφή | Fold with computation history |
| `futu` | futurum (future) + μορφή | Unfold producing multiple layers |
| `zygo` | ζυγός (yoke) + μορφή (form) | Fold with auxiliary algebra |
| `chrono` | χρόνος (time) + μορφή (form) | Generalized time-traveling refold |
| `dyna` | δύναμις (power) + μορφή | Efficient course-of-values recursion |
| `mhylo` | monas + ὕλη + μορφή | Monadic hylomorphism — refold with effects |

#### Core Recursion Types

| Type | Latin Name | Etymology | Purpose |
|------|------------|-----------|---------|
| `Recursiva` | *recursivus* | running back | Trait for recursive structures |
| `Corecursiva` | *co-* + *recursivus* | running together | Trait for corecursive structures |
| `FunctorBasis` | *basis* | foundation | Associates type with base functor |
| `Cofree<F, A>` | *co-* + *liber* | free together | Annotated recursive structure |
| `Free<F, A>` | *liber* | free | Suspended computation |

#### Free Monad Variants

| Variant | Latin Name | Etymology |
|---------|------------|-----------|
| `Pure` | `Purus` | *purus* = pure, clean |
| `Suspended` | `Suspensus` | *suspensus* = hanging, deferred |

#### Either for Apomorphisms

| Variant | Latin Name | Etymology |
|---------|------------|-----------|
| `Left` | `Sinister` | *sinister* = left side |
| `Right` | `Dexter` | *dexter* = right side |
| `Either` | `Aut` | *aut* = or (exclusive disjunction) |

#### Base Functors

| Functor | Purpose | Structure |
|---------|---------|-----------|
| `NatF<R>` | Natural numbers | `ZeroF` \| `SuccF(R)` |
| `ListF<E, R>` | Linked lists | `NilF` \| `ConsF(E, R)` |
| `TreeF<A, R>` | Binary trees | `LeafF(A)` \| `BranchF(R, R)` |
| `MaybeF<R>` | Option-like | `NothingF` \| `JustF(R)` |
| `ExprF<R>` | Expressions | `LitF(i32)` \| `AddF(R, R)` \| `MulF(R, R)` |
| `RoseF<A, R>` | Rose trees | `RoseF(A, Vec<R>)` |

### OrdoFP 4.0 Phase 2: Algebraic Effect System

> *"Effectus est actus causae."*
> — An effect is the act of a cause. (Scholastic philosophy)

The 4.0 Phase 2 algebraic effect system provides type-safe effect handling with
evidence-based handlers inspired by Koka and Polysemy.

#### Effect Monads

| Type | Latin Name | Etymology | Purpose |
|------|------------|-----------|---------|
| `Eff<R, A>` | *Effectus* | effect, result | Effectful computation monad |
| `Sem<R, A>` | *Semantica* | meaning | Polysemy-style extensible effects |
| `EffResult<R, A>` | *Effectus Exitus* | effect outcome | Result of Eff step |
| `SemResult<R, A>` | *Sem Exitus* | meaning outcome | Result of Sem step |
| `EffSuspension` | *Suspensio Effectus* | suspended effect | Awaiting handler |
| `SemSuspension` | *Suspensio Sem* | suspended meaning | Awaiting interpreter |

#### Evidence-Based Handlers

| Type | Latin Name | Etymology | Purpose |
|------|------------|-----------|---------|
| `SignumEffectus<E>` | *signum* | sign, mark | Runtime effect tag |
| `Testimonium<E, H>` | *testimonium* | testimony, evidence | Evidence record |
| `VectorTestimonium` | *vector testimonii* | carrier of evidence | Evidence vector |
| `Clausula<A, B, E, R>` | *clausula* | small clause | Handler clause |
| `ClausulaGenus` | *genus clausulae* | kind of clause | Clause classification |
| `Resumptio<A, R>` | *resumptio* | taking back | One-shot continuation |

#### Handler Clauses (Clausula)

| Variant | Latin Name | Etymology | Semantics |
|---------|------------|-----------|-----------|
| `Fun` | *Functio* | function | Tail-resumptive (automatic resume) |
| `Ctl` | *Controleum* | control | Full control over continuation |
| `Final` | *Finalis* | final | Terminal (no resume available) |

#### Effect Operations

| Function | Latin Root | Purpose |
|----------|------------|---------|
| `run_purus` | *currere purum* | Run pure computation |
| `run_sem` | *currere sem* | Run Sem computation |
| `send` | *mittere* | Send effect operation |
| `interpret` | *interpretari* | Interpret and remove effect |
| `reinterpret` | *re-interpretari* | Transform one effect to another |
| `subsume` | *subsumere* | Merge already-handled effect |
| `embed` | *inserere* | Embed in larger effect row |
| `intercept` | *intercipere* | Intercept without handling |

#### Handler Traits

| Trait | Latin Name | Etymology | Purpose |
|-------|------------|-----------|---------|
| `TractatorEff<E, R>` | *tractator* | handler | Eff effect handler |
| `TractatorEvidentia<E>` | *tractator evidentiae* | evidence handler | Evidence-based handler |
| `Interpres<E, R>` | *interpres* | interpreter | Sem effect interpreter |
| `Reinterpres<E1, E2, R>` | *re-interpres* | re-interpreter | Effect transformer |
| `Membrum<R>` | *membrum* | member | Effect membership marker |

### OrdoFP 4.0 Phase 3: Advanced Fiber Runtime

> *"In motu est vita."*
> — In motion is life. (Classical maxim)

The 4.0 Phase 3 fiber runtime provides ZIO-style effects, work-stealing scheduling,
and concurrent primitives for high-performance asynchronous programming.

#### ZIO-Style Effect Types

| Type | Latin Name | Etymology | Purpose |
|------|------------|-----------|---------|
| `Zio<R, E, A>` | *Zio* | ZIO-inspired | Effect with environment, error, result |
| `Causa<E>` | *causa* | cause | Structured error cause |
| `Exitus<E, A>` | *exitus* | outcome, exit | Success or failure result |
| `Ambitus<R>` | *ambitus* | circuit, environment | Environment wrapper |
| `Task<E, A>` | *Task* | task | ZIO with unit environment |
| `Uio<R, A>` | *Uio* | ZIO-inspired | Effect that cannot fail |
| `UTask<A>` | *UTask* | infallible task | Pure effect, no error |
| `Io<E, A>` | *Io* | input/output | Effect with any environment |

#### Causa (Cause) Variants

| Variant | Latin Name | Etymology | Semantics |
|---------|------------|-----------|-----------|
| `Defectus` | *defectus* | deficiency, failure | Expected failure |
| `Mors` | *mors* | death | Unexpected termination |
| `Interruptio` | *interruptio* | interruption | Fiber interruption |
| `Utrumque` | *utrumque* | both | Parallel failures |
| `Deinde` | *deinde* | then, next | Sequential failures |

#### Work-Stealing Scheduler

| Type | Latin Name | Etymology | Purpose |
|------|------------|-----------|---------|
| `Prioritas` | *prioritas* | priority | Fiber priority level |
| `IndiciumExecutionis` | *indicium* | hint | Execution hint for scheduler |
| `MunusFibrae` | *munus* | task, duty | Fiber task wrapper |
| `OrdinariusConfig` | *ordinarius* | orderly, regular | Scheduler configuration |
| `Statisticae` | *statisticae* | statistics | Scheduler metrics |
| `OrdoLocalis` | *ordo localis* | local order | Per-worker queue |
| `OrdoGlobalis` | *ordo globalis* | global order | Shared work queue |

#### Priority Levels (Prioritas)

| Level | Latin Name | Etymology | Semantics |
|-------|------------|-----------|-----------|
| `Infima` | *infima* | lowest | Background priority |
| `Normalis` | *normalis* | normal | Default priority |
| `Alta` | *alta* | high | Elevated priority |
| `Critica` | *critica* | critical | Highest priority |

#### Execution Hints (IndiciumExecutionis)

| Hint | Latin Name | Etymology | Semantics |
|------|------------|-----------|-----------|
| `Computatio` | *computatio* | computation | CPU-bound work |
| `Obstructio` | *obstructio* | blocking | Blocking I/O |
| `Latentia` | *latentia* | latency | Latency-sensitive |

#### Work-Stealing Policies (PolitiaFurti)

| Policy | Latin Name | Etymology | Purpose |
|--------|------------|-----------|---------|
| `PolitiaFortuita` | *fortuita* | random, chance | Random victim selection |
| `PolitiaCircularis` | *circularis* | circular | Round-robin selection |
| `PolitiaFurti` | *furtum* | theft | Trait for stealing policies |

#### Concurrent Primitives

| Type | Latin Name | Etymology | Purpose |
|------|------------|-----------|---------|
| `Dilatum<A>` | *dilatum* | deferred | Deferred value (Cats Effect) |
| `Referentia<A>` | *referentia* | reference | Atomic reference (Ref) |
| `Semaphorum` | *semaphorum* | signal-bearer | Counting semaphore |
| `MVarSync<A>` | *MVar* | mutable variable | Synchronized mutable box |
| `CaudaBackpressure<A>` | *cauda* | tail, queue | Bounded backpressure queue |

---

## IV. Principia — Foundational Laws

> *"Lex est ordinatio rationis ad bonum commune."*
> — Law is an ordinance of reason for the common good. (ST I-II, q.90, a.4)

The `ordofp_laws` crate verifies that implementations satisfy their governing laws:

### Laws of Compositio (Semigroup)

```
Associativity: (a ⊕ b) ⊕ c = a ⊕ (b ⊕ c)
```

### Laws of Unitas (Monoid)

```
Left Identity:  ε ⊕ a = a
Right Identity: a ⊕ ε = a
Associativity:  (a ⊕ b) ⊕ c = a ⊕ (b ⊕ c)
```

### Laws of Functor

```
Identity:    fmap id = id
Composition: fmap (f . g) = fmap f . fmap g
```

### Laws of Monad

```
Left Identity:  return a >>= f  = f a
Right Identity: m >>= return    = m
Associativity:  (m >>= f) >>= g = m >>= (λx. f x >>= g)
```

### Laws of FunctorAsync (v2.0)

```
Identity:    fmap_async(id).await = id
Composition: fmap_async(f).await.fmap_async(g).await = fmap_async(g ∘ f).await
```

### Laws of MonadAsync (v2.0)

```
Left Identity:  pure(a).flat_map_async(f).await = f(a).await
Right Identity: m.flat_map_async(pure).await    = m
Associativity:  (m.flat_map_async(f).await).flat_map_async(g).await
                = m.flat_map_async(|x| f(x).await.flat_map_async(g)).await
```

### OrdoFP 4.0 Phase 4: Quantitative Types

> *"Semel omnes, omnes semel."*
> — Once for all, all at once. (Classical Latin)

The 4.0 Phase 4 quantitative types system implements Quantitative Type Theory (QTT)
abstractions inspired by Idris 2, providing type-level tracking of value usage.

#### Multiplicitas — Multiplicity System

| Type | Latin Name | Etymology | Purpose |
|------|------------|-----------|---------|
| `Multiplicitas` | *multiplicitas* | manyness | Core multiplicity enum |
| `Nihil` | *nihil* | nothing | Zero usage (erased at runtime) |
| `Semel` | *semel* | once | Linear usage (exactly once) |
| `Omega` | *omega* | Ω | Unrestricted usage |
| `Usage` | *usus* | use | Type-level multiplicity marker |
| `MultiplicitasSemiring` | *semiring* | half-ring | Semiring structure |

#### Multiplicity Operations

| Function | Latin Root | Purpose |
|----------|------------|---------|
| `mult_add` | *addere* | Add multiplicities (choice) |
| `mult_mul` | *multiplicare* | Multiply multiplicities (sequence) |
| `is_subusage` | *sub-usus* | Check subusage relation |

#### Qtt — Quantitative Type Wrapper

| Type | Latin Name | Etymology | Purpose |
|------|------------|-----------|---------|
| `Qtt<A, M>` | *quantitas* | quantity | Value with multiplicity phantom |
| `QttLinearis` | *linearis* | linear | Alias for Qtt with Semel |
| `QttErasum` | *erasum* | erased | Alias for Qtt with Nihil |
| `QttLiber` | *liber* | free | Alias for Qtt with Omega |
| `QttExt` | *extensio* | extension | Extended operations trait |

#### ManusLinearis — Linear Resource Handle

| Type | Latin Name | Etymology | Purpose |
|------|------------|-----------|---------|
| `ManusLinearis<T>` | *manus* | hand | Linear resource handle |
| `ManusGuard<T>` | *custos* | guard | RAII guard for resource access |

#### ManusLinearis Operations

| Method | Latin Root | Purpose |
|--------|------------|---------|
| `acquire` | *acquirere* | Acquire resource ownership |
| `release` | *relaxare* | Release with consumption |
| `guard` | *custodire* | Borrow with RAII guard |
| `use_with` | *uti* | Use resource in closure |

#### ParLinearis — Linear Pairs (Tensor Product)

| Type | Latin Name | Etymology | Purpose |
|------|------------|-----------|---------|
| `ParLinearis<A, B>` | *par* | pair | Tensor product (⊗) |
| `WithLinearis<A, B>` | *cum* | with | Additive product (&) |
| `AdditiveChoice<A, B>` | *electio* | choice | Additive sum (⊕) |

#### Linear Pair Operations

| Function | Latin Root | Purpose |
|----------|------------|---------|
| `tensor` | *tensio* | Create tensor pair |
| `with_pair` | *cum* | Create with-pair |
| `tensor_assoc_l` | *associare* | Left associativity |
| `tensor_assoc_r` | *associare* | Right associativity |
| `tensor_swap` | *permutare* | Swap tensor elements |

#### FunctioLinearis — Linear Function (⊸)

| Type | Latin Name | Etymology | Purpose |
|------|------------|-----------|---------|
| `FunctioLinearis<A, B>` | *functio* | function | Linear implication (lollipop) |
| `FunctioSemel` | *semel* | once | Alias for linear function |
| `Lolly<A, B>` | *lolly* | lollipop | Alias for ⊸ operator |

#### Linear Function Operations

| Function | Latin Root | Purpose |
|----------|------------|---------|
| `linear_apply` | *applicare* | Apply linear function |
| `linear_compose` | *componere* | Compose linear functions |
| `linear_flip` | *flectere* | Flip curried arguments |
| `linear_const` | *constans* | Constant function |
| `linear_curry` | *curry* | Curry binary function |
| `linear_uncurry` | *uncurry* | Uncurry to binary |

#### MonasLinearis — Linear Monad

| Type | Latin Name | Etymology | Purpose |
|------|------------|-----------|---------|
| `MonasLinearis<M>` | *monas* | monad | Linear monad trait |
| `QttMonad<A, M>` | *monas qtt* | QTT monad | Monadic wrapper for Qtt |

#### Linear Monad Laws

```
Left Identity:  purus(a).bind(f) = f(a)
Right Identity: m.bind(purus) = m
Associativity:  m.bind(f).bind(g) = m.bind(|x| f(x).bind(g))
```

#### Linear Logic Connectives

| Symbol | Latin Name | Pronunciation | Meaning |
|--------|------------|---------------|---------|
| `⊸` | *lolly* | "lollipop" | Linear implication |
| `⊗` | *tensor* | "tensor" | Multiplicative conjunction |
| `&` | *cum* | "with" | Additive conjunction |
| `⊕` | *plus* | "plus" | Additive disjunction |
| `!` | *bang* | "of course" | Unrestricted modality |
| `?` | *whimper* | "why not" | Dual of bang |

### OrdoFP 4.0 Phase 5: Row Polymorphism

> *"Ordo est recta ratio rerum ad finem."*
> — Order is the right arrangement of things toward an end. (Scholastic definition)

The 4.0 Phase 5 row polymorphism system implements extensible records and variants
inspired by PureScript/Elm, enabling type-safe field operations with row-polymorphic functions.

#### Ordo — Row Types

| Type | Latin Name | Etymology | Purpose |
|------|------------|-----------|---------|
| `Ordo` | *ordo* | row, rank, order | Row type trait |
| `OrdoOps` | *operationes ordinis* | row operations | Row helper operations |
| `Extensio<L, R>` | *extensio* | extension | Row extension result |
| `Restrictio<L, R>` | *restrictio* | restriction | Row restriction result |
| `Unio<R1, R2>` | *unio* | union | Union of two rows |
| `Disiunctus` | *disiunctus* | disjoint | Disjoint rows marker |
| `Carentia<L>` | *carentia* | lack, absence | Row lacks label |
| `OrdineConiunctio` | *ordo + coniunctio* | row cons | Type-level row cons |

#### Type-Level Booleans

| Type | Latin Name | Etymology | Purpose |
|------|------------|-----------|---------|
| `Bool` | *booleanus* | Boolean | Type-level boolean trait |
| `Verum` | *verum* | true, truth | Type-level true |
| `Falsum` | *falsum* | false | Type-level false |

#### Campus — Field Operations

| Trait | Latin Name | Etymology | Purpose |
|-------|------------|-----------|---------|
| `HabetCampum<L, V, I>` | *habet campum* | has field | Field presence constraint |
| `Extendo<L, V>` | *extendo* | extend, stretch | Record extension |
| `Restricto<L, I>` | *restricto* | restrict, confine | Field removal |
| `Confluo<O>` | *confluo* | flow together | Record merging |
| `Muto<L, V, I>` | *muto* | change, modify | Field modification |
| `Renomino<O, N, I>` | *renomino* | rename | Field renaming |

#### Registrum — Extensible Records

| Type | Latin Name | Etymology | Purpose |
|------|------------|-----------|---------|
| `Registrum<R>` | *registrum* | register, record book | Extensible record wrapper |
| `RegistrumExt<R>` | *registrum extensio* | record extension | Helper trait for records |

#### Registrum Operations

| Method | Latin Root | Purpose |
|--------|------------|---------|
| `extend_field` | *extendere* | Add new field to record |
| `get` | *habere* | Get field reference by label |
| `get_mut` | *habere mutare* | Get mutable field reference |
| `restrict` | *restringere* | Remove field from record |
| `merge` | *confluere* | Merge two records |
| `modify` | *mutare* | Transform field value |

#### Variatio — Extensible Variants

| Type | Latin Name | Etymology | Purpose |
|------|------------|-----------|---------|
| `Variatio<R>` | *variatio* | variation, variety | Extensible variant type |
| `Casus<L, V>` | *casus* | case | Case type alias |
| `CaseResult<R, T>` | *exitus casus* | case result | Pattern match result |
| `MatchBuilder<R, H>` | *aedificator* | builder | Pattern match builder |
| `ExtendoCasum<L, V>` | *extendo casum* | extend case | Variant extension |

#### Variatio Index Markers

| Type | Latin Name | Etymology | Purpose |
|------|------------|-----------|---------|
| `HabetCasum<L, V, I>` | *habet casum* | has case | Case membership marker |
| `CasusHic` | *casus hic* | case here | Head position marker |
| `CasusIbi<T>` | *casus ibi* | case there | Tail position marker |

#### Variatio Operations

| Function | Latin Root | Purpose |
|----------|------------|---------|
| `inject` | *inicere* | Inject value into variant |
| `on` | *super* | Handle single case |
| `try_get` | *temptare* | Try extract value |
| `is` | *esse* | Check if variant holds case |
| `match_on` | *aequare* | Start pattern match |
| `widen` | *dilatare* | Widen variant type |
| `otherwise` | *aliter* | Default value for unmatched |
| `otherwise_with` | *aliter cum* | Computed default |
| `exhaust` | *exhaurire* | Assert exhaustive match |

#### Row-Polymorphic Functions

Row polymorphism enables functions that work with any record containing required fields:

```rust
use ordofp_core::rows::*;

fn greet<R, I>(record: &Registrum<R>) -> String
where
    R: HabetCampum<Name, String, I>,
{
    format!("Hello, {}!", record.get::<Name, String, I>())
}
```

The function accepts any record with a `Name: String` field, regardless of other fields.

### OrdoFP 4.0 Phase 6: Free Monads & Tagless Final

> *"Libertas est potestas faciendi id quod iure licet."*
> — Freedom is the power to do what is permitted by law.

The 4.0 Phase 6 implements Free monads and Tagless Final patterns for building
and interpreting domain-specific languages (DSLs) with multiple interpretations.

#### TransformatioNaturalis — Natural Transformations

| Type | Latin Name | Etymology | Purpose |
|------|------------|-----------|---------|
| `TransformatioNaturalis<F, G>` | *transformatio naturalis* | natural change | Morphism between functors |
| `TransformatioIdentitas<F>` | *transformatio identitas* | identity change | `F ~> F` |
| `TransformatioCompositio` | *transformatio compositio* | composed change | `(G ~> H) ∘ (F ~> G)` |
| `TransformatioFn<F, G>` | *transformatio functionis* | function change | Function-based transformation |

#### Functor Witnesses

| Type | Latin Name | Etymology | Purpose |
|------|------------|-----------|---------|
| `OptionFWitness` | *testis Option* | Option witness | HKT witness for Option |
| `ResultFWitness<E>` | *testis Result* | Result witness | HKT witness for Result |
| `IdentitasFWitness` | *testis identitas* | identity witness | Identity functor |
| `ConstFWitness<C>` | *testis constans* | constant witness | Constant functor |
| `ConstStringFWitness` | *testis stringae* | string witness | For pretty printing |
| `ConstUsizeFWitness` | *testis numeri* | number witness | For counting |

#### Liber — Free Monad

| Type | Latin Name | Etymology | Purpose |
|------|------------|-----------|---------|
| `Liber<F, A>` | *liber* | free | Free monad over functor F |
| `Purus` | *purus* | pure, clean | Pure value constructor |
| `Suspensus` | *suspensus* | hanging | Suspended computation |
| `LiberWitness<F>` | *testis liber* | free witness | HKT witness for Liber |
| `MonadHKT` | *monas HKT* | monad | Monad operations for HKT |

#### Liber Operations

| Function | Latin Root | Purpose |
|----------|------------|---------|
| `purus` | *purus* | Create pure value |
| `suspensus` | *suspensus* | Create suspended computation |
| `est_purus` | *est purus* | Check if pure |
| `est_suspensus` | *est suspensus* | Check if suspended |
| `liftF` | *levare* | Lift functor into Free |
| `plica_liber` | *plico* | Fold Free monad |
| `itero_liber` | *itero* | Iterate Free monad |
| `join_liber` | *iungo* | Flatten nested Free |
| `purus_liber` | *purus* | Helper for pure value |

#### Liberior — Freer Monad

| Type | Latin Name | Etymology | Purpose |
|------|------------|-----------|---------|
| `Liberior<F, A>` | *liberior* | more free | Freer monad (no Functor) |
| `LiberiorSuspensio<F, A>` | *suspensio liberior* | freer suspension | Suspended with continuation |
| `Impurus` | *impurus* | not pure | Impure computation |
| `CatalogusFunctionum` | *catalogus functionum* | function list | Efficient continuations |

#### Liberior Operations

| Function | Latin Root | Purpose |
|----------|------------|---------|
| `mitto_liberior` | *mitto* | Send effect |
| `curro_purus_liberior` | *curro* | Run pure computation |
| `TractatorLiberior` | *tractator* | Effect handler trait |

#### Effect Operation Types

| Type | Latin Name | Etymology | Purpose |
|------|------------|-----------|---------|
| `StatusOperatio<S>` | *operatio status* | state operation | State effect |
| `LectorOperatio<E>` | *operatio lectoris* | reader operation | Reader effect |
| `ScriptorOperatio<W>` | *operatio scriptoris* | writer operation | Writer effect |

#### AlgebraArithmetica — Arithmetic Algebra

| Method | Latin Name | Etymology | Purpose |
|--------|------------|-----------|---------|
| `lit` | *lit* | literal | Integer literal |
| `addo` | *addo* | I add | Addition |
| `multiplico` | *multiplico* | I multiply | Multiplication |
| `subtraho` | *subtraho* | I subtract | Subtraction |
| `nego` | *nego* | I negate | Negation |

#### AlgebraBooleana — Boolean Algebra

| Method | Latin Name | Etymology | Purpose |
|--------|------------|-----------|---------|
| `verum` | *verum* | true | True literal |
| `falsum` | *falsum* | false | False literal |
| `et` | *et* | and | Logical AND |
| `vel` | *vel* | or | Logical OR |
| `non` | *non* | not | Logical NOT |

#### Interpreters (Interpretes)

| Type | Latin Name | Etymology | Purpose |
|------|------------|-----------|---------|
| `InterpresAestimationis` | *interpres aestimationis* | evaluation interpreter | Compute values |
| `InterpresPulcher` | *interpres pulcher* | pretty interpreter | Pretty print |
| `InterpresNumerans` | *interpres numerans* | counting interpreter | Count operations |
| `InterpresOptimans` | *interpres optimans* | optimizing interpreter | Constant folding |

#### Optimization Types

| Type | Latin Name | Etymology | Purpose |
|------|------------|-----------|---------|
| `OptimumExpr<T>` | *optimum expressio* | optimal expression | Optimized result |
| `Constans` | *constans* | constant | Known constant value |
| `Incognitus` | *incognitus* | unknown | Unknown expression |
| `OptimumFWitness` | *testis optimi* | optimal witness | HKT for OptimumExpr |

#### Combined Algebras

| Trait | Latin Name | Etymology | Purpose |
|-------|------------|-----------|---------|
| `AlgebraComparationis` | *algebra comparationis* | comparison algebra | `aequalis`, `minor`, `maior` |
| `AlgebraConditionalis` | *algebra conditionalis* | conditional algebra | `si` (if-then-else) |
| `AlgebraSuperior` | *algebra superior* | higher algebra | `lam`, `app` (HOAS) |
| `Symantica` | *symantica* | semantics | Combined syntax+semantics |

---

### OrdoFP 4.0 Phase 7: Category Theory Foundations

> *"Categoria est praedicamentum universale."*
> — A category is a universal predicament. (Aristotle, adapted)

The 4.0 Phase 7 implements advanced category theory constructs including
enhanced Arrow type classes and Kan extensions.

#### Sagitta — Enhanced Arrow Type Classes

| Type | Latin Name | Etymology | Purpose |
|------|------------|-----------|---------|
| `SagittaElectio` | *sagitta electionis* | arrow of choice | ArrowChoice trait |
| `SagittaApplicatio` | *sagitta applicationis* | arrow of application | ArrowApply trait |
| `SagittaCirculus` | *sagitta circuli* | arrow of the circle | ArrowLoop trait |

#### SagittaElectio — ArrowChoice Methods

| Method | Latin Name | Etymology | Purpose |
|--------|------------|-----------|---------|
| `sinister` | *sinister* | left | Apply to left of Aut |
| `dexter` | *dexter* | right | Apply to right of Aut |
| `confluo` | *confluo* | flow together | Fanin (merge) |
| `addo` | *addo* | I add | Sum/plus on Aut |

#### SagittaApplicatio — ArrowApply Methods

| Method | Latin Name | Etymology | Purpose |
|--------|------------|-----------|---------|
| `applicatio` | *applicatio* | application | Apply arrow to value |

#### SagittaCirculus — ArrowLoop Methods

| Method | Latin Name | Etymology | Purpose |
|--------|------------|-----------|---------|
| `circulus` | *circulus* | circle | Create feedback loop |

#### Functiones Sagittarum — Arrow Function Operations

| Function | Latin Name | Etymology | Purpose |
|----------|------------|-----------|---------|
| `id` | *identitas* | sameness | Identity arrow |
| `compose` | *compono* | put together | Function composition |
| `arr` | *elevo* | lift | Lift to boxed arrow |
| `first` | *primus* | first | Apply to first of pair |
| `second` | *secundus* | second | Apply to second of pair |
| `split` | *divido* | divide | Parallel processing |

#### Utilitas — Utility Functions

| Function | Latin Name | Etymology | Purpose |
|----------|------------|-----------|---------|
| `via_praedicatum` | *via praedicati* | by way of predicate | Route by condition |
| `coalesco` | *coalesco* | grow together | Merge Aut branches |
| `inicio_sinister` | *inicio sinistrum* | begin left | Inject left |
| `inicio_dexter` | *inicio dextrum* | begin right | Inject right |

#### Extensiones Kan — Kan Extensions

| Type | Latin Name | Etymology | Purpose |
|------|------------|-----------|---------|
| `Yoneda<F, A>` | *Yoneda* | Named: Nobuo Yoneda | Yoneda embedding |
| `Coyoneda<F, A>` | *Coyoneda* | co- + Yoneda | Free functor |
| `ExtensioKanDextra` | *extensio Kan dextra* | right Kan extension | Ran |
| `ExtensioKanSinistra` | *extensio Kan sinistra* | left Kan extension | Lan |
| `Codensitas<G, A>` | *codensitas* | co-density | Codensity monad |
| `Densitas<G, A>` | *densitas* | thickness | Density comonad |
| `ConvolutioDiei<F, G, A>` | *convolutio diei* | Day convolution | Monoidal tensor |

#### Yoneda & Coyoneda Operations

| Method | Latin Name | Etymology | Purpose |
|--------|------------|-----------|---------|
| `lift` | *levo* | I lift | Lift into Coyoneda |
| `lower` | *demitto* | I lower | Extract from Yoneda |
| `is_type` | *est typus* | is type | Type checking |

#### Codensitas — Codensity Monad Operations

| Method | Latin Name | Etymology | Purpose |
|--------|------------|-----------|---------|
| `purus` | *purus* | pure | Lift pure value |
| `run_with` | *curro cum* | run with | Execute with continuation |

#### Densitas — Density Comonad Operations

| Method | Latin Name | Etymology | Purpose |
|--------|------------|-----------|---------|
| `extractum` | *extractum* | extracted | Extract value (comonad) |
| `map` | *mappa* | map | Functor mapping |

---

## V. Auctoritates — Sources and References

### Sacred Scripture
- **Matthew 13:52** — *Thesaurus*: "Every scribe trained for the kingdom of heaven is like a householder who brings out of his treasure (*thesaurus*) what is new and what is old."
- **Psalm 19:1** — *Astronomia*: "The heavens declare the glory of God."
- **John 1:1** — *Verbum*: "In the beginning was the Word."

### Aristotle
- **Categories** — Classification of being, foundation for type theory
- **Prior Analytics** — Syllogistic logic, *Omnis* and *Aliquid* quantifiers
- **Metaphysics** — *Unum*, *compositio*, *potentia* and *actus*
- **Physics** — *Motus* (change), foundation for state transformers

### St. Thomas Aquinas
- **Summa Theologiae I, q.11** — On divine unity (*Unitas*)
- **Summa Theologiae I, q.16** — On truth and *compositio*
- **De Veritate** — On the composition and division of judgments
- **Commentary on Aristotle's Metaphysics** — Transcendentals (*unum*, *aliquid*)

### Cicero
- **De Inventione** — Rhetorical patterns, *divisio* and *compositio*
- **Topica** — Logical topics, foundation for type-directed reasoning

### Boethius
- **De Institutione Arithmetica** — Quadrivium structure
- **De Consolatione Philosophiae** — Unity and multiplicity

### Liberal Arts Tradition
- **Martianus Capella** — *De Nuptiis*, codification of Trivium/Quadrivium
- **Isidore of Seville** — *Etymologiae*, encyclopedic naming
- **Hugh of St. Victor** — *Didascalicon*, ordering of knowledge

---

*"In nomine Patris, et Filii, et Spiritus Sancti. Amen."*
