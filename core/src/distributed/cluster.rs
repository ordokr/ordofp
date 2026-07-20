//! Cluster Management
//!
//! > *"Grex est unitas nodorum"*
//! > — A cluster is a unity of nodes. (Latin)
//!
//! This module provides cluster configuration, node discovery,
//! and cluster state management.

use alloc::boxed::Box;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::fmt;
use core::future::Future;
use core::pin::Pin;
use core::time::Duration;

use super::effect::TabulaDirigendi;
use super::node::{InformationesNodi, InscriptioNodi, NodusIdentitas, StatusNodi};

/// Default cluster name.
pub const DEFAULT_CLUSTER_NAME: &str = "ordofp-cluster";

// =============================================================================
// Cluster Configuration
// =============================================================================

/// Configuration for a cluster.
///
/// # Latin Etymology
/// *Configuratio gregis* = cluster configuration.
#[derive(Debug, Clone)]
pub struct ConfiguratioGregis {
    /// Cluster name.
    pub nomen: String,
    /// Discovery method.
    pub inventio: MethodusInventionis,
    /// Heartbeat interval.
    pub intervallum_pulsationis: Duration,
    /// Node timeout (no heartbeat = unhealthy).
    pub mora_nodi: Duration,
    /// Maximum nodes in cluster.
    pub nodi_maximi: Option<usize>,
    /// Replication factor for data.
    pub factor_replicationis: u8,
    /// Consensus protocol.
    pub protocollum_consensus: ProtocollumConsensus,
    /// TLS configuration.
    pub tls: Option<ConfiguratioTls>,
}

impl Default for ConfiguratioGregis {
    fn default() -> Self {
        ConfiguratioGregis {
            nomen: String::from(DEFAULT_CLUSTER_NAME),
            inventio: MethodusInventionis::Staticus(Vec::new()),
            intervallum_pulsationis: Duration::from_secs(5),
            mora_nodi: Duration::from_secs(30),
            nodi_maximi: None,
            factor_replicationis: 1,
            protocollum_consensus: ProtocollumConsensus::Raft,
            tls: None,
        }
    }
}

impl ConfiguratioGregis {
    /// Create a new cluster configuration with a name.
    pub fn new(nomen: impl Into<String>) -> Self {
        ConfiguratioGregis {
            nomen: nomen.into(),
            ..Default::default()
        }
    }

    /// Set discovery method.
    pub fn with_discovery(mut self, inventio: MethodusInventionis) -> Self {
        self.inventio = inventio;
        self
    }

    /// Set heartbeat interval.
    pub fn with_heartbeat(mut self, interval: Duration) -> Self {
        self.intervallum_pulsationis = interval;
        self
    }

    /// Set node timeout.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.mora_nodi = timeout;
        self
    }

    /// Set replication factor.
    pub fn with_replication(mut self, factor: u8) -> Self {
        self.factor_replicationis = factor;
        self
    }

    /// Set consensus protocol.
    pub fn with_consensus(mut self, protocol: ProtocollumConsensus) -> Self {
        self.protocollum_consensus = protocol;
        self
    }
}

// =============================================================================
// Discovery Methods
// =============================================================================

/// Method for discovering nodes in the cluster.
///
/// # Latin Etymology
/// *Methodus inventionis* = method of discovery.
#[derive(Debug, Clone)]
pub enum MethodusInventionis {
    /// Static list of node addresses.
    Staticus(Vec<InscriptioNodi>),

    /// DNS-based discovery.
    Dns {
        /// DNS name to query.
        nomen: String,
        /// Port for discovered nodes.
        portus: u16,
        /// Query interval.
        intervallum: Duration,
    },

    /// Kubernetes-based discovery.
    Kubernetes {
        /// Namespace.
        spatium: String,
        /// Service name.
        servitium: String,
        /// Label selector.
        selector: Option<String>,
    },

