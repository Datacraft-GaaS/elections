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

use eel_auth_rs::{
    audit::{AuditEvent, NoopSink, LogSink, AuditSink},
    subject::Subject,
    spicedb::{SpiceDbClient, Consistency, Decision},
    jwks::JwksVerifier,
};
use serde_json::json;

async fn test_noop_sink() {
    let sink = NoopSink;
    let ev = AuditEvent::new("user1", "read", "doc:1", "allow", json!({}));
    let res = sink.emit(ev).await;
    assert!(res.is_ok());
}

async fn test_log_sink() {
    let sink = LogSink;
    let ev = AuditEvent::new("user1", "write", "doc:2", "deny", json!({}));
    let res = sink.emit(ev).await;
    assert!(res.is_ok());
}

async fn test_subject_serde() {
    let s = Subject {
        sub: "abc-123".into(),
        realm: "eel-staff".into(),
        session_id: Some("sess-1".into()),
        roles: vec!["ro-admin".into()],
        purpose_scopes: vec!["identification".into()],
    };
    let json_str = serde_json::to_string(&s).unwrap();
    let back: Subject = serde_json::from_str(&json_str).unwrap();
    assert_eq!(back.sub, "abc-123");
}

async fn test_jwks_bad_token() {
    let verifier = JwksVerifier::new("http://localhost:19999/nonexistent/certs", "test");
    let res = verifier.verify("not.a.jwt.token").await;
    assert!(res.is_err());
}

async fn test_spicedb_fail_closed() {
    let client = SpiceDbClient::new("http://localhost:19998", NoopSink);
    let subject = Subject {
        sub: "user:u1".into(),
        realm: "test".into(),
        session_id: None,
        roles: vec![],
        purpose_scopes: vec![],
    };
    let decision = client.check(&subject, "read", "doc:1", Consistency::MinimizeLatency, json!({})).await.unwrap();
    assert_eq!(decision, Decision::Deny);
}

#[tokio::test]
async fn all_tests() {
    test_noop_sink().await;
    test_log_sink().await;
    test_subject_serde().await;
    test_jwks_bad_token().await;
    test_spicedb_fail_closed().await;
}
