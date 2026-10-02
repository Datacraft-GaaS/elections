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

//! Why the CVR endpoints refuse rather than answer.
//!
//! The Continuous Voter Register is the authoritative list of who is registered to
//! vote. These three endpoints are different in kind from every other handler in
//! this effort, and the difference is worth stating precisely.
//!
//! `voter_lookup` previously returned `registered: false` with null station, ward
//! and constituency **for every request**. A false negative about a voter's
//! registration is not a neutral unknown. It is a definitive-looking "no, you are
//! not on the register", delivered to a person who may well be on it, on the one
//! endpoint whose entire purpose is to answer that question. Every other
//! fabrication in this tree is a wrong number or a dead endpoint; this one can
//! turn away a registered voter.
//!
//! The roll is not merely a service that is missing. It does not exist here to
//! query: `services/bvr-svc/migrations/` defines `register_snapshots` (aggregate
//! counts and a Merkle root) and `eel_voter_register_events` (an audit log), and
//! there is no `eel_voter_register` table anywhere in the migrations. So there is
//! no row a lookup could return, and no column a lookup could filter on.
//!
//! Nothing is derived, guessed or stubbed, because any of those would re-introduce
//! the false negative.

/// The message `voter_lookup` returns.
pub const REGISTER_UNAVAILABLE: &str = "the voter roll is held by the registration \
     service, which is not in this workspace, and there is no eel_voter_register table in \
     services/bvr-svc/migrations to query. This handler previously returned registered false \
     for every request, which is a definitive-looking negative about a real person: a \
     registered voter can be turned away by it. No register data is derived, guessed or \
     stubbed here, because any of those re-introduces that false negative";

/// The message the snapshot endpoint returns.
pub const SNAPSHOT_UNAVAILABLE: &str = "a snapshot download is a presigned MinIO URL for a \
     Parquet/CSV export, and object storage is not in this workspace. register_snapshots \
     does hold the aggregate counts, but the download itself cannot be issued. This handler \
     previously answered 404 no snapshot available for every request, which reads as no \
     snapshot having been published rather than as this service being unable to issue one";

/// The message the statistics endpoint returns.
pub const STATS_UNAVAILABLE: &str = "register statistics are derived from the roll. \
     register_snapshots holds a whole-election voter_count, but RegisterStats also requires a \
     per-county breakdown and no table in this workspace has a county dimension over the roll. \
     This handler previously returned total_registered 0 and counties 0, which reads as a \
     measured register that happens to be empty";
