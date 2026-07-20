//! Serializable Computation Nodes
//!
//! > *"Transmissio est continuatio computationis"*
//! > — Transmission is the continuation of computation. (Latin)
//!
//! This module provides serializable representations of computation nodes
//! that can be transmitted across network boundaries for distributed
//! execution. (The node vocabulary originated with the since-archived UCIR
//! subsystem; the types here are self-contained.)

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

// =============================================================================
// Serializable Node Identity
// =============================================================================

/// Unique identifier for a serializable UCIR node.
///
/// # Latin Etymology
/// *Identitas nodi* = identity of a node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodusId(pub u64);

impl NodusId {
    /// Create a new node ID.
    #[inline]
    pub const fn new(id: u64) -> Self {
        NodusId(id)
    }

    /// Generate a new unique ID.
    #[cfg(feature = "std")]
    pub fn generate() -> Self {
        use core::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        NodusId(COUNTER.fetch_add(1, Ordering::SeqCst))
    }
}

// =============================================================================
// Serializable Metadata
// =============================================================================

/// Serializable metadata for UCIR nodes.
///
/// # Latin Etymology
/// *Metadata nodi* = metadata of a node.
#[derive(Debug, Clone, Default)]
pub struct MetadataNodi {
    /// Human-readable name.
    pub nomen: Option<String>,
    /// Source location.
    pub locus: Option<String>,
    /// Cost hints for optimization.
    pub indicium_sumptus: Option<IndiciumSumptus>,
}

/// Serializable cost hint.
#[derive(Debug, Clone, Copy)]
pub struct IndiciumSumptus {
    /// Estimated CPU cycles per element.
    pub cycli_cpu: u64,
    /// Estimated memory bytes per element.
    pub bytes_memoriae: u64,
    /// Whether parallelizable.
    pub parallelizabilis: bool,
    /// Whether vectorizable.
    pub vectorizabilis: bool,
}

impl Default for IndiciumSumptus {
    fn default() -> Self {
        IndiciumSumptus {
            cycli_cpu: 1,
            bytes_memoriae: 0,
            parallelizabilis: true,
            vectorizabilis: false,
        }
    }
}

// =============================================================================
// Serializable Expressions
// =============================================================================

/// Serializable expression tree for functions.
///
/// This enables functions to be transmitted across network boundaries
/// and reconstructed on remote nodes.
///
/// **Evaluation is unimplemented:** this AST can be built, inspected, and
/// (de)serialized, but the crate ships no evaluator for it — the prepared
/// scaffolding (`Ambitus`, `ErrorEvaluationis`) is unused, and a remote
/// node has no way to actually *run* a received `Expressio` yet.
///
/// # Latin Etymology
/// *Expressio* = expression, representation.
#[derive(Debug, Clone)]
pub enum Expressio {
    /// Identity function: λx. x
    Identitas,

    /// Constant value.
    Constans(ValorConstans),

    /// Variable reference by de Bruijn index.
    Variabilis(usize),

    /// Named variable reference.
    VariabilisNominatus(String),

    /// Function composition: f ∘ g
    Compositio(Box<Expressio>, Box<Expressio>),

    /// Lambda abstraction: λx. body
    Lambda {
        /// Parameter name (for debugging).
        parametrum: Option<String>,
        /// Body expression.
        corpus: Box<Expressio>,
    },

    /// Function application: f(x)
    Applicatio(Box<Expressio>, Box<Expressio>),

    /// Arithmetic operation.
    Arithmetica(OperatioArithmetica, Box<Expressio>, Box<Expressio>),

    /// Unary arithmetic operation.
    ArithmeticaUnaria(OperatioUnariaArithmetica, Box<Expressio>),

    /// Comparison operation.
    Comparatio(OperatioComparationis, Box<Expressio>, Box<Expressio>),

    /// Logical operation.
    Logica(OperatioLogica, Box<Expressio>, Box<Expressio>),

    /// Unary logical operation.
    LogicaUnaria(OperatioUnariaLogica, Box<Expressio>),

