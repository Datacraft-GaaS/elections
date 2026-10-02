// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Ridge skeleton graph extraction.
//!
//! After Zhang-Suen thinning, the skeleton is a set of pixel-thin curves.
//! This module converts the skeleton into a topological graph:
//!   - Nodes = ridge endpoints (1 skeleton neighbour) + bifurcations (3+ neighbours).
//!   - Edges = ridge segments connecting adjacent nodes, attributed by:
//!     - length (pixels)
//!     - mean curvature (radians per pixel)
//!     - start/end orientation (radians)

use std::collections::{HashMap, HashSet, VecDeque};

use crate::skeleton::RidgeSkeleton;

/// Kind of a graph node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum NodeKind {
    /// Ridge endpoint: exactly 1 skeletal neighbour.
    Endpoint,
    /// Ridge bifurcation: 3 or more skeletal neighbours.
    Bifurcation,
}

/// A node in the ridge graph.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RidgeNode {
    pub id: u32,
    pub kind: NodeKind,
    pub x: u32,
    pub y: u32,
}

/// An edge connecting two nodes.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RidgeEdge {
    pub from: u32,
    pub to: u32,
    /// Number of pixels in the ridge segment.
    pub length: u32,
    /// Mean absolute curvature (radians per pixel), 0 = straight ridge.
    pub curvature: f32,
    /// Ridge orientation at the start node (radians in [0, PI)).
    pub start_orientation: f32,
    /// Ridge orientation at the end node (radians in [0, PI)).
    pub end_orientation: f32,
}

/// Topological graph of ridge skeletons.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RidgeGraph {
    pub nodes: Vec<RidgeNode>,
    pub edges: Vec<RidgeEdge>,
}

impl RidgeGraph {
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }
}

/// Convert a thinned skeleton to a ridge graph.
pub fn skeleton_to_graph(skel: &RidgeSkeleton) -> RidgeGraph {
    let w = skel.width as i32;
    let h = skel.height as i32;

    // Find all node pixels (endpoint or bifurcation).
    let mut node_pixels: HashMap<(i32, i32), u32> = HashMap::new();
    let mut nodes: Vec<RidgeNode> = Vec::new();

    for y in 0..h {
        for x in 0..w {
            if skel.get(x, y) == 0 {
                continue;
            }
            let n = skeleton_neighbour_count(skel, x, y);
            let kind = if n == 1 {
                Some(NodeKind::Endpoint)
            } else if n >= 3 {
                Some(NodeKind::Bifurcation)
            } else {
                None
            };
            if let Some(k) = kind {
                let id = nodes.len() as u32;
                nodes.push(RidgeNode {
                    id,
                    kind: k,
                    x: x as u32,
                    y: y as u32,
                });
                node_pixels.insert((x, y), id);
            }
        }
    }

    // Trace edges: BFS from each node pixel along ridge pixels until next node.
    let mut visited_edges: HashSet<((i32, i32), (i32, i32))> = HashSet::new();
    let mut edges: Vec<RidgeEdge> = Vec::new();

    for (&(sx, sy), &from_id) in &node_pixels {
        for (dx, dy) in neighbour_deltas() {
            let nx = sx + dx;
            let ny = sy + dy;
            if skel.get(nx, ny) == 0 {
                continue;
            }
            let key = canonical_edge((sx, sy), (nx, ny));
            if visited_edges.contains(&key) {
                continue;
            }
            // Trace the ridge from (sx,sy) in direction (nx,ny).
            if let Some(edge) = trace_edge(skel, &node_pixels, from_id, sx, sy, nx, ny) {
                let end_key = canonical_edge((sx, sy), (edge.to as i32, edge.to as i32));
                visited_edges.insert(key);
                // Mark reverse direction too.
                visited_edges.insert(canonical_edge((nx, ny), (sx, sy)));
                let _ = end_key;
                edges.push(edge);
            }
        }
    }

    RidgeGraph { nodes, edges }
}

/// Trace a single ridge segment from start pixel (sx,sy) in direction (nx,ny) until
/// hitting another node pixel or dead end.
fn trace_edge(
    skel: &RidgeSkeleton,
    nodes: &HashMap<(i32, i32), u32>,
    from_id: u32,
    sx: i32,
    sy: i32,
    nx: i32,
    ny: i32,
) -> Option<RidgeEdge> {
    let mut path: Vec<(i32, i32)> = vec![(sx, sy), (nx, ny)];
    let mut prev = (sx, sy);
    let mut cur = (nx, ny);

    loop {
        // If we've reached a node pixel (other than the start), stop.
        if nodes.contains_key(&cur) && cur != (sx, sy) {
            let to_id = nodes[&cur];
            return Some(edge_from_path(from_id, to_id, &path));
        }
        // Advance to next skeleton pixel that isn't the previous one.
        let next = next_pixel(skel, cur, prev);
        match next {
            Some(n) => {
                path.push(n);
                prev = cur;
                cur = n;
            }
            None => {
                // Dead end without reaching another node -- skip.
                return None;
            }
        }
        // Safety: avoid infinitely long paths.
        if path.len() > 10_000 {
            return None;
        }
    }
}

