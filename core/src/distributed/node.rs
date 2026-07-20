//! Distributed Node Management
//!
//! > *"Nodus est fundamentum retis"*
//! > — A node is the foundation of a network. (Latin)
//!
//! This module provides types for identifying and managing nodes
//! in a distributed cluster.

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;
use core::hash::Hash;
use core::time::Duration;

// =============================================================================
// Node Identity
// =============================================================================

/// Unique identifier for a node in the cluster.
///
/// # Latin Etymology
/// *Identitas nodi distribuendi* = identity of a distributed node.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodusIdentitas {
    /// High bits (typically derived from hostname/IP).
    pub alta: u64,
    /// Low bits (typically random or sequential).
    pub ima: u64,
}

impl NodusIdentitas {
    /// Create a new node ID from two 64-bit values.
    pub const fn new(alta: u64, ima: u64) -> Self {
        NodusIdentitas { alta, ima }
    }

    /// Create from a 128-bit value.
    // Both casts intentionally split `value` into its high/low 64-bit
    // halves; no bits are discarded (`to_u128` reconstructs the original
    // value exactly), so the truncation clippy warns about is by design.
    #[allow(clippy::cast_possible_truncation)]
    pub const fn from_u128(value: u128) -> Self {
        NodusIdentitas {
            alta: (value >> 64) as u64,
            ima: value as u64,
        }
    }

    /// Convert to 128-bit value.
    pub const fn to_u128(&self) -> u128 {
        ((self.alta as u128) << 64) | (self.ima as u128)
    }

    /// Nil/zero ID.
    pub const NIL: Self = NodusIdentitas { alta: 0, ima: 0 };

    /// Check if this is the nil ID.
    pub const fn is_nil(&self) -> bool {
        self.alta == 0 && self.ima == 0
    }

    /// Generate a random node ID.
    #[cfg(feature = "std")]
    pub fn generate() -> Self {
        use core::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1);

        // Timestamp + counter suffices for a single-process discriminator;
        // add a hostname hash if IDs must be unique across hosts.
        #[allow(clippy::cast_possible_truncation)] // nanos-since-epoch fits
        // u64 until ~2554 (u64::MAX ns is ~584 years), which safely outlives
        // any process using this as a node-ID discriminator.
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos() as u64);

        NodusIdentitas {
            alta: timestamp,
            ima: COUNTER.fetch_add(1, Ordering::SeqCst),
        }
    }
}

impl fmt::Debug for NodusIdentitas {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NodusId({:016x}{:016x})", self.alta, self.ima)
    }
}

impl fmt::Display for NodusIdentitas {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:016x}{:016x}", self.alta, self.ima)
    }
}

// =============================================================================
// Node Address
// =============================================================================

/// Network address of a node.
///
/// # Latin Etymology
/// *Inscriptio nodi* = address of a node.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InscriptioNodi {
    /// Host address (IP or hostname).
    pub hospes: String,
    /// Port number.
    pub portus: u16,
    /// Protocol scheme.
    pub schema: ProtocollumSchema,
}

impl InscriptioNodi {
    /// Create a new node address.
    pub fn new(hospes: impl Into<String>, portus: u16) -> Self {
        InscriptioNodi {
            hospes: hospes.into(),
            portus,
            schema: ProtocollumSchema::Grpc,
        }
    }

    /// Create with explicit protocol.
    pub fn with_schema(hospes: impl Into<String>, portus: u16, schema: ProtocollumSchema) -> Self {
        InscriptioNodi {
            hospes: hospes.into(),
            portus,
            schema,
        }
    }

    /// Format as URI string.
    pub fn to_uri(&self) -> String {
        let schema = match self.schema {
            ProtocollumSchema::Grpc => "grpc",
            ProtocollumSchema::Http => "http",
            ProtocollumSchema::Https => "https",
            ProtocollumSchema::Tcp => "tcp",
            ProtocollumSchema::Unix => "unix",
        };
        alloc::format!("{}://{}:{}", schema, self.hospes, self.portus)
    }
}

impl fmt::Display for InscriptioNodi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.hospes, self.portus)
    }
}

