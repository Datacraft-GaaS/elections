-- crms-svc baseline placeholder.
--
-- The crms event-store schema (candidate, nomination, clearance_decision,
-- ballot_proof, political_party, coalition_registration, symbol_submission)
-- is owned by crates/crms-domain and has not been written yet. This file
-- keeps the service's migrations/ directory non-empty so a migration runner
-- (e.g. `sqlx migrate run` or the inventory-api-style startup runner) does
-- not treat the service as unmigrated.
SELECT 1;
