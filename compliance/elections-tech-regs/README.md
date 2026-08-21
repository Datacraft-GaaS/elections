# Elections (Technology) Regulations 2017 Compliance

This directory contains the test harness mapping and supporting templates for compliance with the Elections (Technology) Regulations 2017 (Kenya), made under the Elections Act Cap 7.

## Files

| File | Purpose |
|---|---|
| `regulations-mapping.yaml` | Each regulation mapped to test IDs, Pact contracts, and rules-engine tests |
| `certifier-submission-template.md` | Submission package template for independent security certifier (Reg 14) |
| `legal-audit-test-stubs.rs` | Rust test stub template for eel-governance-svc/crates/legal-audit |

## Usage

1. Run `tools/asvs-check scan` and ensure all `LEGAL-REGS-*` test tags appear in passing tests.
2. For each regulation in `regulations-mapping.yaml`, verify that at least one test ID exists with a passing result.
3. Before submitting to the independent security certifier, complete `certifier-submission-template.md` with actual artefact paths and test run references.
4. Copy `legal-audit-test-stubs.rs` to `eel-governance-svc/crates/legal-audit/tests/regulations_test.rs` and implement the bodies before WP-5.1 completion.

## Regulation coverage

Regulations 3 through 17 of the Elections (Technology) Regulations 2017 are mapped. Each regulation is mapped to at least one test tag (`LEGAL-REGS-NNN`) that must appear in a passing test tagged in the eel codebase.
