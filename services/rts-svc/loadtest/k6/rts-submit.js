// k6 load test: RTS form submission
// ADR-0018 SLA: p99 <= 2s at 600 concurrent (1/100 scale of 60k)
import http from "k6/http";
import { check, sleep } from "k6";
import { Rate } from "k6/metrics";

export const errorRate = new Rate("errors");

export const options = {
  scenarios: {
    rts_submit: {
      executor: "constant-vus",
      vus: 600,
      duration: "5m",
    },
  },
  thresholds: {
    http_req_duration: ["p(99)<2000"],
    errors: ["rate<0.01"],
  },
};

const BASE_URL = __ENV.RTS_BASE_URL || "http://localhost:8080";

export default function () {
  const stationCode = `47-${Math.floor(Math.random() * 290)
    .toString()
    .padStart(4, "0")}-${Math.floor(Math.random() * 800)
    .toString()
    .padStart(3, "0")}`;
  const payload = JSON.stringify({
    station_code: stationCode,
    constituency_code: "047",
    county_code: "47",
    registered_voters: 700,
    votes_cast: 500,
    valid_votes: 490,
    rejected_votes: 7,
    spoilt_ballots: 3,
    candidates: [
      { candidate_id: "C001", candidate_name: "Candidate One", votes: 300 },
      { candidate_id: "C002", candidate_name: "Candidate Two", votes: 190 },
    ],
    device_signature: "stub-sig-" + __VU,
  });

  const params = {
    headers: {
      "Content-Type": "application/json",
      "Idempotency-Key": `vu-${__VU}-iter-${__ITER}`,
    },
  };

  const res = http.post(`${BASE_URL}/api/v1/rts/forms`, payload, params);
  const ok = check(res, { "status is 202": (r) => r.status === 202 });
  errorRate.add(!ok);
  sleep(0.1);
}
