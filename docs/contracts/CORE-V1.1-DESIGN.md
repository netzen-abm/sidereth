# SIDERETH Core v1.1 — Remedy & Resolution Infrastructure

**Status:** DRAFT

## Purpose

Core v1.1 adds deterministic infrastructure for recording requested remedies, candidate remedies, remedy applicability, and case resolution without asserting that a remedy is legally available or that an authority acted unlawfully.

Canonical chain:

**Rule → Obligation → Requirement → Evidence → Compliance State → Deadline → Response → Escalation → Remedy → Resolution**

## Scope

Included:

- stable remedy identity and case linkage
- requested-remedy and candidate-remedy distinction
- optional escalation linkage
- remedy type/category
- applicability and availability status
- legal-source and evidence references
- deterministic remedy workflow states
- stable resolution identity and case linkage
- optional remedy/escalation/response linkage
- requested outcome versus recorded outcome distinction
- external outcome evidence reference
- deterministic resolution states
- validated transitions
- deterministic registry access

## Remedy Contract

A remedy record describes a remedy being considered or requested. It does not itself establish legal entitlement.

Each remedy contains:

- stable remedy ID
- case ID
- optional escalation ID
- remedy category
- description
- status
- applicability status
- legal-source references
- evidence references

Applicability states:

- `Unverified`
- `Verified`
- `Uncertain`
- `ReviewRequired`

`Verified` means the record has passed the core's explicit verification boundary. It is not an autonomous legal conclusion.

Remedy states:

- `Candidate`
- `Requested`
- `UnderReview`
- `Submitted`
- `Granted`
- `Denied`
- `Withdrawn`
- `Expired`

`Granted` and `Denied` record a recorded workflow or external outcome. They do not imply that the outcome was legally correct.

## Resolution Contract

Resolution records closure of a case workflow, not legal vindication.

Each resolution contains:

- stable resolution ID
- case ID
- optional remedy ID
- optional escalation ID
- optional response ID
- requested outcome
- recorded outcome
- resolution state
- evidence references
- legal-source references

Resolution states:

- `Open`
- `Resolved`
- `PartiallyResolved`
- `Unresolved`
- `Closed`
- `Reopened`

The distinction between requested and recorded outcome is mandatory so that a user's desired result is never mistaken for an actual result.

## Provenance Boundary

The core must preserve the distinction between:

- user-requested remedy
- candidate remedy
- verified legal source
- applicability status
- evidence
- workflow state
- recorded external outcome
- professional legal judgment

A remedy must not become `Verified` merely because an escalation exists. A resolution must not become `Resolved` merely because a response was submitted.

## Transition Boundary

Transitions are explicit and deterministic. Invalid transitions are rejected.

The core does not infer remedy or resolution transitions from dates, documents, AI output, external events, or presumed government action.

## Human Approval Boundary

The core records workflow state only. Filing, representation, legal strategy, settlement, waiver, or other high-impact action remains behind existing authorization and human/professional review boundaries.

## Explicit Non-Goals

- legal advice
- autonomous remedy selection
- autonomous lawfulness determinations
- remedy guarantees
- limitation-period calculation
- autonomous filing or government communication
- litigation strategy
- live legal-source ingestion
- automatic legal applicability decisions
- notification or scheduling
- AI mutation without authorization
