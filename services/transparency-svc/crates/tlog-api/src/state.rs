use std::sync::Arc;

use sqlx::PgPool;
use tlog_sequencer::Sequencer;

#[derive(Clone)]
pub struct ApiState {
    pub pool: Arc<PgPool>,
    pub sequencer: Arc<Sequencer>,
}
