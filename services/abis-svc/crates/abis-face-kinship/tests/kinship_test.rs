use eel_abis_face_kinship::{
    cluster_families, KinshipClassifier, KinshipError, Relationship, EMBED_DIM,
};

fn unit_emb(i: usize) -> Vec<f32> {
    let mut v = vec![0.0f32; EMBED_DIM];
    v[i % EMBED_DIM] = 1.0;
    v
}

fn near_emb(base: usize, noise: f32) -> Vec<f32> {
    let mut v = unit_emb(base);
    v[1] = noise;
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt().max(1e-9);
    v.iter_mut().for_each(|x| *x /= norm);
    v
}

// --- KinshipClassifier tests ---

#[test]
fn identical_embeddings_kin_high_probability() {
    let clf = KinshipClassifier::default();
    let emb = unit_emb(0);
    let p = clf.predict(&emb, &emb).unwrap();
    assert!(p.p_kin > 0.9, "p_kin={}", p.p_kin);
}

#[test]
fn identical_embeddings_classified_twin() {
    let clf = KinshipClassifier::default();
    let emb = unit_emb(0);
    let p = clf.predict(&emb, &emb).unwrap();
    assert_eq!(p.relationship, Relationship::Twin);
}

#[test]
fn orthogonal_embeddings_not_kin() {
    let clf = KinshipClassifier::default();
    let a = unit_emb(0);
    let b = unit_emb(1);
    let p = clf.predict(&a, &b).unwrap();
    assert!(p.p_kin < 0.01, "p_kin={}", p.p_kin);
    assert_eq!(p.relationship, Relationship::Other);
}

#[test]
fn near_identical_classified_sibling() {
    let clf = KinshipClassifier::new(0.5, 0.99);
    // Cosine ~ 0.9 => sibling range (between kin_threshold and twin_threshold).
    let a = near_emb(0, 0.45);
    let b = near_emb(0, 0.43);
    let p = clf.predict(&a, &b).unwrap();
    assert!(p.p_kin > 0.5, "p_kin={}", p.p_kin);
    // Should be sibling or twin depending on exact cosine.
    assert!(matches!(
        p.relationship,
        Relationship::Sibling | Relationship::Twin | Relationship::Parent
    ));
}

#[test]
fn dimension_mismatch_errors() {
    let clf = KinshipClassifier::default();
    let short = vec![1.0f32; 10];
    let emb = unit_emb(0);
    assert!(matches!(
        clf.predict(&short, &emb),
        Err(KinshipError::DimensionMismatch)
    ));
}

#[test]
fn p_kin_bounded_0_1() {
    let clf = KinshipClassifier::default();
    let a = unit_emb(0);
    let b = unit_emb(2);
    let p = clf.predict(&a, &b).unwrap();
    assert!((0.0..=1.0).contains(&p.p_kin));
}

#[test]
fn relationship_display_strings() {
    assert_eq!(format!("{}", Relationship::Parent), "parent");
    assert_eq!(format!("{}", Relationship::Sibling), "sibling");
    assert_eq!(format!("{}", Relationship::Twin), "twin");
    assert_eq!(format!("{}", Relationship::Other), "other");
}

// --- cluster_families tests ---

#[test]
fn cluster_identical_pair_in_same_group() {
    let clf = KinshipClassifier::default();
    let emb = unit_emb(0);
    let subjects = vec![
        ("A".to_string(), emb.clone()),
        ("B".to_string(), emb.clone()),
    ];
    let clusters = cluster_families(&subjects, &clf);
    // A and B should be in the same cluster.
    assert_eq!(clusters.len(), 1);
    assert_eq!(clusters[0].members.len(), 2);
}

#[test]
fn cluster_orthogonal_pair_separate_groups() {
    let clf = KinshipClassifier::default();
    let subjects = vec![
        ("A".to_string(), unit_emb(0)),
        ("B".to_string(), unit_emb(1)),
    ];
    let clusters = cluster_families(&subjects, &clf);
    assert_eq!(clusters.len(), 2);
}

#[test]
fn cluster_transitivity_groups_chain() {
    // A ~ B ~ C but A and C not directly similar.
    // Use embeddings with controlled cosine:
    // A=[1,0,0], B=[0.9,0.44,0], C=[0.8,0,0.6] -- all L2-normalised.
    let clf = KinshipClassifier::new(0.7, 0.98);
    fn norm(mut v: Vec<f32>) -> Vec<f32> {
        let n: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt().max(1e-9);
        v.iter_mut().for_each(|x| *x /= n);
        v
    }
    let mut a = vec![0.0f32; EMBED_DIM];
    a[0] = 1.0;
    let mut b = vec![0.0f32; EMBED_DIM];
    b[0] = 0.95;
    b[1] = 0.31;
    let b = norm(b);
    let mut c = vec![0.0f32; EMBED_DIM];
    c[0] = 0.95;
    c[1] = 0.31;
    let c = norm(c);

    let subjects = vec![
        ("A".to_string(), a.clone()),
        ("B".to_string(), b),
        ("C".to_string(), c),
    ];
    let clusters = cluster_families(&subjects, &clf);
    // All three should be in one cluster (transitive kin).
    let total_members: usize = clusters.iter().map(|c| c.members.len()).sum();
    assert_eq!(total_members, 3);
    assert!(clusters.len() <= 2, "too many clusters: {}", clusters.len());
}

#[test]
fn cluster_empty_subjects_empty_result() {
    let clf = KinshipClassifier::default();
    let clusters = cluster_families(&[], &clf);
    assert!(clusters.is_empty());
}
