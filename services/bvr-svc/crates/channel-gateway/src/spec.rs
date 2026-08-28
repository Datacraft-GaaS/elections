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
