//! Signed integrity statements (`eel.integrity-statement.v1`) over the
//! current public-log state: latest STH, witness co-signature count and
//! open ingest failures. A non-empty failure set means the log is known
//! incomplete, so the command exits non-zero after printing the statement.

use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde_json::{json, Value};

/// Pure builder for the unsigned statement body.
pub fn build_statement(
    tree_size: u64,
    root_hash_hex: &str,
    witness_cosig_count: i64,
    open_failures: i64,
    generated_at: &str,
) -> Value {
    json!({
        "type": "eel.integrity-statement.v1",
        "tree_size": tree_size,
        "root_hash": root_hash_hex,
        "witness_cosignatures": witness_cosig_count,
        "open_ingest_failures": open_failures,
        "generated_at": generated_at,
    })
}

fn load_signing_key(seed_hex: &str) -> anyhow::Result<ed25519_dalek::SigningKey> {
    let seed: [u8; 32] = hex::decode(seed_hex.trim())
        .map_err(|e| anyhow::anyhow!("signing key not hex: {e}"))?
        .try_into()
        .map_err(|_| anyhow::anyhow!("signing key must be a 32-byte Ed25519 seed"))?;
    Ok(ed25519_dalek::SigningKey::from_bytes(&seed))
}

/// Build, sign and print an integrity statement. Returns the number of open
/// ingest failures so the caller can exit non-zero on an incomplete log.
pub async fn run(db_url: &str, signing_key_hex: &str) -> anyhow::Result<i64> {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(2)
        .connect(db_url)
        .await?;

    let sth: Option<(i64, String)> = sqlx::query_as(
        "SELECT tree_size, encode(root_hash, 'hex') \
         FROM sth ORDER BY tree_size DESC LIMIT 1",
    )
    .fetch_optional(&pool)
    .await?;
    let (tree_size, root_hash_hex) =
        sth.ok_or_else(|| anyhow::anyhow!("no STH rows in log; nothing to attest"))?;

    let cosigs: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM witness_cosigs")
        .fetch_one(&pool)
        .await?;

    let open = tlog_ingest::failures::list_open(&pool).await?;
    let open_failures = open.len() as i64;

    let generated_at = chrono::Utc::now().to_rfc3339();
    let statement = build_statement(
        tree_size as u64,
        &root_hash_hex,
        cosigs.0,
        open_failures,
        &generated_at,
    );

    let key = load_signing_key(signing_key_hex)?;
    use ed25519_dalek::Signer;
    let canonical = serde_json::to_vec(&statement)?;
    let sig = key.sign(&canonical);

    let mut signed = statement;
    signed["signature"] = json!(format!("ed25519:{}", STANDARD.encode(sig.to_bytes())));
    signed["signer_pubkey"] = json!(hex::encode(key.verifying_key().as_bytes()));
    println!("{}", serde_json::to_string_pretty(&signed)?);

    Ok(open_failures)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn statement_has_all_required_fields() {
        let s = build_statement(42, "ab12", 3, 0, "2026-08-25T00:00:00Z");
        assert_eq!(s["type"], "eel.integrity-statement.v1");
        assert_eq!(s["tree_size"], 42);
        assert_eq!(s["root_hash"], "ab12");
        assert_eq!(s["witness_cosignatures"], 3);
        assert_eq!(s["open_ingest_failures"], 0);
        assert_eq!(s["generated_at"], "2026-08-25T00:00:00Z");
    }

    #[test]
    fn statement_serializes_deterministically() {
        let a = serde_json::to_vec(&build_statement(7, "ff", 1, 2, "t")).unwrap();
        let b = serde_json::to_vec(&build_statement(7, "ff", 1, 2, "t")).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn bad_signing_keys_are_rejected() {
        assert!(load_signing_key("zz").is_err());
        assert!(load_signing_key("abcd").is_err()); // 2 bytes, not 32
        assert!(load_signing_key(&"00".repeat(32)).is_ok());
    }
}
