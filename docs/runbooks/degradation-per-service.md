# Per-service degradation modes

Each section below describes a service's normal, degraded, and read-only behaviour, and the commands to trigger each mode. Degraded mode usually means the service continues to operate with reduced functionality (e.g., writes queued, non-critical features disabled). Read-only mode means no state-mutating operations are accepted.

## abis-svc

**Normal:** ABIS biometric deduplication and identification fully operational; all match queries answered in real-time; events emitted to Kafka outbox.

**Degraded:** matching requests queued; async results returned via polling; identification latency increases. Trigger:

```bash
kubectl set env deployment/abis-svc ABIS_MODE=degraded -n eel-system
```

**Read-only:** no new identification requests accepted; existing results queryable. Trigger:

```bash
kubectl set env deployment/abis-svc ABIS_MODE=readonly -n eel-system
```

**Chaos drill:** `chaos/scenarios/eel-pod-kill-abis-svc.yaml`

---

## ai-platform-svc

**Normal:** AI inference endpoints fully operational; all model serving requests answered.

**Degraded:** inference throttled to essential requests only; batch jobs paused. Trigger:

```bash
kubectl set env deployment/ai-platform-svc AI_MODE=degraded -n eel-system
```

**Read-only:** no inference; model metadata queryable only. Trigger:

```bash
kubectl set env deployment/ai-platform-svc AI_MODE=readonly -n eel-system
```

**Chaos drill:** `chaos/scenarios/eel-pod-kill-ai-platform-svc.yaml`

---

## biometric-edge-svc

**Normal:** biometric capture and local matching at edge; events sync to backend.

**Degraded:** offline capture only; sync paused; local queue accumulates. Trigger:

```bash
kubectl set env deployment/biometric-edge-svc EDGE_SYNC=disabled -n eel-system
```

**Read-only:** no new captures; stored captures queryable locally. Trigger:

```bash
kubectl set env deployment/biometric-edge-svc EDGE_MODE=readonly -n eel-system
```

**Chaos drill:** `chaos/scenarios/eel-pod-kill-biometric-edge-svc.yaml`

---

## boundary-svc

**Normal:** API gateway fully operational; all traffic routed; auth enforced.

**Degraded:** rate limits tightened; non-critical endpoints return 503. Trigger:

```bash
kubectl set env deployment/boundary-svc BOUNDARY_MODE=degraded -n eel-system
```

**Read-only:** all write-path routes return 503; GET routes operational. Trigger:

```bash
kubectl set env deployment/boundary-svc BOUNDARY_MODE=readonly -n eel-system
```

**Chaos drill:** `chaos/scenarios/eel-pod-kill-boundary-svc.yaml`

---

## bvi-svc

**Normal:** biometric voter identification fully operational; real-time ID checks.

**Degraded:** ID checks answered from local cache; backend sync paused. Trigger:

```bash
kubectl set env deployment/bvi-svc BVI_MODE=degraded -n eel-system
```

**Read-only:** no new ID checks; existing voter records queryable. Trigger:

```bash
kubectl set env deployment/bvi-svc BVI_MODE=readonly -n eel-system
```

**Chaos drill:** `chaos/scenarios/eel-pod-kill-bvi-svc.yaml`

---

## bvr-svc

**Normal:** biometric voter registration fully operational; new registrations and updates accepted.

**Degraded:** registrations queued locally; backend ingest paused; read queries answered. Trigger:

```bash
kubectl set env deployment/bvr-svc BVR_MODE=degraded -n eel-system
```

**Read-only:** no registrations or updates; voter record queries answered. Trigger:

```bash
kubectl set env deployment/bvr-svc BVR_MODE=readonly -n eel-system
```

**Chaos drill:** `chaos/scenarios/eel-pod-kill-bvr-svc.yaml`

---

## crms-svc

**Normal:** candidate and results management fully operational; all CRUD operations available.

**Degraded:** writes queued; reads answered from cache; non-critical features disabled. Trigger:

```bash
kubectl set env deployment/crms-svc CRMS_MODE=degraded -n eel-system
```

**Read-only:** no writes; candidate data queryable. Trigger:

```bash
kubectl set env deployment/crms-svc CRMS_MODE=readonly -n eel-system
```

**Chaos drill:** `chaos/scenarios/eel-pod-kill-crms-svc.yaml`

---

## ecc-svc

**Normal:** election commission console fully operational; all admin operations available.

**Degraded:** non-critical admin operations disabled; core monitoring and approvals available. Trigger:

```bash
kubectl set env deployment/ecc-svc ECC_MODE=degraded -n eel-system
```

**Read-only:** no admin actions; console is view-only. Trigger:

```bash
kubectl set env deployment/ecc-svc ECC_MODE=readonly -n eel-system
```

**Chaos drill:** `chaos/scenarios/eel-pod-kill-ecc-svc.yaml`

---

## field-mobile-svc

**Normal:** field mobile backend fully operational; push config and event sync active.

**Degraded:** push updates paused; mobile clients use cached config. Trigger:

```bash
kubectl set env deployment/field-mobile-svc FIELD_MOBILE_MODE=degraded -n eel-system
```

**Read-only:** no config pushes or event ingestion; status queries answered. Trigger:

```bash
kubectl set env deployment/field-mobile-svc FIELD_MOBILE_MODE=readonly -n eel-system
```