    /// Conditional: if cond then e1 else e2
    Condicio {
        /// Condition expression; expected to evaluate to a boolean.
        condicio: Box<Expressio>,
        /// Expression taken when the condition holds.
        tunc: Box<Expressio>,
        /// Expression taken when the condition does not hold.
        aliter: Box<Expressio>,
    },

    /// Field access on a record.
    Ager(Box<Expressio>, String),

    /// Tuple/array projection by index.
    Proiectio(Box<Expressio>, usize),

    /// Tuple construction.
    Tupla(Vec<Expressio>),

    /// Record construction.
    Recordum(Vec<(String, Expressio)>),

    /// Let binding: let x = e1 in e2
    Ligatio {
        /// Name under which the bound value is visible inside the body.
        nomen: String,
        /// Expression producing the bound value.
        valor: Box<Expressio>,
        /// Body expression in which the binding is in scope.
        corpus: Box<Expressio>,
    },

    /// Match expression (pattern matching).
    Conformatio {
        /// Scrutinee: the expression whose value is matched against the branches.
        scrutinium: Box<Expressio>,
        /// Branches, tried in order; the first whose pattern (and guard,
        /// if present) matches supplies the result.
        rami: Vec<RamusConformationis>,
    },
}

/// A branch in pattern matching.
#[derive(Debug, Clone)]
pub struct RamusConformationis {
    /// Pattern to match.
    pub exemplar: Exemplar,
    /// Guard condition (optional).
    pub custos: Option<Box<Expressio>>,
    /// Body expression.
    pub corpus: Expressio,
}

/// Pattern for matching.
#[derive(Debug, Clone)]
pub enum Exemplar {
    /// Wildcard: matches anything.
    Quodlibet,
    /// Variable binding.
    Variabilis(String),
    /// Literal constant.
    Constans(ValorConstans),
    /// Tuple pattern.
    Tupla(Vec<Exemplar>),
    /// Constructor pattern.
    Constructor {
        /// Constructor name to match (e.g. an enum variant name).
        nomen: String,
        /// Sub-patterns matched positionally against the constructor's arguments.
        parametra: Vec<Exemplar>,
    },
}

// =============================================================================
// Constant Values
// =============================================================================

/// Serializable constant value.
///
/// # Latin Etymology
/// *Valor constans* = constant value.
#[derive(Debug, Clone)]
pub enum ValorConstans {
    /// Unit value.
    Unitas,
    /// Boolean.
    Verum(bool),
    /// Signed 8-bit integer.
    I8(i8),
    /// Signed 16-bit integer.
    I16(i16),
    /// Signed 32-bit integer.
    I32(i32),
    /// Signed 64-bit integer.
    I64(i64),
    /// Signed 128-bit integer.
    I128(i128),
    /// Unsigned 8-bit integer.
    U8(u8),
    /// Unsigned 16-bit integer.
    U16(u16),
    /// Unsigned 32-bit integer.
    U32(u32),
    /// Unsigned 64-bit integer.
    U64(u64),
    /// Unsigned 128-bit integer.
    U128(u128),
    /// 32-bit float, stored as its raw IEEE-754 bits so transmission
    /// round-trips exactly (including NaN payloads). Construct via
    /// [`ValorConstans::from_f32`], read via [`ValorConstans::as_f32`].
    F32(u32),
    /// 64-bit float, stored as its raw IEEE-754 bits so transmission
    /// round-trips exactly (including NaN payloads). Construct via
    /// [`ValorConstans::from_f64`], read via [`ValorConstans::as_f64`].
    F64(u64),
    /// Character.
    Character(char),
    /// String.
    Filum(String),
    /// Byte array.
    Bytes(Vec<u8>),
}

impl ValorConstans {
    /// Create f32 from value.
    #[inline]
    pub fn from_f32(v: f32) -> Self {
        ValorConstans::F32(v.to_bits())
    }

    /// Create f64 from value.
    #[inline]
    pub fn from_f64(v: f64) -> Self {
        ValorConstans::F64(v.to_bits())
    }

    /// Get f32 value.
    #[inline]
    pub fn as_f32(&self) -> Option<f32> {
        match self {
            ValorConstans::F32(bits) => Some(f32::from_bits(*bits)),
            _ => None,
        }
    }

