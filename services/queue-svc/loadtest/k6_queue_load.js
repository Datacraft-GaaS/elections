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
