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

use base64::{engine::general_purpose::STANDARD, Engine as _};
use chrono::Utc;
use eel_crypto_rs::ed25519::sign;
use uuid::Uuid;

use crate::{
    envelope::{DataClass, Envelope},
    error::EventError,
    spiffe::SpiffeKeyProvider,
};

/// Fluent builder for [`Envelope`].
///
/// Mandatory fields: `event_type`, `source`, `stream_id`, `hlc`.
/// Call [`build`](EventBuilder::build) to sign and produce a complete envelope,
/// or [`build_unsigned`](EventBuilder::build_unsigned) for testing.
#[derive(Default)]
pub struct EventBuilder {
    event_type: Option<String>,
    source: Option<String>,
    subject: Option<String>,
    dataschema: Option<String>,
    data: Option<serde_json::Value>,
    stream_id: Option<String>,
    seq: u64,
    hlc: Option<String>,
    prev_hash: Option<String>,
    correlation_id: Option<String>,
    causation_id: Option<String>,
    election_id: Option<String>,
    data_class: DataClass,
}

impl EventBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn event_type(mut self, t: impl Into<String>) -> Self {
        self.event_type = Some(t.into());
        self
    }

    pub fn source(mut self, s: impl Into<String>) -> Self {
        self.source = Some(s.into());
        self
    }

    pub fn subject(mut self, s: impl Into<String>) -> Self {
        self.subject = Some(s.into());
        self
    }

    pub fn dataschema(mut self, s: impl Into<String>) -> Self {
        self.dataschema = Some(s.into());
        self
    }

    pub fn data(mut self, d: serde_json::Value) -> Self {
        self.data = Some(d);
        self
    }

    pub fn stream_id(mut self, id: impl Into<String>) -> Self {
        self.stream_id = Some(id.into());
        self
    }

    pub fn seq(mut self, s: u64) -> Self {
        self.seq = s;
        self
    }

    pub fn hlc(mut self, h: impl Into<String>) -> Self {
        self.hlc = Some(h.into());
        self
    }

    pub fn prev_hash(mut self, h: impl Into<String>) -> Self {
        self.prev_hash = Some(h.into());
        self
    }

    pub fn correlation_id(mut self, id: impl Into<String>) -> Self {
        self.correlation_id = Some(id.into());
        self
    }

    pub fn causation_id(mut self, id: impl Into<String>) -> Self {
        self.causation_id = Some(id.into());
        self
    }

    pub fn election_id(mut self, id: impl Into<String>) -> Self {
        self.election_id = Some(id.into());
        self
    }

    pub fn data_class(mut self, dc: DataClass) -> Self {
        self.data_class = dc;
        self
    }

    /// Build an unsigned envelope. Useful in tests or when signing is done
    /// downstream.
    pub fn build_unsigned(self) -> Result<Envelope, EventError> {
        Ok(Envelope {
            specversion: "1.0".into(),
            id: Uuid::now_v7().to_string(),
            event_type: self.event_type.ok_or(EventError::MissingField("type"))?,
            source: self.source.ok_or(EventError::MissingField("source"))?,
            subject: self.subject,
            time: Utc::now(),
            datacontenttype: "application/json".into(),
            dataschema: self.dataschema,
            data: self.data.unwrap_or(serde_json::Value::Null),
            eel_stream_id: self
                .stream_id
                .ok_or(EventError::MissingField("stream_id"))?,
            eel_seq: self.seq,
            eel_hlc: self.hlc.ok_or(EventError::MissingField("hlc"))?,
            eel_prev_hash: self.prev_hash,
            eel_correlation_id: self.correlation_id,
            eel_causation_id: self.causation_id,
            eel_producer_key_id: None,
            eel_signature: None,
            eel_election_id: self.election_id,
            eel_data_class: self.data_class,
        })
    }

    /// Build a signed envelope using the provided SPIFFE key provider.
    ///
    /// The signature covers the RFC 8785 canonical JSON of the envelope
    /// with the `eel_signature` field removed.
    pub fn build(self, key_provider: &dyn SpiffeKeyProvider) -> Result<Envelope, EventError> {
        let mut envelope = self.build_unsigned()?;
        envelope.eel_producer_key_id = Some(key_provider.key_id().to_string());

        let canonical = envelope.canonical_bytes_for_signing()?;
        let sig = sign(key_provider.signing_key(), &canonical);
        envelope.eel_signature = Some(format!("ed25519:{}", STANDARD.encode(sig.to_bytes())));

        Ok(envelope)
    }
}
