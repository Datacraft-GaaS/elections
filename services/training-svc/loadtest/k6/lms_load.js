// M38-LOAD: LMS assessment submission load test
// Target: 50k concurrent VUs; p99 < 1000ms
// Profile: see loadtest/profiles/preprod.yaml lms_assessment
// Idempotency-Key is unique per VU to prevent duplicate scoring

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend } from 'k6/metrics';
import { uuidv4 } from 'https://jslib.k6.io/k6-utils/1.4.0/index.js';

const errorRate = new Rate('errors');
const latency = new Trend('latency_ms', true);

export const options = {
    scenarios: {
        lms_assessment_submit: {
            executor: 'ramping-vus',
            startVUs: 0,
            stages: [
                { duration: '60s', target: __ENV.VUS || 500 },
                { duration: __ENV.DURATION || '30s', target: __ENV.VUS || 500 },
                { duration: '30s', target: 0 },
            ],
        },
    },
    thresholds: {
        // M38-LOAD-001: p99 must be under 1000ms at 50k VUs
        latency_ms: ['p(99)<1000'],
        errors: ['rate<0.01'],
    },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:3000';
// Use a fixed set of module IDs for realistic spread
const MODULE_IDS = [
    '00000000-0000-0000-0000-000000000001',
    '00000000-0000-0000-0000-000000000002',
    '00000000-0000-0000-0000-000000000003',
    '00000000-0000-0000-0000-000000000004',
    '00000000-0000-0000-0000-000000000005',
];

export default function () {
    // M38-LOAD-002: unique idempotency key per VU prevents double-scoring
    const idempotencyKey = uuidv4();
    const moduleId = MODULE_IDS[Math.floor(Math.random() * MODULE_IDS.length)];

    const payload = JSON.stringify({
        answers: { q1: 'a', q2: 'b', q3: 'c' },
        idempotency_key: idempotencyKey,
    });

    const res = http.post(
        `${BASE_URL}/v1/training/lms/modules/${moduleId}/assessments`,
        payload,
        {
            headers: { 'Content-Type': 'application/json' },
            tags: { name: 'lms_assessment_submit' },
        }
    );

    const ok = check(res, {
        'status 200': (r) => r.status === 200,
        'has idempotency_key': (r) => {
            try {
                return JSON.parse(r.body).idempotency_key === idempotencyKey;
            } catch {
                return false;
            }
        },
    });

    errorRate.add(!ok);
    latency.add(res.timings.duration);

    sleep(0.5);
}
