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

//! tlog-ingest — Kafka consumer that feeds every platform event into the log.
//!
//! Verifies eel-events-rs signatures; routes tampered events to `<topic>.tamper`.
//! Deduplicates by (source_topic, source_event_id) via PostgreSQL.
//! Back-pressures producers via `TokioSemaphore` bounded by sequencer rate.
#![warn(missing_docs)]

pub mod consumer;
pub mod failures;
pub mod offset_tracker;
pub mod topic_router;

pub use consumer::IngestConsumer;
