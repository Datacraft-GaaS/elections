# Election-Day Mode

Election-day mode is a platform-wide operational flag that activates a freeze window for
election-critical services during the period T-1 through T+7 (day before election through
7 days after). It has three levels:

## Flag hierarchy

```
global (Chair HSM-signed, this directory)
  |
  +-- service (per-service opt-in via label eel.internal/election-day-mode-sensitive: "true")
        |
        +-- module (feature-flag within a service, controlled by the service owner)
```

1. **Global** (`eel-election-day-mode` ConfigMap in `argocd` namespace): set by the Chair or
   designated election officer using the toggle CLI with dual SVID sign-off. Controls the
   Kyverno freeze policy.
2. **Service**: each election-critical service may opt in by labelling its Deployment with
   `eel.internal/election-day-mode-sensitive: "true"`. Services not labelled are excluded from
   the freeze even if the global flag is set.
3. **Module**: service-internal feature flags read the ConfigMap via projected volume or env var.
   Module-level toggling is the service owner's responsibility.

## Files

| File | Purpose |
|---|---|
| `election-day-mode-configmap.yaml` | Default ConfigMap (flag disabled). Deployed to prod only. |
| `freeze-window-policy.yaml` | Kyverno ClusterPolicy that blocks wave-1 Application syncs when flag is active. |
| `toggle-runbook.md` | Pointer to `docs/runbooks/election-day-mode-toggle.md`. |
| `election-day-mode-cli/toggle.sh` | Bash wrapper for kubectl that enforces dual SVID sign-off via cosign. |

## Kyverno validation

```bash
kubectl kyverno validate platform/election-day-mode/freeze-window-policy.yaml
```

Best-effort: requires Kyverno CLI >= 1.11.

## Activate / deactivate

See `toggle-runbook.md`. Never toggle directly with `kubectl edit`; always use the CLI to
ensure the audit trail and cosign attestation are recorded.
