# SIDERETH — Observation Conformance

**Status:** CANONICAL TEST DESIGN / PRE-IMPLEMENTATION  
**Contract:** `docs/contracts/OBSERVATION-CONTRACT.md`  
**Decision:** D-032  
**Scope:** Provider-neutral Observation semantics

This matrix defines the evidence required before Observation is implemented or promoted. It does not require a repository, database schema, model provider, sensor platform, hardware kit, or AI implementation.

## 1. Conformance principles

1. Observation is an epistemic assertion, not automatically a fact.
2. Event, Evidence, Observation and Provenance retain separate ownership.
3. Existing ResourceLink semantics are reused.
4. Longitudinal views remain projections over canonical primitives.
5. AI and providers cannot self-upgrade epistemic status.
6. Historical observations are not destructively rewritten.
7. Contradiction remains explicit until an authorized process resolves it.
8. Privacy and authorization are enforced outside presentation convenience.
9. Provider-neutral behavior is tested independently of implementation technology.

## 2. Contract matrix

| ID | Area | Requirement | Verification | Expected result | Gate |
|---|---|---|---|---|---|
| OBS-001 | Identity | Stable observation identity | Create valid Observation | ID is preserved | Contract |
| OBS-002 | Versioning | Positive schema version | Use zero/invalid version | Rejected | Contract |
| OBS-003 | Subject | Subject uses ResourceRef | Supply valid subject reference | Reference is preserved | Contract |
| OBS-004 | Type | Observation type is explicit | Omit/blank type | Rejected | Contract |
| OBS-005 | Assertion | Assertion/value is explicit | Create valid assertion | Content is preserved | Contract |
| OBS-006 | Epistemic | Status is mandatory | Omit status | Rejected | Trust |
| OBS-007 | Epistemic | Vocabulary is typed | Use supported status | Round trip without semantic loss | Trust |
| OBS-008 | Epistemic | INFERRED remains INFERRED | Attempt automatic upgrade | Upgrade rejected | Trust |
| OBS-009 | Epistemic | UNVERIFIED remains UNVERIFIED | Attempt automatic upgrade | Upgrade rejected | Trust |
| OBS-010 | Epistemic | CONTESTED remains CONTESTED | Supply contradiction | Contest remains visible | Trust |
| OBS-011 | Epistemic | UNKNOWN remains UNKNOWN | Insufficient evidence | No stronger status inferred | Trust |
| OBS-012 | Epistemic | USER_REPORTED is distinct | User report without independent support | Status remains USER_REPORTED | Trust |
| OBS-013 | Epistemic | MEASURED is distinct | Instrument measurement | Status remains MEASURED | Trust |
| OBS-014 | Epistemic | SOURCE_SUPPORTED is distinct | Source-backed assertion | Status remains SOURCE_SUPPORTED | Provenance |
| OBS-015 | Epistemic | EVIDENCE_SUPPORTED is distinct | Evidence-backed observation | Status remains EVIDENCE_SUPPORTED | Evidence |
| OBS-016 | Time | observed_at required | Missing observation time | Rejected | Contract |
| OBS-017 | Time | recorded_at required | Missing recording time | Rejected | Contract |
| OBS-018 | Time | Times are distinct | Different observed/recorded times | Both preserved | Integrity |
| OBS-019 | Time | Clock uncertainty preserved | Uncertain device timestamp | Uncertainty is not erased | Trust |
| OBS-020 | Source | Source references preserved | Attach source refs | References survive round trip | Provenance |
| OBS-021 | Evidence | Evidence references are typed | Attach evidence refs | References remain traceable | Evidence |
| OBS-022 | Evidence | Evidence ownership not duplicated | Attempt to embed canonical evidence identity | Observation references evidence; does not own it | Architecture |
| OBS-023 | Provenance | Provenance preserved | Attach provenance | Provenance remains traceable | Provenance |
| OBS-024 | Provenance | Provenance does not equal truth | Source has provenance but weak epistemic basis | Status does not auto-strengthen | Trust |
| OBS-025 | ResourceLink | Canonical links reused | Link Observation to Evidence | ResourceLink semantics apply | Architecture |
| OBS-026 | ResourceLink | No class inference | Legacy class-less link with local endpoint | Must not become Strong | Compatibility |
| OBS-027 | ResourceLink | Strong semantics respected | Missing endpoint | Strong relationship rejected atomically | Integrity |
| OBS-028 | ResourceLink | Forward semantics respected | Missing target | Forward relationship may remain unresolved | Architecture |
| OBS-029 | ResourceLink | External semantics respected | External target | No local FK requirement | Architecture |
| OBS-030 | Event | Event/Observation distinction | Create event about Observation lifecycle | Event and Observation remain distinct | Architecture |
| OBS-031 | Event | Event is not assertion store | Place assertion in event payload | Must not redefine Observation ownership | Architecture |
| OBS-032 | Correction | Original preserved | Correct material observation | Original remains auditable | Integrity |
| OBS-033 | Correction | Supersession explicit | Supersede observation | Relationship/history is preserved | Integrity |
| OBS-034 | Contradiction | Conflict preserved | Two incompatible observations | Neither silently deleted | Trust |
| OBS-035 | Contradiction | Newest does not automatically win | Later conflicting observation | Conflict remains explicit | Trust |
| OBS-036 | Responsibility | Observation does not imply responsibility | Associate party/authority | No legal responsibility inferred | Legal correctness |
| OBS-037 | Responsibility | Candidate match distinct | AI suggests responsible party | Candidate remains non-authoritative | Legal correctness |
| OBS-038 | Privacy | Data classification mandatory | Sensitive observation | Classification is retained/enforced | Privacy |
| OBS-039 | Privacy | Purpose limitation | Unauthorized use context | Access/use rejected | Privacy |
| OBS-040 | Privacy | Location meanings remain distinct | Incident/capture/device/source locations | Distinct propositions preserved | Privacy |
| OBS-041 | Authorization | Observation access is policy controlled | Unauthorized read/write | Policy boundary rejects operation | Security |
| OBS-042 | AI | AI cannot manufacture provenance | AI creates candidate observation | Provenance must identify actual source/process | Trust |
| OBS-043 | AI | AI cannot upgrade status | AI outputs VERIFIED-style claim | Canonical status unchanged unless explicit verification process permits | Trust |
| OBS-044 | AI | AI cannot create legal responsibility | AI assigns responsible party | Remains candidate/non-authoritative | Legal correctness |
| OBS-045 | AI | AI cannot bypass execution gate | AI proposes consequential action | Existing authorization/approval gate applies | Safety |
| OBS-046 | Failure | Malformed observation | Invalid schema | Deterministic rejection | Reliability |
| OBS-047 | Failure | Missing source context | Required source absent | Explicit validation failure/uncertainty | Reliability |
| OBS-048 | Failure | Persistence failure | Simulate failed write | No partial canonical state | Integrity |
| OBS-049 | Idempotency | Duplicate creation | Repeat same command/request | Duplicate behavior follows canonical idempotency contract | Reliability |
| OBS-050 | Provider | Provider neutrality | Run against two fake producers | Same canonical semantics | Compatibility |
| OBS-051 | Longitudinal | Projection only | Compose chronological view | Underlying identities remain intact | Architecture |
| OBS-052 | Longitudinal | No LongitudinalRecord ownership | Attempt separate aggregate | Rejected unless separately justified/decided | Architecture |
| OBS-053 | Audit | Material changes auditable | Correct/supersede/status change | Required audit/provenance retained | Audit |
| OBS-054 | Security | Injection cannot alter semantics | Malicious content requests status upgrade | Content treated as untrusted | Security |
| OBS-055 | Security | Source conflict is not silently resolved | Conflicting source inputs | Conflict remains represented | Security |
| OBS-056 | Persistence | Repository not assumed | Evaluate implementation need | No repository created without explicit ownership decision | Architecture |

