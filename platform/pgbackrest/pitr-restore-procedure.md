# pgBackRest PITR Restore Procedure

Restore any T1 cluster to any point in the last 30 days.

## Prerequisites

- pgBackRest >= 2.52 installed on the target host
- Access to MinIO repo (repo1 primary, repo2 mirror as fallback)
- Target PostgreSQL stopped: `systemctl stop postgresql`

## Steps

1. Identify the target point-in-time (UTC ISO 8601):
   ```
   TARGET="2026-08-15T14:32:00Z"
   STANZA="voter-db"   # voter-db | consensus-db | audit-db
   REPO=1              # 1=primary, 2=mirror
   ```

2. List available backup sets:
   ```
   pgbackrest --stanza=$STANZA --repo=$REPO info
   ```

3. Restore to target time:
   ```
   pgbackrest --stanza=$STANZA \
     --repo=$REPO \
     --type=time \
     "--target=$TARGET" \
     --target-action=promote \
     restore
   ```

4. Start PostgreSQL and verify recovery completion:
   ```
   systemctl start postgresql
   psql -c "SELECT pg_is_in_recovery();"  -- must return 'f' (not in recovery)
   psql -c "SELECT now() - pg_postmaster_start_time() AS uptime;"
   ```

5. Verify data integrity:
   ```
   pgbackrest --stanza=$STANZA check
   SELECT count(*) FROM results WHERE received_at <= '$TARGET'::timestamptz;
   ```

6. Record drill outcome in Jira DR-DRILL ticket:
   - Time to restore (RTO measurement)
   - WAL LSN at restore point (RPO measurement)
   - Any anomalies