    /// Consul-based discovery.
    Consul {
        /// Consul address.
        inscriptio: InscriptioNodi,
        /// Service name.
        servitium: String,
        /// Datacenter.
        centrum: Option<String>,
    },

    /// etcd-based discovery.
    Etcd {
        /// etcd endpoints.
        endpoints: Vec<InscriptioNodi>,
        /// Key prefix.
        praefixum: String,
    },

    /// Custom discovery provider.
    Proprius {
        /// Provider name.
        nomen: String,
        /// Configuration string.
        configuratio: String,
    },
}

// =============================================================================
// Consensus Protocol
// =============================================================================

/// Consensus protocol for cluster coordination.
///
/// # Latin Etymology
/// *Protocollum consensus* = consensus protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProtocollumConsensus {
    /// Raft consensus.
    #[default]
    Raft,
    /// Paxos consensus.
    Paxos,
    /// ZAB (Zookeeper Atomic Broadcast).
    Zab,
    /// PBFT (Practical Byzantine Fault Tolerance).
    Pbft,
    /// No consensus (single-node or external coordination).
    Nullus,
}

// =============================================================================
// TLS Configuration
// =============================================================================

/// TLS configuration for secure communication.
#[derive(Debug, Clone)]
pub struct ConfiguratioTls {
    /// Certificate path.
    pub certificatum: String,
    /// Private key path.
    pub clavis: String,
    /// CA certificate path.
    pub ca: Option<String>,
    /// Verify client certificates.
    pub verifica_clientes: bool,
}

// =============================================================================
// Cluster State
// =============================================================================

/// Current state of the cluster.
///
/// # Latin Etymology
/// *Status gregis* = cluster state.
#[derive(Debug, Clone)]
pub struct StatusGregis {
    /// Cluster configuration.
    pub configuratio: ConfiguratioGregis,
    /// All known nodes.
    pub nodi: Vec<InformationesNodi>,
    /// Current leader (if using leader-based consensus).
    pub dux: Option<NodusIdentitas>,
    /// Cluster health.
    pub salus: SalusGregis,
    /// Effect routing table.
    pub tabula_dirigendi: TabulaDirigendi,
    /// Cluster generation/epoch.
    pub generatio: u64,
}

impl StatusGregis {
    /// Create initial cluster state.
    #[inline]
    pub fn new(configuratio: ConfiguratioGregis) -> Self {
        StatusGregis {
            configuratio,
            nodi: Vec::with_capacity(16),
            dux: None,
            salus: SalusGregis::Unknown,
            tabula_dirigendi: TabulaDirigendi::new(),
            generatio: 0,
        }
    }

    /// Get healthy nodes.
    #[inline]
    pub fn healthy_nodes(&self) -> impl Iterator<Item = &InformationesNodi> {
        self.nodi.iter().filter(|n| n.is_healthy())
    }

    /// Get nodes that can execute computations.
    #[inline]
    pub fn executor_nodes(&self) -> impl Iterator<Item = &InformationesNodi> {
        self.nodi.iter().filter(|n| n.can_execute())
    }

    /// Get node by ID.
    pub fn get_node(&self, id: NodusIdentitas) -> Option<&InformationesNodi> {
        self.nodi.iter().find(|n| n.identitas == id)
    }

    /// Get mutable node by ID.
    pub fn get_node_mut(&mut self, id: NodusIdentitas) -> Option<&mut InformationesNodi> {
        self.nodi.iter_mut().find(|n| n.identitas == id)
    }

    /// Add or refresh a node. Re-discovered nodes update status, load, and
    /// capabilities (previously silently dropped — cluster state went stale).
    pub fn add_node(&mut self, node: InformationesNodi) {
        // Re-route: capabilities may have changed since last discovery.
        self.tabula_dirigendi.remove_node(node.identitas);
        for effect_id in &node.facultates.effectus_tractati {
            self.tabula_dirigendi.add_route(*effect_id, node.identitas);
        }
        if let Some(existing) = self.nodi.iter_mut().find(|n| n.identitas == node.identitas) {
            *existing = node;
        } else {
            self.nodi.push(node);
        }
        self.update_health();
    }

