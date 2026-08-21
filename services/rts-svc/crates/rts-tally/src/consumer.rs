use eel_eventstore_rs::exactly_once::{ExactlyOnceConsumer};
use eel_eventstore_rs::EventStoreError;
use tracing::info;

pub const TOPIC_FORM34A: &str = "eel.rts.form34a.v1";

pub struct TallyConsumer { inner: ExactlyOnceConsumer }
impl TallyConsumer {
    pub fn new(inner: ExactlyOnceConsumer) -> Self { Self { inner } }
    pub async fn run(&self) -> Result<(), EventStoreError> {
        info!(topic = TOPIC_FORM34A, "tally consumer starting");
        loop {
            self.inner.process_one(|payload| async move {
                info!(bytes = payload.len(), "tally consumer: processing message");
                Ok(None)
            }).await?;
        }
    }
}
