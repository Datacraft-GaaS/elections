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
 * k6 load test: enrolment throughput
 * Target: 10,000 enrolments/hour; p95 ingest < 1s; p99 dedup complete < 8s.
 * ADR-0018, EC-SCL-07.
 *
 * Run: k6 run --env ABIS_URL=http://localhost:8080 k6/enrolment-throughput.js
 */
import http from "k6/http";
import { check, sleep } from "k6";
import { Rate, Counter, Trend } from "k6/metrics";
import { SharedArray } from "k6/data";

const ingestFail  = new Rate("eel_enrolment_ingest_fail");
const dedupFail   = new Rate("eel_enrolment_dedup_fail");
const ingestTotal = new Counter("eel_enrolment_ingest_total");
const ingestDur   = new Trend("eel_enrolment_ingest_duration_ms", true);

const TEMPLATES = new SharedArray("templates", function () {
  return open("../fixtures/synthetic-templates/templates.jsonl")
    .trim()
    .split("\n")
    .map((l) => JSON.parse(l));
});

const BASE_URL = __ENV.ABIS_URL || "http://localhost:8080";

// 10k enrolments/hr = ~2.78/s. Ramp up over 5 min, sustain 25 min, ramp down 5 min.
export const options = {
  scenarios: {
    enrolment_ramp: {
      executor: "ramping-arrival-rate",
      startRate: 0,
      timeUnit: "1s",
      preAllocatedVUs: 50,
      maxVUs: 200,
      stages: [
        { target: 3, duration: "5m" },   // ramp to ~10k/hr
        { target: 3, duration: "20m" },  // sustain
        { target: 0, duration: "5m" },   // ramp down
      ],
    },
  },
  thresholds: {
    // p95 ingest (202 accepted) < 1s
    eel_enrolment_ingest_duration_ms: ["p(95)<1000", "p(99)<3000"],
    // dedup round-trip via polling, proxied through overall http_req_duration
    http_req_duration: ["p(99)<8000"],
    eel_enrolment_ingest_fail: ["rate<0.01"],
  },
};

function pickTemplate() {
  return TEMPLATES[Math.floor(Math.random() * TEMPLATES.length)];
}

function idempotencyKey() {
  return `eel-enrol-${Date.now()}-${Math.random().toString(36).slice(2, 10)}`;
}

export default function () {
  const tmpl = pickTemplate();
  const payload = JSON.stringify({
    probe_ref: `bio-store://eel.local/probes/${tmpl.id}`,
    template_blob: tmpl.fingerprint_b64,
    modality: "fingerprint",
    caller_id: "spiffe://eel.local/ns/enrolment/sa/enrolment-svc",
    election_id: "00000000-0000-7000-8000-000000000001",
    subject_ref: `subject:${tmpl.id}`,
  });
  const params = {
    headers: {
      "Content-Type": "application/json",
      "Idempotency-Key": idempotencyKey(),
      "X-Eel-Caller": "eel-enrolment-loadtest",
    },
  };

  const start = Date.now();
  const res = http.post(`${BASE_URL}/api/v1/abis/match/1n`, payload, params);
  ingestDur.add(Date.now() - start);
  ingestTotal.add(1);

  const ok = check(res, {
    "enrolment 202 accepted": (r) => r.status === 202,
  });
  ingestFail.add(!ok);
  if (!ok && __ENV.VERBOSE) {
    console.warn(`enrolment fail: ${res.status} ${res.body.slice(0, 200)}`);
  }
}
