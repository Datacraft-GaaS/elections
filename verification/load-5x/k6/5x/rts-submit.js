/**
 * k6 scenario: RTS 34A form submission at 5x load (WP-6.4).
 * Target: 300k concurrent at peak; p95 <= 500ms, p99 <= 1000ms.
 * ADR-0018: write-path P99 must remain <= 25ms end-to-end.
 * Run: k6 run verification/load-5x/k6/5x/rts-submit.js
 */

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Trend, Counter } from 'k6/metrics';

const rtsP99 = new Trend('rts_p99_ms', true);
const writePathP99 = new Trend('write_path_p99_ms', true);
const rtsErrors = new Counter('rts_errors');

const BASE_URL = __ENV.EEL_BASE_URL || 'http://localhost:8080';
// 300k concurrent at 5x; env override for CI scaling.
const KIT_COUNT = parseInt(__ENV.EEL_KIT_COUNT || '3000', 10); // default: local 1%

export const options = {
	scenarios: {
		rts_5x_ramp: {
			executor: 'ramping-vus',
			startVUs: 0,
			stages: [
				{ duration: '5m', target: Math.floor(KIT_COUNT * 0.3) },
				{ duration: '20m', target: KIT_COUNT },
				{ duration: '5m', target: Math.floor(KIT_COUNT * 0.1) },
			],
			gracefulRampDown: '60s',
		},
	},
	thresholds: {
		// Appendix-A SLA at 5x: p95 <= 500ms, p99 <= 1000ms.
		http_req_duration: ['p(95)<500', 'p(99)<1000'],
		// ADR-0018 hard gate: write-path P99 <= 25ms.
		write_path_p99_ms: ['p(99)<25'],
		http_req_failed: ['rate<0.01'],
		rts_errors: ['count<1000'],
	},
};

function hexEncode(s) {
	let result = '';
	for (let i = 0; i < s.length; i++) {
		result += ('0' + s.charCodeAt(i).toString(16)).slice(-2);
	}
	return result;
}

export default function () {
	const kitId = `kit-rts5x-${__VU}-${__ITER}`;
	const stationCode = `STN-${Math.floor(__VU % 47000)}`;

	const payload = JSON.stringify({
		kit_id: kitId,
		station_code: stationCode,
		form_type: '34A',
		transmitted_at: new Date().toISOString(),
		candidate_results: [
			{ candidate_id: 'cand-001', votes: Math.floor(Math.random() * 500) },
			{ candidate_id: 'cand-002', votes: Math.floor(Math.random() * 500) },
			{ candidate_id: 'cand-003', votes: Math.floor(Math.random() * 200) },
		],
		attestation_token: `test-attest.${hexEncode(kitId)}.stub`,
		load_profile: '5x',
	});

	const params = {
		headers: {
			'Content-Type': 'application/json',
			'X-Kit-ID': kitId,
			'X-Attestation-Token': `test-attest.${hexEncode(kitId)}.stub`,
			'X-Load-Profile': '5x',
		},
		timeout: '30s',
	};

	const start = Date.now();
	const res = http.post(`${BASE_URL}/rts/submit`, payload, params);
	const elapsed = Date.now() - start;

	rtsP99.add(elapsed);

	// Write-path P99 from response header (set by rts-svc when EEL_EMIT_WRITE_LATENCY=true).
	const writeLatency = parseFloat(res.headers['X-Write-Latency-Ms'] || '0');
	if (writeLatency > 0) {
		writePathP99.add(writeLatency);
	}

	const ok = check(res, {
		'status 200 or 202': (r) => r.status === 200 || r.status === 202,
		'has submission_id': (r) => {
			try {
				return JSON.parse(r.body).submission_id !== undefined;
			} catch (_) {
				return false;
			}
		},
	});

	if (!ok) {
		rtsErrors.add(1);
	}

	// Very short think time: 5x load requires high duty cycle.
	sleep(Math.random() * 0.1);
}
