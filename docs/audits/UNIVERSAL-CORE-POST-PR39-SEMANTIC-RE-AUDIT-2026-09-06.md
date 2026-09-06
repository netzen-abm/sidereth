# SIDERETH Universal Core — Post-PR #39 Semantic Re-Audit

**Date:** 2026-09-06  
**Scope:** Universal Core, authoritative command path, Unit of Work, PostgreSQL adapter, CaseService mutation boundary  
**Reference:** PR #39 — `Core: migrate CaseService to authoritative atomic command path`

## Executive conclusion

PR #39 closes the previously identified P0 mutation-path bypass in `CaseService`: Case creation and transition now enter the provider-neutral authoritative command boundary and persist Case, Event, Audit, Provenance, and Idempotency records through one Unit-of-Work write set.

The Decision Model gate is **not yet open**. The persistence kernel is materially stronger, but several production-grade semantic guarantees remain incomplete and must be resolved before decisioning, legal reasoning, or agentic workflows are allowed to depend on the kernel.

## Verified closed findings

### CORE-013 — CaseService bypass of authoritative UoW
**Status: CLOSED**

`CaseService::execute` now authorizes first, creates an `AtomicCommandPlan`, claims idempotency inside that plan, and executes the case mutation through `execute_authoritative_command`.

### CORE-015 — Transactional idempotency coupling
**Status: CLOSED for CaseService**

The idempotency claim is inserted into the same authoritative write set as the domain mutation. A failed command is rolled back; a duplicate operation is rejected by the same transaction boundary.

### CORE-016 — Event atomicity
**Status: CLOSED for CaseService**

Case and Event writes share the same Unit of Work. Event persistence is no longer a separate post-mutation side effect in the authoritative Case path.

### CORE-017 — PostgreSQL SQLSTATE mapping
**Status: IMPROVED / PARTIALLY CLOSED**

The adapter maps duplicate, integrity, serialization/deadlock conflict, and statement-timeout SQLSTATE classes explicitly. Remaining provider-specific mapping policy should be expanded only when concrete error semantics are required.

### CORE-021 — Provenance coupling
**Status: CLOSED for CaseService**

Provenance is persisted in the same write set as the state-changing Case command.

### CORE-013 rollback integrity extension
**Status: CLOSED**

The PostgreSQL Unit of Work no longer performs an implicit rollback inside `UnitOfWork::execute` and discard the result. The authoritative command executor owns rollback and can therefore preserve a rollback failure as `AuthoritativeCommandError::RollbackFailure`.

## Remaining findings

### CORE-018 — PostgreSQL concurrency architecture
**Status: OPEN / P1**

`Arc<Mutex<Client>>` remains a first implementation, not the production concurrency strategy. Connection pooling, transaction ownership, request isolation, and connection lifecycle need a deliberate provider-neutral policy before production scale.

### CORE-019 — ResourceLink semantics
**Status: OPEN / P1**

Polymorphic resource links intentionally do not use conventional foreign keys. The contract still needs an explicit classification of strong internal references, forward references, and external references, plus duplicate/link conflict semantics.

### CORE-020 — Event aggregate identity
**Status: OPEN / P1

The authoritative Case path still emits compatibility-style `aggregate_type` / `aggregate_id` fields. The canonical future contract should bind event identity to `ResourceRef` while retaining compatibility fields until migration is complete.

### CORE-022 — Authorization/provenance policy enforcement
**Status: OPEN / P1

The Case path enforces authorization before mutation and records provenance, but the universal contract still permits optional authorization/provenance metadata. Policy classes must define when these become mandatory.

### CORE-024 — Timestamp typing
**Status: OPEN / P2

The Case path currently uses service-layer placeholder timestamp values. The universal contract should eventually use a canonical timestamp representation and clock/source policy.

### New finding — PostgreSQL integration semantics
**Status: OPEN / P0/P1 test gap**

The existing test suite verifies the contract and mock transaction behavior, but there is not yet a live PostgreSQL integration suite proving:

1. two concurrent writers using the same expected revision yield exactly one successful mutation;
2. a failed authoritative command leaves Case, Event, Audit, Provenance, and Idempotency unchanged;
3. duplicate operation IDs are transactionally rejected without partial state;
4. rollback failure is surfaced distinctly from the original operation failure;
5. transaction isolation behaves as assumed under concurrent reads/CAS.

These are the highest-value remaining persistence tests.

## Decision Model gate

| Gate | Status |
|---|---|
| Transactional reads | PASS |
| Optimistic CAS | PASS at contract/adapter level; live concurrency test required |
| Atomic command execution | PASS |
| Transactional idempotency | PASS for CaseService |
| Atomic event | PASS for CaseService |
| Atomic audit | PASS for CaseService |
| Atomic provenance | PASS for CaseService |
| Authoritative Case mutation | PASS |
| Explicit authorization boundary | PASS for CaseService |
| Conflict semantics | PASS at contract level; integration verification pending |
| Revision semantics | PASS at contract/adapter level |
| Cross-resource reference semantics | **BLOCKED** |
| Legacy Case mutation path no longer authoritative | PASS for CaseService; repository-wide legacy consumers remain compatibility-only |
| Production transaction/concurrency strategy | **BLOCKED** |
| Live PostgreSQL atomicity/concurrency verification | **BLOCKED** |

## Decision

**DO NOT OPEN THE DECISION MODEL GATE YET.**

The correct next move is not to add more domain intelligence. First complete the live PostgreSQL semantic test suite and formalize ResourceLink/reference semantics. After those are green, re-audit authorization/provenance policy requirements and then reconsider the Decision Model gate.

## Next 2 engineering actions

1. Build a live PostgreSQL integration test matrix for CAS races, rollback atomicity, duplicate idempotency, and transaction isolation.
2. Define and test canonical ResourceLink semantics and lifecycle, including strong, forward, and external references.
