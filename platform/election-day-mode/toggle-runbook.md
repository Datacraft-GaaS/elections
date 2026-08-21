# Election-Day Mode Toggle Runbook

The authoritative runbook lives at:

```
docs/runbooks/election-day-mode-toggle.md
```

Go there for step-by-step instructions, including:

- Pre-activation checklist (confirm election date, freeze bounds, operator on-call roster).
- Dual SVID sign-off procedure (two `eel-election-operators` group members required).
- Using `election-day-mode-cli/toggle.sh` with cosign attestation.
- Verification steps after activation.
- Emergency deactivation procedure.
- Rollback if freeze policy causes an unintended block.
