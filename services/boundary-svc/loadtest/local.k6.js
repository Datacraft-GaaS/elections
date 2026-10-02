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

import http from 'k6/http';
import { check, sleep } from 'k6';

// boundary-svc load test.
// Run: k6 run loadtest/local.k6.js
// Target: HTTP_BASE_URL env var, default http://localhost:8080

const BASE = __ENV.HTTP_BASE_URL || 'http://localhost:8080';

export const options = {
  stages: [
    { duration: '30s', target: 20 },
    { duration: '60s', target: 50 },
    { duration: '30s', target: 0 },
  ],
  thresholds: {
    http_req_duration: ['p(95)<100', 'p(99)<300'],
    http_req_failed: ['rate<0.01'],
  },
};

export default function () {
  const res = http.get(`${BASE}/boundary/constituencies`);
  check(res, {
    'constituencies 200': (r) => r.status === 200,
    'has snapshot_id': (r) => r.json('snapshot_id') !== undefined,
  });
  sleep(0.1);

  const res2 = http.get(`${BASE}/boundary/counties`);
  check(res2, { 'counties 200': (r) => r.status === 200 });
  sleep(0.1);

  const res3 = http.get(`${BASE}/boundary/stations`);
  check(res3, { 'stations 200': (r) => r.status === 200 });
  sleep(0.1);
}
