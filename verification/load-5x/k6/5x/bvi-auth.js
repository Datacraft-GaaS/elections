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
 * k6 scenario: BVI biometric identification at 5x load (WP-6.4).
 * Target: 90k req/s sustained; p95 <= 200ms, p99 <= 500ms.
 * Run: k6 run verification/load-5x/k6/5x/bvi-auth.js
 */

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Trend, Counter, Rate } from 'k6/metrics';

const bviP95 = new Trend('bvi_p95_ms', true);
const bviP99 = new Trend('bvi_p99_ms', true);
const bviErrors = new Counter('bvi_errors');
const bviErrorRate = new Rate('bvi_error_rate');

const BASE_URL = __ENV.EEL_BASE_URL || 'http://localhost:8080';
// 90k req/s at 5x; each VU produces ~1 req/s, so target ~90k VUs.
// Default local: 900 VUs (1% scale).
const TARGET_VUS = parseInt(__ENV.EEL_BVI_VUS || '900', 10);

export const options = {
	scenarios: {
		bvi_5x_morning_peak: {
			executor: 'ramping-vus',
			startVUs: 0,
			stages: [
				{ duration: '3m', target: Math.floor(TARGET_VUS * 0.3) },
				{ duration: '15m', target: TARGET_VUS },
				{ duration: '2m', target: Math.floor(TARGET_VUS * 0.05) },
			],
			gracefulRampDown: '30s',
		},
	},
	thresholds: {
		// Appendix-A SLA at 5x.
		http_req_duration: ['p(95)<200', 'p(99)<500'],
		bvi_p95_ms: ['p(95)<200'],
		bvi_p99_ms: ['p(99)<500'],
		bvi_error_rate: ['rate<0.01'],
	},
};

function generateFingerprintStub(vu, iter) {
	// Stub fingerprint template: 512 hex chars representing a normalised minutiae set.
	const seed = (vu * 1000 + iter) % 65536;
	return seed.toString(16).padStart(4, '0').repeat(128);
}

export default function () {
	const enrolledId = `enr-5x-${Math.floor(__VU % 10000000)}`;

	const payload = JSON.stringify({
		enrolled_id: enrolledId,
		kit_id: `kit-bvi5x-${__VU}`,
		station_code: `STN-${Math.floor(__VU % 47000)}`,
		fingerprint_template: generateFingerprintStub(__VU, __ITER),
		mode: '1:1',
		load_profile: '5x',
	});

	const params = {
		headers: {
			'Content-Type': 'application/json',
			'X-Kit-ID': `kit-bvi5x-${__VU}`,
			'X-Load-Profile': '5x',
		},
		timeout: '10s',
	};

	const start = Date.now();
	const res = http.post(`${BASE_URL}/bvi/identify`, payload, params);
	const elapsed = Date.now() - start;

	bviP95.add(elapsed);
	bviP99.add(elapsed);

	const ok = check(res, {
		'status 200': (r) => r.status === 200,
		'has match_result': (r) => {
			try {
				const b = JSON.parse(r.body);
				return b.match_result !== undefined;
			} catch (_) {
				return false;
			}
		},
	});

	bviErrorRate.add(!ok);
	if (!ok) {
		bviErrors.add(1);
	}

	sleep(Math.random() * 0.05);
}
