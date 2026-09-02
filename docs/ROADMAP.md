# SIDERETH — Master Roadmap V1

## Phase 0 — Foundation & decision baseline
- Canonical product decisions
- Master blueprint
- Decision register
- Repository migration rules
- Branch/legacy audit
- CI foundation

Gate: documentation and repository truth agree.

## Phase 1 — Platform contracts
- Canonical domain model
- Database schema
- Event schema
- OpenAPI/API contract
- Authorization matrix
- Error/idempotency/versioning contracts
- Audit schema
- Storage/encryption model
- Capability contracts

Gate: contracts reviewed before production implementation.

## Phase 2 — Universal core
- Identity
- Policy
- Authorization
- Case
- Incident
- Event processing
- Evidence
- Documents
- Legal Source
- Jurisdiction
- Authority
- Procedure
- Deadline
- Compliance
- Application
- Response
- Escalation/Remedy
- Human Assistance
- Audit/Observability

Gate: useful deterministic workflows without generative AI.

## Phase 3 — Trust, privacy & security
- Source verification/provenance
- evidence integrity
- encryption/key management
- minimization/redaction
- retention/deletion/export
- abuse controls
- security testing
- privacy testing
- audit verification

Gate: sensitive workflows meet security/privacy requirements.

## Phase 4 — Agent platform
- Tool Registry
- Tool Identity
- Tool Gateway
- bounded Tool Runtime
- Memory Bank
- Model Armor
- asynchronous/resumable workflows
- scheduled/event-driven workflows
- human approval checkpoints
- observability

Gate: agents cannot bypass identity, policy, permission or approval controls.

## Phase 5 — First domains
- Panchayat
- Municipality

Build complete, verified journeys rather than broad shallow coverage.

## Phase 6 — Application Guardian
- application readiness
- eligibility
- documentary completeness
- jurisdiction
- fees
- timing
- sequence
- proof of submission
- decision/rejection tracking
- response/escalation

## Phase 7 — Protect Now
- inspection
- search
- seizure
- raid
- questioning
- notice
- chronology/evidence capture
- immediate lawful guidance
- human assistance
- offline-first operation

## Phase 8 — Domain expansion
Tier 1 → Tier 2 → Tier 3, using shared infrastructure and domain adapters.

## Phase 9 — Advanced legal intelligence
- judgment research
- case-law analysis
- precedent comparison
- litigation support
- advanced contracts
- detailed legal research
- legal strategy assistance

## Release gates
Every phase requires: architecture consistency, implementation evidence, automated tests, security/privacy review appropriate to risk, documentation update, and a recorded decision on readiness. Do not merge or release merely because code compiles.