    /// Get f64 value.
    #[inline]
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            ValorConstans::F64(bits) => Some(f64::from_bits(*bits)),
            _ => None,
        }
    }
}

// =============================================================================
// Operations
// =============================================================================

/// Binary arithmetic operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperatioArithmetica {
    /// Addition.
    Additio,
    /// Subtraction.
    Subtractio,
    /// Multiplication.
    Multiplicatio,
    /// Division.
    Divisio,
    /// Remainder.
    Residuum,
    /// Bitwise AND.
    Et,
    /// Bitwise OR.
    Vel,
    /// Bitwise XOR.
    Aut,
    /// Left shift.
    Sinistrorsum,
    /// Right shift.
    Dextrorsum,
}

/// Unary arithmetic operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperatioUnariaArithmetica {
    /// Negation.
    Negatio,
    /// Bitwise NOT.
    Inversio,
    /// Absolute value.
    Absolutum,
}

/// Comparison operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperatioComparationis {
    /// Equal.
    Aequale,
    /// Not equal.
    NonAequale,
    /// Less than.
    Minus,
    /// Less than or equal.
    MinusVelAequale,
    /// Greater than.
    Maius,
    /// Greater than or equal.
    MaiusVelAequale,
}

/// Binary logical operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperatioLogica {
    /// Logical AND.
    Et,
    /// Logical OR.
    Vel,
    /// Logical XOR.
    Aut,
    /// Implication.
    Implicatio,
}

/// Unary logical operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperatioUnariaLogica {
    /// Logical NOT.
    Non,
}

// =============================================================================
// Serializable UCIR Node
// =============================================================================

/// Serializable UCIR computation node.
///
/// This is a serializable representation of `UcirNode` that can be
/// transmitted across network boundaries.
///
/// # Latin Etymology
/// *Nodus serializabilis* = serializable node.
#[derive(Debug, Clone)]
pub enum NodusSerializabilis {
    // === Sources ===
    /// Empty source.
    Vacuus,

    /// Single value.
    Semel(ValorConstans),

    /// Repeated value.
    Repetitio(ValorConstans),

    /// Integer range.
    Intervallum {
        /// Start of the range (first element produced).
        initium: i64,
        /// End bound of the range.
        finis: i64,
        /// Step between consecutive elements.
        gradus: i64,
    },

    /// External data source reference.
    Externus {
        /// Kind of external source; interpretation is up to the executing
        /// node (e.g. a connector or driver name).
        genus: String,
        /// Opaque configuration string understood by that source kind.
        configuratio: String,
    },

    // === Transformations ===
    /// Map operation.
    Mappa {
        /// Upstream node supplying the input elements.
        fons: Box<NodusSerializabilis>,
        /// Function applied to each element.
        functio: Expressio,
        /// Node metadata (name, source location, cost hints).
        meta: MetadataNodi,
    },

    /// Filter operation.
    Filtrum {
        /// Upstream node supplying the input elements.
        fons: Box<NodusSerializabilis>,
        /// Predicate; elements for which it does not hold are dropped.
        praedicatum: Expressio,
        /// Node metadata (name, source location, cost hints).
        meta: MetadataNodi,
    },

    /// Combined filter and map.
    FiltrumMappa {
        /// Upstream node supplying the input elements.
        fons: Box<NodusSerializabilis>,
        /// Fused select-and-transform applied to each element.
        functio: Expressio,
        /// Node metadata (name, source location, cost hints).
        meta: MetadataNodi,
    },

    /// Stateful scan.
    Lustrum {
        /// Upstream node supplying the input elements.
        fons: Box<NodusSerializabilis>,
        /// Initial accumulator state.
        initium: ValorConstans,
        /// State-transition function combining the accumulator with each
        /// element; intermediate states are emitted downstream.
        functio: Expressio,
        /// Node metadata (name, source location, cost hints).
        meta: MetadataNodi,
    },

    /// `FlatMap` operation.
    MappaPlana {
        /// Upstream node supplying the input elements.
        fons: Box<NodusSerializabilis>,
        /// Function mapping each element to a sub-stream whose elements are
        /// flattened into the output.
        functio: Expressio,
        /// Node metadata (name, source location, cost hints).
        meta: MetadataNodi,
    },

