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

import http from "k6/http";
import { check, sleep } from "k6";

export const options = {
  vus: 10,
  duration: "30s",
};

const BASE_URL = __ENV.BASE_URL || "http://localhost:4110";

export default function () {
  const payload = JSON.stringify({
    kit_throughput_per_hour: 120,
    clerk_report_count: 80,
    cv_headcount: 90,
    qr_scan_count: 85,
  });

  const params = { headers: { "Content-Type": "application/json" } };

  const res = http.post(
    `${BASE_URL}/v1/queue/stations/STN001/snapshot`,
    payload,
    params
  );

  check(res, {
    "status 200": (r) => r.status === 200,
    "has station_code": (r) => JSON.parse(r.body).station_code === "STN001",
  });

  sleep(0.1);
}
