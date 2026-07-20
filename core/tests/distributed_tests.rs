//! Failure-path and state-machine tests for `core/src/distributed/`.
//!
//! The inline unit tests cover happy-path construction; these tests drive
//! the cluster state machine through health transitions, quorum edges,
//! re-discovery refresh, routing-table consistency, node selection under
//! mixed capabilities, and the protocol's placeholder-response semantics.

#![cfg(feature = "distributed")]

extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

use ordofp_core::distributed::{
    AdministratorGregis, AedificatorNuntii, AffinitasNodi, ConfiguratioGregis, CorpusNuntii,
    ErrorCorpus, FacultatesNodi, GenusNuntii, GpuFacultas, InformationesNodi, InscriptioNodi,
    MunusNodi, NodusIdentitas, Nuntius, ProtocollumSchema, RequirementaFacultatum, SalusGregis,
    StatusGregis, StatusNodi, VersioProtocolli,
};

// =============================================================================
// Minimal executor for the module's immediately-ready futures
// =============================================================================

fn noop_raw_waker() -> RawWaker {
    fn clone(_: *const ()) -> RawWaker {
        noop_raw_waker()
    }
    fn noop(_: *const ()) {}
    RawWaker::new(
        core::ptr::null(),
        &RawWakerVTable::new(clone, noop, noop, noop),
    )
}

/// Poll a future that must complete without yielding (all futures in the
/// distributed module resolve on first poll).
fn block_on_ready<F: Future + ?Sized>(mut fut: Pin<Box<F>>) -> F::Output {
    // SAFETY: the vtable functions are all no-ops over a null pointer.
    let waker = unsafe { Waker::from_raw(noop_raw_waker()) };
    let mut cx = Context::from_waker(&waker);
    match fut.as_mut().poll(&mut cx) {
        Poll::Ready(out) => out,
        Poll::Pending => panic!("distributed-module future unexpectedly yielded"),
    }
}

// =============================================================================
// Test fixtures
// =============================================================================

fn node_id(n: u64) -> NodusIdentitas {
    NodusIdentitas::new(n, n)
}

/// A healthy executor node handling the given effects, with the given load.
fn executor(n: u64, effects: &[u64], current: u32, max: u32) -> InformationesNodi {
    let mut info = InformationesNodi::new(
        node_id(n),
        InscriptioNodi::new(alloc::format!("node-{n}"), 7000),
    );
    info.status = StatusNodi::Sanus;
    info.facultates = FacultatesNodi {
        nuclei_cpu: 4,
        memoria_bytes: 1 << 30,
        memoria_disponibilis: 1 << 29,
        gpu: None,
        effectus_tractati: effects.to_vec(),
        munera_maxima: max,
        munera_currentia: current,
    };
    info
}

fn cluster_with(nodes: Vec<InformationesNodi>) -> StatusGregis {
    let mut status = StatusGregis::new(ConfiguratioGregis::new("test-grex"));
    for n in nodes {
        status.add_node(n);
    }
    status
}

// =============================================================================
// Cluster health state machine
// =============================================================================

#[test]
fn health_transitions_through_all_states() {
    let mut status = StatusGregis::new(ConfiguratioGregis::new("salus"));
    assert_eq!(status.salus, SalusGregis::Unknown, "empty cluster");

    for n in 1..=4 {
        status.add_node(executor(n, &[], 0, 4));
    }
    assert_eq!(status.salus, SalusGregis::Sanus, "4/4 healthy");

    status.update_node_status(node_id(1), StatusNodi::Aegrotus);
    assert_eq!(status.salus, SalusGregis::Degradatus, "3/4 healthy");

    status.update_node_status(node_id(2), StatusNodi::Inaccessibilis);
    assert_eq!(
        status.salus,
        SalusGregis::Criticus,
        "2/4 healthy is not a majority"
    );

    status.update_node_status(node_id(3), StatusNodi::Descendit);
    status.update_node_status(node_id(4), StatusNodi::Aegrotus);
    assert_eq!(status.salus, SalusGregis::Mortuus, "0/4 healthy");

    // Recovery must be reachable from the dead state.
    for n in 1..=4 {
        status.update_node_status(node_id(n), StatusNodi::Sanus);
    }
    assert_eq!(status.salus, SalusGregis::Sanus, "full recovery");
}

