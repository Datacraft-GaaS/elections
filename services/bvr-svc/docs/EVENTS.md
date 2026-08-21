# BVR Domain Events

All events use CloudEvents 1.0 envelope (eel-events-rs).

| event type | topic | data class | signed |
|---|---|---|---|
| eel.bvr.voter_created.v1 | eel.bvr.voter.v1 | B | yes |
| eel.bvr.voter_amended.v1 | eel.bvr.voter.v1 | B | yes |
| eel.bvr.voter_deregistered.v1 | eel.bvr.voter.v1 | B | yes |
| eel.bvr.register_frozen.v1 | eel.bvr.voter.v1 | C | yes |
| eel.bvr.register_snapshot_published.v1 | eel.bvr.audit.v1 | C | yes |
| eel.bvr.dedup_ordered.v1 | eel.bvr.adjudication.v1 | B | yes |
| eel.bvr.dedup_resolved.v1 | eel.bvr.adjudication.v1 | B | yes |
| eel.bvr.canary_touched.v1 | eel.bvr.security.v1 | D | yes |
