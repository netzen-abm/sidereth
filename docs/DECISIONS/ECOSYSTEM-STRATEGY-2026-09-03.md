# SIDERETH — Ecosystem Strategy Decision Record

**Date:** 2026-09-03
**Status:** LOCKED
**Decision type:** Architecture / Product / Engineering strategy

## 1. Decision

SIDERETH will be built as a **full-fledged Legal & Regulatory Infrastructure ecosystem**, not as a prototype-first, MVP-first, or feature-first application.

Prototype and MVP terminology may be used to describe validation stages when useful, but neither is the governing architecture or strategic objective.

## 2. North-star

> Build the ecosystem, not the application.

SIDERETH must provide shared infrastructure that can support many legal/regulatory domains, workflows, products and interfaces over the long term.

## 3. Shared-infrastructure rule

Every reusable capability should be implemented once and consumed through contracts.

Shared assets include:

- capabilities
- functions
- tools
- resources
- schemas
- workflows
- legal knowledge
- evidence infrastructure
- intelligence services
- policies
- integrations

Applications and surfaces must compose these shared assets rather than recreate them.

## 4. Architectural hierarchy

```text
SIDERETH Ecosystem
  -> Trust Kernel
  -> Shared Capabilities
  -> Platform Services
  -> Intelligence
  -> Domain Packs
  -> Surfaces / Adapters
```

## 5. Five reusable concepts

### Capability
What SIDERETH can do.

### Function
A reusable operation inside a capability.

### Tool
An executable interface to a function or capability.

### Resource
A reusable or consumable asset such as a legal source, dataset, template, schema, model or knowledge collection.

### Workflow
A composition of capabilities producing an outcome.

These concepts must not be collapsed into one abstraction.

## 6. Domain strategy

Domains are **domain packs**, not independent products.

A domain pack supplies domain-specific jurisdiction, authorities, sources, procedures, obligations, requirements, documents, deadlines, decisions, remedies and workflows.

Shared Case, Party, Document, Evidence, Authority, Jurisdiction, Procedure, Deadline, Provenance, Policy, Audit and Workflow infrastructure remains common.

## 7. Surface strategy

Web, Android, iOS, Telegram, WhatsApp, desktop, CLI and partner integrations are independent adapters.

A surface may fail, be replaced or be removed without making another surface or the shared capability layer unusable.

Legal/regulatory business logic must not be duplicated by surface.

## 8. Provider strategy

SIDERETH depends on contracts, not providers.

Storage, search, queues, identity providers, AI providers, OCR engines, workflow engines and cloud services must be replaceable adapters.

No vendor is architecturally canonical.

## 9. Intelligence strategy

AI is a shared, bounded intelligence layer rather than the legal source of truth.

Potential capabilities include OCR, extraction, retrieval, RAG, legal research, document comparison, reasoning, evaluation, agents and memory.

AI and agents must use approved contracts and cannot bypass identity, policy, authorization, data minimisation, audit or human approval.

MCP is an interoperability boundary, not the SIDERETH trust kernel.

Langflow, OpenRAG, Genkit and other frameworks may be evaluated as replaceable implementations behind intelligence contracts.

## 10. Trust-kernel strategy

The Trust Kernel owns authoritative deterministic state and controls including:

- domain invariants
- state transitions
- evidence integrity
- provenance
- authorization
- policy
- persistence contracts
- audit identity
- high-impact approval boundaries

The Trust Kernel must not depend on a specific UI, AI framework, transport, database vendor or cloud service.

## 11. Legal workflow model

Canonical reasoning:

```text
Facts -> Issue -> Jurisdiction -> Authority -> Rule -> Procedure
     -> Evidence -> Deadline -> Options -> Risk -> Escalation
```

Canonical procedural model:

```text
Authority -> Power -> Procedure -> Document -> Deadline
          -> Decision -> Appeal -> Remedy
```

Canonical lifecycle:

```text
Discover -> Classify -> Verify -> Prepare -> Act -> Record
         -> Monitor -> Respond -> Escalate -> Resolve -> Learn
```

## 12. Prototype relationship

Prototype work is valuable as evidence, experimentation and architecture discovery.

It is not the production architecture and must not constrain the ecosystem design.

The Kivy/Python prototype remains reference material. Production surfaces will use the appropriate native or platform technologies while reusing SIDERETH contracts and the deterministic kernel.

## 13. Engineering sequence

The ecosystem will be developed by dependency order, not by arbitrary version-number progression.

### Foundation

- Party
- Document
- Action
- Decision
- Case
- Incident
- Event / Timeline
- Evidence
- Authority
- Jurisdiction
- Legal Source
- Provenance
- Procedure
- Obligation
- Compliance
- Deadline
- Response
- Escalation
- Remedy
- Resolution
- Human Assistance
- Audit
- Authorization / Policy

### Platform

- Capability Registry
- Tool Registry
- Tool Identity
- Tool Gateway
- Tool Runtime
- Workflow Engine
- Policy Engine
- Approval Engine
- asynchronous execution
- retry / resume
- notifications
- integrations
- observability

### Intelligence

- OCR
- extraction
- retrieval
- knowledge graph
- RAG
- legal research
- evaluation
- bounded agents
- memory

### Domain packs

Domains are added when the shared infrastructure and authoritative source requirements are sufficiently mature. Demand, legal readiness, data quality and evidence determine sequencing.

### Surfaces

- Web
- Android
- iOS
- Telegram
- WhatsApp
- Desktop
- CLI
- API
- partner integrations

## 14. Quality rule

A capability is complete only when its contract, implementation, tests, security controls, privacy controls, documentation and observability are present and verified.

Architecture documentation alone does not constitute implementation.

## 15. Safety and legal integrity

SIDERETH must help people navigate the rule of law.

It must not obstruct lawful government action, automatically accuse authorities of illegality or corruption without adequate verified basis, or allow autonomous high-impact legal actions without appropriate approval.

## 16. Existing architecture alignment

This decision formalizes and extends the existing SIDERETH architecture principle that shared domain-independent capabilities are built once and exposed through independent adapters.

The existing Capability Contract remains the basis for capability definition and evolution. The ecosystem architecture expands that contract into a broader model covering functions, tools, resources, workflows, domain packs and surfaces.

## 17. Consequence

Future engineering proposals must answer:

1. Is this a shared capability or a domain-specific composition?
2. What existing capability can be reused?
3. What is the contract?
4. What data does it access?
5. What permissions and risk class apply?
6. Can it run through multiple surfaces?
7. Can its implementation/provider be replaced?
8. How is it tested and observed?
9. Does it introduce unnecessary coupling?
10. Does it belong in the Trust Kernel, Platform, Intelligence, Domain Pack or Surface layer?

If a proposed feature cannot answer these questions, it is not ready for implementation.

## 18. Governing statement

> **SIDERETH is an ecosystem of shared, composable, independently replaceable legal and regulatory capabilities. Applications are compositions. Surfaces are adapters. Providers are replaceable. Intelligence is bounded. Evidence is verifiable. Policy is enforced. Contracts are authoritative.**