#[test]
fn health_recomputed_on_node_removal() {
    let mut status = cluster_with(vec![
        executor(1, &[], 0, 4),
        executor(2, &[], 0, 4),
        executor(3, &[], 0, 4),
    ]);
    status.update_node_status(node_id(3), StatusNodi::Aegrotus);
    assert_eq!(status.salus, SalusGregis::Degradatus);

    // Removing the sick node restores full health.
    status.remove_node(node_id(3));
    assert_eq!(status.salus, SalusGregis::Sanus);

    // Removing everything returns the cluster to Unknown, not Mortuus.
    status.remove_node(node_id(1));
    status.remove_node(node_id(2));
    assert_eq!(status.salus, SalusGregis::Unknown);
}

#[test]
fn quorum_edges() {
    // Empty cluster has no quorum.
    let status = StatusGregis::new(ConfiguratioGregis::new("quorum"));
    assert!(!status.has_quorum());

    // A single healthy node is its own majority.
    let status = cluster_with(vec![executor(1, &[], 0, 4)]);
    assert!(status.has_quorum());

    // Exactly half is NOT a majority: 1 healthy of 2, 2 healthy of 4.
    let mut status = cluster_with(vec![executor(1, &[], 0, 4), executor(2, &[], 0, 4)]);
    status.update_node_status(node_id(2), StatusNodi::Aegrotus);
    assert!(!status.has_quorum(), "1/2 is not a majority");

    let mut status = cluster_with((1..=4).map(|n| executor(n, &[], 0, 4)).collect());
    status.update_node_status(node_id(3), StatusNodi::Aegrotus);
    status.update_node_status(node_id(4), StatusNodi::Aegrotus);
    assert!(!status.has_quorum(), "2/4 is not a majority");

    // 2 healthy of 3 is a majority.
    let mut status = cluster_with((1..=3).map(|n| executor(n, &[], 0, 4)).collect());
    status.update_node_status(node_id(3), StatusNodi::Aegrotus);
    assert!(status.has_quorum(), "2/3 is a majority");
}

#[test]
fn update_status_of_unknown_node_is_noop() {
    let mut status = cluster_with(vec![executor(1, &[], 0, 4)]);
    let before = status.salus;
    status.update_node_status(node_id(99), StatusNodi::Aegrotus);
    assert_eq!(status.salus, before);
    assert_eq!(status.nodi.len(), 1);
}

// =============================================================================
// Re-discovery refresh and routing table consistency
// =============================================================================

#[test]
fn rediscovered_node_updates_in_place_and_reroutes() {
    let mut status = cluster_with(vec![executor(1, &[10, 20], 0, 4)]);
    assert_eq!(
        status.tabula_dirigendi.get_handlers(10),
        Some(&[node_id(1)][..])
    );
    assert_eq!(
        status.tabula_dirigendi.get_handlers(20),
        Some(&[node_id(1)][..])
    );

    // Same identity re-discovered with different capabilities.
    status.add_node(executor(1, &[30], 2, 8));

    assert_eq!(status.nodi.len(), 1, "must update in place, not duplicate");
    assert_eq!(status.nodi[0].facultates.munera_maxima, 8);

    // Stale routes must be gone; the new route must exist.
    assert_eq!(
        status
            .tabula_dirigendi
            .get_handlers(10)
            .map_or(0, <[NodusIdentitas]>::len),
        0,
        "stale route to effect 10 survived re-discovery"
    );
    assert_eq!(
        status.tabula_dirigendi.get_handlers(30),
        Some(&[node_id(1)][..])
    );
}

