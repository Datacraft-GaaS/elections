# reproducibility/make-reproduce/Makefile
# Top-level reproducibility pipeline for eel.
# Usage: make reproduce BUNDLE=<path-or-url>

SHELL := /usr/bin/env bash
.SHELLFLAGS := -euo pipefail -c

BUNDLE      ?=
IMAGES      ?=
A           ?=
B           ?=
REPRO_DIR   := $(CURDIR)
REPORT_DIR  := $(REPRO_DIR)/.repro-reports

.PHONY: reproduce verify-images replay-tally diff-artefact clean-repro help

help:
	@echo "Targets:"
	@echo "  reproduce BUNDLE=<path|url>   Full pipeline: fetch, verify, rebuild, diff"
	@echo "  verify-images IMAGES=<list>   Rebuild images via Nix, sha256-compare"
	@echo "  replay-tally BUNDLE=<path>    Replay events, derive Merkle root, compare"
	@echo "  diff-artefact A=<p> B=<p>     diffoscope wrapper"
	@echo "  clean-repro                   Remove workspace artefacts"

reproduce:
	@test -n "$(BUNDLE)" || (echo "ERROR: BUNDLE is required" >&2; exit 1)
	@mkdir -p "$(REPORT_DIR)"
	bash "$(REPRO_DIR)/scripts/reproduce.sh" "$(BUNDLE)" "$(REPORT_DIR)"

verify-images:
	@test -n "$(IMAGES)" || (echo "ERROR: IMAGES is required" >&2; exit 1)
	@mkdir -p "$(REPORT_DIR)"
	@for img in $(IMAGES); do \
		echo "[verify-images] $$img"; \
		nix build ".#$$img" --no-link 2>&1; \
		actual=$$(nix path-info --hash ".#$$img" 2>/dev/null | awk '{print $$1}'); \
		echo "{\"image\":\"$$img\",\"nix_hash\":\"$$actual\"}" >> "$(REPORT_DIR)/verify-images.jsonl"; \
	done
	@echo "[verify-images] report written to $(REPORT_DIR)/verify-images.jsonl"

replay-tally:
	@test -n "$(BUNDLE)" || (echo "ERROR: BUNDLE is required" >&2; exit 1)
	bash "$(REPRO_DIR)/scripts/replay-tally.sh" "$(BUNDLE)" "$(REPORT_DIR)"

diff-artefact:
	@test -n "$(A)" || (echo "ERROR: A is required" >&2; exit 1)
	@test -n "$(B)" || (echo "ERROR: B is required" >&2; exit 1)
	@mkdir -p "$(REPORT_DIR)"
	diffoscope \
		--json "$(REPORT_DIR)/diff-$(notdir $(A))-vs-$(notdir $(B)).json" \
		--html "$(REPORT_DIR)/diff-$(notdir $(A))-vs-$(notdir $(B)).html" \
		"$(A)" "$(B)" || true

clean-repro:
	rm -rf "$(REPORT_DIR)"
	@echo "Cleaned $(REPORT_DIR)"
