# SIDERETH — Action Execution Gate Contract

## Purpose

This contract defines the deterministic boundary between a proposed consequential Action and permission to execute it.

The gate is part of the core trust boundary. It is not an AI capability, provider feature, transport feature, or UI feature.

## Canonical sequence

```text
Policy
  ↓
Authorization
  ↓
Human Approval
  ↓
Action Approved
  ↓
Execution Permitted
```

Each stage is distinct. Passing one stage does not imply that the later stages have passed.

## Invariants

For an Action requiring explicit approval, execution permission requires all of the following:

1. The Action has an authorization reference.
2. The supplied authorization reference matches the Action authorization reference.
3. Authorization has granted the requested operation.
4. A matching ApprovalRecord exists for the Action.
5. The ApprovalRecord references the same authorization.
6. The approval decision is `Granted`.
7. The approval origin is `Human`.
8. The Action is already in `Approved` state.

Failure of any required condition denies execution.

## AI boundary

An intelligence provider may propose an Action, draft content, retrieve information, or return tool proposals. It cannot manufacture human approval authority.

An approval record whose origin is `Intelligence` or `System` cannot satisfy the consequential execution gate, even if its decision is marked `Granted`.

This is an architectural invariant, not a prompt instruction.

## Authorization boundary

Authorization and approval remain separate concepts:

- Authorization answers whether the requested operation is permitted under applicable policy.
- Human approval records the required human decision for a consequential Action.
- The execution gate combines those prerequisites with Action state before execution is permitted.

The execution gate does not itself define legal authority or policy.

## Failure matrix

| Condition | Execution |
|---|---|
| Missing authorization | Denied |
| Wrong authorization | Denied |
| Authorization denied | Denied |
| Missing approval | Denied |
| Approval targets another Action | Denied |
| Approval targets another authorization | Denied |
| Rejected approval | Denied |
| Revoked approval | Denied |
| Intelligence-produced approval | Denied |
| System-produced approval | Denied |
| Human grant but Action not Approved | Denied |
| Matching authorization + granted human approval + Approved Action | Permitted |

## Scope discipline

This contract does not introduce:

- an Ollama/model runtime;
- autonomous agents;
- external execution adapters;
- OCR or retrieval infrastructure;
- a second approval model for Intelligence;
- legal decision authority;
- a transport-specific approval mechanism.

## Future integration gate

Before any provider or agent is allowed to participate in consequential workflows, integration tests must prove that provider output cannot bypass this contract and that actual execution remains behind the canonical Action boundary.
