# SIDERETH — Authorization & Policy Contract

**Status:** CANONICAL / FOUNDATION CONTRACT  
**Scope:** Provider-neutral authorization decision boundary consumed by protected capabilities and the future Tool Gateway

## 1. Purpose

Authorization determines whether a subject may perform a requested operation against a scoped resource for a declared purpose under applicable policy.

Authorization is a permission decision, not:

- capability registration;
- tool registration;
- human approval;
- legal authority;
- evidence authenticity;
- action execution;
- provider trust;
- workflow completion.

The Tool Gateway will consume this boundary. It must not create a second authorization or policy semantics.

## 2. Canonical request

An authorization request must identify, as applicable:

- stable request identity;
- subject/actor reference;
- action/operation reference;
- resource or case scope;
- declared purpose;
- applicable policy references;
- requested data class/sensitivity;
- jurisdiction/scope context;
- relevant capability/tool references;
- request timestamp and freshness context.

The request is evaluated as a whole. Adapters may translate transport identity, but may not widen subject, resource, purpose, jurisdiction or requested authority.

## 3. Canonical result

A result must contain:

- deterministic decision: `allow`, `deny`, or `not_applicable`;
- policy references used for the decision;
- enforceable constraints where applicable;
- scope/data-minimisation requirements where applicable;
- freshness/expiry information where required;
- provenance/audit context sufficient to reproduce attribution.

`allow` is not unconditional permission. The consuming boundary must enforce all returned constraints.

## 4. Fail-closed semantics

The authorization boundary must fail closed for protected operations when required decision context is:

- missing;
- malformed;
- stale or expired;
- internally conflicting;
- outside declared jurisdiction/scope;
- unable to establish required policy applicability.

A transport error, registry failure, unknown policy, ambiguous policy result or unavailable authorization provider must not be interpreted as `allow`.

## 5. Scope and data minimisation

Authorization must constrain at least the dimensions relevant to the protected operation:

- subject;
- action;
- resource/case;
- purpose;
- jurisdiction;
- data class;
- temporal validity;
- returned constraints.

A downstream adapter, tool, provider or model may reduce scope but may not expand an authorized scope.

Data minimisation is an enforcement obligation, not merely descriptive metadata. A request authorized for a narrower data class or resource scope cannot be transformed into unrestricted access.

## 6. Relationship to capabilities and tools

Capability Registry membership and Tool Registry membership are discovery metadata only. Neither grants permission.

The canonical path is conceptually:

```text
Capability / Tool Resolution
        |
        v
Identity + Authorization + Policy Evaluation
        |
        v
Tool Gateway
        |
        +--> enforce subject/resource/purpose/scope/data constraints
        +--> enforce risk and approval requirements
        +--> invoke bounded implementation only when permitted
```

Registry resolution must not be treated as an authorization result.

## 7. Relationship to Action and human approval

Authorization remains distinct from human approval.

For consequential Actions, authorization is necessary but not sufficient. The canonical Action/Approval/Execution Gate remains authoritative for execution. A successful authorization result cannot manufacture, replace or imply human approval.

Likewise, human approval cannot grant authorization that policy does not permit.

## 8. Relationship to Intelligence

AI may request an authorization evaluation or propose a tool operation, but AI output cannot grant itself permission.

An intelligence provider, model, agent, MCP adapter or external content source cannot alter the canonical authorization decision or its constraints merely by generating metadata or instructions.

## 9. Provider neutrality

Authorization semantics belong to this contract. Policy storage, policy languages, identity providers, databases, deployment topology and evaluation engines are replaceable implementations.

A provider replacement is conformant only when it preserves decision semantics, scope restrictions, failure behavior and audit/provenance requirements.

## 10. Determinism and reproducibility

For the same canonical request, policy state and evaluation context, the decision and enforceable constraints must be reproducible.

Where policy state changes over time, the result must preserve sufficient version/freshness context to explain which policy state produced the decision.

## 11. Non-goals

This contract does not implement:

- a production identity provider;
- a particular policy language;
- a distributed authorization service;
- Tool Gateway execution;
- Tool Runtime;
- MCP;
- human approval workflows;
- legal reasoning or legal authority;
- autonomous agents.

## 12. Conformance requirement

Any implementation intended for protected SIDERETH execution must pass the implementation-independent authorization conformance matrix before it can serve as a Tool Gateway authorization source.
