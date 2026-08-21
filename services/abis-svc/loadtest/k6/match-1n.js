/**
 * k6 load test: 1:N match against 100k gallery.
 * Target: 100 concurrent VUs; p95 < 5s; p99 < 8s.
 * ADR-0018, SLA section "1:N match".
 *
 * Run: k6 run --env ABIS_URL=http://localhost:8080 k6/match-1n.js
 */
import http from "k6/http";
import { check, sleep } from "k6";
import { Rate, Trend } from "k6/metrics";
import { SharedArray } from "k6/data";

const matchFail = new Rate("eel_match1n_fail");
const matchDur  = new Trend("eel_match1n_duration_ms", true);

const TEMPLATES = new SharedArray("templates", function () {
  return open("../fixtures/synthetic-templates/templates.jsonl")
    .trim()
    .split("\n")
    .map((l) => JSON.parse(l));
});

const BASE_URL = __ENV.ABIS_URL || "http://localhost:8080";

export const options = {
  scenarios: {
    match_1n: {
      executor: "ramping-vus",
      startVUs: 10,
      stages: [
        { target: 100, duration: "5m" },
        { target: 100, duration: "20m" },
        { target: 0,   duration: "5m" },
      ],
    },
  },
  thresholds: {
    eel_match1n_duration_ms: ["p(95)<5000", "p(99)<8000"],
    eel_match1n_fail: ["rate<0.005"],
    http_req_failed: ["rate<0.01"],
  },
};

function pickTemplate() {
  return TEMPLATES[Math.floor(Math.random() * TEMPLATES.length)];
}

export default function () {
  const tmpl = pickTemplate();
  const payload = JSON.stringify({
    probe_ref: `bio-store://eel.local/probes/${tmpl.id}`,
    template_blob: tmpl.fingerprint_b64,
    modality: "fingerprint",
    caller_id: "spiffe://eel.local/ns/abis/sa/abis-api",
    election_id: "00000000-0000-7000-8000-000000000001",
    max_candidates: 20,
  });
  const params = {
    headers: {
      "Content-Type": "application/json",
      "Idempotency-Key": `eel-1n-${Date.now()}-${Math.random().toString(36).slice(2)}`,
    },
  };

  const start = Date.now();
  const res = http.post(`${BASE_URL}/api/v1/abis/match/1n`, payload, params);
  matchDur.add(Date.now() - start);

  const ok = check(res, {
    "1n accepted 202": (r) => r.status === 202,
  });
  matchFail.add(!ok);
}
