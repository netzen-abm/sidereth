# SIDERETH Core v1.0 — Response & Escalation Infrastructure

**Status:** DRAFT

## Purpose

Core v1.0 adds deterministic infrastructure for recording responses and escalation paths after the compliance, evidence, deadline, and obligation layers.

Canonical chain:

**Rule → Obligation → Requirement → Evidence → Compliance State → Deadline → Response → Escalation → Remedy**

This milestone records response and escalation workflow state. It does not provide legal advice, determine unlawfulness, or communicate with authorities autonomously.

## Scope

Included:

- response identity and case linkage
- optional obligation linkage
- response content reference
- evidence and legal-source references
- deterministic response states
- escalation identity and case linkage
- optional response linkage
- escalation reason and target reference
- evidence and legal-source references
- deterministic escalation states
- validated state transitions
- deterministic registry access

## Response Contract

Each response contains:

- stable response ID
- case ID
- optional obligation ID
- title
- opaque content reference
- workflow state
- evidence references
- legal-source references

`content_ref` identifies stored response content without making the response registry a content store.

Response states:

- `Draft`
- `ReviewRequired`
- `Approved`
- `Submitted`
- `Withdrawn`
- `Resolved`

`Approved` is a workflow state, not a legal determination. A later adapter may require human or professional approval before reaching it.

`Submitted` records that the workflow considers the response submitted. The core does not perform government submission or prove external receipt.

## Escalation Contract

Each escalation contains:

- stable escalation ID
- case ID
- reason
- target reference
- optional response ID
- workflow state
- evidence references
- legal-source references

Escalation states:

- `Draft`
- `Ready`
- `Submitted`
- `Resolved`
- `Withdrawn`

`target_ref` is an opaque reference. It does not assert that the target has legal competence unless separately established through the authority and legal-source layers.

## Provenance Boundary

Responses and escalations may reference evidence and legal sources, but the core must preserve the distinction between:

- user-provided facts
- verified evidence
- legal propositions
- system workflow state
- draft content
- approved content
- recorded submission state
- professional legal judgment

The system must never convert a workflow state into a conclusion that an authority acted unlawfully or that a remedy is legally guaranteed.

## Transition Boundary

Transitions are explicit and deterministic. Invalid transitions are rejected.

The core does not infer transitions from dates, documents, AI output, external events, or presumed government action.

## Human Approval Boundary

The core can record approval states, but does not grant legal authority to an agent.

Adapters must apply policy and authorization before high-impact actions. External filing, government communication, legal representation, and other high-impact actions require the appropriate human approval and professional review boundary.

## Remedy Boundary

v1.0 records escalation toward a target. It does not implement a universal remedy engine. Remedy availability, jurisdiction, limitation rules, filing requirements, and professional strategy remain future domain-specific or higher-layer concerns.

## Architectural Boundary

The response and escalation layer is a core domain primitive. UI, bots, AI agents, MCP adapters, notifications, and submission adapters must consume it through existing identity, authorization, policy, provenance, evidence, and audit boundaries.

No adapter may bypass the core to mutate legal workflow state.

## Explicit Non-Goals

- autonomous government communication
- autonomous filing
- legal advice
- lawfulness determinations
- remedy guarantees
- live regulatory ingestion
- automatic legal applicability decisions
- notification or scheduling infrastructure
- AI mutation without authorization
- production legal-source verification
- litigation strategy
