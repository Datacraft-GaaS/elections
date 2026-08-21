/**
 * k6 scenario: Public results portal at 5x load (WP-6.4).
 * Target: 25M viewers/hr = 6944 req/s; p95 <= 200ms, p99 <= 500ms.
 * Run: k6 run verification/load-5x/k6/5x/portal-view.js
 */

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Trend, Rate } from 'k6/metrics';

const portalP95 = new Trend('portal_p95_ms', true);
const portalP99 = new Trend('portal_p99_ms', true);
const portalErrorRate = new Rate('portal_error_rate');

const BASE_URL = __ENV.EEL_BASE_URL || 'http://localhost:8080';
// 25M/hr = 6944 req/s at 5x. Each VU targets ~1 req/s.
// Default local: 70 VUs (1% scale).
const TARGET_VUS = parseInt(__ENV.EEL_PORTAL_VUS || '70', 10);

export const options = {
	scenarios: {
		portal_5x: {
			executor: 'ramping-vus',
			startVUs: 0,
			stages: [
				{ duration: '2m', target: Math.floor(TARGET_VUS * 0.2) },
				{ duration: '15m', target: TARGET_VUS },
				{ duration: '3m', target: Math.floor(TARGET_VUS * 0.05) },
			],
			gracefulRampDown: '30s',
		},
	},
	thresholds: {
		// Appendix-A SLA at 5x.
		http_req_duration: ['p(95)<200', 'p(99)<500'],
		portal_p95_ms: ['p(95)<200'],
		portal_p99_ms: ['p(99)<500'],
		portal_error_rate: ['rate<0.01'],
	},
};

export default function () {
	// Cycle 47 counties so every request hits a distinct cache partition.
	const countyId = (__VU % 47) + 1;

	const params = {
		headers: {
			'Accept': 'text/html,application/json',
			'Accept-Language': 'en-US,sw-KE;q=0.9',
			'X-Load-Profile': '5x',
		},
		timeout: '10s',
	};

	const start = Date.now();
	const res = http.get(`${BASE_URL}/results/county/${countyId}`, params);
	const elapsed = Date.now() - start;

	portalP95.add(elapsed);
	portalP99.add(elapsed);

	const ok = check(res, {
		'status 200': (r) => r.status === 200,
		'has results data': (r) => r.body && r.body.length > 100,
	});

	portalErrorRate.add(!ok);

	// Simulate brief page browse before next request.
	sleep(0.8 + Math.random() * 0.4);
}
