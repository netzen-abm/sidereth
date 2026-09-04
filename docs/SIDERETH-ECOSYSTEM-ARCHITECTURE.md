# SIDERETH — Ecosystem Architecture

**Status:** CANONICAL / ARCHITECTURE BASELINE
**Scope:** Full-fledged Legal & Regulatory Infrastructure ecosystem

## 1. Purpose

SIDERETH is being built as an ecosystem, not as a single application, prototype, or feature collection. Its architecture must support many legal and regulatory domains, reusable capabilities, independent surfaces, replaceable providers, bounded intelligence, and long-term evolution.

The governing principle is:

> Build the ecosystem, not the application.

## 2. North-Star Architecture Principle

Build every important capability once and make it reusable everywhere.

Contracts, schemas, protocols, provenance, policy and invariants own the architecture. Programming languages, frameworks, cloud vendors, storage providers, AI providers and transport channels are replaceable implementations or adapters.

## 3. Ecosystem Hierarchy

```text
SIDERETH Ecosystem
    |
    +-- Trust Kernel
    |     +-- Domain state and invariants
    |     +-- Evidence integrity
    |     +-- Provenance
    |     +-- Authorization and policy
    |     +-- Persistence contracts
    |
    +-- Shared Capabilities
    |     +-- Case
    |     +-- Incident
    |     +-- Party
    |     +-- Document
    |     +-- Evidence
    |     +-- Event / Timeline
    |     +-- Authority / Jurisdiction
    |     +-- Procedure / Obligation / Compliance
    |     +-- Deadline
    |     +-- Action / Decision
    |     +-- Response / Appeal / Escalation / Remedy
    |     +-- Resolution
    |     +-- Human Assistance
    |
    +-- Platform Services
    |     +-- Capability Registry
    |     +-- Tool Registry
    |     +-- Tool Gateway
    |     +-- Tool Runtime
    |     +-- Workflow Engine
    |     +-- Policy Engine
    |     +-- Approval Engine
    |     +-- Notification / Integration Services
    |     +-- Audit / Observability
    |
    +-- Intelligence
    |     +-- OCR
    |     +-- Extraction
    |     +-- Search / Retrieval
    |     +-- Knowledge Graph
    |     +-- RAG
    |     +-- Legal research
    |     +-- Reasoning / analysis
    |     +-- Agents
    |     +-- Evaluation
    |     +-- Memory
    |
    +-- Domain Packs
    |     +-- Panchayat
    |     +-- Municipality
    |     +-- Police
    |     +-- Land / Revenue
    |     +-- Tax / GST
    |     +-- Labour
    |     +-- Consumer
    |     +-- Environment
    |     +-- Transport
    |     +-- Utilities
    |     +-- Education / Health / Welfare
    |     +-- Business / Corporate
    |     +-- Courts and specialist domains
    |
    +-- Surfaces / Adapters
          +-- Web
          +-- Android
          +-- iOS
          +-- Telegram
          +-- WhatsApp
          +-- Desktop / CLI
          +-- API / partner integrations
```

## 4. Capability Is the Primary Reusable Unit

A capability describes a reusable ability of SIDERETH. A capability is not a page, chatbot prompt, vendor integration, or domain-specific application.

Each capability should have:

- stable capability ID
- contract version
- purpose
- input schema
- output schema
- permissions
- data classes accessed
- jurisdiction scope
- risk class
- source requirements
- approval requirements
- audit requirements
- supported execution modes
- implementation/adapters
- tests
- documentation
- observability requirements

Risk classes remain:

- `READ_ONLY`
- `USER_DATA`
- `MUTATING`
- `HIGH_IMPACT`

## 5. Capability, Function, Tool, Resource and Workflow

These concepts must remain distinct.

### Capability
What SIDERETH can do.

### Function
A reusable operation within a capability.

### Tool
An executable interface through which a function or capability can be invoked.

### Resource
A consumable or referenceable asset such as a legal source, dataset, template, schema, model or knowledge collection.

### Workflow
A composition of capabilities that produces an outcome.

Example:

```text
Government Notice
    -> Document
    -> Extract
    -> Jurisdiction
    -> Authority
    -> Procedure
    -> Deadline
    -> Evidence
    -> Response
    -> Human Review
```

## 6. Domain Packs

Domains must compose shared infrastructure rather than create parallel systems.

A domain pack may contribute:

- jurisdiction definitions
- authorities
- legal sources
- procedures
- obligations
- requirements
- documents
- deadlines
- decisions
- remedies
- domain-specific workflows
- authoritative source connectors

The underlying Case, Evidence, Document, Party, Authority, Jurisdiction, Deadline, Provenance, Audit, Policy and Workflow infrastructure remains shared.

## 7. Surface Independence

Web, Android, iOS, messaging channels, desktop, CLI and partner APIs are adapters.

A failure or removal of one surface must not invalidate the shared capability layer or other surfaces.

Adapters translate transport-specific representations into canonical contracts and must not duplicate legal or regulatory business logic.