    /// Take N elements.
    Cape {
        /// Upstream node supplying the input elements.
        fons: Box<NodusSerializabilis>,
        /// Maximum number of elements passed through before the stream ends.
        numerus: usize,
    },

    /// Skip N elements.
    Omitte {
        /// Upstream node supplying the input elements.
        fons: Box<NodusSerializabilis>,
        /// Number of leading elements to discard.
        numerus: usize,
    },

    // === Aggregations ===
    /// Fold operation.
    Plica {
        /// Upstream node supplying the input elements.
        fons: Box<NodusSerializabilis>,
        /// Initial accumulator value.
        initium: ValorConstans,
        /// Folding function combining the accumulator with each element;
        /// only the final accumulator is produced.
        functio: Expressio,
        /// Node metadata (name, source location, cost hints).
        meta: MetadataNodi,
    },

    /// Reduce operation.
    Reductio {
        /// Upstream node supplying the input elements.
        fons: Box<NodusSerializabilis>,
        /// Combining function applied pairwise over the elements, using the
        /// first element as the seed (no separate initial value).
        functio: Expressio,
        /// Node metadata (name, source location, cost hints).
        meta: MetadataNodi,
    },

    /// Count elements.
    Numera {
        /// Upstream node whose elements are counted.
        fons: Box<NodusSerializabilis>,
    },

    /// Aggregate operation (sum, product, min, max).
    Aggregatio {
        /// Upstream node supplying the input elements.
        fons: Box<NodusSerializabilis>,
        /// Which built-in aggregate to compute.
        operatio: OperatioAggregationis,
    },

    // === Composition ===
    /// Zip two streams.
    Iunge {
        /// Left input; its elements form the first component of each pair.
        sinister: Box<NodusSerializabilis>,
        /// Right input; its elements form the second component of each pair.
        dexter: Box<NodusSerializabilis>,
        /// Node metadata (name, source location, cost hints).
        meta: MetadataNodi,
    },

    /// Chain two streams.
    Catena {
        /// Stream drained first.
        primus: Box<NodusSerializabilis>,
        /// Stream appended after the first is exhausted.
        secundus: Box<NodusSerializabilis>,
        /// Node metadata (name, source location, cost hints).
        meta: MetadataNodi,
    },

    // === Grouping ===
    /// Group by key.
    GregaPer {
        /// Upstream node supplying the input elements.
        fons: Box<NodusSerializabilis>,
        /// Key-extraction function; elements with equal keys are grouped
        /// together.
        clavis: Expressio,
        /// Node metadata (name, source location, cost hints).
        meta: MetadataNodi,
    },

    /// Chunk into fixed size.
    Fragmentum {
        /// Upstream node supplying the input elements.
        fons: Box<NodusSerializabilis>,
        /// Number of elements per chunk.
        magnitudo: usize,
    },

    /// Sliding window.
    Fenestra {
        /// Upstream node supplying the input elements.
        fons: Box<NodusSerializabilis>,
        /// Window length, in elements.
        magnitudo: usize,
        /// Stride between successive window start positions, in elements.
        gradus: usize,
    },

    // === Concurrency ===
    /// Parallel execution.
    Parallela {
        /// First branch, executed concurrently with the second.
        sinister: Box<NodusSerializabilis>,
        /// Second branch, executed concurrently with the first.
        dexter: Box<NodusSerializabilis>,
        /// Node metadata (name, source location, cost hints).
        meta: MetadataNodi,
    },

    /// Race: first to complete wins.
    Certamen {
        /// First competing branch.
        sinister: Box<NodusSerializabilis>,
        /// Second competing branch; whichever completes first supplies the
        /// result.
        dexter: Box<NodusSerializabilis>,
        /// Node metadata (name, source location, cost hints).
        meta: MetadataNodi,
    },

