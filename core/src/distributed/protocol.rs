//! Distributed Communication Protocol
//!
//! > *"Protocollum est lingua machinarum"*
//! > — Protocol is the language of machines. (Latin)
//!
//! This module defines the wire protocol for distributed
//! communication between nodes.

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;
use core::time::Duration;

use super::node::{InformationesNodi, NodusIdentitas, StatusNodi};
use super::serializable::NodusSerializabilis;

/// Default priority for submitted computations.
const DEFAULT_COMPUTATION_PRIORITY: u32 = 100;

// =============================================================================
// Protocol Version
// =============================================================================

/// Protocol version for compatibility checking.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VersioProtocolli {
    /// Major version (breaking changes).
    pub maior: u16,
    /// Minor version (backward compatible features).
    pub minor: u16,
    /// Patch version (bug fixes).
    pub emendatio: u16,
}

impl VersioProtocolli {
    /// Current protocol version.
    pub const CURRENS: Self = VersioProtocolli {
        maior: 1,
        minor: 0,
        emendatio: 0,
    };

    /// Create a new version.
    #[inline]
    pub const fn new(maior: u16, minor: u16, emendatio: u16) -> Self {
        VersioProtocolli {
            maior,
            minor,
            emendatio,
        }
    }

    /// Check if this version is compatible with another.
    pub fn is_compatible(&self, other: &Self) -> bool {
        // Major version must match, minor version must be >= other
        self.maior == other.maior && self.minor >= other.minor
    }
}

impl fmt::Display for VersioProtocolli {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.maior, self.minor, self.emendatio)
    }
}

// =============================================================================
// Message Types
// =============================================================================

/// Message header for all protocol messages.
#[derive(Debug, Clone)]
pub struct CaputNuntii {
    /// Protocol version.
    pub versio: VersioProtocolli,
    /// Message ID for correlation.
    pub id: u64,
    /// Sender node ID.
    pub mittens: NodusIdentitas,
    /// Recipient node ID (None = broadcast).
    pub recipiens: Option<NodusIdentitas>,
    /// Timestamp (duration since Unix epoch).
    pub tempus: Duration,
    /// Message type.
    pub genus: GenusNuntii,
    /// Request correlation ID (for responses).
    pub correlatio: Option<u64>,
}

/// Message type enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenusNuntii {
    // === Cluster Management ===
    /// Heartbeat/ping.
    Pulsatio,
    /// Join cluster request.
    Coniunctio,
    /// Leave cluster notification.
    Abscessio,
    /// Node status update.
    StatusMutatio,

    // === Discovery ===
    /// Request node list.
    InquisitioNodorum,
    /// Response with node list.
    ResponsumNodorum,

    // === Computation ===
    /// Submit computation.
    SubmissioComputationis,
    /// Computation result.
    ResultatumComputationis,
    /// Cancel computation.
    CancellatioComputationis,

    // === Effects ===
    /// Effect operation request.
    OperatioEffectus,
    /// Effect operation response.
    ResponsumEffectus,

    // === Consensus ===
    /// Leader election vote.
    Suffragium,
    /// Leader announcement.
    AnnuntiatioDucis,
    /// Log replication.
    ReplicatioActorum,

    // === Error ===
    /// Error response.
    Error,
}

// =============================================================================
// Protocol Messages
// =============================================================================

/// Complete protocol message.
#[derive(Debug, Clone)]
pub struct Nuntius {
    /// Message header.
    pub caput: CaputNuntii,
    /// Message payload.
    pub corpus: CorpusNuntii,
}

impl Nuntius {
    /// Create a new message.
    ///
    /// **Placeholder semantics:** the header timestamp (`tempus`) is always
    /// zero — there is no clock in `no_std` scope, so callers that need real
    /// timestamps must set them from system time themselves.
    pub fn new(mittens: NodusIdentitas, genus: GenusNuntii, corpus: CorpusNuntii) -> Self {
        static COUNTER: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(1);

        Nuntius {
            caput: CaputNuntii {
                versio: VersioProtocolli::CURRENS,
                id: COUNTER.fetch_add(1, core::sync::atomic::Ordering::SeqCst),
                mittens,
                recipiens: None,
                tempus: Duration::from_secs(0), // Would be set from system time
                genus,
                correlatio: None,
            },
            corpus,
        }
    }

    /// Set recipient.
    pub fn to(mut self, recipiens: NodusIdentitas) -> Self {
        self.caput.recipiens = Some(recipiens);
        self
    }

