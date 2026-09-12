# SIDERETH — Observation Conformance

**Status:** IMPLEMENTED BASELINE / LIFECYCLE CONFORMANCE IN PROGRESS  
**Contract:** `docs/contracts/OBSERVATION-CONTRACT.md`  
**Decision:** D-032  
**Scope:** Provider-neutral Observation semantics and authoritative creation workflow

This matrix defines the evidence required to establish Observation conformance. The repository now contains a minimal Observation semantic primitive and a bounded authoritative creation workflow. Remaining lifecycle, contradiction, temporal-uncertainty, privacy/purpose, and provider-neutral conformance work is explicitly tracked below.

## 1. Conformance principles

1. Observation is an epistemic assertion, not automatically a fact.
2. Event, Evidence, Observation and Provenance retain separate ownership.
3. Observation origin/modality is distinct from epistemic status.
4. Existing ResourceLink semantics are reused.
5. Longitudinal views remain projections over canonical primitives.
6. AI and providers cannot self-upgrade epistemic status.
7. Historical observations are not destructively rewritten.
8. Contradiction remains explicit until an authorized process resolves it.
9. Privacy and authorization are enforced outside presentation convenience.
10. Provider-neutral behavior is tested independently of implementation technology.

## 2. Status vocabulary

| Status | Meaning |
|---|---|
| **IMPLEMENTED** | Repository implementation exists for the requirement. |
| **TESTED** | Implementation is covered by an automated or equivalent conformance test. |
| **SECURITY-VERIFIED** | Security-specific adversarial behavior has been verified. |
| **PENDING** | Contract requirement is accepted but implementation/conformance evidence is not yet complete. |
| **NOT IN SCOPE YET** | Deliberately deferred by the current bounded implementation scope. |

A requirement is not treated as fully conforming merely because a related field or primitive exists.

## 3. Contract matrix

