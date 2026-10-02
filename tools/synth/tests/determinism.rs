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

// Property tests: same seed produces identical output across two independent runs.
// Uses proptest for seed-space exploration.

use eel_synth::{geo, kits, personnel, verify, voters, ScaleProfile};
use proptest::prelude::*;

// Shared station fixture for property tests.
fn make_stations(seed: u64, n: u64) -> Vec<eel_synth::geo::PollingStation> {
    geo::generate(seed, n.max(5)).unwrap().stations
}

proptest! {
    #[test]
    fn voters_deterministic(seed in 0u64..10_000u64) {
        let stations = make_stations(seed, 10);
        let a = voters::generate(seed, 20, &stations).unwrap();
        let b = voters::generate(seed, 20, &stations).unwrap();
        prop_assert_eq!(a.len(), b.len());
        for (av, bv) in a.iter().zip(b.iter()) {
            prop_assert_eq!(&av.voter_id, &bv.voter_id);
            prop_assert_eq!(&av.national_id, &bv.national_id);
            prop_assert_eq!(&av.first_name, &bv.first_name);
            prop_assert_eq!(&av.last_name, &bv.last_name);
            prop_assert_eq!(&av.dob, &bv.dob);
        }
    }

    #[test]
    fn kits_deterministic(seed in 0u64..10_000u64) {
        let stations = make_stations(seed, 10);
        let a = kits::generate(seed, 10, &stations).unwrap();
        let b = kits::generate(seed, 10, &stations).unwrap();
        prop_assert_eq!(a.len(), b.len());
        for (ak, bk) in a.iter().zip(b.iter()) {
            prop_assert_eq!(&ak.kit_id, &bk.kit_id);
            prop_assert_eq!(&ak.tpm_pubkey_hex, &bk.tpm_pubkey_hex);
        }
    }

    #[test]
    fn personnel_deterministic(seed in 0u64..10_000u64) {
        let stations = make_stations(seed, 10);
        let a = personnel::generate(seed, 10, &stations).unwrap();
        let b = personnel::generate(seed, 10, &stations).unwrap();
        prop_assert_eq!(a.len(), b.len());
        for (ap, bp) in a.iter().zip(b.iter()) {
            prop_assert_eq!(&ap.personnel_id, &bp.personnel_id);
            prop_assert_eq!(&ap.national_id, &bp.national_id);
        }
    }

    #[test]
    fn geo_deterministic(seed in 0u64..10_000u64) {
        let a = geo::generate(seed, 50).unwrap();
        let b = geo::generate(seed, 50).unwrap();
        prop_assert_eq!(a.stations.len(), b.stations.len());
        if !a.stations.is_empty() {
            let fa = &a.stations[0];
            let fb = &b.stations[0];
            prop_assert_eq!(&fa.station_id, &fb.station_id);
            prop_assert!((fa.lat - fb.lat).abs() < f64::EPSILON);
            prop_assert!((fa.lon - fb.lon).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn hash_deterministic(seed in 0u64..100u64) {
        let profile = ScaleProfile::SMOKE;
        let stations_a = make_stations(seed, profile.stations);
        let stations_b = make_stations(seed, profile.stations);
        let h1 = verify::compute_hash(seed, &profile, &stations_a).unwrap();
        let h2 = verify::compute_hash(seed, &profile, &stations_b).unwrap();
        prop_assert_eq!(h1, h2);
    }
}
