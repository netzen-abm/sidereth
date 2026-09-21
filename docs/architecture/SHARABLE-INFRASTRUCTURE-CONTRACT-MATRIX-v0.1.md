# SIDERETH Shareable Infrastructure Contract Matrix v0.1

## Purpose

This matrix freezes the first reusable infrastructure contracts for SIDERETH. It is a conformance boundary, not a feature roadmap.

**Core rule:** one canonical domain, one canonical control plane, multiple implementation technologies only behind explicit contracts.

## Contract matrix

| Contract | Authority | Required inputs | Required output | Failure semantics | Security invariant |
|---|---|---|---|---|---|
| Identity / Subject Context | Canonical identity model | subject, actor, correlation context | validated subject context | reject invalid/ambiguous identity | no operation proceeds on unvalidated identity |
| Authorization | Canonical Authorization Evaluator | AuthorizationRequest | AuthorizationResult | Deny / validation failure | no protected operation bypasses canonical authorization |
| Authorization Enforcement | Canonical enforcement boundary | request + result + operation binding | validated authorization | reject mismatch | authorization must be bound to the exact protected operation |
| Capability | Canonical capability registry | capability reference + constraints | registered capability | reject unknown/mismatched capability | capability semantics cannot be redefined by adapters |
| Capability Lease | Canonical lease authority | capability + subject + validity | active/validated lease | reject expired/revoked/mismatched lease | temporary authority cannot outlive its lease |
| Action | Canonical action model | authorized operation + refs | authoritative action | reject invalid binding/state | action cannot invent authorization or approval |
| Execution Gate | Canonical Execution Gate | authorization + action + approval state | permitted execution or rejection | deny / reject | consequential execution requires canonical gate; required human approval cannot be replaced by AI/system approval |
| Tool Registry | Canonical registry | tool/capability/function/provider/implementation identity | validated registration | reject unknown/mismatched binding | provider execution requires registry binding |
| Tool Gateway | Canonical gateway | invocation + validated execution context | provider execution outcome | reject before execution; Unknown when durable outcome cannot be reconciled | external callers cannot bypass validation or claim durable idempotency |
| Tool Data Access | Canonical least-privilege grant | resource, purpose, scope, data class | bounded grant | reject over-broad access | provider receives no data outside invocation-bound grant |
| Idempotency | Canonical durable lifecycle | idempotency key/reference + operation identity | single durable claim | already-claimed / conflict / Unknown | no external caller directly controls durable claim transition |
| Unit of Work | Canonical transactional boundary | authoritative writes | atomic commit/rollback | rollback on failure | related state cannot become partially committed |
| Audit | Canonical audit sink | authoritative event/action outcome metadata | durable audit record | persistence failure is explicit | audit records describe events; they do not grant authority |
| Provenance | Canonical provenance model | actor/source/input/operation refs | provenance record/ref | persistence failure is explicit | provenance describes lineage; it does not grant authority |
| Event / Correlation Identity | Canonical event model | operation/correlation identity | traceable event identity | reject malformed identity | consequential operations remain traceable end-to-end |
| Conformance | Canonical invariant suite | contract implementation | pass/fail evidence | fail closed | reusable implementations must satisfy canonical security invariants |

## Required lifecycle

For protected consequential execution:

1. establish identity/subject context;
2. construct the canonical authorization request;
3. evaluate canonical authorization;
4. enforce exact operation/resource/data-class binding;
5. resolve capability and validate any required lease;
6. construct/validate the action;
7. pass the canonical Execution Gate;
8. validate tool/function/implementation/provider registry binding;
9. validate invocation-bound data access;
10. claim durable idempotency internally;
11. execute the provider;
12. durably record audit/provenance and outcome;
13. reconcile the lifecycle, including Unknown/indeterminate outcomes.

Invalid binding must fail before provider execution and before idempotency claim.

## Conformance invariants

A reusable implementation is non-conformant if it:

- introduces a competing authorization/policy evaluator;
- permits protected data access without canonical authorization and exact binding;
- permits consequential execution without the canonical Execution Gate;
- executes a provider before registry/provider binding validation;
- lets an external caller directly transition durable idempotency;
- gives a provider data beyond the invocation-bound grant;
- treats AI/system output as authority or human approval;
- silently converts provider/persistence ambiguity into success or failure;
- creates a parallel audit/provenance authority model.

## Domain/application boundary

Domain-specific validation remains above the reusable policy boundary where it represents business invariants rather than authorization semantics.

Example:

Canonical Authorization
→ Domain Binding
→ Authoritative Command / Unit of Work

Observation lifecycle rules such as valid predecessor/successor relationships remain domain invariants; they must not be promoted into a generic policy evaluator merely for reuse.

## Extraction gate

No package split or SDK/API publication should occur until:

1. control-plane convergence is green;
2. protected-data/consequential-execution bypass audit is green;
3. these contracts have executable conformance tests;
4. persistence/lifecycle semantics are stable;
5. an independent consumer can use the boundary without importing SIDERETH-specific application policy.

## Evidence required for v0.1

- repository-wide search for competing authorization/policy engines;
- repository-wide search for direct protected-data access;
- repository-wide search for consequential execution paths;
- adversarial tests for each canonical gate;
- PostgreSQL transactional proof for durable lifecycle claims;
- Foundation and Security & Supply Chain workflows green;
- exactly nine active branches and zero unexpected open PRs.

## Status

This document freezes the contract boundary for the next audit. It does not authorize extraction yet.
