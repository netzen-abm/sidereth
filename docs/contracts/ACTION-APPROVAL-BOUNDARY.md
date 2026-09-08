# SIDERETH — Action Approval Boundary Contract

**Status:** CONTRACTED / CORE

## Purpose

Define the canonical boundary between authorization, human approval, Action state, and execution for consequential operations.

## Invariants

1. **Authorization is not approval.** Authorization evaluates whether an actor/request is permitted under policy. Human approval records an explicit decision for an action that requires it.
2. **Approval is not legal authority.** An approval record does not create legal authority; it records a human decision within an existing authorization/policy context.
3. **Action remains the canonical execution primitive.** AI, agents, transports, and UI surfaces must not create a parallel execution or approval model.
4. **Consequential actions must not become Approved without an approval binding when explicit approval is required.**
5. **Consequential actions must not become Executing without the required authorization and approval bindings.**
6. **Approval must identify the action, approver, authorization context, decision, rationale, provenance, and decision time.**
7. **Approval is auditable and traceable.** The approval record is a first-class referenceable artifact even before a dedicated persistence adapter exists.
8. **Rejected or revoked approval never grants execution.**
9. **AI cannot manufacture an approval grant.** An intelligence response may propose an action or recommendation, but only the canonical approval boundary can bind a human approval.
10. **No provider, model, transport, or UI is authoritative.** They consume this contract.

## Boundary

```text
Intelligence / User Intent
          |
          v
       Action
          |
          +----> Authorization / Policy evaluation
          |
          +----> Explicit human approval when required
          |
          v
   Approved Action
          |
          v
      Execution
          |
          v
 Outcome + Evidence + Audit + Provenance
```

## ApprovalRecord

`ApprovalRecord` contains:

- `approval_id`
- `action_ref`
- `approver_ref`
- `authorization_ref`
- `decision`
- `rationale`
- `provenance_ref`
- `decided_at`

The record is valid only when its required references and decision metadata are present. The Action binding additionally requires the approval to target the exact Action and to reference the same authorization context.

## Action semantics

For `requires_explicit_approval = true`:

```text
Proposed
  |
  | authorization + approval binding
  v
Approved
  |
  | execution gate
  v
Executing
```

The Action cannot skip the approval binding by directly transitioning from Proposed to Approved. A bound approval is also required for execution.

## Rejection / revocation

`Granted` is the only approval decision that can satisfy the positive approval binding. `Rejected` and `Revoked` are recorded decisions but do not grant execution.

A future persistence/workflow integration must additionally prevent an already-approved Action from executing when a valid governing policy revokes or invalidates the approval. That is an integration requirement, not a reason to duplicate approval state in the Intelligence layer.

## Intelligence integration

`IntelligenceRequest.human_approval_required` declares that an intelligence task requires human approval. It does **not** constitute an approval grant. When intelligence proposes a consequential Action, the Action/Authorization/Approval boundary governs whether that proposal may become executable.

## Non-goals

This contract does not define:

- legal authority or delegation law;
- a particular identity provider;
- UI approval screens;
- a database schema or persistence adapter;
- automatic approval;
- autonomous legal decisions;
- model/provider-specific approval APIs.

## Implementation gate

Before production execution of consequential Actions, the repository must have an integration-level test proving:

`policy authorization + explicit approval + Action transition + execution gate`

and adversarial tests proving that missing, mismatched, rejected, or revoked approval cannot authorize execution.