    /// Remove a node.
    pub fn remove_node(&mut self, id: NodusIdentitas) {
        self.tabula_dirigendi.remove_node(id);
        self.nodi.retain(|n| n.identitas != id);
        self.update_health();
    }

    /// Update a node's status.
    pub fn update_node_status(&mut self, id: NodusIdentitas, status: StatusNodi) {
        if let Some(node) = self.get_node_mut(id) {
            node.status = status;
            self.update_health();
        }
    }

    /// Recalculate cluster health.
    fn update_health(&mut self) {
        let total = self.nodi.len();
        let healthy = self.nodi.iter().filter(|n| n.is_healthy()).count();

        self.salus = if total == 0 {
            SalusGregis::Unknown
        } else if healthy == total {
            SalusGregis::Sanus
        } else if healthy > total / 2 {
            SalusGregis::Degradatus
        } else if healthy > 0 {
            SalusGregis::Criticus
        } else {
            SalusGregis::Mortuus
        };
    }

    /// Check if cluster has quorum.
    pub fn has_quorum(&self) -> bool {
        let total = self.nodi.len();
        let healthy = self.nodi.iter().filter(|n| n.is_healthy()).count();
        healthy > total / 2
    }
}

/// Cluster health status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SalusGregis {
    /// Unknown/starting.
    #[default]
    Unknown,
    /// All nodes healthy.
    Sanus,
    /// Some nodes unhealthy but quorum maintained.
    Degradatus,
    /// Below quorum threshold.
    Criticus,
    /// All nodes dead.
    Mortuus,
}

impl fmt::Display for SalusGregis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SalusGregis::Unknown => write!(f, "unknown"),
            SalusGregis::Sanus => write!(f, "healthy"),
            SalusGregis::Degradatus => write!(f, "degraded"),
            SalusGregis::Criticus => write!(f, "critical"),
            SalusGregis::Mortuus => write!(f, "dead"),
        }
    }
}

// =============================================================================
// Cluster Manager
// =============================================================================

/// Manager for cluster operations.
///
/// # Latin Etymology
/// *Administrator gregis* = cluster administrator.
pub struct AdministratorGregis {
    /// Local node ID.
    pub nodus_localis: NodusIdentitas,
    /// Cluster state.
    pub status: StatusGregis,
    /// Discovery provider.
    pub inventor: Option<Arc<dyn InventorNodorum>>,
}

impl AdministratorGregis {
    /// Create a new cluster manager.
    pub fn new(nodus_localis: NodusIdentitas, configuratio: ConfiguratioGregis) -> Self {
        AdministratorGregis {
            nodus_localis,
            status: StatusGregis::new(configuratio),
            inventor: None,
        }
    }

    /// Set the discovery provider.
    pub fn with_discovery(mut self, inventor: Arc<dyn InventorNodorum>) -> Self {
        self.inventor = Some(inventor);
        self
    }

    /// Check if this node is the leader.
    pub fn is_leader(&self) -> bool {
        self.status.dux == Some(self.nodus_localis)
    }

    /// Get the current leader.
    pub fn leader(&self) -> Option<&InformationesNodi> {
        self.status.dux.and_then(|id| self.status.get_node(id))
    }