#[test]
fn removing_node_clears_only_its_routes() {
    let mut status = cluster_with(vec![executor(1, &[10], 0, 4), executor(2, &[10, 20], 0, 4)]);

    status.remove_node(node_id(1));

    assert_eq!(
        status.tabula_dirigendi.get_handlers(10),
        Some(&[node_id(2)][..]),
        "effect 10 must still route to the surviving node"
    );
    assert_eq!(
        status.tabula_dirigendi.get_handlers(20),
        Some(&[node_id(2)][..])
    );
    assert!(status.get_node(node_id(1)).is_none());
}

// =============================================================================
// Node selection
// =============================================================================

#[test]
fn select_nodes_filters_and_ranks_by_load() {
    let config = ConfiguratioGregis::new("select");
    let mut admin = AdministratorGregis::new(node_id(0), config);

    // Candidates handling effect 7 at different loads.
    admin.status.add_node(executor(1, &[7], 3, 4)); // load 0.75
    admin.status.add_node(executor(2, &[7], 1, 4)); // load 0.25
    admin.status.add_node(executor(3, &[7], 2, 4)); // load 0.50

    // Distractors that must all be excluded:
    admin.status.add_node(executor(4, &[8], 0, 4)); // wrong effect
    let mut sick = executor(5, &[7], 0, 4);
    sick.status = StatusNodi::Aegrotus; // unhealthy
    admin.status.add_node(sick);
    let gateway = executor(6, &[7], 0, 4).with_role(MunusNodi::Porta); // non-executor role
    admin.status.add_node(gateway);

    let selected = admin.select_nodes(&[7], 2);
    let ids: Vec<_> = selected.iter().map(|n| n.identitas).collect();
    assert_eq!(
        ids,
        vec![node_id(2), node_id(3)],
        "least-loaded first, truncated to count"
    );

    // Asking for more than exist returns all candidates.
    let all = admin.select_nodes(&[7], 10);
    assert_eq!(all.len(), 3);

    // An unsatisfiable effect yields no candidates.
    assert!(admin.select_nodes(&[999], 3).is_empty());
}

#[test]
fn join_and_leave_update_membership_and_generation() {
    let mut admin = AdministratorGregis::new(node_id(1), ConfiguratioGregis::new("join"));
    assert!(!admin.is_leader());
    assert!(admin.leader().is_none());

    let gen0 = admin.status.generatio;
    block_on_ready(admin.join(executor(1, &[7], 0, 4))).expect("join must succeed");
    assert_eq!(admin.status.nodi.len(), 1);
    assert_eq!(admin.status.generatio, gen0 + 1);

    // Elect ourselves; leader lookup must resolve through node info.
    admin.status.dux = Some(node_id(1));
    assert!(admin.is_leader());
    assert_eq!(
        admin.leader().map(|n| n.identitas),
        Some(node_id(1)),
        "leader() must return the elected node's info"
    );

    block_on_ready(admin.leave()).expect("leave must succeed");
    assert!(admin.status.get_node(node_id(1)).is_none());
    assert_eq!(admin.status.generatio, gen0 + 2);
    assert!(
        admin.leader().is_none(),
        "leader info must be gone once the leader left"
    );
}

// =============================================================================
// Protocol: version compatibility and response semantics
// =============================================================================

#[test]
fn version_compatibility_ignores_patch_and_is_asymmetric_in_minor() {
    let v1_0_0 = VersioProtocolli::new(1, 0, 0);
    let v1_0_9 = VersioProtocolli::new(1, 0, 9);
    let v1_2_0 = VersioProtocolli::new(1, 2, 0);

    // Patch differences never matter.
    assert!(v1_0_0.is_compatible(&v1_0_9));
    assert!(v1_0_9.is_compatible(&v1_0_0));

    // Newer minor talks to older, not vice versa.
    assert!(v1_2_0.is_compatible(&v1_0_0));
    assert!(!v1_0_0.is_compatible(&v1_2_0));

    // Major mismatch in either direction is incompatible.
    let v2 = VersioProtocolli::new(2, 0, 0);
    assert!(!v2.is_compatible(&v1_0_0));
    assert!(!v1_0_0.is_compatible(&v2));
}

