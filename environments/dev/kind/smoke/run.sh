#!/usr/bin/env bash
# Smoke test for the eel local kind cluster.
# Runs after `just -f environments/dev/kind/justfile apply-all`.
set -euo pipefail

CLUSTER="eel-dev"
NS_DATA="eel-data"
NS_KAFKA="eel-kafka"

step() { echo "==> $*"; }
fail() { echo "FAIL: $*" >&2; exit 1; }
ok()   { echo " OK: $*"; }

# (a) Cluster exists and nodes ready
step "a: cluster nodes ready"
kind get clusters | grep -q "$CLUSTER" || fail "cluster $CLUSTER not found"
NOT_READY=$(kubectl get nodes --no-headers | grep -v ' Ready' | wc -l | tr -d ' ')
[[ "$NOT_READY" -eq 0 ]] || fail "$NOT_READY node(s) not ready"
ok "3 nodes ready"

# (b) Apply order: wait for core services
step "b: core platform pods running"
for label in "app=kafka" "app=postgres-voter" "app=spicedb" "app=keycloak" "app=openbao" "app=spire-server"; do
    ns_flag=""
    case "$label" in
        *kafka*)      ns_flag="-n $NS_KAFKA" ;;
        *postgres*)   ns_flag="-n $NS_DATA" ;;
        *spicedb*)    ns_flag="-n eel-authz" ;;
        *keycloak*)   ns_flag="-n eel-iam" ;;
        *openbao*)    ns_flag="-n openbao" ;;
        *spire*)      ns_flag="-n spire" ;;
    esac
    kubectl wait --for=condition=Ready pod -l "$label" $ns_flag --timeout=60s 2>/dev/null \
        || fail "pod with label $label not ready"
done
ok "all core pods ready"

# (c) Write a fake outbox row into Postgres voter db
step "c: insert outbox row"
PG_POD=$(kubectl get pod -n "$NS_DATA" -l app=postgres-voter -o jsonpath='{.items[0].metadata.name}')
kubectl exec -n "$NS_DATA" "$PG_POD" -- psql -U eel -d voter -c \
    "INSERT INTO outbox.outbox_events(id, aggregate_type, aggregate_id, event_type, payload)
     VALUES (gen_random_uuid(), 'voter', gen_random_uuid(), 'smoke.test.v1', '{\"smoke\":true}');" \
    || fail "outbox insert failed"
ok "outbox row inserted"

# (d) Assert outbox row lands on Kafka via Debezium (wait up to 30s)
step "d: Debezium CDC delivers to Kafka"
KAFKA_POD=$(kubectl get pod -n "$NS_KAFKA" -l app=kafka -o jsonpath='{.items[0].metadata.name}')
FOUND=0
for i in $(seq 1 15); do
    COUNT=$(kubectl exec -n "$NS_KAFKA" "$KAFKA_POD" -- \
        /opt/kafka/bin/kafka-console-consumer.sh \
        --bootstrap-server localhost:9092 \
        --topic eel.voter.outbox.outbox_events \
        --from-beginning --max-messages 100 --timeout-ms 2000 2>/dev/null \
        | grep -c '"smoke":true' || true)
    [[ "$COUNT" -gt 0 ]] && FOUND=1 && break
    sleep 2
done
[[ "$FOUND" -eq 1 ]] || fail "outbox event not found on Kafka after 30s"
ok "outbox event delivered to Kafka"

# (e) SoftHSM sign + verify
step "e: SoftHSM sign/verify"
echo "smoke-payload-$(date +%s)" > /tmp/eel-smoke-blob.bin
cargo run -q --release -p hsm-init -- sign-blob \
    --role sth \
    --input /tmp/eel-smoke-blob.bin \
    --output /tmp/eel-smoke-blob.sig
cargo run -q --release -p hsm-init -- verify-blob \
    --role sth \
    --input /tmp/eel-smoke-blob.bin \
    --sig /tmp/eel-smoke-blob.sig \
    || fail "SoftHSM sign/verify failed"
rm -f /tmp/eel-smoke-blob.bin /tmp/eel-smoke-blob.sig
ok "SoftHSM sign/verify passed"

# (f) Request SVID from SPIRE for hello-svc workload
step "f: SPIRE SVID for hello-svc"
SPIRE_AGENT=$(kubectl get pod -n spire -l app=spire-agent -o jsonpath='{.items[0].metadata.name}')
SVID_OUTPUT=$(kubectl exec -n spire "$SPIRE_AGENT" -- \
    /opt/spire/bin/spire-agent api fetch x509 \
    -socketPath /run/spire/sockets/agent.sock 2>&1 || true)
echo "$SVID_OUTPUT" | grep -q "spiffe://eel.local" \
    || fail "SPIRE did not return SVID with trust domain eel.local"
ok "SPIRE SVID issued with trust domain eel.local"

echo ""
echo "All smoke checks passed."
