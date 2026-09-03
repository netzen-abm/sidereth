# SIDERETH Persistence Boundary Audit

Status: REFERENCE

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

The findings in this audit are the gate for Core v1.2-C Persistence Contract Hardening.

## Findings

### P1 — Temporary-file naming is not concurrency-safe

`LocalFileStore::write` originally derived a single temporary path from the destination path. Concurrent writers targeting the same record could therefore collide on the same temporary file.

**v1.2-C action:** temporary paths are now uniquely derived for each write attempt.

### P1 — Create is check-then-write

Creation originally checked `path.exists()` and subsequently wrote. Two concurrent creators could both observe absence and race to create the record.

**v1.2-C action:** creation now prepares the complete serialized representation before publication and uses an exclusive filesystem creation boundary where supported.

### P1 — Idempotency is not atomic

The previous `lookup()` followed by `record()` sequence was not one atomic operation.

**v1.2-C action:** `claim()` is the authoritative idempotency mutation and uses exclusive file creation so the first claimant wins deterministically.

### P1 — Transaction contract has no executable adapter semantics

`Transaction` and `TransactionFactory` exist as provider-neutral interfaces, but the local adapter does not implement them.

**Decision:** unchanged. No transaction support is claimed until isolation, atomic commit, rollback, recovery and crash semantics are implemented and tested.

### P2 — Revision protection is useful but not a complete concurrency contract

Expected revisions provide logical optimistic concurrency, but filesystem updates are not a distributed concurrency mechanism.

**v1.2-C action:** the local reference adapter explicitly remains single-writer for update operations. Distributed/multi-host concurrency is outside this adapter's claim.

### P2 — Atomic rename is host-dependent

Filesystem replacement semantics vary by host/filesystem.

**Decision:** the adapter documents replacement as conditional on host filesystem guarantees and does not represent it as universal ACID transactionality.

### P2 — Schema version is present but migration semantics are undefined

**v1.2-C action:** the reference adapter now fails closed on versions other than its declared supported version. Migration remains an explicit future operation.

### P2 — Error model is too weak for a durable boundary

**v1.2-C action:** persistence operations now expose typed `PersistenceError` values for machine-readable handling.

### P2 — Evidence remains correctly separated

**Decision:** preserve evidence as an independent storage boundary using opaque storage references and content hashes.

### P2 — Authorization and audit remain outside storage

**Decision:** preserve authorization and audit as independent boundaries. Storage adapters do not become authorization engines.

### P3 — Dependency neutrality is healthy

The core remains free of database, cloud, ORM and hosted-service dependencies.

**Decision:** preserve provider neutrality.

## Contract verdict after v1.2-C

| Boundary | Verdict |
| --- | --- |
| Provider neutrality | PASS |
| Domain independence | PASS |
| Restart persistence | PASS for reference scope |
| Optimistic revision check | PASS at logical level |
| Immutable event IDs | PASS |
| Malformed-record detection | PASS |
| Schema version persistence | PASS; explicit compatibility gate |
| Atomic replacement | CONDITIONAL; host-dependent |
| Concurrent create safety | EXCLUSIVE creation boundary |
| Concurrent update safety | SINGLE-WRITER reference scope |
| Idempotency atomicity | PASS at claim boundary |
| Multi-object transactions | NOT IMPLEMENTED |
| Typed persistence errors | PASS |
| Evidence/provider separation | PASS |
| Authorization/storage separation | PASS |

## Architectural decision

SIDERETH continues to own **contracts**, while providers implement those contracts.

No PostgreSQL, Supabase, SQLite, S3, or other provider is promoted into the core architecture merely to solve persistence hardening findings.

The next provider work, if later justified, must be an independent adapter proving replaceability rather than becoming a core dependency.

## Evidence basis

This audit records the v1.2-B boundary assessment and the resulting v1.2-C hardening decisions. The original findings remain useful as architectural rationale even after individual implementation weaknesses are addressed.