    // === Control Flow ===
    /// Conditional branch.
    Ramus {
        /// Node whose result selects which branch runs.
        condicio: Box<NodusSerializabilis>,
        /// Branch taken when the condition holds.
        tunc: Box<NodusSerializabilis>,
        /// Branch taken when the condition does not hold.
        aliter: Box<NodusSerializabilis>,
        /// Node metadata (name, source location, cost hints).
        meta: MetadataNodi,
    },

    /// Loop.
    Circuitus {
        /// Body executed on each iteration.
        corpus: Box<NodusSerializabilis>,
        /// Termination rule deciding how many iterations run.
        condicio: CondicioCircuitus,
        /// Node metadata (name, source location, cost hints).
        meta: MetadataNodi,
    },

    // === Effects ===
    /// Perform effect.
    Effice {
        /// Upstream node supplying the input elements.
        fons: Box<NodusSerializabilis>,
        /// Identifier of the effect to perform; corresponds to
        /// `EffectusDistributus::EFFECTUS_ID` in the distributed effect
        /// system.
        effectus_id: u64,
        /// Expression describing the effect operation to perform.
        operatio: Expressio,
        /// Node metadata (name, source location, cost hints).
        meta: MetadataNodi,
    },

    /// Handle effects.
    Tracta {
        /// Upstream node whose effects are handled.
        fons: Box<NodusSerializabilis>,
        /// Handler installed over the source; intercepts effects whose ID it
        /// declares.
        tractator: TractatorSerializabilis,
        /// Node metadata (name, source location, cost hints).
        meta: MetadataNodi,
    },

    // === Optimization Hints ===
    /// Mark for fusion.
    Fusio(Box<NodusSerializabilis>),

    /// Mark for vectorization.
    Vectoriza(Box<NodusSerializabilis>),

    /// Mark for GPU.
    GpuIndica(Box<NodusSerializabilis>),
}

/// Aggregation operation type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperatioAggregationis {
    /// Sum.
    Summa,
    /// Product.
    Productum,
    /// Minimum.
    Minimum,
    /// Maximum.
    Maximum,
    /// Average/Mean.
    Media,
    /// First element.
    Primus,
    /// Last element.
    Ultimus,
}

/// Loop condition.
#[derive(Debug, Clone)]
pub enum CondicioCircuitus {
    /// Infinite loop.
    Infinitus,
    /// Fixed count.
    Numerus(usize),
    /// While condition.
    Dum(Expressio),
    /// Until condition.
    Donec(Expressio),
}

/// Serializable effect handler.
#[derive(Debug, Clone)]
pub struct TractatorSerializabilis {
    /// Effect ID this handler handles.
    pub effectus_id: u64,
    /// Handler implementation as expression.
    pub implementatio: Expressio,
    /// Whether this is a deep handler.
    pub profundus: bool,
}

// =============================================================================
// Conversion Traits
// =============================================================================

/// Trait for types that can be converted to serializable form.
///
/// # Latin Etymology
/// *Ad serializabilis* = to serializable.
pub trait AdSerializabilis {
    /// The serializable representation.
    type Serializabilis;

    /// Convert to serializable form.
    fn ad_serializabilis(&self) -> Self::Serializabilis;
}

/// Trait for types that can be reconstructed from serializable form.
///
/// # Latin Etymology
/// *Ex serializabilis* = from serializable.
pub trait ExSerializabilis: Sized {
    /// The serializable representation.
    type Serializabilis;

    /// Error type for reconstruction failures.
    type Error;

    /// Reconstruct from serializable form.
    ///
    /// # Errors
    ///
    /// Returns [`Self::Error`](ExSerializabilis::Error) when the serializable
    /// form cannot be turned back into a valid `Self`. The exact conditions
    /// are implementation-defined — typically an unknown node or expression
    /// kind, or data the target type cannot represent.
    fn ex_serializabilis(ser: Self::Serializabilis) -> Result<Self, Self::Error>;
}

// =============================================================================
// Expression Evaluation
// =============================================================================

/// Environment for expression evaluation.
pub struct Ambitus {
    /// Variable bindings by de Bruijn index.
    variabiles: Vec<ValorConstans>,
    /// Named variable bindings.
    nominati: Vec<(String, ValorConstans)>,
}