    /// Set correlation ID.
    pub fn correlating(mut self, id: u64) -> Self {
        self.caput.correlatio = Some(id);
        self
    }

    /// Create a response to this message.
    ///
    /// **Placeholder semantics:** the response's sender is taken from this
    /// message's recipient field (falling back to the original sender) —
    /// the constructor has no notion of "the local node". Wire-up code with
    /// a real node identity should overwrite the sender.
    pub fn respond(&self, genus: GenusNuntii, corpus: CorpusNuntii) -> Self {
        Nuntius::new(
            // Would be local node ID, using sender as placeholder
            self.caput.recipiens.unwrap_or(self.caput.mittens),
            genus,
            corpus,
        )
        .to(self.caput.mittens)
        .correlating(self.caput.id)
    }
}

/// Message body/payload.
#[derive(Debug, Clone)]
pub enum CorpusNuntii {
    /// Empty payload.
    Vacuum,

    /// Heartbeat.
    Pulsatio(PulsatioCorpus),

    /// Join request.
    Coniunctio(ConiunctioCorpus),

    /// Node list.
    Nodi(Vec<InformationesNodi>),

    /// Status update.
    Status(StatusNodi),

    /// Computation submission.
    Computatio(ComputatioCorpus),

    /// Computation result.
    Resultatum(ResultatumCorpus),

    /// Effect operation.
    Effectus(EffectusCorpus),

    /// Effect response.
    ResponsumEffectus(ResponsumEffectusCorpus),

    /// Vote request/response.
    Suffragium(SuffragiumCorpus),

    /// Log entries.
    Acta(ActaCorpus),

    /// Error.
    Error(ErrorCorpus),
}

// =============================================================================
// Specific Message Bodies
// =============================================================================

/// Heartbeat message body.
#[derive(Debug, Clone)]
pub struct PulsatioCorpus {
    /// Current node status.
    pub status: StatusNodi,
    /// Current load factor.
    pub onus: f32,
    /// Active task count.
    pub munera_activa: u32,
    /// Cluster generation this node knows.
    pub generatio: u64,
}

/// Join request body.
#[derive(Debug, Clone)]
pub struct ConiunctioCorpus {
    /// Joining node info.
    pub informationes: InformationesNodi,
    /// Requested role.
    pub munus_petitus: super::node::MunusNodi,
}

/// Computation submission body.
#[derive(Debug, Clone)]
pub struct ComputatioCorpus {
    /// Computation ID.
    pub id: u64,
    /// Serialized computation.
    pub nodus: NodusSerializabilis,
    /// Required effects.
    pub effectus_requiriti: Vec<u64>,
    /// Priority.
    pub prioritas: u32,
    /// Timeout.
    pub mora: Option<Duration>,
}

/// Computation result body.
#[derive(Debug, Clone)]
pub struct ResultatumCorpus {
    /// Computation ID.
    pub computatio_id: u64,
    /// Success or failure.
    pub exitus: ExitusComputationis,
    /// Execution time.
    pub tempus: Duration,
}

/// Computation outcome.
#[derive(Debug, Clone)]
pub enum ExitusComputationis {
    /// Successful with result.
    Successus(Vec<u8>),
    /// Failed with error.
    Defectio(String),
    /// Cancelled.
    Cancellatus,
    /// Timed out.
    MoraExcessit,
}

/// Effect operation body.
#[derive(Debug, Clone)]
pub struct EffectusCorpus {
    /// Effect type ID.
    pub effectus_id: u64,
    /// Operation ID for correlation.
    pub operatio_id: u64,
    /// Serialized operation data.
    pub data: Vec<u8>,
}

/// Effect response body.
#[derive(Debug, Clone)]
pub struct ResponsumEffectusCorpus {
    /// Operation ID.
    pub operatio_id: u64,
    /// Success or failure.
    pub exitus: ExitusEffectus,
}

/// Effect operation outcome.
#[derive(Debug, Clone)]
pub enum ExitusEffectus {
    /// Successful with result.
    Successus(Vec<u8>),
    /// Failed.
    Defectio(String),
    /// Effect not supported.
    NonSuffultus,
}

/// Vote message body.
#[derive(Debug, Clone)]
pub struct SuffragiumCorpus {
    /// Term/epoch.
    pub terminus: u64,
    /// Candidate ID.
    pub candidatus: NodusIdentitas,
    /// Last log index.
    pub ultimus_index: u64,
    /// Last log term.
    pub ultimus_terminus: u64,
    /// Is this a vote grant?
    pub concessum: bool,
}

