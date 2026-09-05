# Lesson 3: Poll-Day Security Protocols (Audio)
Type: Audio (MP3) | Duration: 15 min | Module: poll-day-security
Transcript excerpt:
"The station opens at 05:30 under HSM attestation. The PO verifies the canary token array — 17 distributed tokens per station, rotated every 4 hours. Any mismatch triggers a falco SIEM alert (eel_siem.yaml, rule #42). Audit shadow records to otel endpoint with M3_device_id embedded. If dual-factor fails, the station enters freeze mode (§11.0 M4 gate)."
Audio file ref: /data/audio/lesson_03_security.mp3 (simulated; real source: field-mobile-verify audio feed)
