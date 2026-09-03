# SIDERETH Core v0.9 — Compliance & Obligation State Infrastructure

**Status:** DRAFT

## Purpose

Core v0.9 adds deterministic compliance-state infrastructure between the existing obligation/deadline layer and future response/escalation layers.

Canonical chain:

**Rule -> Obligation -> Requirement -> Evidence -> Compliance State -> Deadline -> Response -> Escalation**

This milestone models what must be satisfied and the evidence/state associated with that requirement. It does not decide whether a person is legally liable or whether an authority acted unlawfully.

## Scope

Included:

- compliance requirement identity
- linkage to an obligation
- source provenance
- explicit evidence references
- deterministic compliance states
- validated state transitions
- deterministic registry access

Excluded:

- live regulatory ingestion
- automatic legal applicability decisions
- notifications or scheduling
- autonomous filing
- legal advice
- lawfulness conclusions
- AI mutation
- production source verification

## Compliance State

Supported states:

- `Unknown`
- `NotApplicable`
- `Required`
- `InProgress`
- `Satisfied`
- `Breached`
- `Disputed`
- `ReviewRequired`

A state is a system representation, not a legal conclusion. In particular, `Breached` means the recorded compliance state indicates a breach condition; it does not by itself establish legal liability.

## Requirement Contract

Each requirement contains:

- stable requirement ID
- obligation ID
- human-readable description
- compliance state
- evidence references
- legal source references

A requirement marked `Satisfied` must have at least one evidence reference. This creates an explicit evidence boundary rather than allowing a bare status claim to masquerade as proof.

## Provenance Boundary

Legal source references remain mandatory. User facts, evidence, system state, and legal propositions must remain distinguishable.

The compliance layer must never silently convert:

- an inference into a fact
- a user assertion into verified evidence
- a system state into a legal conclusion
- missing information into non-applicability

## State Transition Boundary

Transitions are explicit and deterministic. Invalid transitions are rejected.

The implementation does not automatically infer transitions from dates, documents, AI output, or external events.

## Architectural Boundary

The compliance layer is a core domain primitive. Adapters, AI agents, notifications, and user interfaces must consume it through the same policy, authorization, provenance, and audit boundaries established by earlier core milestones.

## Future Integration

Later milestones may connect requirements to:

- verified legal-source propositions
- jurisdiction and authority applicability
- evidence verification
- deadlines
- response workflows
- escalation/remedy workflows
- human professional review

Those integrations must preserve the distinction between **obligation existence**, **applicability**, **evidence of satisfaction**, and **professional legal judgment**.