/// Protocol scheme for node communication.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ProtocollumSchema {
    /// gRPC (HTTP/2 + protobuf).
    #[default]
    Grpc,
    /// Plain HTTP.
    Http,
    /// HTTPS.
    Https,
    /// Raw TCP.
    Tcp,
    /// Unix domain socket.
    Unix,
}

// =============================================================================
// Node Information
// =============================================================================

/// Complete information about a node in the cluster.
///
/// # Latin Etymology
/// *Informationes nodi* = information about a node.
#[derive(Debug, Clone)]
pub struct InformationesNodi {
    /// Unique node identifier.
    pub identitas: NodusIdentitas,
    /// Network address.
    pub inscriptio: InscriptioNodi,
    /// Node role.
    pub munus: MunusNodi,
    /// Node status.
    pub status: StatusNodi,
    /// Node capabilities.
    pub facultates: FacultatesNodi,
    /// Metadata/labels.
    pub tituli: Vec<(String, String)>,
    /// Last heartbeat time (as duration since Unix epoch).
    pub ultima_pulsatio: Option<Duration>,
}

impl InformationesNodi {
    /// Create basic node info.
    #[inline]
    pub fn new(identitas: NodusIdentitas, inscriptio: InscriptioNodi) -> Self {
        InformationesNodi {
            identitas,
            inscriptio,
            munus: MunusNodi::Executor,
            status: StatusNodi::Unknown,
            facultates: FacultatesNodi::default(),
            tituli: Vec::with_capacity(4),
            ultima_pulsatio: None,
        }
    }

    /// Set node role.
    pub fn with_role(mut self, munus: MunusNodi) -> Self {
        self.munus = munus;
        self
    }

    /// Add a label.
    pub fn with_label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.tituli.push((key.into(), value.into()));
        self
    }

    /// Check if node is healthy.
    #[inline]
    pub fn is_healthy(&self) -> bool {
        matches!(self.status, StatusNodi::Sanus)
    }

    /// Check if node can execute computations.
    #[inline]
    pub fn can_execute(&self) -> bool {
        self.is_healthy()
            && matches!(
                self.munus,
                MunusNodi::Executor | MunusNodi::Coordinator | MunusNodi::All
            )
    }
}

/// Role of a node in the cluster.
///
/// # Latin Etymology
/// *Munus* = duty, function, role.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MunusNodi {
    /// Coordinator node (schedules work).
    Coordinator,
    /// Executor node (runs computations).
    #[default]
    Executor,
    /// Both coordinator and executor.
    All,
    /// Gateway node (external API).
    Porta,
    /// Storage node.
    Repositorium,
}

impl fmt::Display for MunusNodi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MunusNodi::Coordinator => write!(f, "coordinator"),
            MunusNodi::Executor => write!(f, "executor"),
            MunusNodi::All => write!(f, "all"),
            MunusNodi::Porta => write!(f, "gateway"),
            MunusNodi::Repositorium => write!(f, "storage"),
        }
    }
}

/// Status of a node.
///
/// # Latin Etymology
/// *Status* = condition, state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StatusNodi {
    /// Unknown status.
    #[default]
    Unknown,
    /// Healthy and ready.
    Sanus,
    /// Starting up.
    Surgit,
    /// Shutting down.
    Descendit,
    /// Unhealthy/failing.
    Aegrotus,
    /// Unreachable.
    Inaccessibilis,
    /// Draining (not accepting new work).
    Exhaurit,
}

impl StatusNodi {
    /// Check if the node can accept new work.
    pub fn can_accept_work(&self) -> bool {
        matches!(self, StatusNodi::Sanus)
    }
}

impl fmt::Display for StatusNodi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StatusNodi::Unknown => write!(f, "unknown"),
            StatusNodi::Sanus => write!(f, "healthy"),
            StatusNodi::Surgit => write!(f, "starting"),
            StatusNodi::Descendit => write!(f, "stopping"),
            StatusNodi::Aegrotus => write!(f, "unhealthy"),
            StatusNodi::Inaccessibilis => write!(f, "unreachable"),
            StatusNodi::Exhaurit => write!(f, "draining"),
        }
    }
}