/// Log replication body.
#[derive(Debug, Clone)]
pub struct ActaCorpus {
    /// Leader term.
    pub terminus: u64,
    /// Leader ID.
    pub dux: NodusIdentitas,
    /// Previous log index.
    pub index_prior: u64,
    /// Previous log term.
    pub terminus_prior: u64,
    /// Log entries.
    pub ingressus: Vec<IngressusActorum>,
    /// Leader commit index.
    pub index_commissi: u64,
}

/// A log entry.
#[derive(Debug, Clone)]
pub struct IngressusActorum {
    /// Entry term.
    pub terminus: u64,
    /// Entry index.
    pub index: u64,
    /// Entry data.
    pub data: Vec<u8>,
}

/// Error message body.
#[derive(Debug, Clone)]
pub struct ErrorCorpus {
    /// Error code.
    pub codex: u32,
    /// Error message.
    pub nuntius: String,
    /// Retriable.
    pub iterabilis: bool,
}

impl ErrorCorpus {
    /// Error code: the addressed node is not known to the cluster.
    pub const NODE_NOT_FOUND: u32 = 1;
    /// Error code: the receiving node has no handler for the requested
    /// effect ID.
    pub const EFFECT_NOT_SUPPORTED: u32 = 2;
    /// Error code: the operation did not complete within its deadline.
    pub const TIMEOUT: u32 = 3;
    /// Error code: a message body could not be (de)serialized.
    pub const SERIALIZATION_ERROR: u32 = 4;
    /// Error code: a leader-only request reached a node that is not the
    /// current leader.
    pub const NOT_LEADER: u32 = 5;
    /// Error code: not enough nodes were reachable to form a quorum.
    pub const NO_QUORUM: u32 = 6;
    /// Error code: the remote computation itself failed after being
    /// successfully delivered.
    pub const COMPUTATION_FAILED: u32 = 7;
    /// Error code: the operation was cancelled before completion.
    pub const CANCELLED: u32 = 8;

    /// Create an error with code and message.
    #[inline]
    pub fn new(codex: u32, nuntius: impl Into<String>) -> Self {
        ErrorCorpus {
            codex,
            nuntius: nuntius.into(),
            iterabilis: false,
        }
    }

    /// Mark as retriable.
    pub fn retriable(mut self) -> Self {
        self.iterabilis = true;
        self
    }
}

// =============================================================================
// Wire Format
// =============================================================================

/// Wire format for serialization.
///
/// # Latin Etymology
/// *Forma fili* = wire format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FormaFili {
    /// Binary format (compact).
    #[default]
    Binarius,
    /// JSON format (human-readable).
    Json,
    /// `MessagePack` format.
    MessagePack,
    /// Protocol Buffers.
    Protobuf,
}

/// Trait for message serialization.
pub trait Serializabilis: Sized {
    /// Serialize to bytes.
    ///
    /// # Errors
    ///
    /// Returns [`ErrorSerialization`] when the value cannot be encoded in the
    /// requested [`FormaFili`] — for example, a format the implementation
    /// does not support. The message lives in
    /// [`nuntius`](ErrorSerialization::nuntius); `offset` is typically `None`
    /// for encoding failures.
    fn serialize(&self, forma: FormaFili) -> Result<Vec<u8>, ErrorSerialization>;

    /// Deserialize from bytes.
    ///
    /// # Errors
    ///
    /// Returns [`ErrorSerialization`] when `bytes` is not a valid encoding of
    /// `Self` in the given [`FormaFili`] (malformed, truncated, or produced
    /// by a different format). Where the failing position is known,
    /// implementations record it in
    /// [`offset`](ErrorSerialization::offset).
    fn deserialize(bytes: &[u8], forma: FormaFili) -> Result<Self, ErrorSerialization>;
}

/// Serialization error.
#[derive(Debug, Clone)]
pub struct ErrorSerialization {
    /// Error message.
    pub nuntius: String,
    /// Byte offset where error occurred.
    pub offset: Option<usize>,
}

impl fmt::Display for ErrorSerialization {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.offset {
            Some(off) => write!(f, "Serialization error at byte {}: {}", off, self.nuntius),
            None => write!(f, "Serialization error: {}", self.nuntius),
        }
    }
}

// =============================================================================
// Message Builder
// =============================================================================

/// Builder for constructing protocol messages.
pub struct AedificatorNuntii {
    mittens: NodusIdentitas,
}

