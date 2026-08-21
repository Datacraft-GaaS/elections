//! Benchmarks for the ABIS matching engine.
//!
//! Run with: cargo bench -p abis-matcher --features bench
//!
//! Targets:
//!   - 1:N face ANN over 100k templates: p99 < 500 ms
//!   - 1:1 fingerprint match: < 5 ms

#[cfg(feature = "bench")]
mod benches {
    use criterion::{black_box, criterion_group, criterion_main, Criterion};

    use abis_matcher::index::{HnswIndex, TemplateIndex};
    use abis_matcher::matcher::{FingerprintMatcher, Matcher};
    use abis_matcher::modality::fingerprint::{FingerPosition, Minutia, MinutiaeType};
    use abis_matcher::modality::FingerprintTemplate;

    /// Pseudo-random f32 seeded from index for reproducible benches.
    fn pseudo_rand(seed: u64, dim: usize) -> Vec<f32> {
        let mut v = Vec::with_capacity(dim);
        let mut s = seed;
        for _ in 0..dim {
            s = s
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            v.push((s >> 33) as f32 / u32::MAX as f32 * 2.0 - 1.0);
        }
        // L2-normalise.
        let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        v.iter_mut().for_each(|x| *x /= norm.max(1e-9));
        v
    }

    fn bench_face_1n(c: &mut Criterion) {
        const N: usize = 100_000;
        const DIM: usize = 512;

        let mut idx = HnswIndex::new(DIM);
        for i in 0..N {
            let emb = pseudo_rand(i as u64, DIM);
            idx.add(&format!("subject-{i}"), &emb).unwrap();
        }
        let probe = pseudo_rand(N as u64 + 1, DIM);

        c.bench_function("face_1n_100k", |b| {
            b.iter(|| {
                let results = idx.search(black_box(&probe), 10).unwrap();
                black_box(results);
            });
        });
    }

    fn bench_fingerprint_11(c: &mut Criterion) {
        fn make_template(seed: u16) -> FingerprintTemplate {
            let minutiae: Vec<Minutia> = (0..40u16)
                .map(|i| Minutia {
                    x: ((seed + i * 12) % 400) + 50,
                    y: ((seed + i * 17) % 400) + 50,
                    theta: ((i * 9) % 360) as f32,
                    mtype: if i % 2 == 0 {
                        MinutiaeType::Ending
                    } else {
                        MinutiaeType::Bifurcation
                    },
                    quality: 80,
                })
                .collect();
            FingerprintTemplate::from_minutiae(FingerPosition::RightIndex, minutiae)
        }

        let probe = make_template(100);
        let gallery = make_template(105); // Nearby template (genuine-like).
        let matcher = FingerprintMatcher::default();

        c.bench_function("fingerprint_11", |b| {
            b.iter(|| {
                let score = matcher.compare(black_box(&probe), black_box(&gallery));
                black_box(score);
            });
        });
    }

    criterion_group!(benches, bench_face_1n, bench_fingerprint_11);
    criterion_main!(benches);
}

#[cfg(not(feature = "bench"))]
fn main() {
    eprintln!("Run with --features bench");
}
