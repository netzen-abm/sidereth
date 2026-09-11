# SIDERETH — Observation Semantics

**Status:** DRAFT / architectural decision proposal  
**Decision ID:** D-032  
**Scope:** Universal Core observation semantics  
**Depends on:** ResourceRef, ResourceLink, Event, Evidence, Provenance, Timeline, Party

## 1. Purpose

Define what an **Observation** means in SIDERETH before introducing implementation types, repositories, longitudinal-record models, or domain-specific observation systems.

The immediate objective is semantic clarity. SIDERETH must distinguish:

- something that happened,
- something that was observed or reported,
- the evidence supporting that observation,
- relationships between resources,
- actions taken because of an observation,
- and outcomes that follow.

## 2. Decision proposal

`Observation` SHOULD be a distinct universal semantic primitive, but it SHOULD NOT become a new aggregate, repository, timeline store, or evidence store at this stage.

An Observation represents an **epistemically bounded assertion about a subject or resource at a stated time, originating from an identified source/context and optionally supported by evidence**.

This distinction is necessary because an observation is not equivalent to an Event and is not equivalent to Evidence.

### 2.1 Event is not Observation

An Event represents a recorded occurrence/change in the SIDERETH event history.

An Observation represents a proposition or report about what was observed, measured, stated, detected, or otherwise perceived.

Example:

```text
Event:
  "inspection_record_created"

Observation:
  "surface condition appears damaged"

Evidence:
  photograph / video / document / sensor capture
```

An Observation may therefore be recorded because of an Event, while an Event may record the lifecycle transition of an Observation. They must not be silently collapsed.

### 2.2 Observation is not Evidence

Evidence is the preserved source artifact or canonical evidence record. An Observation is a semantic assertion derived from a person, sensor, document, system, or other source.

Evidence can support an Observation, but the existence of an Observation does not make it proven.

Derived interpretations, OCR output, summaries, classifications, and AI-generated propositions remain derived artifacts/assertions and do not replace original evidence.

## 3. Epistemic status is mandatory

Every Observation MUST carry an explicit epistemic status or equivalent typed semantic contract.

The status must distinguish, at minimum, concepts such as:

- observed directly;
- reported by a party;
- measured/detected by an instrument or system;
- asserted by a source/document;
- derived/inferred;
- disputed/contested;
- unresolved/unknown.

The final vocabulary requires a separate contract/conformance decision. The implementation MUST NOT invent a single generic `verified` flag that collapses materially different epistemic states.

An AI model MUST NOT promote an Observation to a stronger epistemic state merely by generating confident language.

## 4. Proposed semantic shape

The future canonical Observation contract SHOULD contain, at minimum:

```text
Observation
├── observation_id
├── schema_version
├── subject_ref / observed_resource_ref
├── observation_type
├── observed_at
├── recorded_at
├── assertion/value
├── epistemic_status
├── source_refs
├── evidence_refs
├── provenance_ref
├── context_ref(s)
└── privacy/data classification
```

This is a semantic target, not an implementation mandate yet.

The distinction between `observed_at` and `recorded_at` is intentional. A system must not assume that the time an observation was recorded is the time the underlying phenomenon occurred.

## 5. Relationship to existing SIDERETH primitives

### ResourceRef

Observation identifies its subject through `ResourceRef` rather than embedding another resource's identity model.

### ResourceLink

Observation-to-resource and Observation-to-evidence relationships use the canonical ResourceLink semantics. Strong, Forward, External, and legacy compatibility meanings remain governed by the ResourceLink contract.

No Observation-specific relationship mechanism should be introduced.

### Event

Events record lifecycle and material state transitions. Observation content may be represented in event payloads where appropriate, but event history must not become a second Observation store.

### Evidence

Evidence preserves source material and integrity. Observation references evidence; it does not own or overwrite the evidence.

### Provenance

Provenance records source/actor/input/operation context. Provenance does not itself establish legal authority or truth.

### Timeline

Timeline remains a projection/composition of Event history. A future longitudinal view may compose Events, Observations, Evidence, Relationships, Actions, and Outcomes, but must not create a competing ownership model.

### PartyRelationship

PartyRelationship remains the canonical primitive for party-to-party relationships. Observation must not recreate party relationship semantics.

## 6. Canonical chain

The preferred semantic chain is:

```text
Observation
     │
     ├── supported by ──> Evidence
     │
     ├── connected by ──> ResourceLink / Relationship
     │
     ├── may motivate ──> Action
     │
     └── contributes to ──> Outcome / Resolution
```

