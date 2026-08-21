# training-svc

WP-5.4. Modules: M37, M38, M42, M43, M60.

Service manages voter education content (CMS), poll worker training (LMS),
media and observer accreditation, and medical first response coordination.

Components:
- django/cms: M37 voter-education-cms (11-language CMS, fact-check, IEBC approval)
- django/lms: M38 poll-worker-training-lms (250k workers, cascaded, HSM certs)
- django/accreditation: M42+M43 media and observer accreditation (QR badges, CRL)
- django/medical: M60 medical first response (no-PHI incident dispatch)
- apps/lms-offline: Flutter offline LMS (Android 8+, 5-module cache)
- apps/accred-scanner: Flutter QR scanner (offline CRL, 300ms verify)
- crates/training-api: Rust axum thin API wrapping Django views

Availability: routine 99.5%; 99.9% during T-30 to T-1 and accreditation window.
