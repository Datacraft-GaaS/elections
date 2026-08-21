# training-svc Events

All events emitted via eel-outbox-rs. Signed unless noted.

| Event | Topic | Retention | Signed |
|---|---|---|---|
| eel.training.content_published.v1 | eel.training.content.v1 | operational | yes |
| eel.training.content_expired.v1 | eel.training.content.v1 | operational | yes |
| eel.training.content_archived.v1 | eel.training.content.v1 | operational | yes |
| eel.training.worker_certified.v1 | eel.training.worker.v1 | operational | yes |
| eel.training.worker_certification_revoked.v1 | eel.training.worker.v1 | operational | yes |
| eel.training.refresher_completed.v1 | eel.training.worker.v1 | operational | yes |
| eel.training.briefing_confirmed.v1 | eel.training.worker.v1 | operational | yes |
| eel.training.third_fail_triggered.v1 | eel.training.worker.v1 | operational | yes |
| eel.training.media_badge_issued.v1 | eel.training.accreditation.v1 | operational | yes |
| eel.training.media_badge_revoked.v1 | eel.training.accreditation.v1 | audit | yes |
| eel.training.media_breach_referred.v1 | eel.training.accreditation.v1 | audit | yes |
| eel.training.observer_credential_issued.v1 | eel.training.accreditation.v1 | operational | yes |
| eel.training.observer_credential_revoked.v1 | eel.training.accreditation.v1 | audit | yes |
| eel.training.observer_deployed.v1 | eel.training.accreditation.v1 | operational | yes |
| eel.training.mou_signed.v1 | eel.training.accreditation.v1 | audit | yes |
| eel.training.medical_incident_reported.v1 | eel.training.medical.v1 | operational | no |
| eel.training.medical_cluster_alert.v1 | eel.training.medical.v1 | operational | no |
| eel.training.dispatch_requested.v1 | eel.training.medical.v1 | operational | no |
