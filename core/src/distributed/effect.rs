//! Distributed Effect Handlers
//!
//! > *"Effectus remotus est effectus translatus"*
//! > — A remote effect is a transmitted effect. (Latin)
//!
//! This module provides infrastructure for effect handlers that span
//! multiple nodes in a distributed cluster.
//!
//! **No built-in transport:** the routing/affinity logic here is real, but
//! the crate ships **no `ConnexioRemota` implementation** — nothing actually
//! sends bytes over a network. To distribute effects you must supply your
//! own transport; until then this module is type-level scaffolding plus
//! local routing decisions.

use alloc::boxed::Box;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::fmt;
use core::future::Future;
use core::pin::Pin;

use super::node::{AffinitasNodi, InformationesNodi, NodusIdentitas};
use super::serializable::NodusSerializabilis;

// =============================================================================
// Distributed Effect Trait
// =============================================================================

/// Trait for effects that can be distributed across nodes.
///
/// # Latin Etymology
/// *Effectus distributus* = distributed effect.
pub trait EffectusDistributus: Send + Sync {
    /// Unique effect identifier.
    const EFFECTUS_ID: u64;

    /// Effect name for debugging.
    fn nomen() -> &'static str;

    /// Get the node affinity for this effect operation.
    fn affinitas(&self) -> AffinitasNodi {
        AffinitasNodi::Quodlibet
    }

    /// Serialize this effect for transmission.
    ///
    /// # Errors
    ///
    /// Returns [`ErrorSerializationis`] when the operation cannot be encoded
    /// for transmission (e.g. it contains state with no wire representation);
    /// implementations put a human-readable description in
    /// [`nuntius`](ErrorSerializationis::nuntius).
    fn serialize(&self) -> Result<EffectusSerializatus, ErrorSerializationis>;

    /// Whether this effect requires ordering guarantees.
    fn requires_ordering(&self) -> bool {
        true
    }

    /// Whether this effect can be batched with others.
    fn batchable(&self) -> bool {
        false
    }
}

/// Serialized form of an effect operation.
#[derive(Debug, Clone)]
pub struct EffectusSerializatus {
    /// Effect type ID.
    pub effectus_id: u64,
    /// Serialized operation data.
    pub data: Vec<u8>,
    /// Node affinity.
    pub affinitas: AffinitasNodi,
    /// Correlation ID for request/response matching.
    pub correlatio_id: u64,
}

/// Error during effect serialization.
#[derive(Debug, Clone)]
pub struct ErrorSerializationis {
    /// Error message.
    pub nuntius: String,
}

impl fmt::Display for ErrorSerializationis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Serialization error: {}", self.nuntius)
    }
}

// =============================================================================
// Distributed Handler Trait
// =============================================================================

/// Handler for distributed effects.
///
/// # Latin Etymology
/// *Tractator distributus* = distributed handler.
pub trait TractatorDistributus<E: EffectusDistributus>: Send + Sync {
    /// Output type of the handler.
    type Output;

    /// Handle an effect operation.
    fn handle(
        &self,
        operation: E,
    ) -> Pin<Box<dyn Future<Output = Result<Self::Output, ErrorDistributionis>> + Send + '_>>;

    /// Check if this handler can handle the effect locally.
    fn can_handle_locally(&self) -> bool {
        true
    }
}

/// Error during distributed effect handling.
#[derive(Debug, Clone)]
pub enum ErrorDistributionis {
    /// Network error.
    Rete(String),
    /// Node not found.
    NodusNonInventus(NodusIdentitas),
    /// Effect not supported on target node.
    EffectusNonSuffultus(u64),
    /// Timeout.
    Mora(core::time::Duration),
    /// Serialization error.
    Serializatio(ErrorSerializationis),
    /// Handler error.
    Tractator(String),
    /// Node unavailable.
    NodusInaccessibilis(NodusIdentitas),
}

impl fmt::Display for ErrorDistributionis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorDistributionis::Rete(msg) => write!(f, "Network error: {msg}"),
            ErrorDistributionis::NodusNonInventus(id) => {
                write!(f, "Node not found: {id:?}")
            }
            ErrorDistributionis::EffectusNonSuffultus(id) => {
                write!(f, "Effect {id} not supported")
            }
            ErrorDistributionis::Mora(d) => write!(f, "Timeout after {d:?}"),
            ErrorDistributionis::Serializatio(e) => write!(f, "{e}"),
            ErrorDistributionis::Tractator(msg) => write!(f, "Handler error: {msg}"),
            ErrorDistributionis::NodusInaccessibilis(id) => {
                write!(f, "Node unavailable: {id:?}")
            }
        }
    }
}

// =============================================================================
// Remote Handler Proxy
// =============================================================================

