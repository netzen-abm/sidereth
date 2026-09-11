# SIDERETH — Authorization & Policy Decision Contract

**Status:** CANONICAL / FOUNDATION CONTRACT  
**Scope:** Provider-neutral authorization and policy decision boundary

## 1. Purpose

This contract defines the single canonical authorization/policy decision boundary for SIDERETH. It determines whether a subject may perform a requested action on a resource for a stated purpose under applicable policy.

Authorization is a control-plane decision. It is not legal authority, human approval, evidence authenticity, capability registration, tool registration, workflow completion, or execution.

## 2. Architectural invariants

1. **Deny by default.** No positive authorization is inferred from absence of a denial.
2. **Fail closed.** Missing, malformed, stale, conflicting, unavailable or unverifiable policy data cannot produce an executable allow.
3. **One canonical boundary.** Capabilities, tools, workflows, AI/agents, surfaces and future adapters consume the same authorization boundary.
4. **Authorization is distinct from approval.** Authorization establishes permission under policy; required human approval is a separate gate.
5. **Authorization is distinct from legal authority.** A policy decision does not establish statutory, regulatory, judicial or institutional authority.
6. **Registry membership is not authorization.** Capability or Tool Registry entries cannot grant permission.
7. **Authorization does not execute.** Evaluation is side-effect free and cannot invoke tools, mutate domain state or submit external actions.
8. **Canonical semantics are provider-neutral.** Storage, policy engine, language, framework, deployment topology and identity provider are replaceable.
9. **Purpose limitation is mandatory.** A subject may be authorized for one purpose without being authorized for another.
10. **Resource scope is explicit.** Case, tenant, jurisdiction, data-class and other scope constraints cannot be inferred from a broad identity alone.
11. **Constraints are enforceable outputs.** An allow may carry conditions such as read-only, field scope, time window or destination restriction.
12. **Policy provenance is preserved.** A decision identifies the policy references/version context used for evaluation.
13. **Deterministic evaluation.** Given the same normalized request, policy snapshot and evaluation context, evaluation produces the same decision and constraints.
14. **No silent policy downgrade.** An unavailable newer/required policy cannot silently fall back to an older policy.
15. **Decision evidence is auditable.** Consequential decisions retain attributable policy/evaluation metadata without exposing unnecessary protected data.

## 3. Canonical request

A request must contain at minimum:

- subject reference
- action reference
- resource reference
- purpose
- policy reference(s) or an explicit policy-resolution context

The evaluation context may additionally contain:

- case/workspace scope
- jurisdiction
- data classification
- temporal validity
- delegation/representation context
- authentication assurance
- device/session context
- destination/channel
- risk classification
- consent reference where applicable
- correlation/audit reference

## 4. Canonical decision

The decision vocabulary is:

- `allow` — policy evaluation establishes permission, subject to returned constraints and downstream gates.
- `deny` — policy evaluation establishes that permission is not granted.
- `not_applicable` — the supplied policy set does not govern the request; this is **not** an allow and must not be treated as one by an execution boundary.

A conforming execution path must treat `not_applicable` as non-authorizing unless another independently evaluated policy explicitly grants permission.

## 5. Policy composition

Where multiple applicable policies exist:

- conflicts must be resolved by an explicit deterministic policy rule;
- absence of a conflict rule is fail-closed for consequential access;
- a less restrictive policy cannot silently override a higher-priority restriction;
- policy precedence/version must be part of the evaluation context or resolvable deterministically;
- the evaluator must preserve the references needed to reconstruct the decision.

## 6. Scope and minimisation

Authorization must be evaluated against the narrowest applicable scope. An authorization for one case does not imply access to another case. Authorization to read a resource does not imply authorization to export, disclose, modify, delete or execute against it.

Data-class restrictions must be enforced before protected data crosses a boundary. Returned constraints must be consumable by the downstream capability/tool/workflow boundary.

## 7. Temporal semantics

Policies may be time-bounded. Expired or not-yet-effective policies are not valid grants. Clock/context failures that prevent reliable temporal evaluation fail closed for consequential requests.

## 8. Approval and execution relationship

The canonical sequence is:

```text
Identity / authenticated subject
        ↓
Authorization / Policy Decision
        ↓
Data minimisation + constraints
        ↓
Required human approval (if applicable)
        ↓
Action execution gate
        ↓
Execution
```

An authorization `allow` never manufactures human approval. Human approval never substitutes for missing authorization.

## 9. AI and agent boundary

AI-generated text, recommendations, tool selection, planning or agent intent are inputs to policy evaluation, not authorization themselves.

An agent cannot authorize itself by:

- generating an allow statement;
- selecting a permitted-looking tool;
- asserting user consent;
- asserting human approval;
- relying on registry membership; or
- rewriting policy context.

## 10. Provider and implementation boundary

The contract must remain implementable by different policy engines and storage providers. A future external policy engine may be an adapter, provided it satisfies this contract and cannot weaken SIDERETH's deny-by-default, provenance, scope, approval and execution invariants.

## 11. Minimum error/failure semantics

The evaluator or policy-resolution layer must distinguish at least:

- invalid request
- unknown subject
- unknown resource
- unknown action
- policy unavailable
- policy invalid
- policy stale
- policy conflict
- scope mismatch
- purpose mismatch
- data-class restriction
- temporal restriction
- authorization denied

Failures that prevent reliable authorization must never be converted into `allow`.

## 12. Conformance requirement

Any implementation claiming conformance must pass `AUTHORIZATION-POLICY-CONFORMANCE.md`. Unit tests alone are insufficient for production readiness; integration tests must prove that downstream execution boundaries cannot proceed on denied, not-applicable, malformed, stale, ambiguous or unavailable authorization state.

## 13. Relationship to canonical modules

- **Capability Contract:** defines what a capability means and its declared requirements.
- **Capability Registry:** discovers/resolves capabilities; it does not authorize them.
- **Tool Registry:** discovers/resolves tools; it does not authorize invocation.
- **Action/Approval:** governs consequential human approval and execution.
- **Evidence/Provenance:** governs evidentiary integrity and lineage; it does not grant access.
- **Audit:** records attributable decisions/events; it does not create permission.
- **Legal Authority:** defines institutional/legal authority; it is not equivalent to access authorization.

## 14. Non-goals

This contract does not define:

- authentication protocol;
- identity proofing;
- legal authority;
- human approval UX;
- tool execution;
- workflow orchestration;
- policy authoring UI;
- a specific policy language;
- a specific policy engine;
- a specific database.
