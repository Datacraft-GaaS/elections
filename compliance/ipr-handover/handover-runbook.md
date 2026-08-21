# IPR Handover Runbook

Physical and logical handover procedure from the supplier (Datacraft) to IEBC OIT under GCC 15.4.

## Participants

- Supplier: Technical Lead, Legal Counsel
- IEBC OIT: Chief Information Officer, designated receiving officer
- Witnesses: ODPC liaison, independent auditor (as agreed)

## Pre-conditions

Before scheduling the handover ceremony, confirm:

- [ ] All GCC 15.4 checklist items marked DONE in `compliance/ipr-handover/ipr-checklist.md`
- [ ] Gate G5 signed off by architect and legal
- [ ] Reproducibility: `just make-reproduce` clean on x86_64 and aarch64
- [ ] Source inventory regenerated: `bash compliance/ipr-handover/scripts/generate-source-inventory.sh`
- [ ] SBOM attestations verifiable on all 18 service images
- [ ] HSM key ceremony scheduled with IEBC OIT (see `docs/runbooks/hsm-key-ceremony.md`)

## Step 1: Generate the source tarball

```sh
# From repo root, run as a read-only user (no write access to repo)
git archive --format=tar.gz \
  --prefix=eel-ipr-handover/ \
  HEAD \
  -o /tmp/eel-ipr-handover-$(date +%Y%m%d).tar.gz

# Compute SHA-256
sha256sum /tmp/eel-ipr-handover-$(date +%Y%m%d).tar.gz \
  > /tmp/eel-ipr-handover-$(date +%Y%m%d).tar.gz.sha256
```

## Step 2: Generate the reproducibility bundle

```sh
just make-reproduce <latest-bundle-path>
```

Verify that the Merkle root in the output matches the value in `reproducibility/hash-manifests/`.

## Step 3: Prepare physical media

Copy to two encrypted USB drives (AES-256 hardware encryption, FIPS 140-2 Level 3):

- Source tarball and SHA-256 checksum
- All SBOM attestation files (`compliance/ipr-handover/sbom-index.md` links)
- `compliance/ipr-handover/` directory (full)
- `reproducibility/hash-manifests/` directory
- `docs/` directory (full)

Label both drives with the handover date, git commit hash, and `GCC 15.4 HANDOVER v1`.

## Step 4: Chain-of-custody form

Complete the chain-of-custody form (`compliance/ipr-handover/gcc-15-4-handover-template.md` section 7). Both parties sign and retain a copy. Witnesses countersign.

## Step 5: Key ceremony

Refer to `docs/runbooks/hsm-key-ceremony.md` for the HSM key transfer procedure. The supplier transfers HSM partition credentials to IEBC OIT in the presence of witnesses. The ceremony is video-recorded and the recording is archived.

## Step 6: Dry-run verification

IEBC OIT performs an independent dry-run verification within 5 business days of receiving the media:

1. Install the nix development shell using `nix develop`.
2. Build all images: `just build-all`.
3. Verify reproducibility: `just make-reproduce <bundle-path>`.
4. Run the full test suite: `just test`.
5. Confirm SHA-256 of the source tarball matches the supplied checksum.

The supplier is available for technical support during the dry-run period.

## Step 7: Handover completion

When IEBC OIT confirms successful dry-run verification, both parties sign the completion certificate in `compliance/ipr-handover/gcc-15-4-handover-template.md` section 8. The handover is complete.

## Escrow

A copy of the source tarball is deposited with the agreed escrow agent within 30 days of the handover ceremony. The escrow manifest is in `compliance/ipr-handover/gcc-15-4-handover-template.md` section 6.

## Escalation

If the dry-run verification fails, the supplier has 10 business days to resolve the issue and provide a corrected package. Unresolved issues escalate to the GCC 15.4 dispute resolution mechanism.
