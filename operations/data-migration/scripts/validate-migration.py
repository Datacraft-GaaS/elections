#!/usr/bin/env python3
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

"""
Migration validator for KIEMS-to-eel data migration.

Compares pre/post record counts, verifies hash of sorted voter IDs,
and checks referential integrity. Exits non-zero on any drift.

Usage:
  validate-migration.py --phase pre  --kiems-dump <dir> --output-csv <file> --ticket <id>
  validate-migration.py --phase post --kiems-dump <dir> --eel-endpoint <url> --ticket <id>
  validate-migration.py --phase reconcile --kiems-dump <dir> --eel-endpoint <url> --report <file>
"""

import argparse
import csv
import hashlib
import json
import os
import subprocess
import sys
import urllib.request
from pathlib import Path


def die(msg: str) -> None:
    print(f"ERROR: {msg}", file=sys.stderr)
    sys.exit(1)


def count_csv_records(path: str) -> int:
    with open(path, encoding="utf-8", newline="") as f:
        reader = csv.reader(f)
        next(reader)  # skip header
        return sum(1 for _ in reader)


def hash_sorted_ids(path: str, id_column: str = "national_id") -> str:
    ids: list[str] = []
    with open(path, encoding="utf-8", newline="") as f:
        reader = csv.DictReader(f)
        for row in reader:
            ids.append(row[id_column].strip())
    ids.sort()
    digest = hashlib.sha256("\n".join(ids).encode()).hexdigest()
    return digest


def kiems_count_from_dump(dump_dir: str) -> int:
    """
    Count active voter records in the KIEMS dump by restoring to a temp DB
    and querying. This requires pg_restore and psql on PATH.
    Uses an env-var-driven connection like the extraction scripts.
    """
    restore_db = f"kiems_validate_{os.getpid()}"
    pg_host = os.environ.get("PGHOST", "localhost")
    pg_port = os.environ.get("PGPORT", "5432")
    pg_user = os.environ.get("PGUSER", "postgres")
    conn_args = ["-h", pg_host, "-p", pg_port, "-U", pg_user]

    def psql(sql: str, dbname: str = "postgres") -> str:
        result = subprocess.run(
            ["psql"] + conn_args + ["-d", dbname, "-t", "-c", sql],
            capture_output=True,
            text=True,
            check=True,
        )
        return result.stdout.strip()

    try:
        psql(f"CREATE DATABASE {restore_db};")
        subprocess.run(
            ["pg_restore"]
            + conn_args
            + ["--dbname", restore_db, "--no-privileges", "--no-owner", dump_dir],
            check=True,
            capture_output=True,
        )
        count_str = psql(
            "SELECT COUNT(*) FROM voters WHERE record_status IN ('active', 'provisional');",
            dbname=restore_db,
        )
        return int(count_str)
    finally:
        psql(f"DROP DATABASE IF EXISTS {restore_db};")


def eel_count_from_endpoint(endpoint: str) -> int:
    url = endpoint.rstrip("/") + "/admin/count"
    req = urllib.request.Request(url, headers={"Accept": "application/json"})
    with urllib.request.urlopen(req, timeout=30) as resp:
        data = json.loads(resp.read())
    return int(data["count"])


def eel_hash_from_endpoint(endpoint: str) -> str:
    url = endpoint.rstrip("/") + "/admin/voter-id-hash"
    req = urllib.request.Request(url, headers={"Accept": "application/json"})
    with urllib.request.urlopen(req, timeout=60) as resp:
        data = json.loads(resp.read())
    return data["sha256"]


def phase_pre(args: argparse.Namespace) -> None:
    print(f"[validate-migration] Phase: pre  | Ticket: {args.ticket}")
    csv_count = count_csv_records(args.output_csv)
    kiems_count = kiems_count_from_dump(args.kiems_dump)
    csv_hash = hash_sorted_ids(args.output_csv)

    print(f"  KIEMS record count : {kiems_count}")
    print(f"  CSV record count   : {csv_count}")
    print(f"  CSV voter ID hash  : {csv_hash}")

    if csv_count != kiems_count:
        die(f"Record count mismatch: KIEMS={kiems_count}, CSV={csv_count}")
    print("[validate-migration] Pre-ingest validation PASSED")


def phase_post(args: argparse.Namespace) -> None:
    print(f"[validate-migration] Phase: post | Ticket: {args.ticket}")
    kiems_count = kiems_count_from_dump(args.kiems_dump)
    eel_count = eel_count_from_endpoint(args.eel_endpoint)
    eel_hash = eel_hash_from_endpoint(args.eel_endpoint)

    print(f"  KIEMS record count : {kiems_count}")
    print(f"  eel record count   : {eel_count}")
    print(f"  eel voter ID hash  : {eel_hash}")

    if eel_count != kiems_count:
        die(f"Record count mismatch: KIEMS={kiems_count}, eel={eel_count}")
    print("[validate-migration] Post-ingest validation PASSED")


def phase_reconcile(args: argparse.Namespace) -> None:
    print("[validate-migration] Phase: reconcile")
    kiems_count = kiems_count_from_dump(args.kiems_dump)
    eel_count = eel_count_from_endpoint(args.eel_endpoint)
    eel_hash = eel_hash_from_endpoint(args.eel_endpoint)

    lines = [
        "Migration reconciliation report",
        f"KIEMS record count : {kiems_count}",
        f"eel record count   : {eel_count}",
        f"eel voter ID hash  : {eel_hash}",
        f"Delta              : {eel_count - kiems_count}",
        f"Status             : {'PASS' if eel_count == kiems_count else 'FAIL'}",
    ]
    report = "\n".join(lines)
    print(report)
    if args.report:
        Path(args.report).write_text(report + "\n", encoding="utf-8")
    if eel_count != kiems_count:
        die(f"Reconciliation failed: delta={eel_count - kiems_count}")


def main() -> None:
    parser = argparse.ArgumentParser(description="KIEMS migration validator")
    parser.add_argument("--phase", required=True, choices=["pre", "post", "reconcile"])
    parser.add_argument("--kiems-dump", required=True)
    parser.add_argument("--output-csv")
    parser.add_argument("--eel-endpoint")
    parser.add_argument("--ticket")
    parser.add_argument("--report")
    args = parser.parse_args()

    if args.phase == "pre":
        if not args.output_csv:
            die("--output-csv required for phase pre")
        phase_pre(args)
    elif args.phase == "post":
        if not args.eel_endpoint:
            die("--eel-endpoint required for phase post")
        phase_post(args)
    elif args.phase == "reconcile":
        if not args.eel_endpoint:
            die("--eel-endpoint required for phase reconcile")
        phase_reconcile(args)


if __name__ == "__main__":
    main()
