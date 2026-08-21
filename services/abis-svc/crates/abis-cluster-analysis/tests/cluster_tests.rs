//! Tests for connected-components cluster analysis.

use chrono::Utc;
use eel_abis_cluster_analysis::cluster::connected_components;
use eel_abis_cluster_analysis::detector::detect_clusters;
use eel_abis_cluster_analysis::domain::{ClusterResolution, MatchEdge};
use eel_abis_cluster_analysis::error::ClusterError;
use eel_abis_cluster_analysis::resolver::{resolve_false_alarm, resolve_merge, resolve_split};
use uuid::Uuid;

fn edge(a: Uuid, b: Uuid, score: f64) -> MatchEdge {
    MatchEdge {
        subject_a: a,
        subject_b: b,
        score,
        confirmed_at: Utc::now(),
    }
}

#[test]
fn empty_graph_returns_empty_clusters() {
    let clusters = connected_components(&[]).unwrap();
    assert!(clusters.is_empty());
}

#[test]
fn single_edge_creates_one_cluster() {
    let a = Uuid::now_v7();
    let b = Uuid::now_v7();
    let clusters = connected_components(&[edge(a, b, 0.9)]).unwrap();
    assert_eq!(clusters.len(), 1);
    assert_eq!(clusters[0].member_ids.len(), 2);
}

#[test]
fn two_disjoint_pairs_create_two_clusters() {
    let (a, b, c, d) = (
        Uuid::now_v7(),
        Uuid::now_v7(),
        Uuid::now_v7(),
        Uuid::now_v7(),
    );
    let clusters = connected_components(&[edge(a, b, 0.9), edge(c, d, 0.85)]).unwrap();
    assert_eq!(clusters.len(), 2);
}

#[test]
fn triangle_creates_one_cluster() {
    let (a, b, c) = (Uuid::now_v7(), Uuid::now_v7(), Uuid::now_v7());
    let clusters =
        connected_components(&[edge(a, b, 0.9), edge(b, c, 0.85), edge(a, c, 0.88)]).unwrap();
    assert_eq!(clusters.len(), 1);
    assert_eq!(clusters[0].member_ids.len(), 3);
}

#[test]
fn four_way_collision_detected() {
    let ids: Vec<Uuid> = (0..4).map(|_| Uuid::now_v7()).collect();
    let edges = vec![
        edge(ids[0], ids[1], 0.92),
        edge(ids[1], ids[2], 0.88),
        edge(ids[2], ids[3], 0.85),
    ];
    let clusters = connected_components(&edges).unwrap();
    assert_eq!(clusters.len(), 1);
    assert_eq!(clusters[0].member_ids.len(), 4);
}

#[test]
fn cluster_edge_count_recorded() {
    let (a, b) = (Uuid::now_v7(), Uuid::now_v7());
    let clusters = connected_components(&[edge(a, b, 0.9)]).unwrap();
    assert_eq!(clusters[0].edge_count, 1);
}

#[test]
fn resolve_merge_sets_status() {
    let (a, b) = (Uuid::now_v7(), Uuid::now_v7());
    let mut cluster = connected_components(&[edge(a, b, 0.9)])
        .unwrap()
        .into_iter()
        .next()
        .unwrap();
    resolve_merge(&mut cluster).unwrap();
    assert_eq!(cluster.resolution, ClusterResolution::MergedIntoOne);
    assert!(cluster.resolved_at.is_some());
}

#[test]
fn resolve_split_sets_status() {
    let (a, b) = (Uuid::now_v7(), Uuid::now_v7());
    let mut cluster = connected_components(&[edge(a, b, 0.9)])
        .unwrap()
        .into_iter()
        .next()
        .unwrap();
    resolve_split(&mut cluster).unwrap();
    assert_eq!(cluster.resolution, ClusterResolution::SplitIntoGroups);
}

#[test]
fn resolve_false_alarm_sets_note() {
    let (a, b) = (Uuid::now_v7(), Uuid::now_v7());
    let mut cluster = connected_components(&[edge(a, b, 0.9)])
        .unwrap()
        .into_iter()
        .next()
        .unwrap();
    resolve_false_alarm(&mut cluster, "twins confirmed").unwrap();
    assert_eq!(cluster.resolution, ClusterResolution::FalseAlarm);
    assert_eq!(cluster.notes.as_deref(), Some("twins confirmed"));
}

#[test]
fn double_resolve_fails() {
    let (a, b) = (Uuid::now_v7(), Uuid::now_v7());
    let mut cluster = connected_components(&[edge(a, b, 0.9)])
        .unwrap()
        .into_iter()
        .next()
        .unwrap();
    resolve_merge(&mut cluster).unwrap();
    assert!(matches!(
        resolve_merge(&mut cluster),
        Err(ClusterError::AlreadyResolved)
    ));
}

#[test]
fn detect_clusters_filters_old_edges() {
    let (a, b) = (Uuid::now_v7(), Uuid::now_v7());
    let old = MatchEdge {
        subject_a: a,
        subject_b: b,
        score: 0.9,
        confirmed_at: Utc::now() - chrono::Duration::days(60),
    };
    let since = Utc::now() - chrono::Duration::days(30);
    assert!(detect_clusters(&[old], since).unwrap().is_empty());
}

#[test]
fn detect_clusters_includes_recent_edges() {
    let (a, b) = (Uuid::now_v7(), Uuid::now_v7());
    let recent = MatchEdge {
        subject_a: a,
        subject_b: b,
        score: 0.9,
        confirmed_at: Utc::now(),
    };
    let since = Utc::now() - chrono::Duration::days(30);
    assert_eq!(detect_clusters(&[recent], since).unwrap().len(), 1);
}

#[test]
fn star_topology_creates_one_cluster() {
    let center = Uuid::now_v7();
    let spokes: Vec<Uuid> = (0..5).map(|_| Uuid::now_v7()).collect();
    let edges: Vec<MatchEdge> = spokes.iter().map(|&s| edge(center, s, 0.9)).collect();
    let clusters = connected_components(&edges).unwrap();
    assert_eq!(clusters.len(), 1);
    assert_eq!(clusters[0].member_ids.len(), 6);
}

#[test]
fn cluster_resolution_pending_by_default() {
    let (a, b) = (Uuid::now_v7(), Uuid::now_v7());
    let cluster = connected_components(&[edge(a, b, 0.9)])
        .unwrap()
        .into_iter()
        .next()
        .unwrap();
    assert_eq!(cluster.resolution, ClusterResolution::Pending);
}

#[test]
fn split_after_merge_errors() {
    let (a, b) = (Uuid::now_v7(), Uuid::now_v7());
    let mut cluster = connected_components(&[edge(a, b, 0.9)])
        .unwrap()
        .into_iter()
        .next()
        .unwrap();
    resolve_split(&mut cluster).unwrap();
    assert!(matches!(
        resolve_false_alarm(&mut cluster, "x"),
        Err(ClusterError::AlreadyResolved)
    ));
}
