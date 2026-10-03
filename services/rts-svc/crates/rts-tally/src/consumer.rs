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

use eel_eventstore_rs::exactly_once::ExactlyOnceConsumer;
use eel_eventstore_rs::EventStoreError;
use tracing::info;

pub const TOPIC_FORM34A: &str = "eel.rts.form34a.v1";

pub struct TallyConsumer {
    inner: ExactlyOnceConsumer,
}
impl TallyConsumer {
    pub fn new(inner: ExactlyOnceConsumer) -> Self {
        Self { inner }
    }
    pub async fn run(&self) -> Result<(), EventStoreError> {
        info!(topic = TOPIC_FORM34A, "tally consumer starting");
        loop {
            self.inner
                .process_one(|payload| async move {
                    info!(bytes = payload.len(), "tally consumer: processing message");
                    Ok(None)
                })
                .await?;
        }
    }
}
