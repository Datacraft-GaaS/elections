// k6 load test: 18k req/s BVI 1:1 at 1x, 60 s per SLA.
// Target: p99 <= 100 ms at 18,000 req/s (FR-M06-017, AC-M06-09, EC-SCL-07).
import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate } from 'k6/metrics';

const errorRate = new Rate('errors');

export const options = {
  scenarios: {
    bvi_identification: {
      executor: 'constant-arrival-rate',
      rate: 18000,
      timeUnit: '1s',
      duration: '60s',
      preAllocatedVUs: 500,
      maxVUs: 2000,
    },
  },
  thresholds: {
    http_req_duration: ['p(99)<100'],
    errors: ['rate<0.001'],
  },
};

const BASE_URL = __ENV.BVI_URL || 'http://localhost:8080';

export default function () {
  const payload = JSON.stringify({
    events: [{
      event_id: `${__VU}-${__ITER}-${Date.now()}`,
      voter_id: `voter-${Math.floor(Math.random() * 1000000)}`,
      modality: 'fingerprint',
      outcome: 'matched',
      ts_hlc: new Date().toISOString(),
      kit_serial: `KIT-${__VU}`,
      station_code: `STN-${(__VU % 1000).toString().padStart(4, '0')}`,
    }],
  });

  const res = http.post(`${BASE_URL}/api/v1/bvi/identifications`, payload, {
    headers: { 'Content-Type': 'application/json', 'Idempotency-Key': `${__VU}-${__ITER}` },
  });

  const ok = check(res, {
    'status is 202 or 200': (r) => r.status === 202 || r.status === 200,
    'p99 < 100ms': (r) => r.timings.duration < 100,
  });
  errorRate.add(!ok);
}
