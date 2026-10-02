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
 * k6 scenario: personnel check-ins / M33.
 * Target: 250k check-ins in a 90-minute window (Appendix A / M33 spec).
 * That is 250000 / 5400s = ~46.3 req/s sustained over 90 minutes.
 * Local profile: 250k * 0.01 = 2500 in 90min = ~0.46 req/s (EEL_CHECKIN_RPS).
 *
 * Run: k6 run loadtest/k6/personnel-checkins.js
 */

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Counter, Rate } from 'k6/metrics';

const checkinErrors = new Counter('checkin_errors');
const errorRate = new Rate('checkin_error_rate');

const BASE_URL = __ENV.EEL_BASE_URL || 'http://localhost:8080';

// Local: 46 * 0.01 = ~0.5 req/s -> use at least 1
const TARGET_RPS = Math.max(1, parseInt(__ENV.EEL_CHECKIN_RPS || '1', 10));

export const options = {
	scenarios: {
		// 90-minute polling-open window check-in burst
		morning_checkin: {
			executor: 'constant-arrival-rate',
			rate: TARGET_RPS,
			timeUnit: '1s',
			duration: '90m',
			preAllocatedVUs: TARGET_RPS * 3,
			maxVUs: TARGET_RPS * 10,
		},
	},
	thresholds: {
		// M33 requirement: all 250k complete in the window; no explicit p99 SLA in Appendix A,
		// but use a reasonable budget of 2s p99 to catch outliers.
		http_req_duration: ['p(95)<1000', 'p(99)<2000'],
		http_req_failed: ['rate<0.01'],
		checkin_error_rate: ['rate<0.01'],
	},
};

// Track check-in count for assertion at teardown
let checkinCount = 0;

export default function () {
	const personnelId = `STAFF-${__VU}-${__ITER}`;
	const stationCode = `STN-${Math.floor(Math.random() * 47000)}`;
	const role = ['presiding-officer', 'deputy-po', 'clerk', 'security'][Math.floor(Math.random() * 4)];

	const payload = JSON.stringify({
		personnel_id: personnelId,
		station_code: stationCode,
		role,
		checked_in_at: new Date().toISOString(),
		gps_lat: -1.2 + (Math.random() - 0.5) * 5,
		gps_lon: 36.8 + (Math.random() - 0.5) * 10,
		// Stub attestation for MDM device
		device_token: `mdm-token.${personnelId}.stub`,
	});

	const params = {
		headers: {
			'Content-Type': 'application/json',
			'X-Personnel-ID': personnelId,
		},
		timeout: '10s',
	};

	const res = http.post(`${BASE_URL}/personnel/check-in`, payload, params);

	const ok = check(res, {
		'check-in 200': (r) => r.status === 200 || r.status === 201,
		'check-in p99 within budget': (r) => r.timings.duration < 2000,
	});

	if (!ok) checkinErrors.add(1);
	errorRate.add(!ok);

	checkinCount++;
	// No sleep: arrival-rate executor controls pacing
}

export function handleSummary(data) {
	const totalCheckins = data.metrics.checkin_errors
		? data.metrics.iterations.values.count - data.metrics.checkin_errors.values.count
		: data.metrics.iterations
		  ? data.metrics.iterations.values.count
		  : 0;

	return {
		stdout: `\nCheck-in summary: ${totalCheckins} successful check-ins\n`,
	};
}
