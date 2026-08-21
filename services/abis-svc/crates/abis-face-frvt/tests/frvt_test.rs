use eel_abis_face_frvt::{
    FairnessSlice, FmrFnmr, FrvtCorpus, FrvtImage, FrvtImageSet, FrvtReport, QualityCorrelation,
    RankN, Sex, SkinTone,
};

fn unit_emb(dim: usize) -> Vec<f32> {
    let mut v = vec![0.0f32; 512];
    v[dim % 512] = 1.0;
    v
}

fn build_corpus() -> FrvtCorpus {
    let mut c = FrvtCorpus::new();
    // Subject A: 2 images with same embedding (genuine pair, should match)
    c.add(
        FrvtImage::new("g_A1", "SubjA", FrvtImageSet::Mugshot, unit_emb(0), 0.9).with_demographics(
            30,
            Sex::Male,
            SkinTone::III,
        ),
    );
    c.add(
        FrvtImage::new("g_A2", "SubjA", FrvtImageSet::Mugshot, unit_emb(0), 0.85)
            .with_demographics(30, Sex::Male, SkinTone::III),
    );
    // Subject B: 2 images with different embedding (genuine pair, B1 vs B2)
    c.add(
        FrvtImage::new("g_B1", "SubjB", FrvtImageSet::Mugshot, unit_emb(1), 0.8).with_demographics(
            25,
            Sex::Female,
            SkinTone::V,
        ),
    );
    c.add(
        FrvtImage::new("g_B2", "SubjB", FrvtImageSet::Mugshot, unit_emb(1), 0.75)
            .with_demographics(25, Sex::Female, SkinTone::V),
    );
    c
}

// --- FrvtCorpus tests ---

#[test]
fn corpus_genuine_pairs_count() {
    let c = build_corpus();
    // SubjA: 1 pair, SubjB: 1 pair => 2 genuine pairs
    assert_eq!(c.genuine_pairs().len(), 2);
}

#[test]
fn corpus_impostor_pairs_count() {
    let c = build_corpus();
    // 4 choose 2 = 6 total; 2 genuine => 4 impostor
    assert_eq!(c.impostor_pairs().len(), 4);
}

// --- FmrFnmr tests ---

#[test]
fn fmr_fnmr_perfect_threshold() {
    let c = build_corpus();
    // Threshold 0.5: A embeddings identical => sim=1.0 (accept).
    // B embeddings identical => sim=1.0 (accept).
    // Impostors: A vs B => cosine(e_0, e_1) = 0.0 (reject).
    let m = FmrFnmr::compute(&c, 0.5);
    assert!((m.fnmr - 0.0).abs() < 1e-5, "fnmr={}", m.fnmr);
    assert!((m.fmr - 0.0).abs() < 1e-5, "fmr={}", m.fmr);
}

#[test]
fn fmr_fnmr_very_high_threshold_rejects_all() {
    let c = build_corpus();
    let m = FmrFnmr::compute(&c, 1.1); // threshold above max cosine
    assert!((m.fnmr - 1.0).abs() < 1e-5, "fnmr={}", m.fnmr);
    assert!((m.fmr - 0.0).abs() < 1e-5, "fmr={}", m.fmr);
}

#[test]
fn fmr_fnmr_bounded_0_1() {
    let c = build_corpus();
    let m = FmrFnmr::compute(&c, 0.5);
    assert!((0.0..=1.0).contains(&m.fmr));
    assert!((0.0..=1.0).contains(&m.fnmr));
}

#[test]
fn fmr_fnmr_pair_counts_match_corpus() {
    let c = build_corpus();
    let m = FmrFnmr::compute(&c, 0.5);
    assert_eq!(m.genuine_count, 2);
    assert_eq!(m.impostor_count, 4);
}

// --- RankN tests ---

#[test]
fn rank1_with_gallery_probe_split() {
    let mut c = FrvtCorpus::new();
    c.add(FrvtImage::new(
        "gallery_A",
        "SubjA",
        FrvtImageSet::Mugshot,
        unit_emb(0),
        0.9,
    ));
    c.add(FrvtImage::new(
        "gallery_B",
        "SubjB",
        FrvtImageSet::Mugshot,
        unit_emb(1),
        0.9,
    ));
    c.add(FrvtImage::new(
        "probe_A",
        "SubjA",
        FrvtImageSet::Wild,
        unit_emb(0),
        0.8,
    ));
    let r = RankN::compute(&c, "gallery", "probe", 1);
    // probe_A matches gallery_A perfectly => rank-1 hit
    assert!((r.hit_rate - 1.0).abs() < 1e-4, "hit_rate={}", r.hit_rate);
}

#[test]
fn rank1_empty_probe_zero_hit_rate() {
    let c = build_corpus();
    let r = RankN::compute(&c, "gallery", "probe", 1);
    assert_eq!(r.probe_count, 0);
}

// --- QualityCorrelation tests ---

#[test]
fn quality_correlation_computed() {
    let c = build_corpus();
    let qc = QualityCorrelation::compute(&c, 0.5);
    // Just check it's in [-1, 1].
    assert!((-1.0..=1.0).contains(&qc.pearson_r));
}

#[test]
fn quality_correlation_empty_corpus() {
    let c = FrvtCorpus::new();
    let qc = QualityCorrelation::compute(&c, 0.5);
    assert!((qc.pearson_r).abs() < 1e-5);
}

// --- FairnessSlice tests ---

#[test]
fn fairness_slices_by_skin_tone() {
    let c = build_corpus();
    let slices = FairnessSlice::compute_by_skin_tone(&c, 0.5);
    // Should have slices for SkinTone::III and V.
    assert!(slices.len() >= 2, "slices={}", slices.len());
}

#[test]
fn fairness_slice_fmr_fnmr_bounded() {
    let c = build_corpus();
    let slices = FairnessSlice::compute_by_skin_tone(&c, 0.5);
    for s in &slices {
        assert!((0.0..=1.0).contains(&s.fmr), "fmr={}", s.fmr);
        assert!((0.0..=1.0).contains(&s.fnmr), "fnmr={}", s.fnmr);
    }
}

// --- FrvtReport tests ---

#[test]
fn frvt_report_summary_non_empty() {
    let c = build_corpus();
    let ver = FmrFnmr::compute(&c, 0.5);
    let rank = RankN::compute(&c, "gallery", "probe", 1);
    let qc = QualityCorrelation::compute(&c, 0.5);
    let fs = FairnessSlice::compute_by_skin_tone(&c, 0.5);
    let rep = FrvtReport::new(ver, rank, qc, fs);
    let s = rep.summary();
    assert!(!s.is_empty());
    assert!(s.contains("FRVT"));
}

#[test]
fn frvt_report_valid_verification_fields() {
    let c = build_corpus();
    let ver = FmrFnmr::compute(&c, 0.5);
    assert!((0.0..=1.0).contains(&ver.fmr));
    assert!((0.0..=1.0).contains(&ver.fnmr));
}

#[test]
fn image_set_variants_all_distinct() {
    assert_ne!(FrvtImageSet::Visa, FrvtImageSet::Mugshot);
    assert_ne!(FrvtImageSet::Wild, FrvtImageSet::Kiosk);
}
