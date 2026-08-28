/// Provisioned worker identity for the current session.
///
/// In production these come from the eel-auth JWT claims and the worker's
/// active posting. The values below are valid UUIDs so the dispatch-api
/// accepts them during local development and demo runs.
const String kWorkerId = '00000000-0000-4000-8000-000000000001';
const String kElectionId = '00000000-0000-4000-8000-000000000002';
const String kStationCode = 'STN-022-004';
