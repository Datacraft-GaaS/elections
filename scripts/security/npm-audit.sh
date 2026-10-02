#!/bin/bash
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

# Cross-language audit: npm
find . -name package-lock.json -maxdepth 4 -print0 | while read -r -d '' f; do
  echo "AUDITING: $(dirname $f)"
  (cd $(dirname $f) && npm audit --json 2>/dev/null | grep '"vulnerabilities"' | head -3 || echo "  clean")
done
