# SIDERETH — Observation Semantics

**Status:** LOCKED DECISION RECORD
**Decision ID:** D-032
**Scope:** Universal Core observation semantics
**Canonical contract:** `docs/contracts/OBSERVATION-CONTRACT.md`
**Conformance:** `docs/contracts/OBSERVATION-CONFORMANCE.md`

## 1. Decision

`Observation` is a distinct universal semantic primitive in SIDERETH.

It represents an **epistemically bounded assertion about a subject or resource at a stated time, originating from an identified source/context and optionally supported by evidence**.

Observation is not automatically an aggregate, repository, Event subtype, Evidence record, legal conclusion, or authoritative fact.

The decision is intentionally semantic-first. It does **not** authorize a dedicated Observation repository, LongitudinalRecord aggregate, sensor/hardware implementation, AI provider, responsibility-resolution subsystem, or autonomous legal action.

## 2. Why Observation is distinct

SIDERETH must distinguish:

- something that happened;
- something that was observed, measured, reported, detected or inferred;
- the evidence supporting that observation;
- relationships between resources;
- actions taken because of an observation;
- and outcomes that follow.

An Event represents a recorded occurrence/change or lifecycle transition. An Observation represents an assertion/proposition about something observed, measured, stated, detected or inferred. Evidence preserves source material or a canonical evidence record.

Example:

```text
Event:
  inspection_record_created

Observation:
  surface condition appears damaged

Evidence:
  photograph / video / document / sensor capture
```

Neither Event nor Evidence becomes a substitute for Observation.

## 3. Epistemic semantics

Observation MUST reuse the SIDERETH canonical epistemic vocabulary defined by the Intelligence contract. Observation does not create a parallel status vocabulary.

The current vocabulary is:

- `OBSERVED`
- `USER_REPORTED`
- `EVIDENCE_SUPPORTED`
- `SOURCE_SUPPORTED`
- `SYSTEM_DERIVED`
- `INFERRED`
- `UNVERIFIED`
- `CONTESTED`
- `UNKNOWN`

Source/process modality is a separate concern represented by `observation_origin`, for example `DIRECT_OBSERVATION`, `USER_REPORT`, `MEASUREMENT`, `SOURCE_ASSERTION`, `SYSTEM_DERIVATION`, or `AI_DERIVATION`.

This distinction is deliberate: **MEASUREMENT is an origin/modality, not a new epistemic status.** This avoids semantic drift between Observation and Intelligence.

No provider, model, workflow or UI may silently upgrade epistemic status. In particular:

```text
INFERRED      ≠ OBSERVED
UNVERIFIED    ≠ VERIFIED FACT
USER_REPORTED ≠ EVIDENCE_SUPPORTED
CONTESTED     ≠ RESOLVED
UNKNOWN       ≠ FALSE
```

A stronger state may only be established by an explicit, provenance-bearing verification operation governed by applicable policy.

## 4. Temporal semantics

Observation MUST preserve `observed_at` separately from `recorded_at`.

Device wall-clock time is not automatically authoritative. Where time is uncertain, uncertainty must be represented rather than replaced with manufactured precision.

Future trusted-time, monotonic sequencing or hardware-backed time capabilities remain separate implementations.

## 5. Evidence, provenance and relationships

Observation references Evidence; it does not own, overwrite or replace canonical evidence.

Derived OCR, summaries, classifications and AI outputs remain derived material and cannot silently become original evidence.

Observation uses canonical `ResourceRef` and `ResourceLink` semantics. No Observation-specific relationship primitive is introduced merely to connect observations to evidence, subjects, parties, authorities, actions or outcomes.

Provenance establishes origin/context, not legal authority or truth.

## 6. Correction, supersession and contradiction

Material epistemic changes MUST preserve the original observation identity and provenance, the fact and basis of correction/supersession, the responsible actor/process, and the relationship between prior and subsequent state/history.

Contradictions remain explicit. The newest, highest-confidence or AI-preferred observation does not automatically win.

## 7. Responsibility boundary

Observation does not imply legal responsibility.

Where responsibility is later resolved, SIDERETH must distinguish roles such as responsible party, nearest authority, geographic authority, asset owner, contractor/service provider and contracting/procuring authority. Candidate matches remain non-authoritative until the applicable evidence, relationship, jurisdiction and legal/contractual context support a determination.

## 8. Privacy and security

Observation is subject to authorization, policy and data-classification boundaries.

Location semantics must distinguish, where applicable:

- reported incident location;
- evidence capture location;
- device location at capture;
- location asserted by a source.

These propositions must not be silently collapsed.

## 9. AI boundary

AI may extract, normalize, classify, compare, identify contradictions, associate candidate evidence and propose bounded actions for review.

AI may not manufacture provenance, self-authorize status upgrades, turn inference into fact by wording, create legal responsibility, overwrite original evidence, or bypass authorization/approval/execution gates.

## 10. Longitudinal semantics

A longitudinal view is a projection/composition over canonical primitives:

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

The projection preserves the identity, provenance and epistemic status of underlying resources. `LongitudinalRecord` is not introduced as a canonical aggregate by this decision.

## 11. Persistence and implementation boundary

D-032 does not require a dedicated Observation repository.

Any implementation must first demonstrate an actual ownership/use-case requirement and reuse canonical persistence, transaction, authorization, audit and idempotency infrastructure.

The implementation promotion gate is the Observation conformance matrix. Implementation is permitted only after the applicable semantic and adversarial tests are satisfied.

## 12. Rejected alternatives

### A. Observation as an Event subtype

Rejected because occurrence/lifecycle semantics and epistemic assertion semantics are different concerns.

### B. Every Observation as Evidence

Rejected because reports, measurements, statements and inferences can be semantic assertions without being preserved source artifacts.

### C. Immediate ObservationRepository

Rejected because semantics do not by themselves establish new persistence ownership.

### D. LongitudinalRecord aggregate

Rejected because chronological views can remain projections over existing primitives without duplicating identity, lifecycle and provenance ownership.

### E. Domain-specific observation silos

Rejected because Observation is a reusable universal primitive; domains may specialize observation types without redefining the primitive.

## 13. Consequences

Positive consequences:

- SIDERETH can represent observations without corrupting Event semantics.
- Evidence remains preserved and independently traceable.
- AI-generated observations remain epistemically bounded.
- Measurement and other source modalities are represented without forking the epistemic vocabulary.
- Longitudinal experiences can be composed without a duplicate canonical record model.
- Future sensor, hardware, offline, AI and domain capabilities can attach through stable contracts.

Constraints:

- Implementations must preserve temporal, epistemic and provenance distinctions.
- Verification cannot be inferred from presentation or provider confidence.
- Persistence ownership must be justified separately.
- Conformance evidence is required before implementation promotion.

## 14. Verification state

D-032 is now LOCKED because the canonical Observation contract and conformance design have been created and the semantic conflict between **measurement modality** and **epistemic status** has been resolved without creating a parallel vocabulary.

Implementation status remains **PRE-IMPLEMENTATION**. This decision is architectural authority, not evidence that Observation is implemented.