| ID | Area | Requirement | Verification | Expected result | Current status |
|---|---|---|---|---|---|
| OBS-001 | Identity | Stable observation identity | Create valid Observation | ID is preserved | TESTED |
| OBS-002 | Versioning | Positive schema version | Use zero/invalid version | Rejected | TESTED |
| OBS-003 | Subject | Subject uses ResourceRef | Supply valid subject reference | Reference is preserved | TESTED |
| OBS-004 | Type | Observation type is explicit | Omit/blank type | Rejected | PENDING |
| OBS-005 | Origin | Observation origin is explicit | Create measurement/user-report/direct-observation inputs | Origin is preserved and is not treated as proof | TESTED |
| OBS-006 | Assertion | Assertion/value is explicit | Create valid assertion | Content is preserved | TESTED |
| OBS-007 | Epistemic | Status is mandatory | Omit status | Rejected | PENDING |
| OBS-008 | Epistemic | Canonical vocabulary is reused | Use supported status | Round trip without semantic loss | TESTED |
| OBS-009 | Epistemic | INFERRED remains INFERRED | Attempt automatic upgrade | Upgrade rejected | TESTED |
| OBS-010 | Epistemic | UNVERIFIED remains UNVERIFIED | Attempt automatic upgrade | Upgrade rejected | TESTED |
| OBS-011 | Epistemic | CONTESTED remains CONTESTED | Supply contradiction | Contest remains visible | PENDING |
| OBS-012 | Epistemic | UNKNOWN remains UNKNOWN | Insufficient evidence | No stronger status inferred | PENDING |
| OBS-013 | Epistemic | USER_REPORTED is distinct | User report without independent support | Status remains USER_REPORTED | TESTED |
| OBS-014 | Origin/status | MEASUREMENT is origin, not a parallel epistemic status | Instrument measurement | Origin is MEASUREMENT; epistemic status uses canonical vocabulary | TESTED |
| OBS-015 | Epistemic | SOURCE_SUPPORTED is distinct | Source-backed assertion | Status remains SOURCE_SUPPORTED | PENDING |
| OBS-016 | Epistemic | EVIDENCE_SUPPORTED is distinct | Evidence-backed observation | Status remains EVIDENCE_SUPPORTED | PENDING |
| OBS-017 | Time | observed_at required | Missing observation time | Rejected | TESTED |
| OBS-018 | Time | recorded_at required | Missing recording time | Rejected | TESTED |
| OBS-019 | Time | Times are distinct | Different observed/recorded times | Both preserved | TESTED |
| OBS-020 | Time | Clock uncertainty preserved | Uncertain device timestamp | Uncertainty is not erased | PENDING |
| OBS-021 | Source | Source references preserved | Attach source refs | References survive round trip | PENDING |
| OBS-022 | Evidence | Evidence references are typed | Attach evidence refs | References remain traceable | PENDING |
| OBS-023 | Evidence | Evidence ownership not duplicated | Attempt to embed canonical evidence identity | Observation references evidence; does not own it | PENDING |
| OBS-024 | Provenance | Provenance preserved | Attach provenance | Provenance remains traceable | TESTED |
| OBS-025 | Provenance | Provenance does not equal truth | Source has provenance but weak epistemic basis | Status does not auto-strengthen | TESTED |
| OBS-026 | ResourceLink | Canonical links reused | Link Observation to Evidence | ResourceLink semantics apply | PENDING |
| OBS-027 | ResourceLink | No class inference | Legacy class-less link with local endpoint | Must not become Strong | PENDING |
| OBS-028 | ResourceLink | Strong semantics respected | Missing endpoint | Strong relationship rejected atomically | PENDING |
| OBS-029 | ResourceLink | Forward semantics respected | Missing target | Forward relationship may remain unresolved | PENDING |
| OBS-030 | ResourceLink | External semantics respected | External target | No local FK requirement | PENDING |
| OBS-031 | Event | Event/Observation distinction | Create event about Observation lifecycle | Event and Observation remain distinct | TESTED |
| OBS-032 | Event | Event is not assertion store | Place assertion in event payload | Must not redefine Observation ownership | PENDING |
| OBS-033 | Correction | Original preserved | Correct material observation | Original remains auditable | NOT IN SCOPE YET |
| OBS-034 | Correction | Supersession explicit | Supersede observation | Relationship/history is preserved | NOT IN SCOPE YET |
| OBS-035 | Contradiction | Conflict preserved | Two incompatible observations | Neither silently deleted | NOT IN SCOPE YET |
| OBS-036 | Contradiction | Newest does not automatically win | Later conflicting observation | Conflict remains explicit | NOT IN SCOPE YET |
| OBS-037 | Responsibility | Observation does not imply responsibility | Associate party/authority | No legal responsibility inferred | PENDING |
| OBS-038 | Responsibility | Candidate match distinct | AI suggests responsible party | Candidate remains non-authoritative | NOT IN SCOPE YET |
| OBS-039 | Privacy | Data classification mandatory | Sensitive observation | Classification is retained/enforced | TESTED |
| OBS-040 | Privacy | Purpose limitation | Unauthorized use context | Access/use rejected | PENDING |
| OBS-041 | Privacy | Location meanings remain distinct | Incident/capture/device/source locations | Distinct propositions preserved | PENDING |
| OBS-042 | Authorization | Observation access is policy controlled | Unauthorized read/write | Policy boundary rejects operation | TESTED |
| OBS-043 | AI | AI cannot manufacture provenance | AI creates candidate observation | Provenance must identify actual source/process | NOT IN SCOPE YET |
| OBS-044 | AI | AI cannot upgrade status | AI outputs VERIFIED-style claim | Canonical status unchanged unless explicit verification process permits | TESTED |
| OBS-045 | AI | AI cannot create legal responsibility | AI assigns responsible party | Remains candidate/non-authoritative | NOT IN SCOPE YET |
| OBS-046 | AI | AI cannot bypass execution gate | AI proposes consequential action | Existing authorization/approval gate applies | TESTED at generic execution boundary |
| OBS-047 | Failure | Malformed observation | Invalid schema | Deterministic rejection | TESTED |
| OBS-048 | Failure | Missing source context | Required source absent | Explicit validation failure/uncertainty | PENDING |
| OBS-049 | Failure | Persistence failure | Simulate failed write | No partial canonical state | TESTED through generic authoritative command/UoW path |
| OBS-050 | Idempotency | Duplicate creation | Repeat same command/request | Duplicate behavior follows canonical idempotency contract | TESTED at claim boundary; full prior-result semantics remain generic follow-up |
| OBS-051 | Provider | Provider neutrality | Run against two fake producers | Same canonical semantics | PENDING |
| OBS-052 | Longitudinal | Projection only | Compose chronological view | Underlying identities remain intact | NOT IN SCOPE YET |
| OBS-053 | Longitudinal | No LongitudinalRecord ownership | Attempt separate aggregate | Rejected unless separately justified/decided | NOT IN SCOPE YET |
| OBS-054 | Audit | Material changes auditable | Correct/supersede/status change | Required audit/provenance retained | PENDING |
| OBS-055 | Security | Injection cannot alter semantics | Malicious content requests status upgrade | Content treated as untrusted | TESTED at epistemic boundary |
| OBS-056 | Security | Source conflict is not silently resolved | Conflicting source inputs | Conflict remains represented | NOT IN SCOPE YET |
| OBS-057 | Persistence | Repository not assumed | Evaluate implementation need | No repository created without explicit ownership decision | TESTED / ARCHITECTURE LOCKED |

## 4. Current implementation evidence

The current implementation establishes the following baseline:

