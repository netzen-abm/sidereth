# SIDERETH Universal Core Post-UoW Re-Audit — 2026-09-06

## Scope

Re-audit of the merged Universal Core after PR #31 (provider-neutral Unit-of-Work + PostgreSQL adapter), against the architectural gate that must precede the Decision Model.

Baseline: `main` at `e28dcbae8c7dc27df75e17c30ddc4de84c5f321c`.

## Executive decision

**DECISION MODEL: BLOCKED.**

The Unit-of-Work boundary is a sound infrastructure addition, but the current application/service layer does not yet consume it. The repository therefore contains two persistence paths: the newer provider-neutral UoW contract and the older typed repository/store path. This is an architectural boundary problem, not merely cleanup.

No destructive migration is recommended. Existing contracts remain useful for compatibility and tests; the next increment should introduce a UoW-backed application command path and explicitly classify legacy repositories as compatibility/local adapters.

## Findings

### CORE-013 — P0/P1: application command path bypasses Unit-of-Work

`CaseService::execute` directly calls `CaseStore`, `EventStore`, and `IdempotencyStore`, then writes audit separately. It does not begin, execute, commit, or rollback a UoW.

**Impact:** a case mutation can succeed while event/audit persistence fails, leaving an inconsistent workflow. Idempotency can also be claimed before later mutation failure.

**Decision:** BLOCK Decision Model until at least the authoritative command path is UoW-backed or the architecture explicitly constrains the legacy path to non-authoritative compatibility use.

### CORE-014 — P1: legacy repository and persistence abstractions create a second architecture

`CaseRepository`/`IncidentRepository`/`EventRepository` and `CaseStore`/`IncidentStore`/`EventStore` overlap. The UoW is now the intended cross-resource atomic boundary, but the current service still depends on the older store interfaces.

**Decision:** retain for source compatibility, but stop expanding them. New authoritative workflows should target UoW-backed application ports.

### CORE-015 — P1: idempotency is not transactionally coupled to the business mutation

The service claims the operation before creating/updating the case. With the current LocalFileStore path, the claim is durable independently of later mutation/event/audit success.

**Impact:** a failed operation may become permanently non-retryable even though the business mutation did not complete.

**Decision:** idempotency claim and authoritative mutation must share the same atomic boundary, or use an explicit pending/completed idempotency protocol.

### CORE-016 — P1: audit/event atomicity is not enforced in service layer

The service writes the event and audit after the business mutation. There is no rollback boundary spanning all three operations.

**Impact:** durable state, event spine, and audit trail can diverge.

**Decision:** authoritative state-changing commands must commit resource mutation + event + audit/provenance as one transaction where the provider supports it.

### CORE-017 — P1: PostgreSQL error mapping is too coarse

The PostgreSQL adapter maps most database errors to `Unavailable`, while insert conflicts are also broadly mapped to `Duplicate`.

**Impact:** callers cannot reliably distinguish timeout, serialization failure, constraint conflict, authorization failure, integrity failure, or transient availability failure.

**Decision:** improve error classification before production-grade retry/idempotency semantics are declared.

### CORE-018 — P1: PostgreSQL UoW uses a shared `Arc<Mutex<Client>>`

The adapter serializes access through a mutex around one PostgreSQL client. This is safe as a basic boundary but is not yet a production connection-pooling/concurrency strategy.

**Decision:** acceptable for the adapter increment; defer pooling until connection lifecycle, concurrency, isolation, and operational policy are specified and tested.

### CORE-019 — P1: resource-link referential integrity is not enforced by the generic PostgreSQL schema

The polymorphic link model stores resource type/id pairs, so normal SQL foreign keys cannot enforce cross-table references. The UoW contract currently accepts a link without verifying target existence.

**Impact:** dangling links are possible unless application-level integrity checks exist.

**Decision:** define explicit link-integrity semantics: either allow forward/dangling references by contract, or require existence checks for selected relation classes.

### CORE-020 — P1: event aggregate identity remains stringly typed