// =============================================================================
// Node Capabilities
// =============================================================================

/// Capabilities of a node.
///
/// # Latin Etymology
/// *Facultates* = capabilities, powers.
#[derive(Debug, Clone, Default)]
pub struct FacultatesNodi {
    /// Number of CPU cores.
    pub nuclei_cpu: u32,
    /// Total memory in bytes.
    pub memoria_bytes: u64,
    /// Available memory in bytes.
    pub memoria_disponibilis: u64,
    /// GPU availability.
    pub gpu: Option<GpuFacultas>,
    /// Supported effect handlers.
    pub effectus_tractati: Vec<u64>,
    /// Maximum concurrent tasks.
    pub munera_maxima: u32,
    /// Current task count.
    pub munera_currentia: u32,
}

impl FacultatesNodi {
    /// Calculate load factor (0.0 - 1.0).
    #[inline]
    pub fn load_factor(&self) -> f64 {
        if self.munera_maxima == 0 {
            1.0
        } else {
            f64::from(self.munera_currentia) / f64::from(self.munera_maxima)
        }
    }

    /// Check if node has capacity for more work.
    #[inline]
    pub fn has_capacity(&self) -> bool {
        self.munera_currentia < self.munera_maxima
    }

    /// Check if node can handle a specific effect.
    #[inline]
    pub fn can_handle_effect(&self, effect_id: u64) -> bool {
        self.effectus_tractati.contains(&effect_id)
    }
}

/// GPU capability information.
#[derive(Debug, Clone)]
pub struct GpuFacultas {
    /// GPU vendor.
    pub vendor: String,
    /// GPU model.
    pub modellum: String,
    /// VRAM in bytes.
    pub vram_bytes: u64,
    /// Compute capability version.
    pub versio_computationis: String,
}

// =============================================================================
// Node Affinity
// =============================================================================

/// Affinity specification for node selection.
///
/// # Latin Etymology
/// *Affinitas* = relationship, affinity.
#[derive(Debug, Clone, Default)]
pub enum AffinitasNodi {
    /// No preference - any node is acceptable.
    #[default]
    Quodlibet,

    /// Prefer local execution.
    Localis,

    /// Prefer a specific node.
    Nodus(NodusIdentitas),

    /// Prefer nodes matching labels.
    Tituli(Vec<(String, String)>),

    /// Prefer nodes in a specific zone/region.
    Regio(String),

    /// Prefer nodes with specific capabilities.
    Facultates(RequirementaFacultatum),

    /// Weighted preferences.
    Ponderata(Vec<(AffinitasNodi, u32)>),

    /// Anti-affinity (avoid these nodes).
    Non(Box<AffinitasNodi>),
}

impl AffinitasNodi {
    /// Check if a node matches this affinity.
    pub fn matches(&self, node: &InformationesNodi) -> bool {
        match self {
            AffinitasNodi::Quodlibet => true,
            AffinitasNodi::Localis => false, // Must be handled by caller
            AffinitasNodi::Nodus(id) => node.identitas == *id,
            AffinitasNodi::Tituli(labels) => labels
                .iter()
                .all(|(k, v)| node.tituli.iter().any(|(nk, nv)| nk == k && nv == v)),
            AffinitasNodi::Regio(region) => {
                node.tituli.iter().any(|(k, v)| k == "regio" && v == region)
            }
            AffinitasNodi::Facultates(req) => req.satisfies(&node.facultates),
            AffinitasNodi::Ponderata(affinities) => affinities.iter().any(|(a, _)| a.matches(node)),
            AffinitasNodi::Non(inner) => !inner.matches(node),
        }
    }

