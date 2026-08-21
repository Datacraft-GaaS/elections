//! Criterion benchmark for `geo_check::check`.
//!
//! DoD: p99 < 20 ms local.
//! Run with: cargo bench -p boundary-geofence

use boundary_domain::types::{Coordinate, Polygon};
use boundary_geofence::geo_check;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn make_polygon(n_vertices: usize) -> Polygon {
    let mut ring = Vec::with_capacity(n_vertices + 1);
    for i in 0..n_vertices {
        let angle = 2.0 * std::f64::consts::PI * i as f64 / n_vertices as f64;
        ring.push(Coordinate {
            longitude: 36.8 + 0.1 * angle.cos(),
            latitude: -1.25 + 0.05 * angle.sin(),
        });
    }
    ring.push(ring[0].clone()); // close ring
    Polygon { rings: vec![ring] }
}

fn bench_check(c: &mut Criterion) {
    let poly_100 = make_polygon(100);
    let poly_2000 = make_polygon(2000);

    let mut g = c.benchmark_group("geofence_check");
    g.bench_function("100_vertices_inside", |b| {
        b.iter(|| geo_check::check(black_box(&poly_100), black_box(-1.25), black_box(36.8)));
    });
    g.bench_function("2000_vertices_inside", |b| {
        b.iter(|| geo_check::check(black_box(&poly_2000), black_box(-1.25), black_box(36.8)));
    });
    g.bench_function("2000_vertices_outside", |b| {
        b.iter(|| geo_check::check(black_box(&poly_2000), black_box(-5.0), black_box(36.8)));
    });
    g.finish();
}

criterion_group!(benches, bench_check);
criterion_main!(benches);
