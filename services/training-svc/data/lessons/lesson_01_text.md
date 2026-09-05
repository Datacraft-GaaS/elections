# Lesson 1: Election Procedure Overview (Text)
Type: Text | Duration: 20 min | Module: core-election-procedure

## Learning Objectives
- Understand the legal framework (Elections Act §34, DPA 2019 §6)
- Identify roles: Presiding Officer (PO), Deputy Presiding Officer (DPO), Clerk
- Sequence: Opening → Verification → Voting → Tally → Close

## Content
The election day follows a strict temporal order governed by the Elections Act and the Data Protection Act. The Presiding Officer (PO) holds statutory authority (§34) for station-level decisions. The Deputy Presiding Officer (DPO) assists with verification and acts when the PO is absent. Clerks maintain the voter register and manage biometrics.

Verification uses NBIS v5.0.0 gallery matching with Bozorth3 fingerprint extraction (§3.1 ADR-0022). If a match score exceeds 82% (threshold per ADR-0006), the voter proceeds; otherwise manual adjudication by the PO.

Security: dual-factor authentication (smart card + biometric), HSM-signed audit events (ADR-0019), canary token checks every 15 minutes (falco-rules/eel_siem.yaml).

## Key Terms
- Gallery shard partition (M3 device ID linked)
- Proof hash (SHA-256 of voter_id + station_code + timestamp)
- Correlation key (links to detection rules in falco / wazuh)
