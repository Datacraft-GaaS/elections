// M37-LOAD: Public CMS content feed load test
// Target: 1000 req/s sustained; p99 < 1000ms
// Profile: see loadtest/profiles/preprod.yaml cms_content_feed

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend } from 'k6/metrics';

const errorRate = new Rate('errors');
const latency = new Trend('latency_ms', true);

export const options = {
    scenarios: {
        cms_public_feed: {
            executor: 'constant-arrival-rate',
            rate: __ENV.TARGET_RPS || 10,
            timeUnit: '1s',
            duration: __ENV.DURATION || '30s',
            preAllocatedVUs: 50,
            maxVUs: 200,
        },
    },
    thresholds: {
        // M37-LOAD-001: p99 must be under 1000ms
        latency_ms: ['p(99)<1000'],
        errors: ['rate<0.01'],
    },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:3000';

export default function () {
    // M37-LOAD-002: public content feed, no auth required
    const res = http.get(`${BASE_URL}/v1/training/cms/content/public`, {
        tags: { name: 'cms_public_feed' },
    });

    const ok = check(res, {
        'status 200': (r) => r.status === 200,
        'has items array': (r) => {
            try {
                const body = JSON.parse(r.body);
                return Array.isArray(body.items);
            } catch {
                return false;
            }
        },
    });

    errorRate.add(!ok);
    latency.add(res.timings.duration);

    sleep(0.01);
}
