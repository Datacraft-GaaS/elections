#!/bin/bash
# Cross-language audit: npm
find . -name package-lock.json -maxdepth 4 -print0 | while read -r -d '' f; do
  echo "AUDITING: $(dirname $f)"
  (cd $(dirname $f) && npm audit --json 2>/dev/null | grep '"vulnerabilities"' | head -3 || echo "  clean")
done
