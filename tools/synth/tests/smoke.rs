// Smoke tests: run end-to-end pipeline and verify exit conditions.

use eel_synth::{geo, kits, personnel, verify, voters, ScaleProfile};

#[test]
fn smoke_verify_seed42_exits_ok() {
    // verify::run handles first-run (no checkpoint) by writing and returning Ok.
    // On subsequent calls it compares. Either way exit is Ok for matching seeds.
    let profile = ScaleProfile::SMOKE;
    let result = verify::run(42, &profile);
    assert!(
        result.is_ok(),
        "verify::run(42, SMOKE) must return Ok; got: {:?}",
        result.err()
    );
}

#[test]
fn smoke_full_pipeline_smoke_profile() {
    let seed = 77u64;
    let profile = ScaleProfile::SMOKE;

    let geo_data = geo::generate(seed, profile.stations).expect("geo::generate");
    assert!(!geo_data.stations.is_empty());
    assert_eq!(geo_data.constituencies.len(), 290);

    let voter_records =
        voters::generate(seed, profile.voters, &geo_data.stations).expect("voters::generate");
    assert_eq!(voter_records.len() as u64, profile.voters);

    let kit_list = kits::generate(seed, profile.kits, &geo_data.stations).expect("kits::generate");
    assert_eq!(kit_list.len() as u64, profile.kits);

    let p = personnel::generate(seed, profile.personnel, &geo_data.stations)
        .expect("personnel::generate");
    assert_eq!(p.len() as u64, profile.personnel);
}

#[test]
fn smoke_national_ids_no_collision_with_real_range() {
    const RESERVED_BASE: u64 = 999_000_000;
    let geo_data = geo::generate(42, 10).unwrap();
    let records = voters::generate(42, 50, &geo_data.stations).unwrap();
    for r in &records {
        let id: u64 = r.national_id.parse().unwrap();
        assert!(
            id >= RESERVED_BASE,
            "national ID {} falls below reserved range {}",
            id,
            RESERVED_BASE
        );
    }
}

#[test]
fn smoke_csv_output_is_non_empty() {
    let geo_data = geo::generate(42, 10).unwrap();
    let records = voters::generate(42, 5, &geo_data.stations).unwrap();
    let mut buf = Vec::new();
    eel_synth::voters::write_csv(&records, &mut buf).unwrap();
    assert!(!buf.is_empty(), "CSV output must not be empty");
    // Must include header row.
    let csv_str = String::from_utf8(buf).unwrap();
    assert!(
        csv_str.contains("voter_id"),
        "CSV must have voter_id column header"
    );
}

#[test]
fn smoke_geo_geojson_is_valid_json() {
    let geo_data = geo::generate(42, 20).unwrap();
    let json = eel_synth::geo::to_geojson(&geo_data.stations);
    let parsed: serde_json::Value =
        serde_json::from_str(&json).expect("GeoJSON must be valid JSON");
    assert_eq!(parsed["type"], "FeatureCollection");
}
