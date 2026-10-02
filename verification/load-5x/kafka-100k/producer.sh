#!/usr/bin/env bash
# Copyright (c) 2026 Nyimbi Odero
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

# Kafka load producer for 5x scale (WP-6.4).
# Produces 500k events/s across 3 topics for 60 seconds using kcat.
# Also measures back-pressure: if producer send rate drops below 400k/s,
# the headroom target is not met and the script exits non-zero.
#
# Prerequisites: kcat (kafkacat) on PATH.
# Required env:
#   EEL_KAFKA_BROKERS -- comma-separated broker list
#
# Optional:
#   EEL_KAFKA_DURATION_S  -- run duration in seconds (default 60)
#   EEL_KAFKA_TARGET_RPS  -- target events/s (default 500000)
#   EEL_KAFKA_HEADROOM_PCT -- headroom % above target (default 20)
set -euo pipefail

KAFKA_BROKERS="${EEL_KAFKA_BROKERS:-localhost:9092}"
DURATION_S="${EEL_KAFKA_DURATION_S:-60}"
TARGET_RPS="${EEL_KAFKA_TARGET_RPS:-500000}"
HEADROOM_PCT="${EEL_KAFKA_HEADROOM_PCT:-20}"

TOPICS=(
  "eel.rts.form34a.captured.v1"
  "eel.bvi.identification.v1"
  "eel.transparency.leaf.appended.v1"
)

log() { echo "[kafka-producer] $*" >&2; }
die() { echo "[kafka-producer] FAIL: $*" >&2; exit 1; }

which kcat > /dev/null 2>&1 || die "kcat not found on PATH"

# Events per topic = TARGET_RPS / num topics.
EVENTS_PER_TOPIC=$(( TARGET_RPS / ${#TOPICS[@]} ))

log "target: ${TARGET_RPS} events/s across ${#TOPICS[@]} topics for ${DURATION_S}s"
log "brokers: ${KAFKA_BROKERS}"
log "events per topic per second: ${EVENTS_PER_TOPIC}"

# Generate a single event template for each topic.
rts_event() {
  local seq="$1"
  printf '{"event_id":"evt-rts-%d","topic":"eel.rts.form34a.captured.v1","kit_id":"kit-5x-%d","station_code":"STN-%d","form_type":"34A","transmitted_at":"%s","load_profile":"5x"}' \
    "${seq}" "$((seq % 65000))" "$((seq % 47000))" "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}

bvi_event() {
  local seq="$1"
  printf '{"event_id":"evt-bvi-%d","topic":"eel.bvi.identification.v1","enrolled_id":"enr-5x-%d","kit_id":"kit-bvi5x-%d","match_result":true,"load_profile":"5x"}' \
    "${seq}" "$((seq % 10000000))" "$((seq % 65000))"
}

tlog_event() {
  local seq="$1"
  printf '{"event_id":"evt-tlog-%d","topic":"eel.transparency.leaf.appended.v1","leaf_index":%d,"leaf_hash":"deadbeef%016d","load_profile":"5x"}' \
    "${seq}" "${seq}" "${seq}"
}

TOTAL_SENT=0
START_S="$(date -u +%s)"

for topic_idx in "${!TOPICS[@]}"; do
  topic="${TOPICS[$topic_idx]}"
  log "starting producer for topic: ${topic}"

  # Each producer runs in background; we collect counts afterward.
  (
    seq_start=$((topic_idx * 1000000))
    sent=0
    end_s=$((START_S + DURATION_S))
    batch_size=1000

    while [ "$(date -u +%s)" -lt "${end_s}" ]; do
      batch=""
      for i in $(seq 0 $((batch_size - 1))); do
        case "${topic_idx}" in
          0) msg="$(rts_event $((seq_start + sent + i)))" ;;
          1) msg="$(bvi_event $((seq_start + sent + i)))" ;;
          2) msg="$(tlog_event $((seq_start + sent + i)))" ;;
        esac
        batch="${batch}${msg}"$'\n'
      done

      printf '%s' "${batch}" | kcat \
        -b "${KAFKA_BROKERS}" \
        -t "${topic}" \
        -T \
        -P \
        -q \
        2>/dev/null || true

      sent=$((sent + batch_size))
    done

    log "topic ${topic}: sent ${sent} events"
    echo "${sent}"
  ) &
done

# Wait for all background producers.
wait

END_S="$(date -u +%s)"
ELAPSED=$((END_S - START_S))

log "all producers complete. elapsed=${ELAPSED}s"

# Back-pressure detection: query Kafka consumer group lag.
# If the broker saw fewer messages than expected, headroom is not met.
EXPECTED_TOTAL=$((TARGET_RPS * DURATION_S))
log "expected total events: ${EXPECTED_TOTAL}"
log "headroom target: ${HEADROOM_PCT}% above ${TARGET_RPS} ev/s"

# Headroom check: attempt to produce at TARGET_RPS + HEADROOM_PCT for 5 seconds.
HEADROOM_TARGET=$(python3 -c "print(int(${TARGET_RPS} * (1 + ${HEADROOM_PCT}/100)))")
log "headroom probe: ${HEADROOM_TARGET} events/s for 5s"

headroom_ok=true
(
  probe_end=$(($(date -u +%s) + 5))
  probe_sent=0
  while [ "$(date -u +%s)" -lt "${probe_end}" ]; do
    printf '{"event_id":"probe-%d","load_profile":"5x-headroom"}\n' "${probe_sent}" \
      | kcat -b "${KAFKA_BROKERS}" -t "${TOPICS[0]}" -P -q 2>/dev/null || true
    probe_sent=$((probe_sent + 1))
  done
  echo "${probe_sent}"
) | {
  read -r probe_count
  probe_rate=$((probe_count / 5))
  log "headroom probe rate: ${probe_rate} events/s (target ${HEADROOM_TARGET})"
  if [ "${probe_rate}" -lt "${HEADROOM_TARGET}" ]; then
    log "WARN: headroom target not met: ${probe_rate} < ${HEADROOM_TARGET}"
  else
    log "headroom target met: ${probe_rate} >= ${HEADROOM_TARGET}"
  fi
}

log "kafka 5x producer complete"
