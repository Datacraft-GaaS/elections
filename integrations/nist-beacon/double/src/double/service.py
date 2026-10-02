# Copyright (c) 2026 Nyimbi Odero
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

"""NIST Randomness Beacon double: serves deterministic canned pulses."""

from __future__ import annotations

from fastapi import FastAPI

app = FastAPI(title="eel-adapter-nist-beacon-double")

_CANNED_PULSE = {
    "pulse": {
        "uri": "https://beacon.nist.gov/beacon/2.0/chain/1/pulse/1",
        "version": "Version 2.0",
        "cipherSuite": 0,
        "period": 60,
        "certificateId": "0" * 64,
        "chainIndex": 1,
        "pulseIndex": 1,
        "timeStamp": "2026-08-21T00:00:00.000Z",
        "localRandomValue": "a" * 128,
        "external": {"sourceId": "0" * 64, "statusCode": 0, "value": "0" * 128},
        "listValues": [],
        "previousOutputValue": "0" * 128,
        "hourValue": "0" * 128,
        "dayValue": "0" * 128,
        "monthValue": "0" * 128,
        "yearValue": "0" * 128,
        "preCommitmentValue": "0" * 128,
        "statusCode": 0,
        "signatureValue": "0" * 256,
        "outputValue": "b" * 128,
    }
}


@app.get("/beacon/2.0/pulse/last")
async def pulse_last() -> dict:
    return _CANNED_PULSE


@app.get("/beacon/2.0/chain/{chain_index}/pulse/{pulse_index}")
async def pulse_by_index(chain_index: int, pulse_index: int) -> dict:
    pulse = dict(_CANNED_PULSE["pulse"])
    pulse["chainIndex"] = chain_index
    pulse["pulseIndex"] = pulse_index
    return {"pulse": pulse}


@app.get("/health")
async def health() -> dict[str, str]:
    return {"status": "ok"}
