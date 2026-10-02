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

//! `eel-abis-perf-monitor`: continuous FMR/FNMR + rank-N accuracy monitoring.
//!
//! Recomputes metrics on a labelled holdout set every 6 hours.
//! Produces daily, weekly, and monthly rollup reports.
#![warn(missing_docs)]

pub mod domain;
pub mod error;
pub mod pmr;
pub mod report;
pub mod target;

pub use domain::{HoldoutPair, PerformanceReport, PerformanceTarget, RollupPeriod};
pub use error::PerfMonitorError;
pub use pmr::{compute_fmr, compute_fnmr, compute_rank_n_accuracy, PmrResult};