    /// Join the cluster.
    pub fn join(
        &mut self,
        info: InformationesNodi,
    ) -> Pin<Box<dyn Future<Output = Result<(), ErrorGregis>> + Send + '_>> {
        Box::pin(async move {
            self.status.add_node(info);
            self.status.generatio += 1;
            Ok(())
        })
    }

    /// Leave the cluster.
    pub fn leave(&mut self) -> Pin<Box<dyn Future<Output = Result<(), ErrorGregis>> + Send + '_>> {
        Box::pin(async move {
            self.status.remove_node(self.nodus_localis);
            self.status.generatio += 1;
            Ok(())
        })
    }

    /// Refresh node list from discovery.
    pub fn refresh(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = Result<(), ErrorGregis>> + Send + '_>> {
        Box::pin(async move {
            if let Some(inventor) = &self.inventor {
                let nodes = inventor.discover().await?;
                for node in nodes {
                    self.status.add_node(node);
                }
            }
            Ok(())
        })
    }

    /// Select nodes for a computation.
    pub fn select_nodes(&self, required_effects: &[u64], count: usize) -> Vec<&InformationesNodi> {
        let total = self.status.nodi.len();
        let mut candidates: Vec<_> = Vec::with_capacity(total);
        candidates.extend(self.status.executor_nodes().filter(|n| {
            required_effects
                .iter()
                .all(|e| n.facultates.can_handle_effect(*e))
        }));

        // Sort by load factor (prefer less loaded nodes)
        candidates.sort_by(|a, b| {
            a.facultates
                .load_factor()
                .partial_cmp(&b.facultates.load_factor())
                .unwrap_or(core::cmp::Ordering::Equal)
        });

        candidates.truncate(count);
        candidates
    }
}

impl fmt::Debug for AdministratorGregis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AdministratorGregis")
            .field("nodus_localis", &self.nodus_localis)
            .field("status", &self.status)
            .finish_non_exhaustive()
    }
}

// =============================================================================
// Node Discovery Trait
// =============================================================================

