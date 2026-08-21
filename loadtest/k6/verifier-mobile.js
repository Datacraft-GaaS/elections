/**
 * k6 scenario: public verifier / M13 at 10M concurrent viewers.
 * Mix of SSE (live-count stream) and WebSocket (interactive verifier) connections.
 * Target: 10M concurrent, p99 < 500ms (Appendix A).
 * Run: k6 run loadtest/k6/verifier-mobile.js
 *
 * Note: k6 WebSocket and SSE are handled via ws and http.stream.
 * Local profile uses EEL_VERIFIER_VUS (default 20) to scale.
 */

import http from 'k6/http';
import ws from 'k6/ws';
import { check, sleep } from 'k6';
import { Counter, Rate, Trend } from 'k6/metrics';

const verifierErrors = new Counter('verifier_errors');
const sseLatency = new Trend('verifier_sse_connect_ms', true);
const wsLatency = new Trend('verifier_ws_connect_ms', true);

const BASE_URL = __ENV.EEL_BASE_URL || 'http://localhost:8080';
const WS_BASE_URL = (BASE_URL.replace(/^http/, 'ws'));

// Local: 10M * 0.01 = 100k -> but cap at env var for local runs
const TARGET_VUS = parseInt(__ENV.EEL_VERIFIER_VUS || '20', 10);

export const options = {
	scenarios: {
		// SSE live-count stream (majority of viewers)
		sse_viewers: {
			executor: 'ramping-vus',
			startVUs: 0,
			stages: [
				{ duration: '1m', target: Math.floor(TARGET_VUS * 0.7) },
				{ duration: '8m', target: Math.floor(TARGET_VUS * 0.7) },
				{ duration: '1m', target: 0 },
			],
			gracefulRampDown: '30s',
		},
		// WebSocket interactive verifier
		ws_verifiers: {
			executor: 'constant-vus',
			vus: Math.max(1, Math.floor(TARGET_VUS * 0.3)),
			duration: '10m',
		},
	},
	thresholds: {
		// Appendix-A: p99 < 500ms for M13
		http_req_duration: ['p(99)<500'],
		http_req_failed: ['rate<0.01'],
		verifier_errors: ['count<50'],
		verifier_sse_connect_ms: ['p(99)<500'],
		verifier_ws_connect_ms: ['p(99)<500'],
	},
};

export default function () {
	const scenario = __ENV.K6_SCENARIO_NAME;
	if (scenario === 'ws_verifiers') {
		runWebSocketVerifier();
	} else {
		runSseViewer();
	}
}

function runSseViewer() {
	const t0 = Date.now();
	const res = http.get(`${BASE_URL}/verifier/stream`, {
		headers: {
			'Accept': 'text/event-stream',
			'Cache-Control': 'no-cache',
		},
		timeout: '15s',
	});

	sseLatency.add(Date.now() - t0);

	const ok = check(res, {
		'SSE stream connected': (r) => r.status === 200,
		'SSE p99 within 500ms': (r) => r.timings.duration < 500,
	});
	if (!ok) verifierErrors.add(1);

	sleep(Math.random() * 10 + 5); // viewer watches for 5-15s
}

function runWebSocketVerifier() {
	const voterId = `V${Math.floor(Math.random() * 22_000_000)}`;
	const t0 = Date.now();

	const res = ws.connect(
		`${WS_BASE_URL}/verifier/ws?voter_id=${voterId}`,
		{},
		function (socket) {
			wsLatency.add(Date.now() - t0);

			socket.on('open', () => {
				socket.send(JSON.stringify({ type: 'verify', voter_id: voterId }));
			});

			socket.on('message', (msg) => {
				check(msg, {
					'WS response is JSON': (m) => { try { JSON.parse(m); return true; } catch { return false; } },
				});
				socket.close();
			});

			socket.on('error', () => {
				verifierErrors.add(1);
				socket.close();
			});

			// Timeout guard
			socket.setTimeout(() => socket.close(), 5000);
		}
	);

	check(res, { 'WS connected': (r) => r && r.status === 101 });
	sleep(1);
}
