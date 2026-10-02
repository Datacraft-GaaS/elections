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
 * k6 load test: 1:1 match (identity verification).
 * Target: 500 rps sustained; p95 < 100ms; p99 < 200ms.
 * ADR-0018, SLA "1:1 match".
 *
 * Run: k6 run --env ABIS_URL=http://localhost:8080 k6/match-11.js
 */
import http from "k6/http";
import { check } from "k6";
import { Rate, Trend } from "k6/metrics";
import { SharedArray } from "k6/data";

const matchFail = new Rate("eel_match11_fail");
const matchDur  = new Trend("eel_match11_duration_ms", true);

const TEMPLATES = new SharedArray("templates", function () {
  return open("../fixtures/synthetic-templates/templates.jsonl")
    .trim()
    .split("\n")
    .map((l) => JSON.parse(l));
});

const BASE_URL = __ENV.ABIS_URL || "http://localhost:8080";

export const options = {
  scenarios: {
    match_11: {
      executor: "constant-arrival-rate",
      rate: 500,
      timeUnit: "1s",
      duration: "15m",
      preAllocatedVUs: 100,
      maxVUs: 500,
    },
  },
  thresholds: {
    eel_match11_duration_ms: ["p(95)<100", "p(99)<200"],
    eel_match11_fail: ["rate<0.001"],
    http_req_failed: ["rate<0.005"],
  },
};

function pickTemplate() {
  return TEMPLATES[Math.floor(Math.random() * TEMPLATES.length)];
}

export default function () {
  const probe = pickTemplate();
  const gallery = TEMPLATES[Math.floor(Math.random() * TEMPLATES.length)];
  const payload = JSON.stringify({
    probe_ref: `bio-store://eel.local/probes/${probe.id}`,
    gallery_ref: `bio-store://eel.local/gallery/voter-${gallery.id}`,
    template_blob: probe.fingerprint_b64,
    gallery_blob: gallery.fingerprint_b64,
    modality: "fingerprint",
    caller_id: "spiffe://eel.local/ns/bvi/sa/bvi-backend",
    threshold_override: null,
  });
  const params = {
    headers: {
      "Content-Type": "application/json",
      "Idempotency-Key": `eel-11-${Date.now()}-${Math.random().toString(36).slice(2)}`,
    },
  };

  const start = Date.now();
  const res = http.post(`${BASE_URL}/api/v1/abis/match/11`, payload, params);
  matchDur.add(Date.now() - start);

  const ok = check(res, {
    "11 match 200": (r) => r.status === 200,
  });
  matchFail.add(!ok);
}
