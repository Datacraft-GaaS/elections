#!/usr/bin/env bash
# Seed each module with SPEC.md + .gitkeep placeholders.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
for d in "$root"/modules/mod-*; do
	slug="$(basename "$d")"
	for sub in shared server desktop mobile; do
		[ -e "$d/$sub/.gitkeep" ] || : > "$d/$sub/.gitkeep"
	done
	spec="$d/docs/SPEC.md"
	if [ ! -e "$spec" ]; then
		cat > "$spec" <<EOF
# $slug — spec

Status: draft

## Purpose
One sentence: what problem this module solves.

## Domain model
Entities, invariants, key operations. Lives in \`shared/\`.

## Server surface
API endpoints or message contracts exposed by \`server/\`.

## Desktop surface
Screens, commands, integration points in \`desktop/\`.

## Mobile surface
Screens, integration points in \`mobile/\`.

## Dependencies
Other modules or packages this depends on. Justify each.

## Open questions
EOF
	fi
done
echo "seeded $(ls -d "$root"/modules/mod-* | wc -l | tr -d ' ') modules"