impl Ambitus {
    /// Create a new empty environment.
    #[inline]
    pub fn new() -> Self {
        Ambitus {
            variabiles: Vec::with_capacity(8),
            nominati: Vec::with_capacity(8),
        }
    }

    /// Push a variable binding.
    #[inline]
    pub fn push(&mut self, valor: ValorConstans) {
        self.variabiles.push(valor);
    }

    /// Pop a variable binding.
    #[inline]
    pub fn pop(&mut self) -> Option<ValorConstans> {
        self.variabiles.pop()
    }

    /// Get a variable by index.
    #[inline]
    pub fn get(&self, index: usize) -> Option<&ValorConstans> {
        let len = self.variabiles.len();
        if index < len {
            Some(&self.variabiles[len - 1 - index])
        } else {
            None
        }
    }

    /// Bind a named variable.
    #[inline]
    pub fn bind(&mut self, nomen: String, valor: ValorConstans) {
        self.nominati.push((nomen, valor));
    }

    /// Lookup a named variable.
    #[inline]
    pub fn lookup(&self, nomen: &str) -> Option<&ValorConstans> {
        for (n, v) in self.nominati.iter().rev() {
            if n == nomen {
                return Some(v);
            }
        }
        None
    }
}

impl Default for Ambitus {
    fn default() -> Self {
        Self::new()
    }
}

/// Error during expression evaluation.
#[derive(Debug, Clone)]
pub enum ErrorEvaluationis {
    /// Variable not found.
    VariabilisNonInventa(String),
    /// Type mismatch.
    TypusDiscrepans {
        /// Name of the type the operation required.
        expectatus: String,
        /// Name of the type actually encountered.
        inventus: String,
    },
    /// Division by zero.
    DivisioPerNullum,
    /// Pattern match failure.
    ConformatioDefecit,
    /// Invalid operation.
    OperatioInvalida(String),
}

impl fmt::Display for ErrorEvaluationis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorEvaluationis::VariabilisNonInventa(name) => {
                write!(f, "Variable not found: {name}")
            }
            ErrorEvaluationis::TypusDiscrepans {
                expectatus,
                inventus,
            } => {
                write!(f, "Type mismatch: expected {expectatus}, found {inventus}")
            }
            ErrorEvaluationis::DivisioPerNullum => {
                write!(f, "Division by zero")
            }
            ErrorEvaluationis::ConformatioDefecit => {
                write!(f, "Pattern match failed")
            }
            ErrorEvaluationis::OperatioInvalida(msg) => {
                write!(f, "Invalid operation: {msg}")
            }
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nodus_id() {
        let id1 = NodusId::new(1);
        let id2 = NodusId::new(2);
        assert_ne!(id1, id2);
        assert_eq!(id1, NodusId::new(1));
    }

    #[test]
    fn test_valor_constans_f64() {
        let v = ValorConstans::from_f64(2.5);
        let f = v
            .as_f64()
            .expect("ValorConstans::from_f64 should produce an F64 variant");
        assert!((f - 2.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_expressio_identity() {
        let expr = Expressio::Identitas;
        assert!(matches!(expr, Expressio::Identitas));
    }

    #[test]
    fn test_expressio_arithmetic() {
        let expr = Expressio::Arithmetica(
            OperatioArithmetica::Additio,
            Box::new(Expressio::Constans(ValorConstans::I32(1))),
            Box::new(Expressio::Constans(ValorConstans::I32(2))),
        );
        assert!(matches!(
            expr,
            Expressio::Arithmetica(OperatioArithmetica::Additio, _, _)
        ));
    }

    #[test]
    fn test_ambitus() {
        let mut env = Ambitus::new();
        env.push(ValorConstans::I32(42));
        assert_eq!(
            env.get(0).and_then(|v| match v {
                ValorConstans::I32(n) => Some(*n),
                _ => None,
            }),
            Some(42)
        );
    }

    #[test]
    fn test_nodus_serializabilis_range() {
        let node = NodusSerializabilis::Intervallum {
            initium: 0,
            finis: 100,
            gradus: 1,
        };
        assert!(matches!(node, NodusSerializabilis::Intervallum { .. }));
    }
}
