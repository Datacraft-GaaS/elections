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
 * k6 load test: abis-svc
 * WP-3.6, M03, EC-SCL-07
 *
 * Scenario: 10k enrolments/hour sustained + 100 match ops/sec at 1x.
 * Maps to local.yaml 1/100 scale: 100 enrolments/hour + 1 match ops/sec.
 *
 * Run: k6 run --env ABIS_URL=http://localhost:8080 abis.js
 */
import http from "k6/http";
import { check, sleep } from "k6";
import { Rate, Counter } from "k6/metrics";

const matchFailRate = new Rate("match_fail_rate");
const matchCount = new Counter("match_requests_total");

const BASE_URL = __ENV.ABIS_URL || "http://localhost:8080";

export const options = {
  scenarios: {
    // Sustained 100 enrolment match ops/hour = ~0.028/sec; round to 1 VU with long sleep
    enrolment_1n: {
      executor: "constant-vus",
      vus: 1,
      duration: "60s",
      exec: "enrolment1n",
    },
    // 1 match op/sec (1x scale of 100/sec)
    match_ops: {
      executor: "constant-arrival-rate",
      rate: 1,
      timeUnit: "1s",
      duration: "60s",
      preAllocatedVUs: 5,
      exec: "matchOp",
    },
  },
  thresholds: {
    http_req_duration: ["p(95)<5000", "p(99)<8000"],
    match_fail_rate: ["rate<0.01"],
  },
};

function randomId() {
  return Math.random().toString(36).substring(2, 10);
}

export function enrolment1n() {
  const payload = JSON.stringify({
    probe_ref: `bio-store://eel.local/probes/${randomId()}`,
    modality: "fingerprint",
    caller_id: "spiffe://eel.local/ns/bvr/sa/bvr-backend",
    election_id: "00000000-0000-7000-8000-000000000001",
  });
  const params = {
    headers: {
      "Content-Type": "application/json",
      "Idempotency-Key": randomId(),
    },
  };
  const res = http.post(`${BASE_URL}/api/v1/abis/match/1n`, payload, params);
  const ok = check(res, {
    "1n match 202": (r) => r.status === 202,
  });
  matchFailRate.add(!ok);
  matchCount.add(1);
  sleep(36); // ~100/hour = one every 36 seconds
}

export function matchOp() {
  const payload = JSON.stringify({
    probe_ref: `bio-store://eel.local/probes/${randomId()}`,
    gallery_ref: `bio-store://eel.local/gallery/voter-${randomId()}`,
    modality: "iris",
    caller_id: "spiffe://eel.local/ns/bvi/sa/bvi-backend",
  });
  const params = {
    headers: {
      "Content-Type": "application/json",
      "Idempotency-Key": randomId(),
    },
  };
  const res = http.post(`${BASE_URL}/api/v1/abis/match/11`, payload, params);
  const ok = check(res, {
    "11 match 200": (r) => r.status === 200,
  });
  matchFailRate.add(!ok);
  matchCount.add(1);
}
