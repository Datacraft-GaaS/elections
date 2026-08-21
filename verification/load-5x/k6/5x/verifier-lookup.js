/**
 * k6 scenario: Kit verifier at 5x load (WP-6.4).
 * Target: 50M concurrent sessions; proof retrieval p99 <= 30s.
 * Run: k6 run verification/load-5x/k6/5x/verifier-lookup.js
 */

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Trend, Rate } from 'k6/metrics';

const verifierP99 = new Trend('verifier_p99_ms', true);
const proofP99 = new Trend('proof_p99_ms', true);
const verifierErrorRate = new Rate('verifier_error_rate');

const BASE_URL = __ENV.EEL_BASE_URL || 'http://localhost:8080';
// 50M concurrent at 5x. Each VU holds a long-lived session.
// Default local: 200 VUs (scale down for local run).
const TARGET_VUS = parseInt(__ENV.EEL_VERIFIER_VUS || '200', 10);

export const options = {
	scenarios: {
		verifier_5x: {
			executor: 'ramping-vus',
			startVUs: 0,
			stages: [
				{ duration: '2m', target: Math.floor(TARGET_VUS * 0.1) },
				{ duration: '10m', target: TARGET_VUS },
				{ duration: '2m', target: 0 },
			],
			gracefulRampDown: '30s',
		},
	},
	thresholds: {
		// Appendix-A: proof retrieval p99 <= 30s; session lookup p99 <= 1s.
		http_req_duration: ['p(99)<30000'],
		proof_p99_ms: ['p(99)<30000'],
		verifier_p99_ms: ['p(99)<1000'],
		verifier_error_rate: ['rate<0.01'],
	},
};

export default function () {
	const stationCode = `STN-${Math.floor(__VU % 47000)}`;

	const params = {
		headers: {
			'Accept': 'application/json',
			'X-Load-Profile': '5x',
		},
		timeout: '45s',
	};

	// Session heartbeat: fast lookup.
	const startLookup = Date.now();
	const resLookup = http.get(`${BASE_URL}/verifier/session/${__VU}`, params);
	verifierP99.add(Date.now() - startLookup);

	check(resLookup, {
		'session 200 or 404': (r) => r.status === 200 || r.status === 404,
	});

	// Inclusion proof fetch: may be slow (p99 <= 30s).
	const startProof = Date.now();
	const resProof = http.get(`${BASE_URL}/results/proof/${stationCode}`, params);
	const proofElapsed = Date.now() - startProof;
	proofP99.add(proofElapsed);

	const ok = check(resProof, {
		'proof 200': (r) => r.status === 200,
		'has proof data': (r) => {
			try {
				return JSON.parse(r.body).proof !== undefined;
			} catch (_) {
				return false;
			}
		},
	});

	verifierErrorRate.add(!ok);

	// Verifier sessions have a dwell time before the next lookup.
	sleep(1 + Math.random() * 2);
}