**Chaos drill:** `chaos/scenarios/eel-pod-kill-field-mobile-svc.yaml`

---

## governance-svc

**Normal:** governance rules and flag management fully operational.

**Degraded:** flag changes blocked except by Security Officer; existing flags enforced. Trigger:

```bash
kubectl set env deployment/governance-svc GOV_MODE=degraded -n eel-system
```

**Read-only:** no flag changes; current governance state queryable. Trigger:

```bash
kubectl set env deployment/governance-svc GOV_MODE=readonly -n eel-system
```

**Chaos drill:** `chaos/scenarios/eel-pod-kill-governance-svc.yaml`

---

## inventory-svc

**Normal:** device and supply inventory fully operational; check-in/check-out and custody events active.

**Degraded:** custody events queued; read queries answered from cache. Trigger:

```bash
kubectl set env deployment/inventory-svc INV_MODE=degraded -n eel-system
```

**Read-only:** no custody changes; inventory state queryable. Trigger:

```bash
kubectl set env deployment/inventory-svc INV_MODE=readonly -n eel-system
```

**Chaos drill:** `chaos/scenarios/eel-pod-kill-inventory-svc.yaml`

---

## portal-svc

**Normal:** public portal fully operational; voter information and results queryable.

**Degraded:** dynamic content served from cache; live result updates paused. Trigger:

```bash
kubectl set env deployment/portal-svc PORTAL_MODE=degraded -n eel-system
```

**Read-only:** static cached content only; no live queries to backend. Trigger:

```bash
kubectl set env deployment/portal-svc PORTAL_MODE=readonly -n eel-system
```

**Chaos drill:** `chaos/scenarios/eel-pod-kill-portal-svc.yaml`

---

## queue-svc

**Normal:** event queue relay fully operational; all outbox events relayed to Kafka.

**Degraded:** relay rate throttled; backpressure applied to producers. Trigger:

```bash
kubectl set env deployment/queue-svc QUEUE_MODE=degraded -n eel-system
```

**Read-only:** no new events enqueued; queue state queryable. Trigger:

```bash
kubectl set env deployment/queue-svc QUEUE_MODE=readonly -n eel-system
```

**Chaos drill:** `chaos/scenarios/eel-pod-kill-queue-svc.yaml`

---

## rts-svc

**Normal:** results transmission fully operational; tally events and declarations processed in real-time.

**Degraded:** declarations queued; tally queries answered from cache; live updates paused. Trigger:

```bash
kubectl set env deployment/rts-svc RTS_MODE=degraded -n eel-system
```

**Read-only:** no new tally submissions or declarations; results queryable. Trigger:

```bash
kubectl set env deployment/rts-svc RTS_MODE=readonly -n eel-system
```

**Chaos drill:** `chaos/scenarios/eel-pod-kill-rts-svc.yaml`

---

## support-admin-svc

**Normal:** support administration fully operational; ticket management and escalation active.

**Degraded:** non-critical admin features disabled; escalation and read queries available. Trigger:

```bash
kubectl set env deployment/support-admin-svc SUPPORT_MODE=degraded -n eel-system
```

**Read-only:** no ticket creation or updates; existing tickets queryable. Trigger:

```bash
kubectl set env deployment/support-admin-svc SUPPORT_MODE=readonly -n eel-system
```

**Chaos drill:** `chaos/scenarios/eel-pod-kill-support-admin-svc.yaml`

---

## training-svc

**Normal:** LMS fully operational; content delivery, assessments, and cert issuance active.

**Degraded:** new cert issuance paused; content delivery from cache; assessments queued. Trigger:

```bash
kubectl set env deployment/training-svc TRAINING_MODE=degraded -n eel-system
```

**Read-only:** no new enrollments or assessments; completed cert status queryable. Trigger:

```bash
kubectl set env deployment/training-svc TRAINING_MODE=readonly -n eel-system
```

**Chaos drill:** `chaos/scenarios/eel-pod-kill-training-svc.yaml`

---

## transparency-svc

**Normal:** tlog ingest and query fully operational; all leaves accepted and witnesses notified.

**Degraded:** submission gate paused (per `witness-divergence-response.md`); leaf queries still served. Trigger:

```bash
curl -sf -X POST \
  http://transparency-svc.eel-system.svc.cluster.local:9001/admin/submission-gate \
  -H "Content-Type: application/json" \
  -d '{"state":"paused","reason":"degraded operation"}'
```

**Read-only:** no new leaves accepted; all query endpoints operational. This is the same as the paused state above; resume with:

```bash
curl -sf -X POST \
  http://transparency-svc.eel-system.svc.cluster.local:9001/admin/submission-gate \
  -H "Content-Type: application/json" \
  -d '{"state":"open"}'
```

**Chaos drill:** `chaos/scenarios/eel-pod-kill-transparency-svc.yaml`

---

## vprs-svc

**Normal:** voter and party registration fully operational; all registration and amendment workflows active.

**Degraded:** amendments queued; registration queries answered from cache. Trigger:

```bash
kubectl set env deployment/vprs-svc VPRS_MODE=degraded -n eel-system
```

**Read-only:** no registrations or amendments; current voter and party records queryable. Trigger:

```bash
kubectl set env deployment/vprs-svc VPRS_MODE=readonly -n eel-system
```

**Chaos drill:** `chaos/scenarios/eel-pod-kill-vprs-svc.yaml`