/// Trait for node discovery providers.
///
/// # Latin Etymology
/// *Inventor nodorum* = discoverer of nodes.
pub trait InventorNodorum: Send + Sync {
    /// Discover nodes in the cluster.
    fn discover(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<InformationesNodi>, ErrorGregis>> + Send + '_>>;

    /// Register this node with the discovery service.
    fn register(
        &self,
        info: &InformationesNodi,
    ) -> Pin<Box<dyn Future<Output = Result<(), ErrorGregis>> + Send + '_>>;

    /// Deregister this node.
    fn deregister(
        &self,
        id: NodusIdentitas,
    ) -> Pin<Box<dyn Future<Output = Result<(), ErrorGregis>> + Send + '_>>;

    /// Watch for changes.
    fn watch(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<EventusGregis, ErrorGregis>> + Send + '_>>;
}

/// Cluster event from watching.
#[derive(Debug, Clone)]
pub enum EventusGregis {
    /// Node joined. Boxed: `InformationesNodi` is much larger than the other
    /// variants, and join events are rare relative to status traffic.
    NodusAddidit(Box<InformationesNodi>),
    /// Node left.
    NodusAbscessit(NodusIdentitas),
    /// Node status changed.
    StatusMutatus {
        /// Node whose status changed.
        nodus: NodusIdentitas,
        /// The status the node transitioned to.
        status: StatusNodi,
    },
    /// Leader changed.
    DuxMutatus(Option<NodusIdentitas>),
}

// =============================================================================
// Errors
// =============================================================================

/// Cluster operation error.
#[derive(Debug, Clone)]
pub enum ErrorGregis {
    /// Discovery error.
    Inventio(String),
    /// Consensus error.
    Consensus(String),
    /// Network error.
    Rete(String),
    /// Configuration error.
    Configuratio(String),
    /// Node not found.
    NodusNonInventus(NodusIdentitas),
    /// No quorum.
    SineQuorum,
    /// Already joined.
    IamConiunctus,
    /// Not joined.
    NonConiunctus,
}

impl fmt::Display for ErrorGregis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorGregis::Inventio(msg) => write!(f, "Discovery error: {msg}"),
            ErrorGregis::Consensus(msg) => write!(f, "Consensus error: {msg}"),
            ErrorGregis::Rete(msg) => write!(f, "Network error: {msg}"),
            ErrorGregis::Configuratio(msg) => write!(f, "Configuration error: {msg}"),
            ErrorGregis::NodusNonInventus(id) => write!(f, "Node not found: {id:?}"),
            ErrorGregis::SineQuorum => write!(f, "No quorum"),
            ErrorGregis::IamConiunctus => write!(f, "Already joined cluster"),
            ErrorGregis::NonConiunctus => write!(f, "Not joined to cluster"),
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::distributed::{FacultatesNodi, MunusNodi};

    fn test_node(id: u64) -> InformationesNodi {
        InformationesNodi {
            identitas: NodusIdentitas::new(0, id),
            inscriptio: InscriptioNodi::new("localhost", 8080 + id as u16),
            munus: MunusNodi::Executor,
            status: StatusNodi::Sanus,
            facultates: FacultatesNodi::default(),
            tituli: Vec::new(),
            ultima_pulsatio: None,
        }
    }

    #[test]
    fn test_configuratio_default() {
        let config = ConfiguratioGregis::default();
        assert_eq!(config.nomen, "ordofp-cluster");
        assert_eq!(config.factor_replicationis, 1);
    }

    /// M15 regression: re-discovered nodes' status/load/capabilities updates
    /// were silently dropped, leaving cluster state permanently stale.
    #[test]
    fn add_node_updates_known_nodes() {
        let mut status = StatusGregis::new(ConfiguratioGregis::default());
        let mut node = test_node(1);
        status.add_node(node.clone());
        assert_eq!(
            status.get_node(node.identitas).unwrap().status,
            StatusNodi::Sanus
        );

        node.status = StatusNodi::Aegrotus;
        status.add_node(node.clone());

        assert_eq!(
            status.get_node(node.identitas).unwrap().status,
            StatusNodi::Aegrotus
        );
        assert_eq!(
            status.nodi.len(),
            1,
            "re-discovery must not duplicate the node"
        );
    }

    #[test]
    fn test_status_gregis_add_node() {
        let mut status = StatusGregis::new(ConfiguratioGregis::default());
        status.add_node(test_node(1));
        status.add_node(test_node(2));

        assert_eq!(status.nodi.len(), 2);
        assert_eq!(status.salus, SalusGregis::Sanus);
    }

    #[test]
    fn test_status_gregis_remove_node() {
        let mut status = StatusGregis::new(ConfiguratioGregis::default());
        status.add_node(test_node(1));
        status.add_node(test_node(2));
        status.remove_node(NodusIdentitas::new(0, 1));

        assert_eq!(status.nodi.len(), 1);
    }

    #[test]
    fn test_status_gregis_health() {
        let mut status = StatusGregis::new(ConfiguratioGregis::default());

        // Add 3 healthy nodes
        for i in 1..=3 {
            status.add_node(test_node(i));
        }
        assert_eq!(status.salus, SalusGregis::Sanus);

        // Mark one as unhealthy
        status.update_node_status(NodusIdentitas::new(0, 1), StatusNodi::Aegrotus);
        assert_eq!(status.salus, SalusGregis::Degradatus);

        // Still has quorum
        assert!(status.has_quorum());
    }

    #[test]
    fn test_administrator_select_nodes() {
        let mut admin =
            AdministratorGregis::new(NodusIdentitas::new(0, 0), ConfiguratioGregis::default());

        // Add nodes with different effect capabilities
        let mut node1 = test_node(1);
        node1.facultates.effectus_tractati = alloc::vec![1, 2];
        node1.facultates.munera_maxima = 10;
        node1.facultates.munera_currentia = 2;

        let mut node2 = test_node(2);
        node2.facultates.effectus_tractati = alloc::vec![1, 3];
        node2.facultates.munera_maxima = 10;
        node2.facultates.munera_currentia = 5;

        admin.status.add_node(node1);
        admin.status.add_node(node2);

        // Select nodes that can handle effect 1
        let selected = admin.select_nodes(&[1], 2);
        assert_eq!(selected.len(), 2);

        // Select nodes that can handle effect 2
        let selected = admin.select_nodes(&[2], 2);
        assert_eq!(selected.len(), 1);
    }
}
