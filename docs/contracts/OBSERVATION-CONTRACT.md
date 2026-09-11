# SIDERETH — Observation Contract

**Status:** CANONICAL CONTRACT PROPOSAL / PRE-IMPLEMENTATION  
**Decision:** D-032  
**Scope:** Universal Core Observation semantics  
**Depends on:** ResourceRef, ResourceLink, Event, Evidence, Provenance, Timeline, Party

## 1. Purpose

This contract defines the semantic boundary for an Observation before implementation.

An **Observation** is an epistemically bounded assertion about a subject or resource at a stated time, originating from an identified source/context and optionally supported by evidence.

Observation is a semantic primitive. It is not automatically an aggregate, repository, Event subtype, Evidence record, legal conclusion, or authoritative fact.

## 2. Normative boundary

Observation MUST preserve the distinction between:

- the phenomenon or proposition being observed;
- when it was observed;
- when the observation was recorded;
- who or what produced the observation;
- what source/context produced it;
- what evidence supports it;
- what epistemic status it currently has;
- and what relationships connect it to other canonical resources.

No implementation may infer stronger epistemic meaning merely from storage location, resource type, relation name, provider identity, model confidence, or presence of evidence.

## 3. Canonical semantic shape

A conforming Observation SHOULD contain:

| Field | Requirement | Meaning |
|---|---|---|
| `observation_id` | REQUIRED | Stable identity of this observation assertion |
| `schema_version` | REQUIRED | Contract schema version |
| `subject_ref` | REQUIRED | `ResourceRef` identifying what is being observed |
| `observation_type` | REQUIRED | Typed semantic category of observation |
| `observed_at` | REQUIRED | Time at which the underlying observation was made, with uncertainty represented explicitly where needed |
| `recorded_at` | REQUIRED | Time the observation entered the authoritative SIDERETH record |
| `assertion` / `value` | REQUIRED | Structured or bounded representation of what is being asserted/observed |
| `epistemic_status` | REQUIRED | Explicit status of the observation's evidentiary/epistemic position |
| `source_refs` | REQUIRED | References to sources, actors, devices, systems or documents from which the observation originates |
| `evidence_refs` | OPTIONAL | References to supporting evidence |
| `provenance_ref` | REQUIRED when provenance is applicable | Provenance record for creation/transformation |
| `context_refs` | OPTIONAL | Case, Incident, jurisdiction, workflow or other contextual references |
| `data_class` | REQUIRED | Privacy/security classification |

This table is a semantic contract, not a requirement that every field become a database column.

## 4. Subject semantics

`subject_ref` identifies the object, resource, party, event, place, asset or other entity about which the observation is made.

The Observation MUST NOT copy the subject's canonical identity fields into a second identity model.

An Observation may itself become the subject of another observation or relationship if the future contract permits this, but recursive semantics must be explicitly bounded.

## 5. Observation type

`observation_type` identifies the semantic kind of observation without determining its truth.

Examples may include:

- condition observed;
- measurement;
- location observation;
- statement/report;
- status observation;
- sensor detection;
- document assertion;
- system detection.

Domain packs may define specialized observation types, but must consume the universal Observation contract rather than redefine it.

## 6. Epistemic status

The canonical vocabulary MUST distinguish materially different epistemic states.

The initial vocabulary is:

- `OBSERVED` — directly observed or recorded as an observation by the declared source;
- `USER_REPORTED` — reported by an identified party without the system independently establishing the proposition;
- `MEASURED` — produced by a declared measurement process or instrument;
- `SOURCE_SUPPORTED` — supported by an identified source/document under applicable verification rules;
- `EVIDENCE_SUPPORTED` — supported by identified evidence under applicable verification rules;
- `SYSTEM_DERIVED` — deterministically derived by a declared system process;
- `INFERRED` — inferred from other information rather than directly established;
- `UNVERIFIED` — recorded but not sufficiently established under the applicable verification policy;
- `CONTESTED` — materially disputed or contradicted;
- `UNKNOWN` — insufficient information to establish a stronger status.

These statuses are descriptive epistemic states, not legal conclusions.

### 6.1 No automatic upgrade

No provider, model, workflow, or UI may silently upgrade an Observation from a weaker to a stronger epistemic status.

In particular:

```text
INFERRED      ≠ OBSERVED
UNVERIFIED    ≠ VERIFIED FACT
USER_REPORTED ≠ EVIDENCE_SUPPORTED
CONTESTED     ≠ RESOLVED
UNKNOWN       ≠ FALSE
```

A later verification operation may establish a new state only through an explicit, provenance-bearing operation governed by the applicable policy.

## 7. Temporal semantics

`observed_at` and `recorded_at` are distinct.

The system MUST NOT treat a device wall-clock timestamp as unquestionable truth.

Where time is uncertain, the representation must preserve that uncertainty rather than manufacturing precision.