#[test]
fn message_ids_are_unique_and_correlation_links_responses() {
    let a = Nuntius::new(node_id(1), GenusNuntii::Pulsatio, CorpusNuntii::Vacuum);
    let b = Nuntius::new(node_id(1), GenusNuntii::Pulsatio, CorpusNuntii::Vacuum);
    assert_ne!(a.caput.id, b.caput.id, "message IDs must be unique");

    let request = Nuntius::new(
        node_id(1),
        GenusNuntii::InquisitioNodorum,
        CorpusNuntii::Vacuum,
    )
    .to(node_id(2));
    let response = request.respond(
        GenusNuntii::ResponsumNodorum,
        CorpusNuntii::Nodi(Vec::new()),
    );

    assert_eq!(response.caput.mittens, node_id(2), "responder is recipient");
    assert_eq!(response.caput.recipiens, Some(node_id(1)));
    assert_eq!(response.caput.correlatio, Some(request.caput.id));
    assert_eq!(response.caput.genus, GenusNuntii::ResponsumNodorum);
}

#[test]
fn respond_to_broadcast_falls_back_to_original_sender() {
    // A broadcast request has no recipient; the documented placeholder
    // semantics fall back to the original sender as the responder identity.
    let broadcast = Nuntius::new(node_id(9), GenusNuntii::Pulsatio, CorpusNuntii::Vacuum);
    assert_eq!(broadcast.caput.recipiens, None);

    let response = broadcast.respond(GenusNuntii::Pulsatio, CorpusNuntii::Vacuum);
    assert_eq!(response.caput.mittens, node_id(9));
    assert_eq!(response.caput.recipiens, Some(node_id(9)));
}

#[test]
fn builder_messages_carry_expected_genus_and_bodies() {
    let builder = AedificatorNuntii::new(node_id(1));

    let comp = builder.computatio(
        42,
        ordofp_core::distributed::NodusSerializabilis::Vacuus,
        vec![7],
    );
    assert_eq!(comp.caput.genus, GenusNuntii::SubmissioComputationis);
    match comp.corpus {
        CorpusNuntii::Computatio(body) => {
            assert_eq!(body.id, 42);
            assert_eq!(body.effectus_requiriti, vec![7]);
        }
        other => panic!("expected Computatio body, got {other:?}"),
    }

    let eff = builder.effectus(7, 3, vec![1, 2, 3]);
    assert_eq!(eff.caput.genus, GenusNuntii::OperatioEffectus);

    let err = builder.error(ErrorCorpus::NO_QUORUM, "no quorum");
    assert_eq!(err.caput.genus, GenusNuntii::Error);
    match err.corpus {
        CorpusNuntii::Error(body) => {
            assert_eq!(body.codex, ErrorCorpus::NO_QUORUM);
            assert!(!body.iterabilis, "errors are non-retriable by default");
        }
        other => panic!("expected Error body, got {other:?}"),
    }
}

// =============================================================================
// Affinity and capability requirements
// =============================================================================

#[test]
fn affinity_label_matching_requires_all_labels() {
    let node = executor(1, &[], 0, 4)
        .with_label("regio", "eu-west")
        .with_label("tier", "gold");

    let both = AffinitasNodi::Tituli(vec![
        ("regio".into(), "eu-west".into()),
        ("tier".into(), "gold".into()),
    ]);
    let partial = AffinitasNodi::Tituli(vec![
        ("regio".into(), "eu-west".into()),
        ("tier".into(), "silver".into()),
    ]);

    assert!(both.matches(&node));
    assert!(!partial.matches(&node), "ALL labels must match");
    // Score counts matching labels individually even when matches() is false.
    assert_eq!(both.score(&node), 20);
    assert_eq!(partial.score(&node), 10);

    assert!(AffinitasNodi::Regio("eu-west".into()).matches(&node));
    assert!(!AffinitasNodi::Regio("us-east".into()).matches(&node));
}

