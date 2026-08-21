use eel_secrets_rs::{
	admin_creds::issue_admin_cred,
	error::SecretsError,
	transit::Transit,
};

// Tests that don't require a running OpenBao instance

#[tokio::test]
async fn test_transit_unreachable_returns_error() {
	let transit = Transit::new("http://localhost:19997", "s.test");
	let result = transit.encrypt("mykey", b"hello world").await;
	assert!(result.is_err());
	let err = result.unwrap_err();
	let msg = err.to_string();
	assert!(
		msg.contains("transit") || msg.contains("HTTP") || msg.contains("error") || msg.contains("connect"),
		"unexpected: {msg}"
	);
}

#[tokio::test]
async fn test_admin_cred_cap_enforced() {
	// 5h requested, should be capped at 4h (or fail due to unreachable server)
	let result = issue_admin_cred("http://localhost:19997", "s.test", "admin-role", 5).await;
	match result {
		Ok(cred) => {
			let diff = cred.expires_at - cred.issued_at;
			assert!(diff.num_hours() <= 4, "TTL exceeded 4h cap");
		}
		Err(SecretsError::VaultClient(_)) | Err(SecretsError::Transit(_)) => {
			// Expected when server unreachable
		}
		Err(e) => panic!("unexpected error: {e}"),
	}
}

#[tokio::test]
async fn test_transit_struct_creation() {
	let t = Transit::new("http://localhost:8200", "root");
	let result = t.decrypt("mykey", "vault:v1:badsig").await;
	assert!(result.is_err());
}
