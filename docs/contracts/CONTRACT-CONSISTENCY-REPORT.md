# SIDERETH — Contract Consistency Report V1

## Purpose
Verify that the canonical domain model, JSON schemas, Rust primitives, event/state contracts, authorization, error/idempotency/versioning, and audit/storage contracts describe the same system boundary.

## Audit baseline
- Domain model: `docs/contracts/CANONICAL-DOMAIN-MODEL.md`
- Capability boundary: `docs/contracts/CAPABILITY-CONTRACT.md`
- Events/states: `docs/contracts/CASE-INCIDENT-EVENTS.md`
- Authorization: `docs/contracts/AUTHORIZATION-MATRIX.md`
- API semantics: `docs/contracts/API-ERROR-IDEMPOTENCY-VERSIONING.md`
- Audit/storage: `docs/contracts/AUDIT-STORAGE-ENCRYPTION.md`
- Schemas: `schemas/case.schema.json`, `schemas/incident.schema.json`
- Rust foundation: `src/lib.rs`

## Findings

### C-001 — Contract ownership
**Status: Resolved by policy.**

The canonical domain model is authoritative for domain concepts; JSON Schema is authoritative for interchange validation; Rust types are the executable foundation representation; event contracts govern state-changing history. None may silently redefine another.

### C-002 — State/event boundary
**Status: Resolved by baseline.**

Case and Incident lifecycle states are represented separately from append-only events. A derived state must be reproducible from valid events plus deterministic transition rules.

### C-003 — Source provenance
**Status: Resolved by baseline.**

System-generated legal propositions require source references. User facts and captured evidence are not to be represented as legal authority merely because an AI or workflow classified them.

### C-004 — Authorization/high-impact actions
**Status: Resolved by baseline.**

Mutating and high-impact actions require scoped authorization. Legal submissions, appeals, consequential external communications and comparable high-impact actions require explicit human approval.

### C-005 — Evidence integrity
**Status: Resolved by baseline.**

Original evidence is immutable. OCR, extraction, summaries and analyses are derived artifacts and must not overwrite originals.

### C-006 — Versioning/idempotency
**Status: Resolved by contract draft.**

API/event schemas require explicit versions; commands that may be retried require idempotency semantics; concurrent updates require conflict detection rather than silent last-write-wins behavior.

### C-007 — Audit boundary
**Status: Resolved by contract draft.**

Security/policy decisions and high-impact user approvals are auditable. Sensitive payloads should not be copied into audit records unnecessarily.

## Remaining implementation gates
1. Generate/maintain a database schema from the canonical model without introducing competing semantics.
2. Add executable JSON Schema validation tests.
3. Add Rust state-transition tests covering valid and invalid transitions.
4. Add event replay tests proving deterministic reconstruction of lifecycle state.
5. Add authorization tests for allow/deny/high-impact approval cases.
6. Add idempotency/concurrency tests at the command boundary.
7. Define the OpenAPI contract from the same capability and domain model.

## Rule
No production API or UI should be treated as canonical if it invents a competing domain meaning. Contracts are a shared infrastructure layer for every future surface and adapter.
