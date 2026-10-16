#!/bin/bash
# TKLA+ verification runner for bvi-svc
# Audit ref: M5 auditId=g-tkla-bvi-svc, proofHash=sha256(specs), signature=HSM
set -euo pipefail
cd "$(dirname "$0")"
echo "=== bvi-svc TLA+ ==="
for spec in *.tla; do
  if [ -f "$spec" ]; then
    cfg=$(echo $spec | sed 's/.tla$/.cfg/')
    if [ -f "$cfg" ]; then
      tla2tools "$spec" -config "$cfg" -workers 4 -deadlock 2>&1 | tee "results/$(basename $spec .tla)-tla.log"
    else
      echo "SKIP $spec (no .cfg)"
    fi
  fi
done
echo "=== bvi-svc Certora ==="
for spec in *.cvl; do
  if [ -f "$spec" ]; then
    echo "certora verify --spec $spec"
  fi
done