/// Proxy for invoking effect handlers on remote nodes.
///
/// # Latin Etymology
/// *Procurator remotus* = remote proxy.
#[derive(Clone)]
pub struct ProcuratorRemotus<E> {
    /// Target node.
    pub nodus: NodusIdentitas,
    /// Connection state (abstract).
    pub connexio: Arc<dyn ConnexioRemota>,
    /// Effect type marker.
    _effectus: core::marker::PhantomData<E>,
}

impl<E: EffectusDistributus> ProcuratorRemotus<E> {
    /// Create a new remote proxy.
    #[inline]
    pub fn new(nodus: NodusIdentitas, connexio: Arc<dyn ConnexioRemota>) -> Self {
        ProcuratorRemotus {
            nodus,
            connexio,
            _effectus: core::marker::PhantomData,
        }
    }

    /// Invoke the effect on the remote node.
    pub fn invoke(
        &self,
        operation: E,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, ErrorDistributionis>> + Send + '_>> {
        Box::pin(async move {
            let serialized = operation
                .serialize()
                .map_err(ErrorDistributionis::Serializatio)?;

            self.connexio.send(self.nodus, serialized).await
        })
    }
}

impl<E> fmt::Debug for ProcuratorRemotus<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProcuratorRemotus")
            .field("nodus", &self.nodus)
            .finish_non_exhaustive()
    }
}

/// Trait for remote connections.
pub trait ConnexioRemota: Send + Sync {
    /// Send an effect to a remote node.
    fn send(
        &self,
        nodus: NodusIdentitas,
        effectus: EffectusSerializatus,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, ErrorDistributionis>> + Send + '_>>;

    /// Check if the node is reachable.
    fn is_reachable(&self, nodus: NodusIdentitas) -> bool;
}

// =============================================================================
// Distributed Handler Wrapper
// =============================================================================

/// Handler that routes effects to local or remote handlers.
///
/// # Latin Etymology
/// *Tractator dirigens* = routing handler.
pub struct TractatorDirigens<E: EffectusDistributus> {
    /// Local handler for this effect.
    local: Option<Box<dyn TractatorDistributus<E, Output = Vec<u8>>>>,
    /// Remote handler proxies by node.
    remoti: Vec<(NodusIdentitas, ProcuratorRemotus<E>)>,
    /// Local node ID.
    nodus_localis: NodusIdentitas,
    /// Load balancing strategy.
    strategia: StrategiaDistributionis,
}

impl<E: EffectusDistributus> TractatorDirigens<E> {
    /// Create a new routing handler.
    #[inline]
    pub fn new(nodus_localis: NodusIdentitas) -> Self {
        TractatorDirigens {
            local: None,
            remoti: Vec::with_capacity(8),
            nodus_localis,
            strategia: StrategiaDistributionis::RoundRobin { index: 0 },
        }
    }

    /// Set the local handler.
    pub fn with_local(
        mut self,
        handler: impl TractatorDistributus<E, Output = Vec<u8>> + 'static,
    ) -> Self {
        self.local = Some(Box::new(handler));
        self
    }

    /// Add a remote handler.
    pub fn with_remote(mut self, nodus: NodusIdentitas, proxy: ProcuratorRemotus<E>) -> Self {
        self.remoti.push((nodus, proxy));
        self
    }

    /// Set the distribution strategy.
    pub fn with_strategy(mut self, strategia: StrategiaDistributionis) -> Self {
        self.strategia = strategia;
        self
    }

