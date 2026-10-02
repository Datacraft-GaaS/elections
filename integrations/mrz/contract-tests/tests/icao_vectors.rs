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

//! ICAO 9303 test vectors for MRZ parsing.

use eel_adapter_mrz_rs::{check_digit, parse_td3};

#[test]
fn icao_check_digit_520727() {
    assert_eq!(check_digit("520727"), 3);
}

#[test]
fn icao_check_digit_l898902c3() {
    assert_eq!(check_digit("L898902C3"), 6);
}

#[test]
fn icao_td3_sample_passport() {
    let l1 = "P<UTOERIKSSON<<ANNA<MARIA<<<<<<<<<<<<<<<<<<<";
    let l2 = "L898902C36UTO7408122F1204159ZE184226B<<<<<10";
    let mrz = parse_td3(l1, l2).expect("ICAO sample TD3 must parse");
    assert_eq!(mrz.document_number, "L898902C3");
    assert!(mrz.name.contains("ERIKSSON"));
    assert_eq!(mrz.date_of_birth, "740812");
    assert_eq!(mrz.sex, "F");
}

#[test]
fn bad_checksum_returns_error() {
    let l1 = "P<UTOERIKSSON<<ANNA<MARIA<<<<<<<<<<<<<<<<<<<";
    let l2 = "L898902C99UTO7408122F1204159ZE184226B<<<<<10";
    assert!(parse_td3(l1, l2).is_err());
}
