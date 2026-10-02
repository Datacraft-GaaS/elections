-- Copyright (c) 2026 Nyimbi Odero
--
-- Licensed under the Apache License, Version 2.0 (the "License");
-- you may not use this file except in compliance with the License.
-- You may obtain a copy of the License at
--
--     http://www.apache.org/licenses/LICENSE-2.0
--
-- Unless required by applicable law or agreed to in writing, software
-- distributed under the License is distributed on an "AS IS" BASIS,
-- WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
-- See the License for the specific language governing permissions and
-- limitations under the License.

-- The SAR ticket table stored the requester's RAW national ID.
--
-- openapi-contracts/bvr/openapi.yaml asks for `requestor_national_id_hash`, a
-- 64-hex SHA-256, and class A data should not be written to a ticket table in the
-- clear. `create_sar` was also accepting the raw number over the wire.
--
-- Renamed rather than added: the old column had exactly one writer, and leaving it
-- in place would keep a column whose name invites the raw value back.
ALTER TABLE sar_requests
    RENAME COLUMN requester_national_id TO requestor_national_id_hash;

-- A hash is 64 characters; the raw number was 8-10. Anything already written
-- under the old column is a raw national ID, so it cannot be retro-hashed
-- without the salt, and it must not silently stand in as a hash.
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM sar_requests WHERE requestor_national_id_hash !~ '^[0-9a-f]{64}$') THEN
        RAISE EXCEPTION
            'sar_requests contains values under requestor_national_id_hash that are not \
             64-hex SHA-256 hashes. These are raw national ID numbers and must be \
             removed or re-derived before this column is trusted. Aborting.';
    END IF;
END $$;
