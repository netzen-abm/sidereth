# SIDERETH — Universal Action Model

**Status:** DESIGN / IMPLEMENTATION BASELINE  
**Version:** 1.0

## 1. Purpose

Action is the universal execution primitive for SIDERETH. It represents an intended or executed operation by a user, professional, authorized system, workflow, integration, or agent.

Action is domain-independent. Domain packs specialize action kinds and validation rules; they do not redefine the Action identity, lifecycle, authorization, provenance, or audit semantics.

## 2. Boundary

```text
Intent
  ↓
Action (proposed)
  ↓
Policy + Authorization + Preconditions
  ↓
Approval when required
  ↓
Execution
  ↓
Outcome
  ↓
Evidence / Decision / Response / Event
  ↓
Audit + Provenance
```

An Action is not itself:
- an Event;
- a Decision;
- a legal proposition;
- an authorization grant;
- an evidence record;
- a workflow definition.

Those remain separate canonical primitives and are referenced by stable IDs.

## 3. Canonical fields

- `action_id`: stable unique identity.
- `schema_version`: explicit contract version.
- `kind`: broad operation category.
- `status`: lifecycle state.
- `actor_id`: Party or system actor responsible for the action.
- `context_refs`: matter/context references such as Case or Incident.
- `target_refs`: canonical objects affected by the action.
- `intent`: human-readable declared purpose.
- `authorization_ref`: authorization/policy decision reference when applicable.
- `precondition_refs`: conditions that must be satisfied before execution.
- `input_refs`: references to inputs rather than uncontrolled embedded copies.
- `output_refs`: references to produced canonical or derived objects.
- `evidence_refs`: supporting evidence references.
- `requires_explicit_approval`: high-impact approval boundary.
- `provenance_ref`: origin/provenance reference.
- `created_at`, `updated_at`: lifecycle timestamps.

## 4. Lifecycle

```text
Proposed
   ├── Approved → Executing → Completed
   │                         └→ Failed
   ├── Rejected
   └── Cancelled
```

Invalid transitions must be rejected deterministically.

An Action may not become `Executing` unless its required authorization and approval conditions have been satisfied by the governing policy layer.

## 5. Authorization boundary

Action does not grant authority merely because an actor created it. Authorization remains an independent policy-controlled primitive.

High-impact legal actions must require explicit approval where policy requires it. Autonomous agents must never acquire authority merely by invoking the Action model.

The existing authorization baseline requires least privilege, purpose limitation, case-scoped access, explicit consent for external sharing, no autonomous high-impact legal action, auditable policy decisions, and deny-by-default behavior.

## 6. Relationship to universal primitives

```text
Party       → who acts
Case        → matter context
Incident    → interaction context
Document    → information acted upon
Evidence    → supporting material
Authority   → institutional context
Jurisdiction→ governing scope
Action      → operation intended/executed
Decision    → authorized outcome
Event       → recorded occurrence
```

Action references these primitives; it does not absorb their semantics.

## 7. Event relationship

Execution and lifecycle changes may emit domain events. The Action remains the durable execution object; an Event records an occurrence about that object or its surrounding aggregate.

`causation_id` and `correlation_id` should connect emitted events to the originating Action where the event contract supports those fields.

## 8. AI and agent boundary

AI may propose or prepare Actions when explicitly enabled. AI does not become the legal authority for an Action. Agent execution remains subject to identity, authorization, policy, preconditions, approval gates, audit, and capability/tool boundaries.

## 9. Integrity and provenance

Inputs, outputs, evidence, and external effects must remain traceable through stable references and provenance. Derived AI interpretations must not silently become authoritative source facts.

## 10. Idempotency and retries

Execution infrastructure should associate externally effectful Actions with an idempotency key or equivalent execution claim before production execution. Retries must not silently duplicate an external legal or financial effect.

This execution mechanism is infrastructure work and is not claimed as complete by the core Action model alone.

## 11. Domain-pack rule

A domain pack may add:
- specialized action kinds;
- domain-specific preconditions;
- required evidence;
- authority constraints;
- approval requirements;
- procedure references.

It must consume the universal Action contract and cannot create a parallel action lifecycle.

## 12. Definition of Done

Action is complete only when:

1. canonical contract is approved;
2. implementation is present;
3. schema is versioned;
4. lifecycle and validation tests pass;
5. authorization/policy integration is verified;
6. provenance and audit integration are verified;
7. idempotency/retry semantics are implemented for effectful execution;
8. security/privacy controls are verified;
9. documentation is synchronized;
10. CI validates the complete boundary.
