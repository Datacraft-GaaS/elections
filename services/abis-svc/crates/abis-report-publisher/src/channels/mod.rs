//! Channel adapters. Each module exposes `deliver(ctx) -> DeliveryOutcome`.

pub mod email;
pub mod webhook;
pub mod kafka;
pub mod sftp;
pub mod gov_printer;
pub mod kbc_sftp;
pub mod api_pull;

use crate::domain::{Channel, DeliveryContext, DeliveryOutcome};

/// Dispatch a delivery to the appropriate channel adapter.
pub async fn dispatch(channel: &Channel, ctx: DeliveryContext) -> DeliveryOutcome {
	match channel {
		Channel::Email => email::deliver(ctx).await,
		Channel::Webhook => webhook::deliver(ctx).await,
		Channel::Kafka => kafka::deliver(ctx).await,
		Channel::Sftp => sftp::deliver(ctx).await,
		Channel::GovPrinter => gov_printer::deliver(ctx).await,
		Channel::KbcSftp => kbc_sftp::deliver(ctx).await,
		Channel::ApiPull => api_pull::deliver(ctx).await,
	}
}
