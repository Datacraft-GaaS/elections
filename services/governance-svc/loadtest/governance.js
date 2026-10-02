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

// WP-5.1 governance-svc load test scenarios
import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate } from 'k6/metrics';

export const errorRate = new Rate('errors');

const BASE_RLA = 'http://localhost:8091';
const BASE_COURT = 'http://localhost:8092';
const BASE_LEGAL = 'http://localhost:8093';
const BASE_REPORTING = 'http://localhost:8094';
const BASE_OAG = 'http://localhost:8095';

export const options = {
  scenarios: {
    rla_list: {
      executor: 'constant-arrival-rate',
      rate: 50,
      timeUnit: '1s',
      duration: '60s',
      preAllocatedVUs: 20,
      exec: 'listAudits',
    },
    court_petitions: {
      executor: 'constant-arrival-rate',
      rate: 20,
      timeUnit: '1s',
      duration: '60s',
      preAllocatedVUs: 10,
      exec: 'listPetitions',
    },
    legal_verdicts: {
      executor: 'constant-arrival-rate',
      rate: 30,
      timeUnit: '1s',
      duration: '60s',
      preAllocatedVUs: 10,
      exec: 'listVerdicts',
    },
  },
  thresholds: {
    'http_req_duration{scenario:rla_list}': ['p(95)<300'],
    'http_req_duration{scenario:court_petitions}': ['p(95)<300'],
    errors: ['rate<0.01'],
  },
};

export function listAudits() {
  const res = http.get(`${BASE_RLA}/api/v1/governance/rla/audits`);
  errorRate.add(!check(res, { 'status 200': (r) => r.status === 200 }));
  sleep(0.1);
}

export function listPetitions() {
  const res = http.get(`${BASE_COURT}/api/v1/governance/court/petitions`);
  errorRate.add(!check(res, { 'status 200': (r) => r.status === 200 }));
  sleep(0.1);
}

export function listVerdicts() {
  const res = http.get(`${BASE_LEGAL}/api/v1/governance/legal-audit/transmissions`);
  errorRate.add(!check(res, { 'status 200': (r) => r.status === 200 }));
  sleep(0.1);
}
