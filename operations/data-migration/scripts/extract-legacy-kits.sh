#!/usr/bin/env bash
# Reads a KIEMS BVR PostgreSQL dump directory and extracts device registrations,
# mapping them to the format expected by inventory-svc's /admin/import/kits endpoint.
#
# Usage: extract-legacy-kits.sh <DUMP_DIR> <OUTPUT_CSV>

set -euo pipefail

DUMP_DIR="${1:?Usage: $0 <dump_dir> <output_csv>}"
OUTPUT_CSV="${2:?Usage: $0 <dump_dir> <output_csv>}"

if [[ ! -d "$DUMP_DIR" ]]; then
	echo "ERROR: dump directory not found: $DUMP_DIR" >&2
	exit 1
fi

RESTORE_DB="kiems_kits_extract_$$"
PGHOST="${PGHOST:-localhost}"
PGPORT="${PGPORT:-5432}"
PGUSER="${PGUSER:-postgres}"

cleanup() {
	psql -h "$PGHOST" -p "$PGPORT" -U "$PGUSER" postgres \
		-c "DROP DATABASE IF EXISTS $RESTORE_DB;" 2>/dev/null || true
}
trap cleanup EXIT

echo "[extract-legacy-kits] Creating ephemeral restore database: $RESTORE_DB"
psql -h "$PGHOST" -p "$PGPORT" -U "$PGUSER" postgres \
	-c "CREATE DATABASE $RESTORE_DB;"

echo "[extract-legacy-kits] Restoring dump into $RESTORE_DB"
pg_restore \
	--host "$PGHOST" \
	--port "$PGPORT" \
	--username "$PGUSER" \
	--dbname "$RESTORE_DB" \
	--no-privileges \
	--no-owner \
	"$DUMP_DIR"

echo "[extract-legacy-kits] Extracting device registrations to CSV"
psql -h "$PGHOST" -p "$PGPORT" -U "$PGUSER" -d "$RESTORE_DB" \
	--no-psqlrc \
	-c "
COPY (
  SELECT
    d.serial_number                  AS serial,
    d.imei                           AS imei,
    d.model_name                     AS model,
    CASE d.device_type
      WHEN 'BVR' THEN 'kit'
      WHEN 'BVI' THEN 'kit'
      WHEN 'RTS' THEN 'kit'
      ELSE 'unknown'
    END                              AS category,
    d.device_type                    AS kiems_device_type,
    d.assigned_station_code          AS assigned_station_code,
    d.last_assigned_officer          AS last_assigned_officer,
    d.procurement_date               AS procurement_date,
    d.decommission_date              AS decommission_date,
    d.status                         AS kiems_status
  FROM devices d
  WHERE d.status NOT IN ('scrapped', 'lost_unrecovered')
  ORDER BY d.serial_number
) TO STDOUT WITH (FORMAT CSV, HEADER true, DELIMITER ',', ENCODING 'UTF8');
" > "$OUTPUT_CSV"

RECORD_COUNT=$(tail -n +2 "$OUTPUT_CSV" | wc -l | tr -d ' ')
echo "[extract-legacy-kits] Extracted $RECORD_COUNT device records to $OUTPUT_CSV"

if [[ "$RECORD_COUNT" -eq 0 ]]; then
	echo "ERROR: zero records extracted; check dump and SQL query" >&2
	exit 1
fi
