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

"""Views for diaspora admin: bulk CSV import (WP-5.8, M24)."""

from __future__ import annotations

import csv
import io
from dataclasses import dataclass, field

from django.contrib.admin.views.decorators import staff_member_required
from django.http import HttpRequest, HttpResponse, JsonResponse
from django.utils.decorators import method_decorator
from django.views import View


@dataclass
class RowReport:
    line: int
    ok: bool
    error: str | None = None
    voter_id: str | None = None


@dataclass
class ImportReport:
    total: int = 0
    accepted: int = 0
    rejected: int = 0
    rows: list[RowReport] = field(default_factory=list)


EXPECTED_HEADER = ["id", "national_id_last4", "name_nfc", "dob_yyyy", "station_code"]


def _validate_row(line: int, row: dict[str, str]) -> RowReport:
    """Validate one roll row. `name_nfc` is not read here.

    The header check in `post` requires the column to be present, but nothing
    validates its content -- unlike `id`, `national_id_last4`, `dob_yyyy` and
    `station_code`, each of which has a rule below. A row with an empty name
    is therefore accepted. Reading it into an unused local made that look
    handled; it is not, and `RowReport` has no field to carry it.
    """
    vid = row.get("id", "").strip()
    nid4 = row.get("national_id_last4", "").strip()
    dob = row.get("dob_yyyy", "").strip()
    code = row.get("station_code", "").strip()

    if not vid:
        return RowReport(line=line, ok=False, error="id is empty")

    if len(nid4) != 4 or not nid4.isdigit():
        return RowReport(
            line=line,
            ok=False,
            error=f"national_id_last4 must be 4 digits, got '{nid4}'",
        )

    try:
        year = int(dob)
        if not (1900 <= year <= 2010):
            raise ValueError
    except ValueError:
        return RowReport(
            line=line, ok=False, error=f"dob_yyyy '{dob}' out of range 1900-2010"
        )

    if not code:
        return RowReport(line=line, ok=False, error="station_code is empty")

    return RowReport(line=line, ok=True, voter_id=vid)


@method_decorator(staff_member_required, name="dispatch")
class ImportRollView(View):
    """Accept a CSV upload, validate rows, and return a JSON import report."""

    def get(self, request: HttpRequest) -> HttpResponse:
        html = """
		<form method="post" enctype="multipart/form-data">
			<input type="file" name="csv_file" accept=".csv" required>
			<button type="submit">Import Roll CSV</button>
		</form>
		"""
        return HttpResponse(html)

    def post(self, request: HttpRequest) -> JsonResponse:
        f = request.FILES.get("csv_file")
        if f is None:
            return JsonResponse({"error": "csv_file required"}, status=400)

        text = f.read().decode("utf-8", errors="replace")
        reader = csv.DictReader(io.StringIO(text))

        if reader.fieldnames is None or list(reader.fieldnames) != EXPECTED_HEADER:
            return JsonResponse(
                {"error": f"invalid header; expected {EXPECTED_HEADER}"},
                status=400,
            )

        report = ImportReport()
        for idx, row in enumerate(reader, start=2):
            r = _validate_row(idx, dict(row))
            report.rows.append(r)
            report.total += 1
            if r.ok:
                report.accepted += 1
            else:
                report.rejected += 1

        return JsonResponse(
            {
                "total": report.total,
                "accepted": report.accepted,
                "rejected": report.rejected,
                "rows": [
                    {
                        "line": r.line,
                        "ok": r.ok,
                        "voter_id": r.voter_id,
                        "error": r.error,
                    }
                    for r in report.rows
                ],
            }
        )
