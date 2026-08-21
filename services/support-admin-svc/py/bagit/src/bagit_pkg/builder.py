"""BagIt bag builder (BagIt 1.0 spec, RFC 8493)."""

from __future__ import annotations

import hashlib
import shutil
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from uuid6 import uuid7

from .models import BagManifest


def _sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as fh:
        for chunk in iter(lambda: fh.read(65536), b""):
            h.update(chunk)
    return h.hexdigest()


class BagBuilder:
    """Create a BagIt 1.0 bag from a source directory.

    Structure produced:
      <dest_path>/
        bagit.txt
        bag-info.txt
        manifest-sha256.txt
        data/
          <copied files>
    """

    def create_bag(
        self,
        source_path: str | Path,
        dest_path: str | Path,
        metadata: dict[str, Any] | None = None,
    ) -> BagManifest:
        src = Path(source_path)
        dst = Path(dest_path)
        assert src.exists(), f"source_path does not exist: {src}"
        dst.mkdir(parents=True, exist_ok=True)

        data_dir = dst / "data"
        data_dir.mkdir(exist_ok=True)

        # Copy all files from source into data/
        manifest: dict[str, str] = {}
        total_bytes = 0
        stream_count = 0

        if src.is_file():
            files = [src]
            dest_files = [data_dir / src.name]
        else:
            files = sorted(p for p in src.rglob("*") if p.is_file())
            dest_files = [data_dir / p.relative_to(src) for p in files]

        for src_file, dst_file in zip(files, dest_files):
            dst_file.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(src_file, dst_file)
            digest = _sha256_file(dst_file)
            rel = dst_file.relative_to(dst)
            manifest[str(rel)] = digest
            total_bytes += dst_file.stat().st_size
            stream_count += 1

        payload_oxum = f"{total_bytes}.{stream_count}"
        bagging_date = datetime.now(timezone.utc).strftime("%Y-%m-%d")

        # bagit.txt
        (dst / "bagit.txt").write_text(
            "BagIt-Version: 1.0\nTag-File-Character-Encoding: UTF-8\n",
            encoding="utf-8",
        )

        # manifest-sha256.txt
        lines = "\n".join(
            f"{digest}  {rel}" for rel, digest in sorted(manifest.items())
        )
        (dst / "manifest-sha256.txt").write_text(lines + "\n", encoding="utf-8")

        # bag-info.txt
        info_lines: list[str] = []
        if metadata:
            for key, val in metadata.items():
                info_lines.append(f"{key}: {val}")
        info_lines.append(f"Payload-Oxum: {payload_oxum}")
        info_lines.append(f"Bagging-Date: {bagging_date}")
        (dst / "bag-info.txt").write_text(
            "\n".join(info_lines) + "\n", encoding="utf-8"
        )

        bag_id = str(uuid7())
        return BagManifest(
            bag_id=bag_id,
            bag_path=str(dst),
            manifest=manifest,
            payload_oxum=payload_oxum,
            bagging_date=bagging_date,
            created_at=datetime.now(timezone.utc),
        )
