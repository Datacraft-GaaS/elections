# Kit mass revocation: SPIRE SVID and step-ca CRL propagation

## Purpose

Revoke SVIDs for all BVR/BVI/RTS kits in a workload class and publish a new CRL via step-ca. CRL propagation to all edge agents must complete within 60 seconds. Kits with revoked SVIDs enter offline-only mode and cannot submit events to the backend.

## Prerequisites

- `spire-server` CLI configured with admin access to the SPIRE server pod.
- `step` CLI (step-ca admin) configured with `CA_URL` and `CA_FINGERPRINT` environment variables.
- The kit workload class to revoke is identified (e.g., `spiffe://eel.io/kit/bvr` or a specific workload selector).
- Access to edge agent status dashboards (or `kubectl exec` on agent pods).
- An incident ticket open as `<TICKET>`.
- Two authorised officers present (revocation affects all kits in the class).

## Commands

### Step 1: identify SVIDs to revoke

```bash
kubectl exec -n eel-system deploy/spire-server -- \
  spire-server entry show -selector "eel:kit-class:bvr"
# note the entry IDs and SPIFFE IDs to be revoked
```

### Step 2: revoke SPIRE entries for the kit workload class

```bash
# revoke all entries matching the kit class:
kubectl exec -n eel-system deploy/spire-server -- \
  spire-server entry delete \
  -entryID <ENTRY_ID_1> -entryID <ENTRY_ID_2>
# for a full class revocation, use the selector:
kubectl exec -n eel-system deploy/spire-server -- \
  spire-server bundle show -format spiffe | grep "eel:kit-class"
```

### Step 3: mark SVIDs as revoked in step-ca CRL

```bash
# get the serial numbers of the kit certificates:
step ca certificate list --all | grep "kit-class:bvr"
# revoke each certificate:
step ca revoke --reasonCode 0 <SERIAL_1>
step ca revoke --reasonCode 0 <SERIAL_2>
# (repeat for all serials in the class)
```

### Step 4: publish new CRL

```bash
step ca crl --out /tmp/eel-crl.pem
# distribute to all edge agents via ConfigMap:
kubectl create configmap eel-crl \
  --from-file=crl.pem=/tmp/eel-crl.pem \
  --dry-run=client -o yaml \
  | kubectl apply -n eel-system -f -
```

### Step 5: verify CRL propagation within 60 seconds

```bash
# start a timer:
START=$(date +%s)
# poll agent CRL version:
watch -n 2 'kubectl exec -n eel-system ds/spire-agent -- \
  spire-agent healthcheck'
# check edge nodes directly:
for agent_pod in $(kubectl get pods -n eel-system -l app=spire-agent -o name); do
  kubectl exec -n eel-system $agent_pod -- \
    curl -sf http://localhost:9980/health | grep '"svid_valid"'
done
END=$(date +%s)
echo "Propagation time: $((END - START))s"
# must be <= 60
```

### Step 6: confirm kits enter offline-only mode

```bash
# kits with revoked SVIDs should stop appearing in the backend submission logs:
kubectl logs -n eel-system deploy/bvr-svc --tail=50 | grep "SVID_REVOKED\|offline"
```

### Step 7: emit tlog entry

```bash
grpcurl -plaintext \
  -d "{\"event_type\":\"meta-events.kit-mass-revocation.v1\",\"payload\":{\"incident_ticket\":\"<TICKET>\",\"kit_class\":\"<CLASS>\",\"propagation_seconds\":\"<SECONDS>\",\"officer_a\":\"<ID_A>\",\"officer_b\":\"<ID_B>\",\"timestamp\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"}}" \
  transparency-svc.eel-system.svc.cluster.local:9000 \
  eel.tlog.v1.TlogIngest/Append
```

## Verification steps

1. `spire-server entry show -selector "eel:kit-class:<CLASS>"` returns no entries.
2. CRL propagation time (step 5) is below 60 seconds.
3. Backend logs show no new submissions from the revoked kit class.
4. step-ca CRL contains the revoked serials (`step ca crl | openssl crl -noout -text`).
5. Transparency log contains the kit-mass-revocation leaf.

## Rollback procedure

Revocation is permanent. To restore kit access, issue new SPIRE entries and new step-ca certificates for the re-provisioned kits. Follow the kit provisioning runbook (referenced in `operations/mdm/lifecycle.md`). Do not re-issue to the same hardware if the revocation reason was theft or compromise.

## Escalation contacts

- Security Officer (contact via encrypted comms)
- Site Reliability Lead (on-call via PagerDuty rotation)
- Field Operations Lead (on-call via PagerDuty rotation) for coordination with field teams

## Estimated duration

10 to 20 minutes for revocation. CRL propagation must complete within 60 seconds of publication.

## Chaos-drill reference

This runbook is exercised by: `chaos/scenarios/eel-svid-revocation.yaml` and `chaos/scenarios/eel-cert-expiry-15min.yaml`
