# SIDERETH Core v1.2-C — Persistence Contract Hardening

Status: DRAFT

## Objective

Harden the provider-neutral persistence boundary identified by the v1.2 persistence audit without selecting or requiring a storage provider.

## Contract decisions

### 1. Typed failures

Persistence operations return `PersistenceError` rather than static strings. Errors distinguish conflict, duplicate, unavailable, integrity, validation, unsupported schema, not-found and idempotency outcomes.

### 2. Concurrency

The domain contract requires stale mutations to fail deterministically. The reference local adapter remains a single-writer reference implementation for update operations; it does not claim distributed or multi-host locking.

### 3. Create semantics

Creation must never silently replace an existing stable identifier. The local adapter prepares the complete serialized object before publishing it and uses an exclusive filesystem creation boundary where supported.

### 4. Idempotency

Idempotency is no longer modeled as `lookup` followed by `record`. `claim` is the authoritative mutation and must atomically establish ownership of an operation identifier or report that it was already claimed.

### 5. Schema compatibility

Persisted objects carry an explicit schema version. The reference adapter accepts only its declared supported version and fails closed on unsupported versions. Migration is an explicit future operation, not an implicit read-time mutation.

### 6. Atomic replacement

Temporary-file replacement is an adapter guarantee only where the host filesystem provides the required semantics. SIDERETH does not treat filesystem rename as universal ACID transactionality.

### 7. Transactions

The generic transaction interface remains a contract boundary. No adapter may imply multi-object transaction support until isolation, atomic commit, rollback, crash recovery and failure semantics are explicitly implemented and tested.

## Provider neutrality

No database, ORM, cloud SDK, object-store SDK or hosted service is introduced by this milestone.

The hardening applies to contracts and the reference local adapter only.

## Explicit non-goals

- distributed locking
- multi-host transaction processing
- database selection
- cloud provider selection
- automatic schema migration
- production backup/replication
- autonomous recovery that invents domain state
