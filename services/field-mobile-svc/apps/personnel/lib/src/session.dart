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

/// Provisioned worker identity for the current session.
///
/// In production these come from the eel-auth JWT claims and the worker's
/// active posting. The values below are valid UUIDs so the dispatch-api
/// accepts them during local development and demo runs.
const String kWorkerId = '00000000-0000-4000-8000-000000000001';
const String kElectionId = '00000000-0000-4000-8000-000000000002';
const String kStationCode = 'STN-022-004';
