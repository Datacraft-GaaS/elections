/**
 * Copyright (c) 2026 Nyimbi Odero
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

/**
 * k6 load test — biometric-edge-svc (WP-3.8)
 *
 * Scenario: 46,000 virtual kits at 1x heartbeat rate.
 * For local/CI execution, scale is reduced to 460 virtual users (1:100).
 * Measure edge-api p99 under sustained heartbeat load.
 *
 * Run:
 *   k6 run --vus 460 --duration 60s loadtest/edge_k6.js
 */

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend } from 'k6/metrics';

const errorRate = new Rate('errors');
const heartbeatLatency = new Trend('heartbeat_latency_ms', true);
const enrolmentLatency = new Trend('enrolment_latency_ms', true);

export const options = {
  vus: 460,
  duration: '60s',
  thresholds: {
    http_req_duration: ['p(99)<1000'], // edge-api SLA p99 < 1 s
    errors: ['rate<0.01'],
  },
};

const BASE_URL = __ENV.EDGE_API_URL || 'http://localhost:8090';

function randomSerial(prefix) {
  return `${prefix}-${Math.floor(Math.random() * 46000).toString().padStart(5, '0')}`;
}

function randomStation() {
  const county = Math.floor(Math.random() * 47) + 1;
  const ward   = Math.floor(Math.random() * 20) + 1;
  const ps     = Math.floor(Math.random() * 50) + 1;
  return `${county.toString().padStart(3, '0')}-${ward.toString().padStart(3, '0')}-${ps.toString().padStart(3, '0')}`;
}

export default function () {
  const isEnrolmentKit = Math.random() < 0.5;
  const serial  = randomSerial(isEnrolmentKit ? 'BVRD-KE' : 'BVID-KE');
  const station = randomStation();
  const now     = new Date().toISOString();

  // Heartbeat (every kit, every iteration).
  const heartbeatPayload = JSON.stringify({
    device_serial: serial,
    station_code:  station,
    timestamp:     now,
    battery_pct:   Math.floor(Math.random() * 100),
    uptime_s:      Math.floor(Math.random() * 86400),
  });

  const hbRes = http.post(`${BASE_URL}/api/v1/edge/telemetry`, heartbeatPayload, {
    headers: { 'Content-Type': 'application/json' },
    tags: { name: 'heartbeat' },
  });
  check(hbRes, { 'heartbeat 202': (r) => r.status === 202 });
  errorRate.add(hbRes.status !== 202);
  heartbeatLatency.add(hbRes.timings.duration);

  // 10% of BVR kits submit an enrolment.
  if (isEnrolmentKit && Math.random() < 0.1) {
    const enrolPayload = JSON.stringify({
      idempotency_key:  `idem-${serial}-${Date.now()}`,
      device_serial:    serial,
      capture_timestamp: now,
      biometric_ref:    `v1:sha256:${Math.random().toString(36).substring(2)}`,
      modality:         'fingerprint',
      quality_score:    Math.floor(60 + Math.random() * 40),
      device_sig:       'stub-sig',
      station_code:     station,
      national_id_hash: `sha256:${Math.random().toString(36).substring(2)}`,
    });

    const enrRes = http.post(`${BASE_URL}/api/v1/edge/enrolment/upload`, enrolPayload, {
      headers: { 'Content-Type': 'application/json' },
      tags: { name: 'enrolment' },
    });
    check(enrRes, { 'enrolment 202': (r) => r.status === 202 });
    errorRate.add(enrRes.status !== 202);
    enrolmentLatency.add(enrRes.timings.duration);
  }

  sleep(1); // 1 heartbeat/s per VU models 46k kits at 1x rate with 460 VUs
}
