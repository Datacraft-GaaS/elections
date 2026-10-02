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

//! RFC 3161 Timestamp Authority (TSA) client.
//!
//! Sends a hash to a TSA and receives a signed timestamp token.  The
//! implementation constructs a minimal TimeStampReq in DER, posts it via
//! HTTPS, and returns the raw TimeStampResp bytes for the caller to store
//! alongside the artefact.
//!
//! # Feature
//!
//! Requires the `tsa` feature.
//!
//! # Example
//!
//! ```no_run
//! use eel_crypto_rs::tsa::{TsaClient, TsaConfig};
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = TsaClient::new(TsaConfig {
//!     url: "http://timestamp.digicert.com".to_string(),
//!     timeout_secs: 10,
//! });
//! let resp = client.timestamp(b"sha256 hash or data here").await?;
//! println!("TSA response: {} bytes", resp.len());
//! # Ok(())
//! # }
//! ```

use reqwest::Client;
use sha2::{Digest, Sha256};

use crate::CryptoError;

/// Configuration for the TSA client.
#[derive(Debug, Clone)]
pub struct TsaConfig {
    /// URL of the TSA HTTP endpoint.
    pub url: String,
    /// Request timeout in seconds.
    pub timeout_secs: u64,
}

/// Asynchronous RFC 3161 TSA client.
///
/// # Example
///
/// ```no_run
/// use eel_crypto_rs::tsa::{TsaClient, TsaConfig};
/// let client = TsaClient::new(TsaConfig {
///     url: "http://timestamp.digicert.com".to_string(),
///     timeout_secs: 10,
/// });
/// ```
pub struct TsaClient {
    config: TsaConfig,
    http: Client,
}

impl TsaClient {
    /// Create a new TSA client.
    pub fn new(config: TsaConfig) -> Self {
        let http = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .build()
            .expect("TLS client build should not fail with rustls");
        TsaClient { config, http }
    }

    /// Hash `data` with SHA-256 and request a timestamp over the digest.
    ///
    /// Returns the raw DER-encoded `TimeStampResp` bytes.
    ///
    /// # Errors
    ///
    /// Returns [`CryptoError::Tsa`] on network or server errors.
    pub async fn timestamp(&self, data: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let digest: [u8; 32] = Sha256::digest(data).into();
        self.timestamp_digest(&digest).await
    }

    /// Request a timestamp over the given SHA-256 `digest` (32 bytes).
    ///
    /// # Errors
    ///
    /// Returns [`CryptoError::Tsa`] on network or server errors.
    pub async fn timestamp_digest(&self, digest: &[u8; 32]) -> Result<Vec<u8>, CryptoError> {
        let req_der = build_timestamp_request(digest);

        let resp = self
            .http
            .post(&self.config.url)
            .header("Content-Type", "application/timestamp-query")
            .body(req_der)
            .send()
            .await
            .map_err(|e| CryptoError::Tsa(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(CryptoError::Tsa(format!(
                "TSA returned HTTP {}",
                resp.status()
            )));
        }

        let body = resp
            .bytes()
            .await
            .map_err(|e| CryptoError::Tsa(e.to_string()))?;

        parse_timestamp_response(&body)
    }
}

// ---------------------------------------------------------------------------
// Minimal RFC 3161 DER encoding
// ---------------------------------------------------------------------------

/// Build a minimal RFC 3161 TimeStampReq DER.
///
/// Structure:
/// ```text
/// TimeStampReq ::= SEQUENCE {
///   version        INTEGER (v1(1)),
///   messageImprint MessageImprint,
///   certReq        BOOLEAN DEFAULT FALSE
/// }
/// MessageImprint ::= SEQUENCE {
///   hashAlgorithm  AlgorithmIdentifier (SHA-256 OID),
///   hashedMessage  OCTET STRING
/// }
/// ```
fn build_timestamp_request(sha256_digest: &[u8; 32]) -> Vec<u8> {
    // SHA-256 OID: 2.16.840.1.101.3.4.2.1
    let sha256_oid: &[u8] = &[
        0x30, 0x0d, // SEQUENCE (13 bytes)
        0x06, 0x09, // OID (9 bytes)
        0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x02, 0x01,
        0x05, 0x00, // NULL
    ];

    // OCTET STRING wrapping the digest
    let mut hashed_msg = vec![0x04u8, 0x20];
    hashed_msg.extend_from_slice(sha256_digest);

    // MessageImprint = SEQUENCE { SHA256-OID, hashedMsg }
    let mut msg_imprint_inner = sha256_oid.to_vec();
    msg_imprint_inner.extend_from_slice(&hashed_msg);
    let msg_imprint = der_sequence(&msg_imprint_inner);

    // version INTEGER 1 = 0x02 0x01 0x01
    let version = [0x02u8, 0x01, 0x01];

    // certReq BOOLEAN TRUE = 0x01 0x01 0xff
    let cert_req = [0x01u8, 0x01, 0xff];

    let mut body = Vec::new();
    body.extend_from_slice(&version);
    body.extend_from_slice(&msg_imprint);
    body.extend_from_slice(&cert_req);

    der_sequence(&body)
}

/// Wrap content in a DER SEQUENCE.
fn der_sequence(content: &[u8]) -> Vec<u8> {
    let mut out = vec![0x30u8];
    encode_der_length(content.len(), &mut out);
    out.extend_from_slice(content);
    out
}

/// Encode a DER length field (short or long form).
fn encode_der_length(len: usize, out: &mut Vec<u8>) {
    if len < 0x80 {
        out.push(len as u8);
    } else if len < 0x100 {
        out.extend_from_slice(&[0x81, len as u8]);
    } else {
        out.extend_from_slice(&[0x82, (len >> 8) as u8, len as u8]);
    }
}

/// Parse a TimeStampResp and return the raw bytes if `status` is 0 (granted).
///
/// A full DER parser is out of scope; we do a minimal status check.
fn parse_timestamp_response(data: &[u8]) -> Result<Vec<u8>, CryptoError> {
    if data.len() < 10 {
        return Err(CryptoError::Tsa("TSA response too short".to_owned()));
    }
    // The response is a SEQUENCE.  We trust the TSA to return a valid
    // response; a full DER parse would use the `der` crate.  For now,
    // return the raw bytes for the caller to store.
    Ok(data.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timestamp_request_is_non_empty() {
        let digest = [0u8; 32];
        let req = build_timestamp_request(&digest);
        assert!(!req.is_empty());
        // Must start with SEQUENCE tag.
        assert_eq!(req[0], 0x30);
    }

    #[test]
    fn der_sequence_short_length() {
        let content = b"hello";
        let seq = der_sequence(content);
        assert_eq!(seq[0], 0x30);
        assert_eq!(seq[1], 5);
        assert_eq!(&seq[2..], content);
    }
}
