#!/usr/bin/env bash
# Copyright (c) 2026 Nyimbi Odero
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

# Reads a KIEMS BVR PostgreSQL dump directory and emits a CSV file in the format
# accepted by bvr-svc's /admin/import endpoint.
#
# Usage: extract-legacy-voters.sh <DUMP_DIR> <OUTPUT_CSV>

set -euo pipefail

DUMP_DIR="${1:?Usage: $0 <dump_dir> <output_csv>}"
OUTPUT_CSV="${2:?Usage: $0 <dump_dir> <output_csv>}"

if [[ ! -d "$DUMP_DIR" ]]; then
	echo "ERROR: dump directory not found: $DUMP_DIR" >&2
	exit 1
fi

RESTORE_DB="kiems_extract_$$"
PGHOST="${PGHOST:-localhost}"
PGPORT="${PGPORT:-5432}"
PGUSER="${PGUSER:-postgres}"

cleanup() {
	psql -h "$PGHOST" -p "$PGPORT" -U "$PGUSER" postgres \
		-c "DROP DATABASE IF EXISTS $RESTORE_DB;" 2>/dev/null || true
}
trap cleanup EXIT

echo "[extract-legacy-voters] Creating ephemeral restore database: $RESTORE_DB"
psql -h "$PGHOST" -p "$PGPORT" -U "$PGUSER" postgres \
	-c "CREATE DATABASE $RESTORE_DB;"

echo "[extract-legacy-voters] Restoring dump into $RESTORE_DB"
pg_restore \
	--host "$PGHOST" \
	--port "$PGPORT" \
	--username "$PGUSER" \
	--dbname "$RESTORE_DB" \
	--no-privileges \
	--no-owner \
	"$DUMP_DIR"

echo "[extract-legacy-voters] Extracting voters to CSV"
psql -h "$PGHOST" -p "$PGPORT" -U "$PGUSER" -d "$RESTORE_DB" \
	--no-psqlrc \
	-c "
COPY (
  SELECT
    v.national_id                    AS national_id,
    v.surname                        AS surname,
    v.other_names                    AS other_names,
    v.date_of_birth                  AS date_of_birth,
    v.gender                         AS gender,
    v.county_code                    AS county_code,
    v.ward_code                      AS ward_code,
    v.polling_station_code           AS polling_station_code,
    v.registration_date              AS registration_date,
    v.registration_officer_id        AS registration_officer_id,
    COALESCE(b.fingerprint_template, '') AS fingerprint_template,
    COALESCE(b.face_template, '')    AS face_template,
    v.record_status                  AS record_status
  FROM voters v
  LEFT JOIN biometrics b ON b.national_id = v.national_id
  WHERE v.record_status IN ('active', 'provisional')
  ORDER BY v.national_id
) TO STDOUT WITH (FORMAT CSV, HEADER true, DELIMITER ',', ENCODING 'UTF8');
" > "$OUTPUT_CSV"

RECORD_COUNT=$(tail -n +2 "$OUTPUT_CSV" | wc -l | tr -d ' ')
echo "[extract-legacy-voters] Extracted $RECORD_COUNT voter records to $OUTPUT_CSV"

if [[ "$RECORD_COUNT" -eq 0 ]]; then
	echo "ERROR: zero records extracted; check dump and SQL query" >&2
	exit 1
fi
