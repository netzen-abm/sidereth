# SIDERETH Core v1.2-B — Test Matrix

Status: DRAFT

| Area | Required evidence |
|---|---|
| Create | Case and incident survive process restart |
| Read | Persisted values round-trip without semantic loss |
| Update | Expected revision succeeds and increments revision |
| Concurrency | Stale revision is rejected without overwriting newer data |
| Events | Duplicate event IDs are rejected; existing events are unchanged |
| Idempotency | Recorded operation remains discoverable after restart |
| Schema | Zero/invalid schema version is rejected |
| Corruption | Invalid persisted JSON returns a typed adapter failure |
| Isolation | Separate roots cannot read each other's records |
| Domain independence | Adapter implementation changes do not require domain-model changes |
| Recovery | Interrupted temporary write does not replace an existing valid record |

## Acceptance rule

The adapter is mergeable only when the same SIDERETH domain semantics are demonstrated through the adapter and the strict repository CI remains green.