/// Return the next skeleton pixel along the ridge (excluding the pixel we came from).
fn next_pixel(skel: &RidgeSkeleton, cur: (i32, i32), prev: (i32, i32)) -> Option<(i32, i32)> {
    for (dx, dy) in neighbour_deltas() {
        let n = (cur.0 + dx, cur.1 + dy);
        if n == prev {
            continue;
        }
        if skel.get(n.0, n.1) == 1 {
            return Some(n);
        }
    }
    None
}

/// Build a RidgeEdge from the traced pixel path.
fn edge_from_path(from: u32, to: u32, path: &[(i32, i32)]) -> RidgeEdge {
    let length = (path.len() as u32).saturating_sub(1).max(1);

    // Orientation at start: angle from pixel 0 to pixel 1.
    let start_orientation = if path.len() >= 2 {
        pixel_angle(path[0], path[1])
    } else {
        0.0
    };
    // Orientation at end.
    let end_orientation = if path.len() >= 2 {
        pixel_angle(path[path.len() - 2], path[path.len() - 1])
    } else {
        0.0
    };

    // Curvature: mean turning angle per step.
    let curvature = if path.len() >= 3 {
        let total: f32 = path
            .windows(3)
            .map(|w| {
                let a1 = pixel_angle(w[0], w[1]);
                let a2 = pixel_angle(w[1], w[2]);
                angle_diff_abs(a1, a2)
            })
            .sum();
        total / (path.len() - 2) as f32
    } else {
        0.0
    };

    RidgeEdge {
        from,
        to,
        length,
        curvature,
        start_orientation,
        end_orientation,
    }
}

/// Angle from p1 to p2 in [0, PI) (orientation, not direction).
fn pixel_angle(p1: (i32, i32), p2: (i32, i32)) -> f32 {
    let dx = (p2.0 - p1.0) as f32;
    let dy = (p2.1 - p1.1) as f32;
    let a = dy.atan2(dx);
    // Map to [0, PI).
    let a = if a < 0.0 { a + std::f32::consts::PI } else { a };
    a % std::f32::consts::PI
}

fn angle_diff_abs(a: f32, b: f32) -> f32 {
    let mut d = (a - b).abs();
    if d > std::f32::consts::PI / 2.0 {
        d = std::f32::consts::PI - d;
    }
    d
}

/// Count skeleton neighbours of pixel (x,y).
fn skeleton_neighbour_count(skel: &RidgeSkeleton, x: i32, y: i32) -> u8 {
    neighbour_deltas()
        .iter()
        .map(|&(dx, dy)| skel.get(x + dx, y + dy))
        .sum()
}

/// 8-connectivity neighbour offsets.
fn neighbour_deltas() -> [(i32, i32); 8] {
    [
        (-1, -1),
        (0, -1),
        (1, -1),
        (1, 0),
        (1, 1),
        (0, 1),
        (-1, 1),
        (-1, 0),
    ]
}

/// Canonical (ordered) edge key to avoid duplicates.
fn canonical_edge(a: (i32, i32), b: (i32, i32)) -> ((i32, i32), (i32, i32)) {
    if a <= b {
        (a, b)
    } else {
        (b, a)
    }
}

/// BFS-based connected component for skeleton paths.
pub fn connected_components(skel: &RidgeSkeleton) -> Vec<Vec<(u32, u32)>> {
    let w = skel.width as i32;
    let h = skel.height as i32;
    let mut visited = vec![false; (w * h) as usize];
    let mut components: Vec<Vec<(u32, u32)>> = Vec::new();

    for y in 0..h {
        for x in 0..w {
            let idx = (y * w + x) as usize;
            if skel.get(x, y) == 0 || visited[idx] {
                continue;
            }
            let mut comp: Vec<(u32, u32)> = Vec::new();
            let mut queue: VecDeque<(i32, i32)> = VecDeque::new();
            queue.push_back((x, y));
            visited[idx] = true;
            while let Some((cx, cy)) = queue.pop_front() {
                comp.push((cx as u32, cy as u32));
                for (dx, dy) in neighbour_deltas() {
                    let nx = cx + dx;
                    let ny = cy + dy;
                    if nx < 0 || ny < 0 || nx >= w || ny >= h {
                        continue;
                    }
                    let nidx = (ny * w + nx) as usize;
                    if skel.get(nx, ny) == 1 && !visited[nidx] {
                        visited[nidx] = true;
                        queue.push_back((nx, ny));
                    }
                }
            }
            components.push(comp);
        }
    }

    components
}
