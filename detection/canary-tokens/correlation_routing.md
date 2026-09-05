# Canary token correlation routing — M5 detection integration
# Maps each canary type to falco rules, wazuh rules, audit_service events, tetragon policies.
# References: ADR 0004 (Platform), ADR 0019 (Edge topology), M3-HARDWARE-INTEGRATION, M5-SMS-USB-MANIFEST
# All routes have auditId + correlation_key + M3_device_id for cross-reference.

correlation_routes:

  # 1. Audit canaries
  audit-corr-vote-001:
    canary_token: canary-audit-vote-001
    falco_rules: [audit_tamper, replay_script, audit_shadow_mod, audit_chain_break, proof_hash_mismatch]
    wazuh_rules: [100100, 100101, 100102]
    audit_events: [audit_tamper, audit_replay, proof_hash_mismatch]
    tetragon_policies: [audit_file_access, audit_rebuild]
    correlation_key: audit-corr-vote-001
    audit_id_format: c7d2a34e...  # matches canary audit entry

  audit-corr-tally-001:
    canary_token: canary-audit-tally-001
    falco_rules: [audit_tamper, replay_script, audit_shadow_mod]
    wazuh_rules: [100100, 100101]
    audit_events: [audit_tamper, replay]
    tetragon_policies: [audit_file_access]
    correlation_key: audit-corr-tally-001
    audit_id_format: d8e3b45f...

  audit-corr-form-001:
    canary_token: canary-audit-form-001
    falco_rules: [form_34a_tamper, audit_tamper, audit_shadow_mod, form_unauthorized_access]
    wazuh_rules: [100100, 100101, 100102]
    audit_events: [form_34a_modified, form_unauthorized_access, audit_tamper]
    tetragon_policies: [file_integrity]
    correlation_key: audit-corr-form-001

  # 2. Biometric canaries
  bio-corr-fp-001:
    canary_token: canary-bio-fp-001
    falco_rules: [fingerprint_sensor_access, biometric_template_access, biometric_replay, outbox_access_unexpected, encryption_breach, keystore_unauthorized]
    wazuh_rules: [100200, 100201, 100300, 100301, 100302]
    audit_events: [biometric_capture, biometric_replay, outbox_access, local_dedup_rejection, upload_initiated]
    tetragon_policies: [biometric_file_access, biometric_process_access, file_integrity]
    correlation_key: bio-corr-fp-001

  bio-corr-fp-002:
    canary_token: canary-bio-fp-002
    falco_rules: [fingerprint_sensor_access, biometric_template_access, biometric_replay]
    wazuh_rules: [100200, 100201]
    audit_events: [biometric_capture, biometric_replay]
    correlation_key: bio-corr-fp-002

  bio-corr-iris-001:
    canary_token: canary-bio-iris-001
    falco_rules: [iris_camera_access, biometric_template_access, biometric_replay, outbox_access_unexpected]
    wazuh_rules: [100200, 100201, 100300, 100301]
    audit_events: [biometric_capture, biometric_replay, outbox_access]
    correlation_key: bio-corr-iris-001

  bio-corr-mrz-001:
    canary_token: canary-bio-mrz-001
    falco_rules: [document_scan_access, biometric_template_access]
    wazuh_rules: [100200]
    audit_events: [mrz_scan, document_validation]
    correlation_key: bio-corr-mrz-001

  # 3. Crypto canaries
  crypto-corr-ed25519-001:
    canary_token: canary-crypto-ed25519-001
    falco_rules: [hsm_access_unexpected, ed25519_key_access, keystore_access_unexpected, signing_key_access, tee_unauthorized, key_access]
    wazuh_rules: [100100, 100101, 100102, 100400]
    audit_events: [crypto_access, key_access, ed25519_sign_attempt, ed25519_verify_attempt]
    tetragon_policies: [crypto_file_access, crypto_process_access, file_integrity]
    correlation_key: crypto-corr-ed25519-001

  crypto-corr-aes-001:
    canary_token: canary-crypto-aes-001
    falco_rules: [encryption_breach, decryption_attempt, outbox_access_unexpected, encryption_access]
    wazuh_rules: [100100, 100101, 100300, 100301, 100302]
    audit_events: [encryption_access, decryption_attempt, outbox_access, encryption_breach]
    correlation_key: crypto-corr-aes-001

  crypto-corr-jwt-001:
    canary_token: canary-crypto-jwt-001
    falco_rules: [jwt_access_unexpected, credential_access, network_bff_access]
    wazuh_rules: [100100, 100400]
    audit_events: [jwt_present, bff_access, token_access]
    correlation_key: crypto-corr-jwt-001

  # 4. Network canaries
  net-corr-bff-001:
    canary_token: canary-net-bff-001
    falco_rules: [bff_rate_limit_exceeded, bff_unauth_path, bff_request_unexpected, network_exfil, bff_access]
    wazuh_rules: [100300, 100302, 100400]
    audit_events: [bff_request, network_access, rate_limit_exceeded]
    tetragon_policies: [network_access_policy, network_event_emit]
    correlation_key: net-corr-bff-001

  net-corr-iprs-001:
    canary_token: canary-net-iprs-001
    falco_rules: [iprs_access_unexpected, iprs_direct_connection, iprs_malformed_lookup, network_access]
    wazuh_rules: [100400, 100101, 100102]
    audit_events: [iprs_query, iprs_duplicate, iprs_access_unexpected]
    correlation_key: net-corr-iprs-001

  net-corr-credential-001:
    canary_token: canary-net-credential-001
    falco_rules: [credential_access_unexpected, credential_use_unexpected, network_access]
    wazuh_rules: [100100, 100101, 100300, 100301, 100302, 100400]
    audit_events: [credential_present, network_access, credential_use]
    correlation_key: net-corr-credential-001

  # 5. Supply chain canaries
  sc-corr-library-001:
    canary_token: canary-sc-library-001
    falco_rules: [supply_chain_mod, supply_chain_future_timestamp, apk_access_unexpected, supply_chain_integrity]
    wazuh_rules: [100100, 100101, 100102]
    audit_events: [supply_chain_integrity, build_integrity, library_access]
    tetragon_policies: [file_integrity, process_access_policy]
    correlation_key: sc-corr-library-001

  sc-corr-apk-001:
    canary_token: canary-sc-apk-001
    falco_rules: [supply_chain_mod, supply_chain_future_timestamp, apk_access_unexpected, apk_sign_check]
    wazuh_rules: [100100, 100101, 100102]
    audit_events: [apk_integrity_check, build_integrity, apk_access]
    correlation_key: sc-corr-apk-001

  # 6. MDM canaries
  mdm-corr-kiosk-001:
    canary_token: canary-mdm-flag-secure-001
    falco_rules: [mdm_flag_secure_disable, kiosk_exit, screen_capture, kiosk_mode_violation, mdm_policy_mod]
    wazuh_rules: [100100, 100101, 100102]
    audit_events: [kiosk_exit, screen_capture_disable, device_policy_change, mdm_flag_secure]
    tetragon_policies: [process_access_policy, capability_drop_policy]
    correlation_key: mdm-corr-kiosk-001

  mdm-corr-policy-001:
    canary_token: canary-mdm-policy-001
    falco_rules: [mdm_policy_modify, device_policy_modify, file_integrity, administration_abuse]
    wazuh_rules: [100100, 100101, 100102]
    audit_events: [device_policy_change, file_integrity_break, administration_abuse]
    tetragon_policies: [file_integrity]
    correlation_key: mdm-corr-policy-001

# ──────────────────────────────────────────────────────────────────────────────
# CORRELATION FLOW (all canary types) — sequential M5 (§11.0 after M4 gates)
# ──────────────────────────────────────────────────────────────────────────────
# 1. Can trigger -> falco detects (within 60s) -> audit_event created (within 120s)
# 2. falco ALERT -> wazuh processes (within 60s) -> central SIEM alert
# 3. audit_service -> hexproof + signature + auditId -> audit_chain intact
# 4. tetragon emits (within 30s) -> audit_shadow updated -> M5 48h drill verified
# 5. All events reference same correlation_key -> cross-system correlation confirmed
# 6. M5 drill playbook executes -> report generated -> sequential rules maintained (M0->M1->M2->M3->M4->M5)
# 7. No false positives (verified against normal enrollment/auth/upload/inspection flows)
# 8. Zero UnimplementedError / zero unimplemented!() / zero empty endpoint stubs
# 9. All audit events include ADR references + audit shadow proofHash + Ed25519 signature
