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

use abis_standards_icao::{
    bac::{derive_key, pace_derive_key, KeyMode},
    dg1::{check_digit, MachineReadableZone},
    dg14::SecurityOptions,
    dg15::ActiveAuthPublicKey,
    dg2::{FaceImageData, ImageFormat},
    dg3::MultipleFingerprintData,
    dg4::{IrisData, IrisEye},
    sod::DocumentSecurityObject,
    IcaoError,
};
use sha2::{Digest, Sha256};

// TD3 MRZ test vector: P<UTOERIKSSON<<ANNA<MARIA<<<<<<<<<<<<<<<<<<<<<
//                       L898902C36UTO7408122F1204159ZE184226B<<<<<<1
// 88 chars total
fn sample_mrz() -> [u8; 88] {
    let line1 = b"P<UTOERIKSSON<<ANNA<MARIA<<<<<<<<<<<<<<<<<<<";
    let line2 = b"L898902C36UTO7408122F1204159ZE184226B<<<<<<1";
    assert_eq!(line1.len(), 44);
    assert_eq!(line2.len(), 44);
    let mut raw = [0u8; 88];
    raw[..44].copy_from_slice(line1);
    raw[44..].copy_from_slice(line2);
    raw
}

#[test]
fn mrz_parse_valid() {
    let raw = sample_mrz();
    let mrz = MachineReadableZone::parse(&raw).expect("parse failed");
    assert_eq!(mrz.raw, raw);
}

#[test]
fn mrz_surname() {
    let raw = sample_mrz();
    let mrz = MachineReadableZone::parse(&raw).unwrap();
    assert_eq!(mrz.surname().unwrap(), "ERIKSSON");
}

#[test]
fn mrz_given_names() {
    let raw = sample_mrz();
    let mrz = MachineReadableZone::parse(&raw).unwrap();
    assert_eq!(mrz.given_names().unwrap(), "ANNA<MARIA");
}

#[test]
fn mrz_document_number() {
    let raw = sample_mrz();
    let mrz = MachineReadableZone::parse(&raw).unwrap();
    assert_eq!(mrz.document_number().unwrap(), "L898902C3");
}

#[test]
fn mrz_birth_date() {
    let raw = sample_mrz();
    let mrz = MachineReadableZone::parse(&raw).unwrap();
    assert_eq!(mrz.birth_date().unwrap(), "740812");
}

#[test]
fn mrz_expiry_date() {
    let raw = sample_mrz();
    let mrz = MachineReadableZone::parse(&raw).unwrap();
    assert_eq!(mrz.expiry_date().unwrap(), "120415");
}

#[test]
fn mrz_too_short_returns_error() {
    let result = MachineReadableZone::parse(b"TOOSHORT");
    assert!(matches!(result, Err(IcaoError::InvalidMrz(_))));
}

#[test]
fn check_digit_document_number() {
    // ICAO 9303 example: "L898902C3" -> check digit 6
    assert_eq!(check_digit(b"L898902C3"), 6);
}

#[test]
fn check_digit_birth_date() {
    // "740812" -> check digit 2
    assert_eq!(check_digit(b"740812"), 2);
}

#[test]
fn dg2_parse_jpeg() {
    let mut data = vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10];
    data.extend_from_slice(b"JFIF\0");
    let face = FaceImageData::parse(&data).unwrap();
    assert_eq!(face.image_format, ImageFormat::Jpeg);
}

#[test]
fn dg2_parse_jpeg2000() {
    // JPEG 2000 codestream magic
    let data = vec![0xFF, 0x4F, 0xFF, 0x51];
    let face = FaceImageData::parse(&data).unwrap();
    assert_eq!(face.image_format, ImageFormat::Jpeg2000);
}

#[test]
fn dg3_parse_fingerprint_position() {
    // position=2 (right index), followed by dummy image bytes
    let data = vec![0x02, 0xAB, 0xCD];
    let mfp = MultipleFingerprintData::parse(&data).unwrap();
    assert_eq!(mfp.records.len(), 1);
    assert_eq!(mfp.records[0].position, 2);
}

#[test]
fn dg4_parse_iris_left() {
    // 0x02 = Left
    let data = vec![0x02, 0xDE, 0xAD];
    let iris = IrisData::parse(&data).unwrap();
    assert_eq!(iris.eye, IrisEye::Left);
}

#[test]
fn sod_passive_auth_valid() {
    let dg1_data = b"some DG1 content";
    let hash: Vec<u8> = Sha256::digest(dg1_data).to_vec();

    let sod = DocumentSecurityObject {
        hash_algorithm: "SHA-256".into(),
        data_group_hashes: vec![(1, hash)],
        signed_attrs: vec![],
    };
    assert!(sod.verify_passive_auth(1, dg1_data).unwrap());
}

#[test]
fn sod_passive_auth_wrong_data() {
    let dg1_data = b"correct DG1";
    let hash: Vec<u8> = Sha256::digest(dg1_data).to_vec();

    let sod = DocumentSecurityObject {
        hash_algorithm: "SHA-256".into(),
        data_group_hashes: vec![(1, hash)],
        signed_attrs: vec![],
    };
    assert!(!sod.verify_passive_auth(1, b"tampered DG1").unwrap());
}

#[test]
fn sod_round_trip() {
    let hash: Vec<u8> = Sha256::digest(b"dg1").to_vec();
    let original = DocumentSecurityObject {
        hash_algorithm: "SHA-256".into(),
        data_group_hashes: vec![(1, hash.clone()), (2, hash.clone())],
        signed_attrs: vec![0xAB, 0xCD],
    };
    let encoded = original.encode();
    let parsed = DocumentSecurityObject::parse(&encoded).unwrap();
    assert_eq!(parsed.hash_algorithm, "SHA-256");
    assert_eq!(parsed.data_group_hashes.len(), 2);
}

#[test]
fn bac_key_derivation_returns_16_bytes() {
    let mrz_info = b"L898902C<369UTO7408122F1204159";
    let key = derive_key(mrz_info, KeyMode::Encryption).unwrap();
    assert_eq!(key.len(), 16);
}

#[test]
fn bac_key_derivation_enc_mac_differ() {
    let mrz_info = b"L898902C<369UTO7408122F1204159";
    let enc = derive_key(mrz_info, KeyMode::Encryption).unwrap();
    let mac = derive_key(mrz_info, KeyMode::MacComputation).unwrap();
    assert_ne!(enc, mac);
}

#[test]
fn dg14_parse_empty() {
    let opts = SecurityOptions::parse(&[]).unwrap();
    assert!(opts.supported_protocols.is_empty());
}

#[test]
fn dg15_parse_minimal_pubkey() {
    let oid = b"1.2.840.113549.1.1.1";
    let mut data = vec![oid.len() as u8];
    data.extend_from_slice(oid);
    data.extend_from_slice(&[0x30, 0x0D]);
    let key = ActiveAuthPublicKey::parse(&data).unwrap();
    assert_eq!(key.algorithm, "1.2.840.113549.1.1.1");
    assert_eq!(key.key_data, &[0x30, 0x0D]);
}

#[test]
fn pace_derive_key_stub_returns_zeroes() {
    let key = pace_derive_key("L898902C3", "740812", "120415").unwrap();
    assert_eq!(key, vec![0u8; 16]);
}
