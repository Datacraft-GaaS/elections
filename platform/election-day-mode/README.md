# platform/election-day-mode/

Election-day mode is a platform-wide operational flag that activates a freeze window for election-critical services during the period T-1 through T+7 (day before election through 7 days after). It has three levels.

## Flag hierarchy

```
global (Chair HSM-signed, this directory)
  |
  +-- service (per-service opt-in via label eel.internal/election-day-mode-sensitive: "true")
        |
        +-- module (feature-flag within a service, controlled by the service owner)
```

1. **Global** (`eel-election-day-mode` ConfigMap in `argocd` namespace): set by the Chair or designated election officer using the toggle CLI with dual SVID sign-off. Controls the Kyverno freeze policy.
2. **Service**: each election-critical service opts in by labelling its Deployment with `eel.internal/election-day-mode-sensitive: "true"`. Services not labelled are excluded from the freeze even if the global flag is set.
3. **Module**: service-internal feature flags read the ConfigMap via projected volume or env var. Module-level toggling is the service owner's responsibility.

## Files

| File | Purpose |
|---|---|
| `election-day-mode-configmap.yaml` | Default ConfigMap (flag disabled). Deployed to prod only. |
| `freeze-window-policy.yaml` | Kyverno `ClusterPolicy` that blocks wave-1 Application syncs when flag is active. |
| `toggle-runbook.md` | Pointer to `docs/runbooks/election-day-mode-toggle.md`. |
| `election-day-mode-cli/toggle.sh` | Bash wrapper for kubectl that enforces dual SVID sign-off via cosign. |

## Activate / deactivate

```bash
# Activate (requires two SVID-signed approvals):
bash platform/election-day-mode/election-day-mode-cli/toggle.sh activate \
  --reason "Election Day 2027-08-09" \
  --approver-1 alice.svid \
  --approver-2 bob.svid

# Deactivate:
bash platform/election-day-mode/election-day-mode-cli/toggle.sh deactivate \
  --reason "Results certified 2027-08-16" \
  --approver-1 alice.svid \
  --approver-2 bob.svid
```

Never toggle directly with `kubectl edit`; always use the CLI to ensure the audit trail and cosign attestation are recorded.

## Validate Kyverno policy

```bash
kubectl kyverno validate platform/election-day-mode/freeze-window-policy.yaml
```

## Limitations

- The freeze applies to ArgoCD `Application` sync only. Manual `kubectl apply` is not blocked (require RBAC controls separately).
- Module-level flags are the responsibility of individual service owners; the global flag does not cascade automatically to module toggles.

## Operator responsibilities

- Activation and deactivation events are logged to the transparency log (transparency-svc).
- The dual-approver requirement mirrors the four-eyes principle for IEBC result certification.
- Test activation and deactivation in the staging cluster at least 72 hours before election day.

## Related docs

- `docs/runbooks/election-day-mode-toggle.md`: full toggle procedure
- `platform/kyverno-policies/README.md`: base admission policies
- `platform/argocd/README.md`: ArgoCD sync controlled by the freeze policy
