# Surge procedure: T-1 and T-0 scaling

## Overview

Peak check-in load occurs at station opening (T-0: 06:00 local) and around the midday refresh (T-0: 12:00 local). T-1 (the day before) sees administrative pre-check-in traffic. This procedure covers horizontal scaling of `field-mobile-svc` and supporting services ahead of the surge.

## T-1 preparation (day before election)

```bash
# scale field-mobile-svc to pre-surge replicas:
kubectl scale deployment/field-mobile-svc -n eel-system --replicas=20

# scale queue-svc to handle increased outbox throughput:
kubectl scale deployment/queue-svc -n eel-system --replicas=10

# verify HPA limits are set correctly:
kubectl get hpa -n eel-system field-mobile-svc
# minReplicas should be 20, maxReplicas should be 60

# pre-warm Valkey cache with station configuration:
curl -sf -X POST \
  http://field-mobile-svc.eel-system.svc.cluster.local:8080/admin/cache-warm \
  -H "Content-Type: application/json" \
  -d '{"scope":"stations","county_ids":"all"}'
```

## T-0 morning surge (05:00 to 08:00 local)

```bash
# verify current replica count:
kubectl get deployment/field-mobile-svc -n eel-system -o jsonpath='{.spec.replicas}'

# monitor check-in rate:
watch -n 5 'curl -sf http://field-mobile-svc.eel-system.svc.cluster.local:9090/metrics \
  | grep eel_checkins_total'

# monitor Kafka consumer lag:
kafka-consumer-groups.sh --bootstrap-server $KAFKA_BOOTSTRAP \
  --describe --group eel-dispatch-relay

# if lag > 1000 events, scale queue-svc:
kubectl scale deployment/queue-svc -n eel-system --replicas=20
```

## Automatic scale-out (HPA)

The HPA for `field-mobile-svc` is configured to scale on `eel_checkin_rps` (custom metric via Prometheus adapter):

- Target: 500 check-ins per second per replica.
- Min replicas: 20 (T-1 through T-0).
- Max replicas: 60.
- Scale-up stabilisation: 30 seconds.
- Scale-down stabilisation: 10 minutes (prevents thrashing during midday lull).

## Post-surge scale-down (after T-0: 19:00 local, polls closed)

```bash
# reduce to post-election replicas:
kubectl scale deployment/field-mobile-svc -n eel-system --replicas=5
kubectl scale deployment/queue-svc -n eel-system --replicas=3
# confirm Kafka consumer lag is zero before scaling down:
kafka-consumer-groups.sh --bootstrap-server $KAFKA_BOOTSTRAP \
  --describe --group eel-dispatch-relay | grep LAG
```

## Escalation during surge

If check-in rate exceeds HPA capacity or errors spike:

1. Page SRE on-call via PagerDuty.
2. Invoke `docs/runbooks/incident-response-p0.md` if write availability drops below 99%.
3. Activate USSD and SMS fallback channels (`comms-tree.md`).
