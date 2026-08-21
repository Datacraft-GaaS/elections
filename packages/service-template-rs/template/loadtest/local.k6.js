// k6 load test stub for {{project-name}}.
// Run: k6 run loadtest/local.k6.js -e BASE_URL=http://localhost:8080
import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
  vus: 10,
  duration: '30s',
  thresholds: {
    // SLA: p95 < 200ms, error rate < 1%.
    http_req_duration: ['p(95)<200'],
    http_req_failed: ['rate<0.01'],
  },
};

const BASE = __ENV.BASE_URL || 'http://localhost:8080';

export default function () {
  const r = http.get(`${BASE}/healthz`);
  check(r, { 'healthz 200': (res) => res.status === 200 });
  sleep(0.1);
}