## 8. Provider Neutrality

SIDERETH depends on contracts, not providers.

Storage, search, queues, identity providers, AI providers, OCR engines, workflow runtimes and external services must be replaceable through explicit adapter boundaries.

Examples of storage implementations may include local storage, SQLite, PostgreSQL, object storage or private cloud. No provider is canonical merely because it is used by one deployment.

## 9. Trust Kernel

The Trust Kernel contains authoritative deterministic state and controls:

- domain invariants
- canonical state transitions
- evidence integrity
- legal-source provenance
- authorization
- policy enforcement
- persistence contracts
- audit identity
- high-impact approval boundaries

The kernel must not depend on a particular UI, AI framework, transport, database vendor or cloud provider.

## 10. Intelligence Boundary

Intelligence is a shared augmentation layer, not the source of legal authority.

Potential intelligence capabilities include:

- OCR
- extraction
- classification
- retrieval
- RAG
- legal research
- document comparison
- reasoning
- summarization
- agentic workflow execution
- evaluation

AI systems may consume approved capability contracts. They cannot gain permissions merely by generating requests and cannot bypass identity, policy, authorization, data minimisation, audit or human approval boundaries.

MCP is an interoperability boundary. It is not the source of truth for identity, policy, permissions, case state, evidence or audit.

Langflow, OpenRAG, Genkit and similar technologies may be evaluated as replaceable intelligence implementations. None is a SIDERETH core dependency by architectural default.

## 11. Privacy and Data Boundary

Local-first is the default.

Sensitive data should remain under user control whenever practical. External processing requires purpose limitation, data minimisation, explicit authorization where required, encryption and audit.

AI-disabled operation must remain possible for core deterministic workflows.

No agent receives unrestricted access to the user's case data. Data must cross boundaries only through explicit policy-controlled interfaces.

## 12. Canonical Legal Reasoning

```text
Facts
  -> Issue
  -> Jurisdiction
  -> Authority
  -> Rule
  -> Procedure
  -> Evidence
  -> Deadline
  -> Options
  -> Risk
  -> Escalation
```

## 13. Canonical Procedural Model

```text
Authority
  -> Power
  -> Procedure
  -> Document
  -> Deadline
  -> Decision
  -> Appeal
  -> Remedy
```

## 14. Canonical Lifecycle

```text
Discover
  -> Classify
  -> Verify
  -> Prepare
  -> Act
  -> Record
  -> Monitor
  -> Respond
  -> Escalate
  -> Resolve
  -> Learn
```

## 15. Universal Capability Roadmap

The architecture should mature shared primitives in dependency order rather than chase isolated application features.

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
- Citation / Provenance
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
- Async execution
- Retry / resume
- Notification
- Integration
- Observability

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

### Domain Packs

Add domains by evidence, demand, legal readiness and infrastructure maturity rather than by arbitrary version-number momentum.

## 16. Event and Evidence Integrity

Material state changes must be represented by validated events. Derived state should be reproducible from valid event history.

Original evidence is immutable in the logical domain. Derived artifacts such as OCR, extraction, summaries and analysis must be separate and source-linked.

Prototype findings around event reconstruction, evidence overwrite, crash recovery and clock semantics are treated as engineering lessons for hardening the shared infrastructure rather than as reasons to preserve prototype implementation choices.

## 17. Command Boundary

The canonical command flow is:

```text
Intent
  -> Authorization
  -> Validation
  -> Mutation
  -> Event
  -> Audit
  -> Result
```

The current command boundary does not claim distributed exactly-once semantics. Provider-neutral transactional atomicity remains a future infrastructure capability and must be implemented honestly where supported rather than inferred from orchestration alone.

## 18. High-Impact Actions

Agents and automation may prepare and organize work, but high-impact actions require explicit approval and professional review where required.

Examples include:

- legal submissions
- government filings
- consequential external communication
- litigation strategy
- other legally consequential actions

SIDERETH must not obstruct lawful government action and must not automatically characterize an authority's conduct as unlawful or corrupt without adequate verified basis.

## 19. Prototype Relationship

Prototype applications are validation instruments and evidence sources. They are not architectural authorities.

A prototype may prove a workflow, expose defects, validate UX assumptions or demonstrate a kernel contract. Production architecture must still satisfy the ecosystem contracts, security model, privacy requirements, test requirements and operational evidence.

The Kivy/Python prototype is therefore retained as reference evidence. Production mobile surfaces should use native platform technologies while reusing the same SIDERETH contracts and deterministic kernel.

## 20. Definition of Ecosystem Completion

A capability is complete only when the relevant contract, implementation, tests, security controls, privacy controls, documentation and observability are present and verified.

An ecosystem layer is not complete merely because a document describes it.

## 21. Governing Rule

> **Build shared infrastructure once. Compose capabilities into workflows. Make every surface an adapter. Keep providers replaceable. Keep intelligence bounded. Preserve user control. Make evidence and provenance verifiable.**
