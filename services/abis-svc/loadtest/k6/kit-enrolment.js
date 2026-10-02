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
 * k6 load test: kit enrolment with attestation header.
 * Target: 500 concurrent kit enrolments; p95 < 3s.
 * ADR-0018, WP-3.6, kit-facing path.
 *
 * Run: k6 run --env ABIS_URL=http://localhost:8080 k6/kit-enrolment.js
 */
import http from "k6/http";
import { check } from "k6";
import { Rate, Trend } from "k6/metrics";
import { SharedArray } from "k6/data";

const kitFail = new Rate("eel_kit_enrolment_fail");
const kitDur  = new Trend("eel_kit_enrolment_duration_ms", true);

const TEMPLATES = new SharedArray("templates", function () {
  return open("../fixtures/synthetic-templates/templates.jsonl")
    .trim()
    .split("\n")
    .map((l) => JSON.parse(l));
});

const BASE_URL = __ENV.ABIS_URL || "http://localhost:8080";
// Synthetic attestation token. Real kit provides TDX/SEV report; load test uses dev token.
const ATTEST_HDR = __ENV.ATTEST_TOKEN || "dev-attest-placeholder";

export const options = {
  scenarios: {
    kit_enrolment: {
      executor: "ramping-vus",
      startVUs: 50,
      stages: [
        { target: 500, duration: "3m" },
        { target: 500, duration: "10m" },
        { target: 0,   duration: "2m" },
      ],
    },
  },
  thresholds: {
    eel_kit_enrolment_duration_ms: ["p(95)<3000"],
    eel_kit_enrolment_fail: ["rate<0.01"],
    http_req_failed: ["rate<0.01"],
  },
};

function pickTemplate() {
  return TEMPLATES[Math.floor(Math.random() * TEMPLATES.length)];
}

export default function () {
  const tmpl = pickTemplate();
  const payload = JSON.stringify({
    probe_ref: `bio-store://eel.local/kit-probes/${tmpl.id}`,
    template_blob: tmpl.fingerprint_b64,
    iris_blob: tmpl.iris_b64 || tmpl.fingerprint_b64,
    modality: "fingerprint+iris",
    caller_id: "spiffe://eel.local/ns/kit/sa/enrolment-kit",
    election_id: "00000000-0000-7000-8000-000000000001",
    subject_ref: `subject:kit-${tmpl.id}`,
    kit_serial: `EKLK-${tmpl.id.slice(0, 8).toUpperCase()}`,
  });
  const params = {
    headers: {
      "Content-Type": "application/json",
      "Idempotency-Key": `eel-kit-${Date.now()}-${Math.random().toString(36).slice(2)}`,
      "X-Eel-Attestation": ATTEST_HDR,
      "X-Eel-Kit-Serial": `EKLK-${tmpl.id.slice(0, 8).toUpperCase()}`,
    },
  };

  const start = Date.now();
  const res = http.post(`${BASE_URL}/api/v1/abis/match/1n`, payload, params);
  kitDur.add(Date.now() - start);

  const ok = check(res, {
    "kit enrolment 202": (r) => r.status === 202,
  });
  kitFail.add(!ok);
}