The chain is not necessarily linear. It may be revisited when new evidence, contradiction, correction, or outcome information appears.

A more accurate system-level model is therefore:

```text
       Evidence
          ▲
          │ supports
          │
Observation ─── Relationship ─── Resource/Party/Authority
    │
    │ may motivate
    ▼
  Action
    │
    ▼
 Outcome / Resolution
```

## 7. Responsibility must remain typed

Observation must not imply that the observed subject is legally responsible.

Where future workflows determine responsibility, SIDERETH must distinguish at least:

- responsible party;
- nearest authority;
- geographic authority;
- asset owner;
- contractor/service provider;
- contracting/procuring authority;
- other legally relevant roles.

A probable match is not canonical truth. Responsibility resolution requires explicit relationship semantics, provenance, jurisdiction and applicable legal/contractual context.

## 8. Longitudinal semantics

A longitudinal view SHOULD be a **projection/composition layer**, not a new canonical record type by default.

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

The projection should preserve the identity and provenance of each underlying primitive.

Do NOT introduce `LongitudinalRecord` merely to make chronological presentation easier.

## 9. Privacy and security

Observation data may be highly sensitive even when the underlying evidence is not publicly disclosed.

Therefore future implementation must apply:

- data classification;
- purpose limitation;
- least-privilege access;
- provenance;
- authorization policy;
- disclosure controls;
- explicit separation between protected evidence and presentation artifacts.

Location observations require particular care. A reported incident location, evidence capture location, device location, and location asserted by a source are distinct propositions and must not be collapsed into one coordinate field.

## 10. AI boundary

AI may:

- extract candidate observations;
- classify or normalize observations;
- identify possible contradictions;
- link observations to candidate evidence;
- propose actions for human review.

AI must not:

- manufacture observation provenance;
- upgrade epistemic status without evidence/policy authority;
- convert inference into fact;
- create legal responsibility by assertion;
- bypass authorization or approval gates;
- replace original evidence.

## 11. Implementation boundary

No Observation implementation should be merged until the following are separately defined and reviewed:

1. canonical Observation contract;
2. epistemic-status vocabulary;
3. subject/context semantics;
4. temporal semantics (`observed_at` versus `recorded_at`);
5. evidence/provenance binding;
6. correction and supersession semantics;
7. contradiction/challenge semantics;
8. privacy/data-class semantics;
9. ResourceLink integration;
10. provider-neutral conformance tests;
11. persistence requirements, if any;
12. projection requirements for longitudinal views.

The first implementation should remain minimal and should reuse existing Event, Evidence, Provenance, ResourceLink, Action, Response and Resolution infrastructure.

## 12. Rejected alternatives

### A. Make Observation an Event subtype

Rejected because occurrence/lifecycle semantics and epistemic assertion semantics are different concerns. Forcing Observation into Event would encourage event history to become a universal assertion store.

### B. Make every Observation Evidence

Rejected because a user report, measurement, statement, or inference may be represented semantically without being the preserved source artifact itself.

### C. Create ObservationRepository immediately

Rejected because the current core has not established a persistence ownership requirement. A repository-first design risks creating unnecessary storage ownership before semantics are stable.

### D. Create LongitudinalRecord as a new aggregate

Rejected because chronological views can be projections over existing primitives. A new aggregate would duplicate identity, lifecycle and provenance responsibilities.

### E. Put all observation semantics inside domain packs

Rejected because observation is potentially reusable across legal, regulatory, compliance, evidence, incident and future domains. Domain packs should specialize observation types rather than redefine the primitive.

## 13. Verification gate

Before D-032 can become LOCKED:

- semantic distinction from Event and Evidence is reviewed;
- epistemic vocabulary is explicit;
- ResourceLink semantics are proven and merged;
- privacy/security implications are reviewed;
- correction/supersession and contradiction behavior are specified;
- implementation ownership is established;
- no duplicate primitive/repository is introduced;
- conformance tests are defined before implementation.

## 14. Current recommendation

**Proceed with Observation as a distinct semantic primitive at the contract level, but do not implement persistence yet.**

The immediate next engineering artifact should be the canonical Observation contract and its conformance matrix. Only after those are stable should a minimal implementation be considered.

This preserves SIDERETH's universal-core principle while preventing premature proliferation of repositories, aggregates, or domain-specific models.