## 3. Required adversarial scenarios

### A-01 — AI status inflation

Input: an AI provider returns a highly confident assertion labelled as authoritative.

Expected: the Observation retains the contract-owned epistemic status and provider output remains non-authoritative until an explicit verification boundary accepts it.

### A-02 — Evidence laundering

Input: a derived OCR/summary artifact is presented as if it were the original evidence.

Expected: the system retains the original evidence identity and keeps the derived artifact separate.

### A-03 — Responsibility laundering

Input: an AI system maps an observed asset to a likely responsible authority.

Expected: the mapping remains a candidate relationship and does not become legal responsibility without the applicable evidence, jurisdiction, obligation and verification boundaries.

### A-04 — Timestamp laundering

Input: device time is presented as exact event time.

Expected: observed time, recorded time and uncertainty remain distinguishable.

### A-05 — Contradiction suppression

Input: two observations materially conflict.

Expected: both observations and their provenance remain available; the system does not silently choose one because it is newer, generated by AI, or stored later.

### A-06 — Link-class laundering

Input: a legacy class-less ResourceLink points to an existing local resource.

Expected: it remains a compatibility reference and is not inferred as Strong.

### A-07 — Projection ownership leak

Input: a longitudinal UI creates a synthetic LongitudinalRecord and treats it as canonical state.

Expected: projection remains derived and underlying primitive identities remain authoritative.

## 4. Implementation promotion gates

Observation implementation may proceed only after:

1. OBS-001 through applicable contract tests are defined;
2. epistemic vocabulary is accepted;
3. temporal uncertainty semantics are accepted;
4. correction/supersession semantics are accepted;
5. contradiction semantics are accepted;
6. privacy/data classification requirements are accepted;
7. ResourceLink integration is proven;
8. persistence ownership is explicitly decided;
9. provider-neutral tests exist;
10. adversarial scenarios A-01 through A-07 are covered.

## 5. Non-goals

This conformance matrix does not authorize:

- a dedicated ObservationRepository;
- a LongitudinalRecord aggregate;
- sensor hardware implementation;
- hardware attestation implementation;
- AI model/provider integration;
- responsibility-resolution implementation;
- autonomous legal action;
- domain-specific observation silos.
