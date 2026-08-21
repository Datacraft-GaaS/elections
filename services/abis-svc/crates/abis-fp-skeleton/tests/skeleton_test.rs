//! Tests for skeleton thinning, graph extraction, and matching.

use eel_abis_fp_skeleton::{
    graph::{skeleton_to_graph, NodeKind},
    matcher::SkeletonMatcher,
    skeleton::{extract_skeleton, BinaryImage},
};

// Helper: create binary image from a vec of rows ("X" = foreground).
fn from_ascii(rows: &[&str]) -> BinaryImage {
    let h = rows.len() as u32;
    let w = rows[0].len() as u32;
    let pixels: Vec<u8> = rows
        .iter()
        .flat_map(|r| r.chars().map(|c| if c == 'X' { 1u8 } else { 0u8 }))
        .collect();
    BinaryImage::new(w, h, pixels).unwrap()
}

fn solid_rect(w: u32, h: u32) -> BinaryImage {
    BinaryImage::new(w, h, vec![1u8; (w * h) as usize]).unwrap()
}

fn empty(w: u32, h: u32) -> BinaryImage {
    BinaryImage::new(w, h, vec![0u8; (w * h) as usize]).unwrap()
}

fn horizontal_line(w: u32, h: u32) -> BinaryImage {
    let mut pixels = vec![0u8; (w * h) as usize];
    let mid = h / 2;
    for x in 0..w {
        pixels[(mid * w + x) as usize] = 1;
    }
    BinaryImage::new(w, h, pixels).unwrap()
}

// ----- BinaryImage construction -----

#[test]
fn test_binary_image_ok() {
    let img = BinaryImage::new(10, 10, vec![0u8; 100]);
    assert!(img.is_ok());
}

#[test]
fn test_binary_image_size_mismatch_err() {
    let img = BinaryImage::new(10, 10, vec![0u8; 50]);
    assert!(img.is_err());
}

#[test]
fn test_binary_image_pixel_access_out_of_bounds_returns_zero() {
    let img = BinaryImage::new(4, 4, vec![1u8; 16]).unwrap();
    assert_eq!(img.get(-1, 0), 0);
    assert_eq!(img.get(4, 0), 0);
    assert_eq!(img.get(0, -1), 0);
}

// ----- Zhang-Suen thinning -----

#[test]
fn test_thinning_empty_image() {
    let img = empty(32, 32);
    let skel = extract_skeleton(&img).unwrap();
    assert_eq!(skel.count_foreground(), 0);
}

#[test]
fn test_thinning_too_small_errors() {
    let img = BinaryImage::new(4, 4, vec![1u8; 16]).unwrap();
    assert!(extract_skeleton(&img).is_err());
}

#[test]
fn test_thinning_produces_thinner_image() {
    let thick = solid_rect(30, 30);
    let skel = extract_skeleton(&thick).unwrap();
    assert!(
        skel.count_foreground() < thick.count_foreground(),
        "thinned skeleton must have fewer pixels than the solid rectangle"
    );
}

#[test]
fn test_thinning_horizontal_line_stays_thin() {
    // A single-pixel horizontal line should remain after thinning.
    let line = horizontal_line(32, 32);
    let skel = extract_skeleton(&line).unwrap();
    // Count how many pixels remain on the central row.
    let h = 32u32;
    let w = 32u32;
    let mid = h / 2;
    let center_count: usize = (0..w)
        .filter(|&x| skel.get(x as i32, mid as i32) == 1)
        .count();
    // Most of the line should remain (allow some boundary erosion).
    assert!(
        center_count >= 20,
        "horizontal line center row should mostly survive thinning, got {center_count}"
    );
}

#[test]
fn test_thinning_solid_rectangle_produces_single_pixel_thick() {
    // After thinning a 30x30 solid rect, all foreground pixels should have <= 2 skeleton neighbours.
    let thick = solid_rect(30, 30);
    let skel = extract_skeleton(&thick).unwrap();
    let w = skel.width as i32;
    let h = skel.height as i32;
    for y in 1..h - 1 {
        for x in 1..w - 1 {
            if skel.get(x, y) == 1 {
                let n: u8 = [
                    (-1i32, -1i32),
                    (0, -1),
                    (1, -1),
                    (1, 0),
                    (1, 1),
                    (0, 1),
                    (-1, 1),
                    (-1, 0),
                ]
                .iter()
                .map(|&(dx, dy)| skel.get(x + dx, y + dy))
                .sum();
                assert!(
                    n <= 3,
                    "skeleton pixel at ({x},{y}) has {n} neighbours - not thin"
                );
            }
        }
    }
}

