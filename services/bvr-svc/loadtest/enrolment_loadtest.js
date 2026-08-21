// k6 load test: voter enrolment at 1 000 registrations/sec target.
// Local SLA: 10/sec (1x). Adjust TARGET_RPS for full load.
// Run: k6 run enrolment_loadtest.js

import http from "k6/http";
import { check, sleep } from "k6";
import { Counter, Rate, Trend } from "k6/metrics";

const TARGET_RPS = __ENV.TARGET_RPS ? parseInt(__ENV.TARGET_RPS) : 10;
const BASE_URL = __ENV.BASE_URL || "http://localhost:8080";

const enrolmentErrors = new Counter("enrolment_errors");
const enrolmentRate   = new Rate("enrolment_success_rate");
const enrolmentDuration = new Trend("enrolment_duration_ms");

export const options = {
  scenarios: {
    voter_enrolments: {
      executor: "constant-arrival-rate",
      rate: TARGET_RPS,
      timeUnit: "1s",
      duration: "60s",
      preAllocatedVUs: TARGET_RPS * 2,
      maxVUs: TARGET_RPS * 10,
    },
  },
  thresholds: {
    http_req_duration:      ["p(95)<500", "p(99)<1000"],
    enrolment_success_rate: ["rate>0.99"],
    http_req_failed:        ["rate<0.01"],
  },
};

let voterSeq = 0;

function makeEnrolment() {
  const seq = ++voterSeq;
  return JSON.stringify({
    idempotency_key: `lt-${__VU}-${seq}-${Date.now()}`,
    national_id:     `KE${String(seq).padStart(8, "0")}`,
    full_name:       `Test Voter ${seq}`,
    dob:             "1990-01-01",
    county_code:     String(((seq - 1) % 47) + 1).padStart(3, "0"),
    polling_station_id: "001-001-001",
    election_scope:  "2027-general",
    biometric_template_refs: [],
  });
}

export default function () {
  const start = Date.now();
  const res = http.post(
    `${BASE_URL}/api/v1/bvr/enrolments`,
    makeEnrolment(),
    { headers: { "Content-Type": "application/json" } }
  );

  const ok = check(res, {
    "status 202":   (r) => r.status === 202,
    "has voter_id": (r) => {
      try { return !!JSON.parse(r.body).voter_id; } catch { return false; }
    },
  });

  enrolmentDuration.add(Date.now() - start);
  if (ok) {
    enrolmentRate.add(1);
  } else {
    enrolmentErrors.add(1);
    enrolmentRate.add(0);
  }
}