    /// Select a node for handling the effect.
    pub fn select_node(
        &mut self,
        operation: &E,
        nodes: &[InformationesNodi],
    ) -> Option<NodusIdentitas> {
        let affinity = operation.affinitas();

        // If affinity specifies local, use local
        if matches!(affinity, AffinitasNodi::Localis) {
            return Some(self.nodus_localis);
        }

        // If affinity specifies a specific node, only honor the pin when
        // that node passes the same health/role check the unpinned path
        // applies below (`can_execute`) — a pinned-but-dead node must fall
        // through to normal selection rather than being returned as-is.
        if let AffinitasNodi::Nodus(id) = affinity
            && nodes.iter().any(|n| n.identitas == id && n.can_execute())
        {
            return Some(id);
        }

        // Filter nodes that match affinity and are healthy
        let candidates: Vec<_> = nodes
            .iter()
            .filter(|n| n.can_execute() && affinity.matches(n))
            .collect();

        if candidates.is_empty() {
            return None;
        }

        // Apply distribution strategy
        match &mut self.strategia {
            StrategiaDistributionis::RoundRobin { index } => {
                let selected = &candidates[*index % candidates.len()];
                *index = (*index + 1) % candidates.len();
                Some(selected.identitas)
            }
            StrategiaDistributionis::LeastLoaded => candidates
                .iter()
                .min_by(|a, b| {
                    a.facultates
                        .load_factor()
                        .partial_cmp(&b.facultates.load_factor())
                        .unwrap_or(core::cmp::Ordering::Equal)
                })
                .map(|n| n.identitas),
            StrategiaDistributionis::AffinityScore => candidates
                .iter()
                .max_by_key(|n| affinity.score(n))
                .map(|n| n.identitas),
            StrategiaDistributionis::Random => {
                // Simple deterministic "random" for no_std. The cast below
                // is bounded by the subsequent `% candidates.len()`, so any
                // truncation of `ima` on 32-bit `usize` targets still
                // yields a valid in-bounds index into `candidates`.
                #[allow(clippy::cast_possible_truncation)]
                let idx = (candidates[0].identitas.ima as usize) % candidates.len();
                Some(candidates[idx].identitas)
            }
            StrategiaDistributionis::LocalFirst => {
                if candidates.iter().any(|n| n.identitas == self.nodus_localis) {
                    Some(self.nodus_localis)
                } else {
                    Some(candidates[0].identitas)
                }
            }
        }
    }
}

/// Distribution strategy for load balancing.
#[derive(Debug, Clone)]
pub enum StrategiaDistributionis {
    /// Round-robin across nodes.
    RoundRobin {
        /// Rotating cursor into the healthy-candidate list; advanced
        /// (modulo the candidate count) on every selection.
        index: usize,
    },
    /// Select least loaded node.
    LeastLoaded,
    /// Select by affinity score.
    AffinityScore,
    /// "Random" selection — **deterministic in practice**: with no RNG in
    /// `no_std` scope, the implementation picks an index derived from the
    /// first candidate's node ID, so the same candidate set always yields
    /// the same choice. Treat it as an arbitrary-but-stable pick.
    Random,
    /// Prefer local, fallback to remote.
    LocalFirst,
}

// =============================================================================
// Effect Routing Table
// =============================================================================

/// Routing table for distributed effects.
///
/// Maps effect IDs to the nodes that can handle them.
///
/// # Latin Etymology
/// *Tabula dirigendi* = routing table.
#[derive(Debug, Clone, Default)]
pub struct TabulaDirigendi {
    /// Routes: `effect_id` -> list of capable nodes.
    pub viae: Vec<(u64, Vec<NodusIdentitas>)>,
}

impl TabulaDirigendi {
    /// Create a new empty routing table.
    #[inline]
    pub fn new() -> Self {
        TabulaDirigendi {
            viae: Vec::with_capacity(16),
        }
    }

    /// Add a route for an effect.
    pub fn add_route(&mut self, effectus_id: u64, nodus: NodusIdentitas) {
        if let Some((_, nodes)) = self.viae.iter_mut().find(|(id, _)| *id == effectus_id) {
            if !nodes.contains(&nodus) {
                nodes.push(nodus);
            }
        } else {
            self.viae.push((effectus_id, alloc::vec![nodus]));
        }
    }

    /// Remove a node from all routes.
    pub fn remove_node(&mut self, nodus: NodusIdentitas) {
        for (_, nodes) in &mut self.viae {
            nodes.retain(|n| *n != nodus);
        }
    }

    /// Get nodes that can handle an effect.
    pub fn get_handlers(&self, effectus_id: u64) -> Option<&[NodusIdentitas]> {
        self.viae
            .iter()
            .find(|(id, _)| *id == effectus_id)
            .map(|(_, nodes)| nodes.as_slice())
    }
}

// =============================================================================
// Distributed Computation
// =============================================================================

/// A computation that can be distributed across nodes.
///
/// # Latin Etymology
/// *Computatio distributa* = distributed computation.
#[derive(Debug, Clone)]
pub struct ComputatioDistributa {
    /// Computation ID.
    pub id: u64,
    /// Serialized computation.
    pub nodus: NodusSerializabilis,
    /// Required effects.
    pub effectus_requiriti: Vec<u64>,
    /// Affinity requirements.
    pub affinitas: AffinitasNodi,
    /// Priority (lower is higher priority).
    pub prioritas: u32,
    /// Timeout.
    pub mora_maxima: Option<core::time::Duration>,
}

impl ComputatioDistributa {
    /// Create a new distributed computation.
    #[inline]
    pub fn new(id: u64, nodus: NodusSerializabilis) -> Self {
        ComputatioDistributa {
            id,
            nodus,
            effectus_requiriti: Vec::with_capacity(4),
            affinitas: AffinitasNodi::Quodlibet,
            prioritas: 100,
            mora_maxima: None,
        }
    }

    /// Add required effect.
    pub fn require_effect(mut self, effectus_id: u64) -> Self {
        self.effectus_requiriti.push(effectus_id);
        self
    }

    /// Set affinity.
    pub fn with_affinity(mut self, affinitas: AffinitasNodi) -> Self {
        self.affinitas = affinitas;
        self
    }