    /// Calculate affinity score for a node (higher is better).
    pub fn score(&self, node: &InformationesNodi) -> u32 {
        match self {
            AffinitasNodi::Quodlibet => 1,
            AffinitasNodi::Localis => 0, // Must be handled by caller
            AffinitasNodi::Nodus(id) => {
                if node.identitas == *id {
                    100
                } else {
                    0
                }
            }
            AffinitasNodi::Tituli(labels) => {
                let matching = labels
                    .iter()
                    .filter(|(k, v)| node.tituli.iter().any(|(nk, nv)| nk == k && nv == v))
                    .count();
                (matching * 10) as u32
            }
            AffinitasNodi::Regio(region) => {
                if node.tituli.iter().any(|(k, v)| k == "regio" && v == region) {
                    50
                } else {
                    0
                }
            }
            AffinitasNodi::Facultates(req) => {
                if req.satisfies(&node.facultates) {
                    30
                } else {
                    0
                }
            }
            AffinitasNodi::Ponderata(affinities) => affinities
                .iter()
                // A score is a ranking value, not an exact quantity — on
                // overflow, saturating to u32::MAX still ranks this
                // candidate correctly as "very strongly preferred" rather
                // than wrapping into a small/misleading number.
                .map(|(a, weight)| a.score(node).saturating_mul(*weight))
                .max()
                .unwrap_or(0),
            AffinitasNodi::Non(inner) => {
                if inner.matches(node) {
                    0
                } else {
                    10
                }
            }
        }
    }
}

/// Requirements for node capabilities.
#[derive(Debug, Clone, Default)]
pub struct RequirementaFacultatum {
    /// Minimum CPU cores.
    pub nuclei_cpu_min: Option<u32>,
    /// Minimum memory.
    pub memoria_min: Option<u64>,
    /// Requires GPU.
    pub gpu_requiritur: bool,
    /// Required effect handlers.
    pub effectus_requiriti: Vec<u64>,
}

impl RequirementaFacultatum {
    /// Check if capabilities satisfy requirements.
    pub fn satisfies(&self, cap: &FacultatesNodi) -> bool {
        if let Some(min_cpu) = self.nuclei_cpu_min
            && cap.nuclei_cpu < min_cpu
        {
            return false;
        }

        if let Some(min_mem) = self.memoria_min
            && cap.memoria_disponibilis < min_mem
        {
            return false;
        }

        if self.gpu_requiritur && cap.gpu.is_none() {
            return false;
        }

        for effect_id in &self.effectus_requiriti {
            if !cap.effectus_tractati.contains(effect_id) {
                return false;
            }
        }

        true
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_nodus_identitas() {
        let id1 = NodusIdentitas::new(1, 2);
        let id2 = NodusIdentitas::new(1, 2);
        let id3 = NodusIdentitas::new(1, 3);

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_nodus_identitas_u128() {
        let value: u128 = 0x0123_4567_89AB_CDEF_FEDC_BA98_7654_3210;
        let id = NodusIdentitas::from_u128(value);
        assert_eq!(id.to_u128(), value);
    }

    #[test]
    fn test_inscriptio_nodi() {
        let addr = InscriptioNodi::new("localhost", 8080);
        assert_eq!(addr.hospes, "localhost");
        assert_eq!(addr.portus, 8080);
        assert_eq!(addr.to_uri(), "grpc://localhost:8080");
    }

    #[test]
    fn test_status_nodi() {
        assert!(StatusNodi::Sanus.can_accept_work());
        assert!(!StatusNodi::Aegrotus.can_accept_work());
        assert!(!StatusNodi::Exhaurit.can_accept_work());
    }

    #[test]
    fn test_facultates_load_factor() {
        let cap = FacultatesNodi {
            munera_maxima: 100,
            munera_currentia: 50,
            ..Default::default()
        };
        assert!((cap.load_factor() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_affinitas_matches() {
        let node = InformationesNodi {
            identitas: NodusIdentitas::new(1, 1),
            inscriptio: InscriptioNodi::new("localhost", 8080),
            munus: MunusNodi::Executor,
            status: StatusNodi::Sanus,
            facultates: FacultatesNodi::default(),
            tituli: vec![("regio".into(), "us-west".into())],
            ultima_pulsatio: None,
        };

        assert!(AffinitasNodi::Quodlibet.matches(&node));
        assert!(AffinitasNodi::Regio("us-west".into()).matches(&node));
        assert!(!AffinitasNodi::Regio("us-east".into()).matches(&node));
    }
}
