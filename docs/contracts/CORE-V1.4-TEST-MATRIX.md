# SIDERETH Core v1.4 — Test Matrix

**Status:** DRAFT

| Area | Required verification |
|---|---|
| Intent | Every supported command has explicit typed intent |
| Actor identity | Actor is carried into event and audit records |
| Operation identity | Every mutation requires an operation identity |
| Correlation | Command result/event preserves correlation identity |
| Authorization | Authorization occurs before idempotency and mutation |
| Validation | Invalid command targets are rejected deterministically |
| Domain invariant | Invalid case transitions are rejected |
| Concurrency | Transition uses the caller-provided expected revision |
| Idempotency | A repeated operation is rejected before a second mutation |
| Mutation | Case state is persisted through `CaseStore` only |
| Event | Successful mutation produces an attributable `EventEnvelope` |
| Event integrity | Duplicate event identity is rejected by the event store |
| Audit | Successful command produces an attributable audit record |
| Typed errors | Service failures remain machine-readable |
| Provider neutrality | No provider/transport/AI SDK enters the command boundary |
| Partial failure | State/event/audit failure semantics are explicit and tested where adapters permit |
| Determinism | Same valid command produces structurally deterministic identifiers/results |
| High-impact boundary | No government submission, legal judgment, or autonomous external action |

## Acceptance gate

A v1.4 implementation may merge only when Foundation CI passes and the implementation does not imply distributed atomicity that the underlying persistence contracts do not provide.
