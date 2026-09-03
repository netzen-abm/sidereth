# SIDERETH Core v1.2-B — Reference Local Durable Adapter

Status: DRAFT

## Objective

Provide a technology-neutral reference implementation using the host filesystem and the existing generic serialization dependencies.

This is a reference adapter, not the canonical SIDERETH storage architecture and not a production deployment recommendation.

## Design

The adapter stores each aggregate/object as an independently addressed JSON file beneath a caller-selected root directory.

```text
SIDERETH Core
    |
Persistence Contract
    |
Local File Adapter
    |
Host Filesystem
```

No cloud SDK, database driver, ORM, or hosted service is required.

## Durability semantics

- writes use a temporary file followed by atomic rename where supported by the host filesystem
- duplicate creation is rejected
- updates require the expected revision
- every successful update increments revision
- event append is immutable by event ID
- idempotency records are durable
- malformed persisted objects fail rather than being silently repaired
- schema version is persisted with each object

## Portability

The adapter is intentionally isolated from domain logic.

A future relational, object, encrypted-device, or remote adapter must implement the same contract without changing domain types.

## Scope

Included:
- Case
- Incident
- Event
- idempotency records
- restart persistence
- optimistic concurrency
- corruption/malformed-record detection

Not included:
- encryption-at-rest implementation
- distributed locking
- multi-host transactions
- production backup
- replication
- access-control policy
- legal hold policy implementation
- cloud provider integration

Those remain higher-level or provider-specific concerns governed by existing SIDERETH contracts.
