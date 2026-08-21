/**
 * k6 scenario: public results portal visitor load (M11).
 * Target: 5M visitors/hr = 1389 req/s; p95 <= 200ms, p99 <= 500ms (Appendix A).
 * Also exercises proof-viewer endpoint (proofs <= 30s p99).
 * Run: k6 run loadtest/k6/portal-visitors.js
 */

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend } from 'k6/metrics';

const errorRate = new Rate('portal_error_rate');
const proofP99 = new Trend('portal_proof_p99_ms', true);

const BASE_URL = __ENV.EEL_BASE_URL || 'http://localhost:8080';

// Local: 1389 * 0.01 = ~14 req/s
const TARGET_RPS = parseInt(__ENV.EEL_PORTAL_RPS || '14', 10);

export const options = {
	scenarios: {
		results_flood: {
			executor: 'constant-arrival-rate',
			rate: TARGET_RPS,
			timeUnit: '1s',
			duration: '10m',
			preAllocatedVUs: TARGET_RPS * 2,
			maxVUs: TARGET_RPS * 10,
		},
		proof_viewers: {
			executor: 'constant-vus',
			vus: 5,
			duration: '10m',
			startTime: '2m',
		},
	},
	thresholds: {
		// Appendix-A SLA: p95 <= 200ms, p99 <= 500ms
		http_req_duration: ['p(95)<200', 'p(99)<500'],
		http_req_failed: ['rate<0.001'],    // portal must be highly available
		portal_error_rate: ['rate<0.001'],
		// Proof viewer: <= 30s p99 (Appendix A)
		portal_proof_p99_ms: ['p(99)<30000'],
	},
};

export default function () {
	const scenario = __ENV.K6_SCENARIO_NAME;

	if (scenario === 'proof_viewers') {
		runProofViewer();
	} else {
		runPortalVisitor();
	}
}

function runPortalVisitor() {
	const countyId = Math.floor(Math.random() * 47) + 1;

	const reqs = {
		index: ['GET', `${BASE_URL}/results`, null],
		county: ['GET', `${BASE_URL}/results/county/${countyId}`, null],
	};

	const res = http.get(reqs.county[1], {
		headers: { 'Accept': 'application/json, text/html' },
		timeout: '5s',
	});

	check(res, {
		'portal 200': (r) => r.status === 200,
		'portal p95 within budget': (r) => r.timings.duration < 200,
	});

	errorRate.add(res.status !== 200);
	sleep(Math.random() * 3); // user browse time
}

function runProofViewer() {
	const stationCode = `STN-${Math.floor(Math.random() * 47000)}`;
	const res = http.get(`${BASE_URL}/results/proof/${stationCode}`, {
		headers: { 'Accept': 'application/json' },
		timeout: '35s',
	});

	check(res, {
		'proof 200': (r) => r.status === 200,
		'proof within 30s': (r) => r.timings.duration < 30000,
	});

	proofP99.add(res.timings.duration);
	sleep(5);
}
