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

//! Email channel adapter using lettre async SMTP with STARTTLS.
//!
//! PGP encryption is optional and requires the `pgp` feature flag + the
//! `rpgp` crate. Enabling `pgp` without adding `rpgp` to Cargo.toml will
//! produce a compile-time error via the cfg_attr check below.

use chrono::Utc;
use lettre::{
    message::{header, SinglePart},
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use tracing::{info, warn};

use crate::domain::{DeliveryContext, DeliveryOutcome};

// Deliberate guard: enabling `pgp` without the rpgp dependency would silently
// ship an unencrypted channel. It fires whenever the feature is on, including
// under `cargo clippy --all-features`, so lint runs must not enable it.
#[cfg(feature = "pgp")]
compile_error!(
    "The `pgp` feature requires the `rpgp` crate to be added to Cargo.toml. \
	 Add `rpgp = { version = \"0.14\" }` under [dependencies] to enable PGP encryption."
);

/// Deliver a report artefact via SMTP STARTTLS.
///
/// Configuration via environment variables:
/// - EEL_SMTP_HOST  (default: localhost)
/// - EEL_SMTP_PORT  (default: 587)
/// - EEL_SMTP_USER
/// - EEL_SMTP_PASS
/// - EEL_SMTP_FROM  (default: reports@eel.internal)
pub async fn deliver(ctx: DeliveryContext) -> DeliveryOutcome {
    let smtp_host = std::env::var("EEL_SMTP_HOST").unwrap_or_else(|_| "localhost".into());
    let smtp_port: u16 = std::env::var("EEL_SMTP_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(587);
    let smtp_from =
        std::env::var("EEL_SMTP_FROM").unwrap_or_else(|_| "reports@eel.internal".into());

    let recipient = ctx.subscriber.endpoint.clone();

    let from_addr: lettre::Address = match smtp_from.parse() {
        Ok(a) => a,
        Err(e) => {
            return DeliveryOutcome::Permanent {
                error: format!("invalid from address '{smtp_from}': {e}"),
            }
        }
    };

    let to_addr: lettre::Address = match recipient.parse() {
        Ok(a) => a,
        Err(e) => {
            return DeliveryOutcome::Permanent {
                error: format!("invalid recipient address '{recipient}': {e}"),
            }
        }
    };

    let body = format!(
        "Report artefact: {}\nFormat: {}\nRun: {}\nSHA-256: {}\nMinIO ref: {}",
        ctx.artifact.artifact_id,
        ctx.artifact.format,
        ctx.delivery.run_id,
        hex::encode(&ctx.artifact.sha256),
        ctx.artifact.blob_ref,
    );

    let email = match Message::builder()
        .from(lettre::message::Mailbox::new(None, from_addr))
        .to(lettre::message::Mailbox::new(None, to_addr))
        .subject(format!(
            "[eel] Report: {} ({})",
            ctx.delivery.run_id, ctx.artifact.format
        ))
        .singlepart(
            SinglePart::builder()
                .header(header::ContentType::TEXT_PLAIN)
                .body(body),
        ) {
        Ok(m) => m,
        Err(e) => {
            return DeliveryOutcome::Permanent {
                error: format!("failed to build email message: {e}"),
            }
        }
    };

    let mailer: AsyncSmtpTransport<Tokio1Executor> =
        match AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&smtp_host) {
            Ok(builder) => {
                let builder = builder.port(smtp_port);
                // Add credentials if configured.
                let builder = if let (Ok(user), Ok(pass)) = (
                    std::env::var("EEL_SMTP_USER"),
                    std::env::var("EEL_SMTP_PASS"),
                ) {
                    builder.credentials(Credentials::new(user, pass))
                } else {
                    builder
                };
                builder.build()
            }
            Err(e) => {
                return DeliveryOutcome::Retryable {
                    error: format!("SMTP relay configuration error: {e}"),
                }
            }
        };

    match mailer.send(email).await {
        Ok(response) => {
            info!(
                delivery_id = %ctx.delivery.delivery_id,
                recipient = %recipient,
                code = ?response.code(),
                "email delivered"
            );
            DeliveryOutcome::Succeeded {
                remote_receipt_id: Some(response.code().to_string()),
                remote_receipt_at: Some(Utc::now()),
            }
        }
        Err(e) => {
            warn!(
                delivery_id = %ctx.delivery.delivery_id,
                error = %e,
                "SMTP send failed"
            );
            DeliveryOutcome::Retryable {
                error: format!("SMTP send failed: {e}"),
            }
        }
    }
}
