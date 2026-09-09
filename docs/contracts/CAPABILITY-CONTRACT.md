# SIDERETH — Capability Contract

**Status:** CANONICAL / FOUNDATION CONTRACT
**Scope:** Shared capability definition, governance and conformance boundary

## 1. Purpose

SIDERETH capabilities are shared infrastructure. Web, mobile, bots, integrations and future surfaces are adapters; they do not create parallel business logic.

A capability is the stable contract through which reusable SIDERETH behavior is defined, implemented, exposed, tested and composed.

This contract governs the capability boundary. It does not require a particular programming language, framework, cloud vendor, storage provider, AI provider or transport.

## 2. Capability envelope

Each capability must declare, as applicable:

- `capability_id`
- `version`
- name
- purpose
- lifecycle status
- input schema
- output schema
- required permissions
- data classes accessed
- jurisdiction scope
- risk class
- source requirements
- approval requirements
- audit requirements
- supported execution modes (`sync`, `async`, `offline`, `resumable` where applicable)
- dependency references
- implementation references
- adapter references
- observability requirements
- failure/error semantics

A capability contract must be sufficiently explicit for an independent implementation to be tested for conformance without depending on another implementation's internal details.

## 3. Identity and versioning

`capability_id` is stable across compatible implementations.

Capability versions are contract versions, not implementation or provider versions.

Breaking contract changes require a new major capability version or an explicit, documented migration path. Consumers must be able to reject unsupported versions deterministically.

Implementation/provider versions may change without changing the capability identity when the implementation remains contract-conformant.

## 4. Risk classes

- `READ_ONLY`: public or otherwise authorized retrieval with no canonical mutation.
- `USER_DATA`: scoped access to user-authorized case, party, document or evidence data.
- `MUTATING`: changes canonical SIDERETH state.
- `HIGH_IMPACT`: creates consequential external effects or legally significant actions and requires explicit human approval and professional review where required.

Risk classification is part of the contract and must not be weakened by an adapter, tool, model or workflow invocation.

## 5. Authorization, policy and approval

A capability declaration does not itself grant permission.

The execution path must evaluate the applicable identity, authorization and policy controls before access to protected data or execution of protected operations.

Authorization remains distinct from human approval. Where a capability participates in a consequential Action, the canonical Action/Approval/Execution Gate remains authoritative.

AI or system output cannot manufacture human approval authority.

## 6. Adapter rule

Adapters translate transport-specific input/output into canonical capability contracts. They must not bypass authorization, policy, data minimisation, audit, provenance or approval controls.

An adapter may add presentation, transport or provider-specific behavior only when that behavior does not change the canonical capability semantics.

A surface failure must not imply failure of the underlying capability or another independent surface.

## 7. Implementation and provider neutrality

A capability may have multiple implementations or providers.

The canonical contract owns semantics; implementations own mechanism.

SIDERETH must not make a provider, framework or language part of a capability contract unless that dependency is itself explicitly required by the capability's semantics.

Examples of replaceable implementation choices include storage, search, AI/model providers, queues, external integrations and transport protocols.

## 8. AI rule

AI may consume capability contracts but cannot acquire permissions merely by generating a request.

The Tool Gateway and applicable policy/authorization boundaries evaluate identity, policy, scope and risk before execution.

AI-generated interpretations are not automatically canonical legal state, evidence, authorization, approval or decision records.

## 9. MCP rule

MCP is an interoperability adapter for exposing selected SIDERETH capabilities to compatible AI hosts. MCP is not the source of truth for permissions, identity, case state, evidence, legal provenance or audit.

## 10. Function, tool and resource relationships

A capability may expose reusable functions, controlled executable tools and consumable resources.

- A **Function** is a reusable operation within a capability.
- A **Tool** is a controlled executable interface for performing an operation.
- A **Resource** is data or knowledge consumed by a capability.
- A **Workflow** composes capabilities into a controlled outcome.

These relationships must not be used to bypass the capability contract.

Conceptually:

```text
Capability
   |
   +-- Contract
   +-- Functions
   +-- Tools
   +-- Resources
   +-- Policies
   +-- Workflow participation
   +-- Implementations
   +-- Adapters
   +-- Tests
   +-- Documentation
   +-- Observability
```

## 11. Execution semantics

Where execution is supported, the contract must define the relevant execution mode and externally observable semantics, including as applicable:

- synchronous vs asynchronous behavior
- idempotency expectations
- retry safety
- timeout behavior
- resumability
- offline/degraded operation
- ordering requirements
- concurrency constraints
- failure classification

A capability must not silently turn a retryable operation into a duplicate consequential operation.

## 12. Data and provenance

Capabilities must declare the data classes they access and any jurisdictional or source constraints relevant to their operation.

Where outputs depend on external, legal or evidentiary sources, provenance requirements must be explicit.

Legal propositions require source provenance under the SIDERETH legal-source boundary. Evidence integrity and provenance remain governed by the canonical Evidence Trust and Provenance contracts.

## 13. Audit and observability

Capabilities that access protected data, mutate canonical state, execute tools or participate in consequential workflows must declare the required audit and observability records.

At minimum, consequential operations must remain attributable to the relevant request/case/action context and must preserve the applicable provenance and authorization references.

Operational observability must not replace canonical audit records.

## 14. Capability lifecycle

```text
Proposed
 -> Designed
 -> Contracted
 -> Implemented
 -> Tested
 -> Security Reviewed
 -> Operationally Verified
 -> Active
 -> Deprecated
 -> Retired
```

Lifecycle status is governance metadata. It does not by itself prove implementation readiness.

## 15. Conformance requirements

A capability implementation is conformant only when the applicable requirements are demonstrated through:

1. contract/schema validation;
2. deterministic unit or contract tests;
3. relevant integration tests;
4. security/privacy checks;
5. provenance and audit checks where required;
6. failure and boundary-condition tests;
7. documentation sufficient for independent consumption;
8. operational verification where the capability is promoted to `Active`.

Provider-specific tests may supplement, but may not replace, contract conformance tests.

## 16. Non-goals

This contract does not define:

- a universal programming language;
- a mandatory storage technology;
- a mandatory AI/model provider;
- a mandatory workflow framework;
- a mandatory transport;
- autonomous legal authority;
- autonomous consequential execution.

Those concerns are implementation, adapter or separate governance boundaries unless explicitly elevated into a canonical contract.

## 17. Evolution

Capability contracts are versioned. Breaking changes require a new major contract version or an explicit migration path.

Changes that affect authorization, data classification, risk, provenance, approval or consequential execution require explicit architectural review and an updated decision record where material.

Before creating a parallel capability specification, the documentation index and existing canonical contracts must be checked first. Existing contracts should be extended when the scope is already covered.
