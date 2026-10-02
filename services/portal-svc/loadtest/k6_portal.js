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
 * k6 load test: portal-svc public results portal (M11-LOAD-, WP-3.9).
 *
 * Target SLA: p95 <= 200 ms, p99 <= 500 ms at 5M visitors/hour (AC-M11-01).
 * Local scale: 1/100 (50k/hr = ~14 req/s) with --duration=5m --vus=50.
 *
 * Run: k6 run --vus 50 --duration 5m loadtest/k6_portal.js
 */
import http from 'k6/http'
import { check, sleep } from 'k6'
import { Trend, Rate } from 'k6/metrics'

const p95Latency = new Trend('portal_p95_latency', true)
const p99Latency = new Trend('portal_p99_latency', true)
const errorRate = new Rate('portal_error_rate')

export const options = {
  scenarios: {
    // Homepage: national tally static page (highest volume)
    home_page: {
      executor: 'constant-vus',
      vus: 30,
      duration: '5m',
      exec: 'homeScenario',
    },
    // Snapshot API: /api/v1/portal/snapshots/latest
    snapshot_api: {
      executor: 'constant-vus',
      vus: 10,
      duration: '5m',
      exec: 'snapshotScenario',
    },
    // Merkle proof: /api/v1/portal/snapshots/{id}/verify
    proof_api: {
      executor: 'constant-vus',
      vus: 5,
      duration: '5m',
      exec: 'proofScenario',
    },
    // Bulk download redirect: /api/v1/portal/snapshots/bulk.tar.gz
    bulk_download: {
      executor: 'constant-vus',
      vus: 5,
      duration: '5m',
      exec: 'bulkScenario',
    },
  },
  thresholds: {
    // p95 <= 200 ms, p99 <= 500 ms per SLA (FR-M11-022, AC-M11-01)
    http_req_duration: ['p(95)<200', 'p(99)<500'],
    portal_error_rate: ['rate<0.001'],
  },
}

const BASE_URL = __ENV.PORTAL_BASE_URL || 'http://localhost:8090'
const SAMPLE_UUID = '00000000-0000-0000-0000-000000000000'

export function homeScenario() {
  const res = http.get(`${BASE_URL}/`)
  const ok = check(res, {
    'home 200': (r) => r.status === 200,
    'home < 200ms': (r) => r.timings.duration < 200,
  })
  p95Latency.add(res.timings.duration)
  errorRate.add(!ok)
  sleep(Math.random() * 0.5)
}

export function snapshotScenario() {
  const res = http.get(`${BASE_URL}/api/v1/portal/snapshots/latest`)
  const ok = check(res, {
    'snapshot 200': (r) => r.status === 200,
    'has merkle-root header': (r) => r.headers['X-Eel-Merkle-Root'] !== undefined,
    'has snapshot-id header': (r) => r.headers['X-Eel-Snapshot-Id'] !== undefined,
    'snapshot < 200ms': (r) => r.timings.duration < 200,
  })
  p95Latency.add(res.timings.duration)
  errorRate.add(!ok)
  sleep(Math.random() * 0.2)
}

export function proofScenario() {
  const res = http.get(`${BASE_URL}/api/v1/portal/snapshots/${SAMPLE_UUID}/verify`)
  const ok = check(res, {
    'proof 200': (r) => r.status === 200,
    'proof < 200ms': (r) => r.timings.duration < 200,
    'proof has leaf_hash': (r) => {
      try { return JSON.parse(r.body).leaf_hash !== undefined } catch { return false }
    },
  })
  p99Latency.add(res.timings.duration)
  errorRate.add(!ok)
  sleep(Math.random() * 0.5)
}

export function bulkScenario() {
  // Bulk download should redirect (302); do not follow to avoid hammering MinIO in load test.
  const res = http.get(`${BASE_URL}/api/v1/portal/snapshots/bulk.tar.gz`, {
    redirects: 0,
  })
  const ok = check(res, {
    'bulk redirect 302': (r) => r.status === 302 || r.status === 307,
    'bulk < 100ms': (r) => r.timings.duration < 100,
  })
  errorRate.add(!ok)
  sleep(Math.random() * 1.0)
}