`EventEnvelope` has `aggregate_type: String` + `aggregate_id: Id`, then maps strings to `ResourceType` through a fallback to `Other`.

**Impact:** typos can silently become `Other`; the canonical boundary is weaker than `ResourceRef`.

**Decision:** future event contract should use or derive from `ResourceRef`; retain current fields only for compatibility during migration.

### CORE-021 — P1: provenance is modeled but not coupled to state-changing persistence

`Provenance` is a first-class contract, but current `CaseService` command execution does not create or persist a provenance record as part of the authoritative mutation boundary.

**Decision:** every authoritative state-changing operation should have a provenance reference and durable provenance record, transactionally linked where applicable.

### CORE-022 — P1: lifecycle transition contract permits missing authorization/provenance references

`LifecycleTransition` exposes optional authorization/provenance references, but validation does not enforce when they are mandatory for sensitive transitions.

**Decision:** keep the universal contract flexible; enforce requirement through explicit policy/action class contracts rather than silently making every lifecycle transition legally authoritative.

### CORE-023 — P2: `ResourceWriteMode` has no explicit wire naming policy

The enum currently lacks `snake_case` serialization while other public contract enums explicitly use it.

**Decision:** normalize wire serialization in a small compatibility-safe hardening change and add deterministic JSON tests.

### CORE-024 — P2: timestamp validation remains syntactic

Core contracts generally require non-empty timestamp strings; JSON schemas may require `date-time`, but Rust validation does not consistently parse/validate timestamps.

**Decision:** defer stronger temporal typing until a canonical time policy is adopted across all contracts.

## Verified strengths

- `ResourceRef` is now an explicit ecosystem boundary while preserving `Id = String` source compatibility.
- UoW is provider-neutral and separates domain atomicity from persistence-provider mechanics.
- PostgreSQL adapter performs real `BEGIN` / `COMMIT` / `ROLLBACK` operations.
- Drop-based rollback protects an active PostgreSQL UoW from being silently left open.
- Revision semantics provide an optimistic-concurrency primitive.
- Local file persistence uses atomic create/update patterns and atomic idempotency claim creation.
- Action, lifecycle, audit, provenance, and event contracts are distinct rather than collapsed into one overloaded model.
- Security/supply-chain checks remain part of the repository verification baseline.

## Gate result

| Gate | Result |
|---|---|
| Lifecycle ↔ Event ↔ Audit conceptual separation | PASS with integration gap |
| Authorization ↔ Action separation | PASS with enforcement gap |
| Authorization ↔ UoW coupling | BLOCKED |
| Provenance coverage | BLOCKED |
| Document invariant ↔ persistence atomicity | PASS at domain/UoW boundary; application adoption incomplete |
| ResourceRef ↔ PostgreSQL polymorphic persistence | PASS with referential-integrity policy gap |
| DerivedArtifact duplication | OPEN P1 architectural cleanup |
| Privacy coverage | OPEN P1 |
| Schema/version evolution | PASS baseline; policy hardening remains |
| Idempotency + retries | BLOCKED |
| Concurrency/revision semantics | PASS baseline; service adoption incomplete |
| Rollback semantics | PASS for PostgreSQL UoW; not used by CaseService |
| Legacy persistence bypass | BLOCKED |
| Provider neutrality | PASS at UoW contract level |

## Required next increment

1. Introduce an authoritative UoW-backed application command port that can atomically persist domain mutation, idempotency outcome, event, audit, and provenance.
2. Classify existing repositories/stores as compatibility/local adapters and prevent new domain services from bypassing the authoritative command/UoW path.
3. Harden PostgreSQL error classification and define resource-link integrity semantics.
4. Normalize remaining wire-contract inconsistencies (`ResourceWriteMode`) without breaking source compatibility.
5. Re-audit and only then unlock the Decision Model.

## Architectural principle reaffirmed

**One canonical domain. One authoritative mutation boundary. Multiple adapters and surfaces.**

AI, transport, UI, storage provider, and integration code may vary; authoritative legal/regulatory state must not acquire multiple competing mutation semantics.
