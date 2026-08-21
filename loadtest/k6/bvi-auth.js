/**
 * k6 scenario: BVI 1:1 biometric identification (M05/M06).
 * Target: 18k req/s; p95 <= 800ms, p99 <= 1500ms (Appendix A).
 * bvi-svc internal: p99 <= 100ms.
 * Run: k6 run loadtest/k6/bvi-auth.js
 */

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend } from 'k6/metrics';

const errorRate = new Rate('bvi_error_rate');
const bviP99 = new Trend('bvi_p99_ms', true);

const BASE_URL = __ENV.EEL_BASE_URL || 'http://localhost:8080';

// Local profile: 18k * 0.01 = 180 req/s. Achieved via arrival-rate executor.
const TARGET_RPS = parseInt(__ENV.EEL_BVI_RPS || '180', 10);
const LEGACY_PCT = parseFloat(__ENV.EEL_LEGACY_PCT || '0.22');

export const options = {
	scenarios: {
		bvi_steady: {
			executor: 'constant-arrival-rate',
			rate: TARGET_RPS,
			timeUnit: '1s',
			duration: '12m',
			preAllocatedVUs: Math.ceil(TARGET_RPS * 0.5),
			maxVUs: TARGET_RPS * 2,
		},
	},
	thresholds: {
		// Appendix-A SLA: p95 <= 800ms, p99 <= 1500ms
		http_req_duration: ['p(95)<800', 'p(99)<1500'],
		http_req_failed: ['rate<0.005'],
		bvi_error_rate: ['rate<0.005'],
	},
};

export default function () {
	const kitId = `kit-bvi-${__VU}-${__ITER}`;
	const isLegacy = Math.random() < LEGACY_PCT;
	const path = isLegacy ? '/legacy/bvi/identify' : '/bvi/identify';

	const payload = JSON.stringify({
		kit_id: kitId,
		voter_id: `V${Math.floor(Math.random() * 22_000_000)}`,
		modalities: ['fingerprint', 'face'],
		quality_score: 0.85 + Math.random() * 0.15,
		attestation_token: `test-attest.${hexEncode(kitId)}.stub`,
	});

	const params = {
		headers: {
			'Content-Type': 'application/json',
			'X-Kit-ID': kitId,
			'X-Attestation-Token': `test-attest.${hexEncode(kitId)}.stub`,
		},
		timeout: '10s',
	};

	const res = http.post(`${BASE_URL}${path}`, payload, params);

	const ok = check(res, {
		'bvi identify 200': (r) => r.status === 200,
		'bvi p99 within budget': (r) => r.timings.duration < 1500,
	});

	bviP99.add(res.timings.duration);
	errorRate.add(!ok);

	// No explicit sleep: arrival-rate executor controls pacing
}

function hexEncode(s) {
	let out = '';
	for (let i = 0; i < s.length; i++) {
		out += s.charCodeAt(i).toString(16).padStart(2, '0');
	}
	return out;
}
