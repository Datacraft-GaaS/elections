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

// M42-LOAD, M43-LOAD: Badge verification load test (scanner path)
// Target: 10000 req/min (~167 req/s); p99 < 300ms
// Profile: see loadtest/profiles/preprod.yaml badge_verification
// No PII in response verified in assertion

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend } from 'k6/metrics';

const errorRate = new Rate('errors');
const latency = new Trend('latency_ms', true);

export const options = {
    scenarios: {
        badge_scan: {
            executor: 'constant-arrival-rate',
            rate: __ENV.TARGET_RPS || 100,
            timeUnit: '1s',
            duration: __ENV.DURATION || '30s',
            preAllocatedVUs: 20,
            maxVUs: 100,
        },
    },
    thresholds: {
        // M42-LOAD-001, M43-LOAD-001: badge verify p99 must be under 300ms
        latency_ms: ['p(99)<300'],
        errors: ['rate<0.01'],
    },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:3000';

// Sample serials for realistic distribution
function randomSerial() {
    const prefix = Math.random() < 0.5 ? 'MEDIA' : 'OBS';
    const num = Math.floor(Math.random() * 100000).toString().padStart(6, '0');
    return `${prefix}-${num}`;
}

export default function () {
    const serial = randomSerial();

    // M42-LOAD-002, M43-LOAD-002: GET /v1/training/badges/:serial/verify
    const res = http.get(`${BASE_URL}/v1/training/badges/${serial}/verify`, {
        tags: { name: 'badge_verify' },
    });

    const ok = check(res, {
        'status 200': (r) => r.status === 200,
        'no PII applicant_name': (r) => {
            try {
                const body = JSON.parse(r.body);
                return !('applicant_name' in body);
            } catch {
                return false;
            }
        },
        'no PII id_number': (r) => {
            try {
                const body = JSON.parse(r.body);
                return !('id_number' in body);
            } catch {
                return false;
            }
        },
        'has status field': (r) => {
            try {
                return typeof JSON.parse(r.body).status === 'string';
            } catch {
                return false;
            }
        },
    });

    errorRate.add(!ok);
    latency.add(res.timings.duration);

    sleep(0.01);
}
