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

//! Per-station queue state.
//!
//! `get_queue_snapshot` used to live here and returned `queue_length: 0` for any
//! station, which reads as "no one is waiting" for every station in the country.
//! The real one is in [`crate::handlers::verify`], reading the store that agents
//! check in against; this module keeps the queue module's namespace.

pub use crate::handlers::verify::{get_queue_snapshot, queue_check_in, queue_check_out};
