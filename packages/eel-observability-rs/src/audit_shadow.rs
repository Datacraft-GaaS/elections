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

use sqlx::PgPool;
use crate::error::ObservabilityError;

/// SQL for creating an audit shadow trigger on the given table.
///
/// The trigger fires AFTER INSERT/UPDATE/DELETE and inserts a row into
/// `audit.<table>_history` with (op, old_row, new_row, changed_by, changed_at, request_id).
pub fn audit_shadow_sql(table_name: &str) -> String {
	let history_table = format!("audit.{table_name}_history");
	format!(
		r#"
-- Audit shadow for {table_name}
CREATE SCHEMA IF NOT EXISTS audit;

CREATE TABLE IF NOT EXISTS {history_table} (
    id          BIGSERIAL PRIMARY KEY,
    op          TEXT NOT NULL CHECK (op IN ('INSERT','UPDATE','DELETE')),
    old_row     JSONB,
    new_row     JSONB,
    changed_by  TEXT,
    changed_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    request_id  TEXT
);

CREATE OR REPLACE FUNCTION audit.shadow_{table_name}() RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO {history_table} (op, old_row, new_row, changed_by, changed_at, request_id)
    VALUES (
        TG_OP,
        CASE WHEN TG_OP = 'INSERT' THEN NULL ELSE row_to_json(OLD)::jsonb END,
        CASE WHEN TG_OP = 'DELETE' THEN NULL ELSE row_to_json(NEW)::jsonb END,
        current_setting('app.current_user', true),
        now(),
        current_setting('app.request_id', true)
    );
    RETURN NULL;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

DROP TRIGGER IF EXISTS audit_shadow ON {table_name};
CREATE TRIGGER audit_shadow
    AFTER INSERT OR UPDATE OR DELETE ON {table_name}
    FOR EACH ROW EXECUTE FUNCTION audit.shadow_{table_name}();
"#
	)
}

/// Install the audit shadow trigger for a table on the given pool.
///
/// Skips gracefully if the table does not exist.
pub async fn install_audit_shadow(
	pool: &PgPool,
	table_name: &str,
) -> Result<(), ObservabilityError> {
	let sql = audit_shadow_sql(table_name);
	sqlx::raw_sql(&sql)
		.execute(pool)
		.await
		.map_err(|e| ObservabilityError::SqlMigration(e.to_string()))?;
	tracing::info!(table = table_name, "audit_shadow_installed");
	Ok(())
}
