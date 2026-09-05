#!/bin/bash
# Cross-language audit: Flutter/Dart
find . -name pubspec.yaml -maxdepth 4 | head -5 | while read f; do
  dir=$(dirname $f)
  echo "DART LOCK: $dir/pubspec.lock"
  if [ -f "$dir/pubspec.lock" ]; then echo "  ✅ pubspec.lock present ($f)"; fi
done