Future time-integrity capabilities may incorporate receipt time, monotonic sequencing, trusted time, hardware-backed time or other signals, but those are separate capabilities.

## 8. Evidence binding

Evidence is the preserved source artifact or canonical evidence record. Observation is the semantic assertion.

An Observation may reference one or more Evidence resources using canonical `ResourceLink`/`ResourceRef` semantics.

Evidence support does not automatically mean legal authenticity or truth. Evidence integrity and epistemic status remain distinct.

Derived artifacts such as OCR, summaries, classifications and AI outputs remain derived material and MUST NOT replace the original evidence.

## 9. Provenance binding

Observation creation and material transformation MUST preserve provenance appropriate to the risk and data class.

Provenance may identify:

- actor/source;
- input references;
- operation;
- time;
- relevant transformation context.

Provenance establishes origin/context; it does not by itself establish legal authority or truth.

## 10. ResourceLink integration

Observation uses the canonical ResourceLink contract for relationships to other resources.

No Observation-specific relationship table or relationship primitive may be introduced merely to connect observations to evidence, subjects, authorities, parties, actions or outcomes.

Strong, Forward and External semantics remain governed by the ResourceLink contract. A legacy class-less link must not be reinterpreted as Strong by inference.

## 11. Event integration

Events represent recorded occurrences and material state transitions in event history.

Observation represents an assertion about something observed, measured, reported, detected or inferred.

An Event may record the lifecycle transition of an Observation, while an Observation may reference an Event as context. Neither becomes a substitute for the other.

## 12. Correction, supersession and contradiction

Observations MUST NOT be destructively rewritten when their epistemic position changes materially.

A future conforming implementation MUST preserve:

- the original observation identity and provenance;
- the fact that a correction/supersession occurred;
- the actor/process responsible;
- the reason or basis where required;
- the relationship between the prior and subsequent observation.

Corrections and supersessions should normally be represented as new state/history linked to the prior observation rather than erasing the prior record.

A contradiction is not automatically resolved by choosing the newest, highest-confidence, or AI-preferred observation.

## 13. Responsibility boundary

Observation MUST NOT encode legal responsibility merely because a party or authority is associated with the observed subject.

Responsibility resolution remains a separate semantic capability requiring appropriate relationships, jurisdiction, provenance and applicable obligation/legal context.

A candidate match MUST remain distinguishable from evidence-supported responsibility.

## 14. Privacy and security

Observation data is subject to SIDERETH authorization, policy and data-classification boundaries.

Location observations require explicit semantic separation among, where applicable:

- reported incident location;
- evidence capture location;
- device location at capture;
- location asserted by a source.

These must not be silently collapsed into a single coordinate meaning.

## 15. AI boundary

AI may propose or derive candidate observations, normalize observation data, identify contradictions, and associate candidate evidence.

AI MUST NOT:

- manufacture provenance;
- self-authorize a status upgrade;
- turn inference into fact by wording alone;
- create legal responsibility;
- overwrite original evidence;
- bypass authorization, approval or execution gates.

An AI-produced Observation remains subject to the same epistemic and provenance requirements as any other source.

## 16. Longitudinal projection

A longitudinal view is a projection over canonical resources, not a second ownership model.

It may compose:

```text
Events
Observations
Evidence
Relationships
Actions
Responses
Resolutions
Outcomes
```

The projection MUST preserve the identity, provenance and epistemic status of its underlying resources.

No `LongitudinalRecord` aggregate is implied by this contract.

## 17. Persistence boundary

This contract does not require a dedicated Observation repository.

Persistence is a separate implementation decision that must demonstrate a real ownership/use-case requirement and must reuse canonical persistence, transaction, authorization, audit and idempotency infrastructure.

A first implementation SHOULD prefer the smallest persistence mechanism that preserves the contract rather than introducing a new subsystem.

## 18. Non-goals

This contract does not define:

- legal truth;
- legal responsibility;
- medical truth;
- a sensor/device standard;
- hardware attestation;
- a database schema;
- a longitudinal aggregate;
- an AI model;
- an OCR engine;
- a retrieval/vector database;
- autonomous legal action;
- domain-specific responsibility graphs.

## 19. Conformance gate

An implementation is conforming only when the applicable Observation conformance matrix passes and evidence demonstrates:

1. identity/version preservation;
2. explicit epistemic status;
3. source/provenance preservation;
4. distinct observed/recorded time semantics;
5. evidence references without evidence ownership duplication;
6. ResourceLink semantic correctness;
7. correction/supersession preservation;
8. contradiction preservation;
9. privacy/data-class enforcement;
10. AI status-upgrade resistance;
11. provider neutrality;
12. deterministic failure semantics;
13. auditability where required;
14. no duplicate repository/aggregate introduced without an explicit decision.