impl AedificatorNuntii {
    /// Create a new message builder.
    #[inline]
    pub fn new(mittens: NodusIdentitas) -> Self {
        AedificatorNuntii { mittens }
    }

    /// Build a heartbeat message.
    #[inline]
    pub fn pulsatio(&self, status: StatusNodi, onus: f32, munera: u32, generatio: u64) -> Nuntius {
        Nuntius::new(
            self.mittens,
            GenusNuntii::Pulsatio,
            CorpusNuntii::Pulsatio(PulsatioCorpus {
                status,
                onus,
                munera_activa: munera,
                generatio,
            }),
        )
    }

    /// Build a join request.
    #[inline]
    pub fn coniunctio(&self, info: InformationesNodi) -> Nuntius {
        Nuntius::new(
            self.mittens,
            GenusNuntii::Coniunctio,
            CorpusNuntii::Coniunctio(ConiunctioCorpus {
                munus_petitus: info.munus,
                informationes: info,
            }),
        )
    }

    /// Build a computation submission.
    #[inline]
    pub fn computatio(&self, id: u64, nodus: NodusSerializabilis, effectus: Vec<u64>) -> Nuntius {
        Nuntius::new(
            self.mittens,
            GenusNuntii::SubmissioComputationis,
            CorpusNuntii::Computatio(ComputatioCorpus {
                id,
                nodus,
                effectus_requiriti: effectus,
                prioritas: DEFAULT_COMPUTATION_PRIORITY,
                mora: None,
            }),
        )
    }

    /// Build an effect operation request.
    #[inline]
    pub fn effectus(&self, effectus_id: u64, operatio_id: u64, data: Vec<u8>) -> Nuntius {
        Nuntius::new(
            self.mittens,
            GenusNuntii::OperatioEffectus,
            CorpusNuntii::Effectus(EffectusCorpus {
                effectus_id,
                operatio_id,
                data,
            }),
        )
    }

    /// Build an error response.
    #[inline]
    pub fn error(&self, codex: u32, nuntius: impl Into<String>) -> Nuntius {
        Nuntius::new(
            self.mittens,
            GenusNuntii::Error,
            CorpusNuntii::Error(ErrorCorpus::new(codex, nuntius)),
        )
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_versio_compatibility() {
        let v1 = VersioProtocolli::new(1, 0, 0);
        let v2 = VersioProtocolli::new(1, 1, 0);
        let v3 = VersioProtocolli::new(2, 0, 0);

        assert!(v2.is_compatible(&v1));
        assert!(!v1.is_compatible(&v2));
        assert!(!v3.is_compatible(&v1));
    }

    #[test]
    fn test_nuntius_creation() {
        let sender = NodusIdentitas::new(1, 1);
        let msg = Nuntius::new(sender, GenusNuntii::Pulsatio, CorpusNuntii::Vacuum);

        assert_eq!(msg.caput.mittens, sender);
        assert_eq!(msg.caput.genus, GenusNuntii::Pulsatio);
    }

    #[test]
    fn test_nuntius_response() {
        let sender = NodusIdentitas::new(1, 1);
        let recipient = NodusIdentitas::new(2, 2);

        let request = Nuntius::new(sender, GenusNuntii::InquisitioNodorum, CorpusNuntii::Vacuum)
            .to(recipient);

        let response = request.respond(
            GenusNuntii::ResponsumNodorum,
            CorpusNuntii::Nodi(Vec::new()),
        );

        assert_eq!(response.caput.recipiens, Some(sender));
        assert_eq!(response.caput.correlatio, Some(request.caput.id));
    }

    #[test]
    fn test_aedificator_nuntii() {
        let sender = NodusIdentitas::new(1, 1);
        let builder = AedificatorNuntii::new(sender);

        let msg = builder.pulsatio(StatusNodi::Sanus, 0.5, 10, 1);
        assert_eq!(msg.caput.genus, GenusNuntii::Pulsatio);

        if let CorpusNuntii::Pulsatio(body) = msg.corpus {
            assert_eq!(body.status, StatusNodi::Sanus);
            assert!((body.onus - 0.5).abs() < f32::EPSILON);
        } else {
            panic!("Wrong message body type");
        }
    }

    #[test]
    fn test_error_corpus() {
        let err = ErrorCorpus::new(ErrorCorpus::TIMEOUT, "Operation timed out").retriable();

        assert_eq!(err.codex, ErrorCorpus::TIMEOUT);
        assert!(err.iterabilis);
    }
}