#[test]
fn anti_affinity_inverts_and_weighted_takes_max() {
    let node = executor(1, &[], 0, 4);

    let avoid = AffinitasNodi::Non(Box::new(AffinitasNodi::Nodus(node_id(1))));
    assert!(!avoid.matches(&node));
    assert_eq!(avoid.score(&node), 0);

    let avoid_other = AffinitasNodi::Non(Box::new(AffinitasNodi::Nodus(node_id(2))));
    assert!(avoid_other.matches(&node));
    assert_eq!(avoid_other.score(&node), 10);

    // Weighted affinity picks the strongest branch and saturates rather
    // than wrapping on overflow.
    let weighted = AffinitasNodi::Ponderata(vec![
        (AffinitasNodi::Quodlibet, 2),
        (AffinitasNodi::Nodus(node_id(1)), u32::MAX),
    ]);
    assert!(weighted.matches(&node));
    assert_eq!(weighted.score(&node), u32::MAX, "must saturate, not wrap");
}

#[test]
fn capability_requirements_reject_each_deficiency() {
    let mut cap = FacultatesNodi {
        nuclei_cpu: 8,
        memoria_bytes: 1 << 32,
        memoria_disponibilis: 1 << 31,
        gpu: None,
        effectus_tractati: vec![7],
        munera_maxima: 4,
        munera_currentia: 0,
    };

    let mut req = RequirementaFacultatum {
        nuclei_cpu_min: Some(8),
        memoria_min: Some(1 << 31),
        gpu_requiritur: false,
        effectus_requiriti: vec![7],
    };
    assert!(req.satisfies(&cap), "all requirements met");

    req.nuclei_cpu_min = Some(16);
    assert!(!req.satisfies(&cap), "cpu below minimum");
    req.nuclei_cpu_min = Some(8);

    req.memoria_min = Some(1 << 32);
    assert!(!req.satisfies(&cap), "available memory below minimum");
    req.memoria_min = Some(1 << 31);

    req.gpu_requiritur = true;
    assert!(!req.satisfies(&cap), "gpu required but absent");
    cap.gpu = Some(GpuFacultas {
        vendor: "test".into(),
        modellum: "unit".into(),
        vram_bytes: 1 << 30,
        versio_computationis: "1.0".into(),
    });
    assert!(req.satisfies(&cap), "gpu requirement now met");

    req.effectus_requiriti = vec![7, 8];
    assert!(!req.satisfies(&cap), "missing effect handler");
}

#[test]
fn load_factor_edges() {
    let mut cap = FacultatesNodi {
        munera_maxima: 0,
        munera_currentia: 0,
        ..FacultatesNodi::default()
    };
    assert!(
        (cap.load_factor() - 1.0).abs() < f64::EPSILON,
        "zero-capacity node must report full load, not divide by zero"
    );
    assert!(!cap.has_capacity());

    cap.munera_maxima = 4;
    cap.munera_currentia = 4;
    assert!((cap.load_factor() - 1.0).abs() < f64::EPSILON);
    assert!(!cap.has_capacity(), "at capacity");

    cap.munera_currentia = 3;
    assert!(cap.has_capacity());
}

#[test]
fn node_address_uri_covers_every_schema() {
    let cases = [
        (ProtocollumSchema::Grpc, "grpc://host:1"),
        (ProtocollumSchema::Http, "http://host:1"),
        (ProtocollumSchema::Https, "https://host:1"),
        (ProtocollumSchema::Tcp, "tcp://host:1"),
        (ProtocollumSchema::Unix, "unix://host:1"),
    ];
    for (schema, expected) in cases {
        assert_eq!(
            InscriptioNodi::with_schema("host", 1, schema).to_uri(),
            expected
        );
    }
}
