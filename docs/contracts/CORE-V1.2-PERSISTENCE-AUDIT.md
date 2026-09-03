# SIDERETH Persistence Boundary Audit

Status: DRAFT

## Audit target

Current `main` after Core v1.2-B.

Audited boundaries:
- `src/persistence.rs`
- `src/local_store.rs`
- authorization and audit interaction
- evidence storage separation
- dependency neutrality
- durability and concurrency semantics
- schema/version handling
- idempotency
- restart/recovery
- portability claims

## Executive decision

**Do not introduce a database or cloud provider yet.**

The persistence boundary is correctly provider-neutral at the dependency level, and the local adapter proves that a non-database implementation can satisfy the current contracts. However, the reference adapter should not yet be treated as a concurrency-safe production persistence implementation.

The next engineering change should strengthen the persistence contracts and local adapter semantics before adding another provider.

## Findings

### P1 — Temporary-file naming is not concurrency-safe

`LocalFileStore::write` derives a single temporary path from the destination path. Concurrent writers targeting the same record can therefore collide on the same temporary file.

**Required action:** use uniquely named temporary files and define the rename/replace atomicity contract explicitly.

### P1 — Create is check-then-write

Creation checks `path.exists()` and subsequently writes. Two concurrent creators can both observe absence and race to create the record.

**Required action:** make creation exclusive at the filesystem primitive level, or explicitly scope the adapter as single-writer until an exclusive-create mechanism is implemented.

### P1 — Idempotency is not atomic

`lookup()` followed by `record()` is not one atomic operation. Concurrent workers can both observe an operation as absent and both proceed.

**Required action:** define an atomic claim/insert-or-existing operation in the idempotency contract. Keep lookup for read-only inspection if useful.

### P1 — Transaction contract has no executable adapter semantics

`Transaction` and `TransactionFactory` exist as provider-neutral interfaces, but `LocalFileStore` does not implement them. The current adapter therefore does not demonstrate multi-object transactionality.

**Required action:** keep transactions out of the local adapter until the contract specifies isolation, atomic commit, rollback behavior, failure recovery, and crash semantics. Do not imply transaction support merely because the interface exists.

### P2 — Revision protection is useful but not a complete concurrency contract

Updates compare an expected revision before writing, which provides optimistic concurrency at the logical level. The filesystem write itself is still subject to concurrent-writer races.

**Required action:** make the concurrency guarantee explicit: either serialize writers, use an exclusive lock/claim protocol, or document the adapter as single-writer.

### P2 — Atomic rename is host-dependent

The design correctly qualifies rename as atomic only where supported. That qualification must remain prominent because filesystem semantics differ across platforms and filesystems.

**Required action:** document supported durability assumptions and failure behavior rather than treating rename as a universal ACID primitive.

### P2 — Schema version is present but migration semantics are undefined

Persisted objects carry a schema version, which is correct. The current adapter validates only that the version is non-zero and does not define supported-version negotiation or migration behavior.

**Required action:** establish a canonical schema compatibility/migration policy before persisted schema versions begin changing.

### P2 — Error model is still too weak for a durable boundary

The persistence interfaces use `Result<_, &'static str>`. This is sufficient for the reference proof but loses machine-readable distinctions needed by higher layers for retry, conflict handling, corruption response, authorization separation, and observability.

**Required action:** introduce typed persistence errors before production adapters are built.

### P2 — Evidence remains correctly separated

The evidence subsystem uses an opaque storage reference and content hash as integrity identity. It is not coupled to the local case/incident store.

**Decision:** preserve this separation. Do not collapse evidence persistence into generic aggregate persistence.

### P2 — Authorization and audit remain outside storage

Authorization is a policy boundary and audit is a separate sink. The persistence contract does not directly embed a provider or policy implementation.

**Decision:** preserve this separation. Storage adapters must not become authorization engines.

### P3 — Dependency neutrality is healthy

The core Cargo manifest contains only generic serialization and hashing dependencies; no database, cloud, ORM, or hosted-service SDK is present.

**Decision:** no provider dependency should be added to the core crate.

## Contract verdict

| Boundary | Verdict |
| --- | --- |
| Provider neutrality | PASS |
| Domain independence | PASS |
| Restart persistence | PASS for reference scope |
| Optimistic revision check | PASS at logical level |
| Immutable event IDs | PASS |
| Malformed-record detection | PASS |
| Schema version persistence | PASS; migration policy pending |
| Atomic replacement | CONDITIONAL; host-dependent |
| Concurrent create safety | FAIL for production use |
| Concurrent update safety | CONDITIONAL |
| Idempotency atomicity | FAIL for production use |
| Multi-object transactions | NOT IMPLEMENTED |
| Typed persistence errors | NOT YET SUFFICIENT |
| Evidence/provider separation | PASS |
| Authorization/storage separation | PASS |

## Architectural decision

SIDERETH continues to own **contracts**, while providers implement those contracts.

No PostgreSQL, Supabase, SQLite, S3, or other provider is to be promoted into the core architecture merely to solve the findings above.

The next milestone should be a **Persistence Contract Hardening** change that:

1. introduces typed persistence errors;
2. defines concurrency semantics;
3. makes idempotency claim semantics explicit;
4. defines schema compatibility/version policy;
5. clarifies filesystem atomicity and recovery guarantees;
6. adds tests for concurrent or explicitly single-writer behavior;
7. keeps the core provider-neutral.

Only after this contract is stable should a second independent adapter be considered as a replaceability proof.

## Evidence basis

The audit is based on the current `main` implementation and the v1.2-B design. The persistence contract exposes provider-neutral Case, Incident, Event, transaction, and idempotency interfaces; the local adapter persists JSON records under a caller-selected root and uses revision checks and temporary-file replacement. The design explicitly excludes production encryption, distributed locking, multi-host transactions, replication, and provider integration.