- Observation has a stable semantic type and canonical wire identity.
- Observation origin and epistemic status are separate concepts.
- `SYSTEM_DERIVATION` is the canonical wire value for system-derived observations.
- Observation creation uses the existing authoritative command and Unit-of-Work infrastructure.
- Authorization is bound to the exact actor, action, resource and applicable data classification/validity constraints.
- Observation creation persists the Observation together with its creation Event, Audit, Provenance and idempotency claim atomically.
- No Observation-specific repository or storage subsystem has been introduced.

This baseline does **not** establish full lifecycle conformance.

## 5. Required adversarial scenarios

### A-01 — AI status inflation

Input: an AI provider returns a highly confident assertion labelled as authoritative.

Expected: the Observation retains the contract-owned epistemic status and provider output remains non-authoritative until an explicit verification boundary accepts it.

**Current status:** BASELINE TESTED at the Observation semantic boundary; provider integration itself is deferred.

### A-02 — Evidence laundering

Input: a derived OCR/summary artifact is presented as if it were the original evidence.

Expected: the system retains the original evidence identity and keeps the derived artifact separate.

**Current status:** PENDING.

### A-03 — Responsibility laundering

Input: an AI system maps an observed asset to a likely responsible authority.

Expected: the mapping remains a candidate relationship and does not become legal responsibility without the applicable evidence, jurisdiction, obligation and verification boundaries.

**Current status:** DEFERRED until the relevant responsibility capability exists.

### A-04 — Timestamp laundering

Input: device time is presented as exact event time.

Expected: observed time, recorded time and uncertainty remain distinguishable.

**Current status:** Partial baseline: observed/recorded times are distinct; uncertainty semantics remain pending.

### A-05 — Contradiction suppression

Input: two observations materially conflict.

Expected: both observations and their provenance remain available; the system does not silently choose one because it is newer, generated by AI, or stored later.

**Current status:** DEFERRED until correction/contradiction history semantics are implemented.

### A-06 — Link-class laundering

Input: a legacy class-less ResourceLink points to an existing local resource.

Expected: it remains a compatibility reference and is not inferred as Strong.

**Current status:** ResourceLink semantics are already implemented and tested at their canonical boundary; Observation-specific integration evidence remains pending.

### A-07 — Projection ownership leak

Input: a longitudinal UI creates a synthetic LongitudinalRecord and treats it as canonical state.

Expected: projection remains derived and underlying primitive identities remain authoritative.

**Current status:** Deferred; no longitudinal aggregate exists.

## 6. Implementation promotion gates

The original pre-implementation gates are now superseded by staged implementation gates:

### Gate 1 — Semantic primitive

- identity/version semantics defined and tested;
- origin/modality and epistemic status separated;
- canonical epistemic vocabulary reused;
- observed/recorded time distinction established;
- provenance and data classification represented;
- no duplicate repository/aggregate introduced.

**Status: PASSED.**

### Gate 2 — Authoritative creation

- authorization precedes authoritative mutation;
- authorization is bound to the exact operation/resource/context;
- creation uses canonical command/UoW infrastructure;
- Observation, Event, Audit, Provenance and idempotency claim are atomic;
- epistemic status cannot be silently upgraded;
- security/foundation gates pass at exact head.

**Status: PASSED for bounded Create Observation workflow.**

### Gate 3 — Lifecycle integrity

Before correction/supersession is implemented, the repository must define and test:

1. original observation preservation;
2. explicit correction relationship/history;
3. explicit supersession semantics;
4. contradiction preservation;
5. no automatic newest/highest-confidence winner;
6. audit/provenance preservation across material change;
7. deterministic authorization for lifecycle operations.

**Status: PENDING.**

### Gate 4 — Cross-boundary conformance

Before provider, hardware, sensor or longitudinal capabilities are promoted:

1. temporal uncertainty semantics are accepted;
2. evidence binding and derived-artifact separation are proven;
3. privacy/purpose enforcement is proven;
4. ResourceLink integration is proven;
5. provider-neutral tests exist;
6. relevant adversarial scenarios A-01 through A-07 are covered;
7. longitudinal projection remains non-authoritative.

**Status: PENDING.**

## 7. Non-goals

This conformance matrix does not authorize:

- a dedicated ObservationRepository;
- a LongitudinalRecord aggregate;
- sensor hardware implementation;
- hardware attestation implementation;
- AI model/provider integration;
- responsibility-resolution implementation;
- autonomous legal action;
- domain-specific observation silos.

## 8. Next bounded conformance work

The next implementation target is **Observation lifecycle integrity**, specifically correction, supersession and contradiction preservation, using existing canonical resources and relationships.

It MUST NOT introduce:

- an Observation state machine;
- an Observation-specific repository;
- a LongitudinalRecord aggregate;
- a second command framework;
- provider-specific semantics.