    /// Set priority.
    pub fn with_priority(mut self, prioritas: u32) -> Self {
        self.prioritas = prioritas;
        self
    }

    /// Set timeout.
    pub fn with_timeout(mut self, timeout: core::time::Duration) -> Self {
        self.mora_maxima = Some(timeout);
        self
    }
}

/// Result of a distributed computation.
#[derive(Debug, Clone)]
pub struct ResultatumDistributum {
    /// Computation ID.
    pub computatio_id: u64,
    /// Node that executed the computation.
    pub executor: NodusIdentitas,
    /// Result data (serialized).
    pub data: Vec<u8>,
    /// Execution time.
    pub tempus_executionis: core::time::Duration,
    /// Any effects that were performed.
    pub effectus_effecti: Vec<u64>,
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::super::node::{FacultatesNodi, InscriptioNodi, MunusNodi, StatusNodi};
    use super::*;
    use alloc::vec;

    struct EffectusProbationis {
        affinitas: AffinitasNodi,
    }

    impl EffectusDistributus for EffectusProbationis {
        const EFFECTUS_ID: u64 = 1;

        fn nomen() -> &'static str {
            "probatio"
        }

        fn affinitas(&self) -> AffinitasNodi {
            self.affinitas.clone()
        }

        fn serialize(&self) -> Result<EffectusSerializatus, ErrorSerializationis> {
            Ok(EffectusSerializatus {
                effectus_id: Self::EFFECTUS_ID,
                data: Vec::new(),
                affinitas: self.affinitas.clone(),
                correlatio_id: 0,
            })
        }
    }

    fn probatio_node(id: u64, status: StatusNodi) -> InformationesNodi {
        InformationesNodi {
            identitas: NodusIdentitas::new(0, id),
            inscriptio: InscriptioNodi::new("localhost", 8080 + id as u16),
            munus: MunusNodi::Executor,
            status,
            facultates: FacultatesNodi::default(),
            tituli: Vec::new(),
            ultima_pulsatio: None,
        }
    }

    /// Pinned-affinity regression: a pinned node that is unhealthy must not
    /// be returned by `select_node` — it should fall through to normal
    /// selection instead, mirroring the health check the unpinned path
    /// applies (`can_execute`).
    #[test]
    fn select_node_pinned_but_unhealthy_falls_through() {
        let dead_id = NodusIdentitas::new(0, 1);
        let nodes = vec![probatio_node(1, StatusNodi::Aegrotus)];

        let mut router = TractatorDirigens::<EffectusProbationis>::new(NodusIdentitas::new(0, 0));
        let op = EffectusProbationis {
            affinitas: AffinitasNodi::Nodus(dead_id),
        };

        // The dead pinned node must never be returned.
        assert_ne!(router.select_node(&op, &nodes), Some(dead_id));
        // No other candidate matches this affinity, so normal selection
        // (which also excludes unhealthy nodes) yields None.
        assert_eq!(router.select_node(&op, &nodes), None);
    }

    #[test]
    fn select_node_pinned_and_healthy_is_returned() {
        let healthy_id = NodusIdentitas::new(0, 1);
        let nodes = vec![probatio_node(1, StatusNodi::Sanus)];

        let mut router = TractatorDirigens::<EffectusProbationis>::new(NodusIdentitas::new(0, 0));
        let op = EffectusProbationis {
            affinitas: AffinitasNodi::Nodus(healthy_id),
        };

        assert_eq!(router.select_node(&op, &nodes), Some(healthy_id));
    }

    #[test]
    fn test_tabula_dirigendi() {
        let mut table = TabulaDirigendi::new();
        let node1 = NodusIdentitas::new(1, 1);
        let node2 = NodusIdentitas::new(2, 2);

        table.add_route(1, node1);
        table.add_route(1, node2);
        table.add_route(2, node1);

        let handlers = table
            .get_handlers(1)
            .expect("route 1 should have handlers after two add_route calls");
        assert_eq!(handlers.len(), 2);

        table.remove_node(node1);
        let handlers = table
            .get_handlers(1)
            .expect("route 1 should still have one handler after removing node1");
        assert_eq!(handlers.len(), 1);
        assert_eq!(handlers[0], node2);
    }

    #[test]
    fn test_computatio_distributa() {
        let comp = ComputatioDistributa::new(1, NodusSerializabilis::Vacuus)
            .require_effect(10)
            .require_effect(20)
            .with_priority(50);

        assert_eq!(comp.effectus_requiriti, vec![10, 20]);
        assert_eq!(comp.prioritas, 50);
    }

    #[test]
    fn test_error_display() {
        let err = ErrorDistributionis::EffectusNonSuffultus(42);
        let msg = alloc::format!("{err}");
        assert!(msg.contains("42"));
    }
}
