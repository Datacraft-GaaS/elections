//! Store trait for printing — swap boundary for `AppState`.
//!
//! ```sql
//! -- migrations/VXXX__printing_orders.sql (sketch)
//! CREATE TABLE printing_orders (
//!   order_id UUID PRIMARY KEY,
//!   election_id UUID NOT NULL,
//!   position_code TEXT NOT NULL,
//!   constituency_code TEXT NOT NULL,
//!   ballot_quantity INT NOT NULL,
//!   proof_id UUID NOT NULL,
//!   security_features TEXT[] NOT NULL,
//!   status TEXT NOT NULL,
//!   printer_reference TEXT NOT NULL,
//!   created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
//!   updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
//! );
//! ```

use async_trait::async_trait;
use uuid::Uuid;

use crate::spec::{DeliveryRecord, DeliveryRecordRequest, PrintOrder, PrintOrderRequest, QCDecision};

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("not found")]
    NotFound,
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

#[async_trait]
pub trait PrintStore: Send + Sync {
    async fn create_order(&self, req: PrintOrderRequest) -> Result<PrintOrder, StoreError>;
    async fn get_order(&self, id: Uuid) -> Result<Option<PrintOrder>, StoreError>;
    async fn list_orders(&self) -> Result<Vec<PrintOrder>, StoreError>;
    async fn submit_qc(&self, id: Uuid, decision: QCDecision) -> Result<Option<PrintOrder>, StoreError>;
    async fn record_delivery(&self, req: DeliveryRecordRequest) -> Result<DeliveryRecord, StoreError>;
}
