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

//! Typed models for `ussd/gateway.yaml`, `ivr/openapi.yaml`, `whatsapp/openapi.yaml`.

use serde::{Deserialize, Serialize};

/// Mirrors `USSDSessionRequest`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UssdSessionRequest {
    pub session_id: String,
    pub msisdn_hash: String,
    pub text: String,
    pub service_code: String,
    pub network_code: String,
}

/// Mirrors `USSDSessionResponse`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UssdSessionResponse {
    pub response_text: String,
    pub action: String,
}

/// Mirrors `IvrVoiceRequest` (simplified).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IvrVoiceRequest {
    pub call_id: String,
    pub caller_id_hash: String,
    pub dtmf: Option<String>,
}

/// Mirrors WhatsApp inbound payload (simplified).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsappMessage {
    pub from: String,
    pub body: String,
    pub message_id: Option<String>,
}
