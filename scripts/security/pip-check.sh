#!/bin/bash
# Cross-language audit: Python (pip audit via pip-audit if installed)
if command -v pip-audit >/dev/null 2>&1; then
  find . -name "uv.lock" -maxdepth 4 | head -5 | while read f; do echo "PIP-AUDIT: $f"; done
  echo "pip-audit installed — run: pip-audit --requirement requirements.txt"
else
  echo "pip-audit NOT installed (pip install pip-audit)"
fi