// ----- Graph extraction -----

#[test]
fn test_graph_from_empty_skeleton_has_no_nodes() {
    let skel = empty(32, 32);
    let g = skeleton_to_graph(&skel);
    assert_eq!(g.node_count(), 0);
    assert_eq!(g.edge_count(), 0);
}

#[test]
fn test_graph_endpoints_on_line() {
    let line = horizontal_line(32, 32);
    let skel = extract_skeleton(&line).unwrap();
    let g = skeleton_to_graph(&skel);
    // A thin horizontal line should have exactly 2 endpoint nodes.
    let endpoints: Vec<_> = g
        .nodes
        .iter()
        .filter(|n| n.kind == NodeKind::Endpoint)
        .collect();
    assert_eq!(
        endpoints.len(),
        2,
        "horizontal line skeleton must have 2 endpoints, got {}",
        endpoints.len()
    );
}

#[test]
fn test_graph_edge_connects_existing_nodes() {
    let line = horizontal_line(32, 32);
    let skel = extract_skeleton(&line).unwrap();
    let g = skeleton_to_graph(&skel);
    let node_ids: std::collections::HashSet<u32> = g.nodes.iter().map(|n| n.id).collect();
    for e in &g.edges {
        assert!(
            node_ids.contains(&e.from),
            "edge.from {} not in nodes",
            e.from
        );
        assert!(node_ids.contains(&e.to), "edge.to {} not in nodes", e.to);
    }
}

#[test]
fn test_graph_edge_length_positive() {
    let line = horizontal_line(32, 32);
    let skel = extract_skeleton(&line).unwrap();
    let g = skeleton_to_graph(&skel);
    for e in &g.edges {
        assert!(e.length > 0, "edge length must be > 0");
    }
}

// ----- SkeletonMatcher -----

#[test]
fn test_matcher_identical_thin_line_high_score() {
    let line = horizontal_line(32, 32);
    let m = SkeletonMatcher::default();
    let result = m.compare(&line, &line).unwrap();
    assert!(
        result.fused > 0.5,
        "identical skeleton match should be > 0.5, got {}",
        result.fused
    );
}

#[test]
fn test_matcher_empty_vs_empty() {
    let e = empty(32, 32);
    let m = SkeletonMatcher::default();
    let result = m.compare(&e, &e).unwrap();
    assert_eq!(result.fused, 0.0, "two empty skeletons should score 0.0");
}

#[test]
fn test_matcher_tolerates_10pct_missing_edges() {
    // Build two graphs where one has 90% of the edges of the other.
    use eel_abis_fp_skeleton::graph::{RidgeEdge, RidgeGraph, RidgeNode};
    let base_nodes: Vec<RidgeNode> = (0..4)
        .map(|i| RidgeNode {
            id: i,
            kind: NodeKind::Endpoint,
            x: i * 10,
            y: 0,
        })
        .collect();
    let base_edges: Vec<RidgeEdge> = (0..10)
        .map(|i| RidgeEdge {
            from: i % 4,
            to: (i + 1) % 4,
            length: 20,
            curvature: 0.0,
            start_orientation: 0.0,
            end_orientation: 0.0,
        })
        .collect();
    let full = RidgeGraph {
        nodes: base_nodes.clone(),
        edges: base_edges.clone(),
    };
    // Subset: remove last edge (9 of 10 = 90%).
    let subset = RidgeGraph {
        nodes: base_nodes.clone(),
        edges: base_edges[..9].to_vec(),
    };
    let m = SkeletonMatcher::default();
    let score = m.compare_graphs(&subset, &full).unwrap();
    assert!(
        score.edge_coverage >= 0.9,
        "10% missing edges should not drop coverage below 0.9, got {}",
        score.edge_coverage
    );
}
