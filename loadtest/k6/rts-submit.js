/**
 * k6 scenario: RTS 34A form submission (M08/M10).
 * Target: 60k concurrent at peak; p95 <= 500ms, p99 <= 1000ms.
 * Run: k6 run loadtest/k6/rts-submit.js
 */

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Trend, Counter } from 'k6/metrics';

const rtsP99 = new Trend('rts_p99_ms', true);
const rtsErrors = new Counter('rts_errors');

const BASE_URL = __ENV.EEL_BASE_URL || 'http://localhost:8080';
const KIT_COUNT = parseInt(__ENV.EEL_KIT_COUNT || '600', 10); // 60k * 0.01 local

export const options = {
	scenarios: {
		polling_open: {
			executor: 'ramping-vus',
			startVUs: 0,
			stages: [
				{ duration: '2m', target: Math.floor(KIT_COUNT * 0.3) },   // ramp up
				{ duration: '8m', target: KIT_COUNT },                       // peak: 60k concurrent
				{ duration: '2m', target: Math.floor(KIT_COUNT * 0.1) },   // wind down
			],
			gracefulRampDown: '30s',
		},
	},
	thresholds: {
		// Appendix-A SLA: p95 <= 500ms, p99 <= 1000ms @ 60k concurrent
		http_req_duration: ['p(95)<500', 'p(99)<1000'],
		http_req_failed: ['rate<0.01'],  // < 1% errors
		rts_errors: ['count<100'],
	},
};

export default function () {
	const kitId = `kit-rts-${__VU}-${__ITER}`;
	const payload = JSON.stringify({
		kit_id: kitId,
		station_code: `STN-${Math.floor(__VU % 47000)}`,
		form_type: '34A',
		transmitted_at: new Date().toISOString(),
		candidate_results: [
			{ candidate_id: 'cand-001', votes: Math.floor(Math.random() * 500) },
			{ candidate_id: 'cand-002', votes: Math.floor(Math.random() * 500) },
		],
		// Stub attestation token (EEL_KIT_ATTEST_TEST_MODE=true)
		attestation_token: `test-attest.${hexEncode(kitId)}.stub`,
	});

	const params = {
		headers: {
			'Content-Type': 'application/json',
			'X-Kit-ID': kitId,
			'X-Attestation-Token': `test-attest.${hexEncode(kitId)}.stub`,
		},
		timeout: '30s',
	};

	const res = http.post(`${BASE_URL}/rts/submit`, payload, params);

	const ok = check(res, {
		'rts submit 200': (r) => r.status === 200 || r.status === 202,
		'rts p99 within budget': (r) => r.timings.duration < 1000,
	});

	rtsP99.add(res.timings.duration);
	if (!ok) rtsErrors.add(1);

	// Kits space transmissions by ~1s to simulate realistic pacing
	sleep(1);
}

function hexEncode(s) {
	let out = '';
	for (let i = 0; i < s.length; i++) {
		out += s.charCodeAt(i).toString(16).padStart(2, '0');
	}
	return out;
}
