# SIDERETH — Ecosystem Roadmap

**Status:** CANONICAL / STRATEGIC ROADMAP

## Governing approach

This roadmap is capability-led rather than prototype/MVP/version-led. Version numbers may identify releases, but they do not determine architectural priority.

The objective is a full-fledged ecosystem of reusable infrastructure.

## Track 1 — Trust Kernel

Continue hardening and completing universal deterministic infrastructure:

1. Party
2. Document
3. Action
4. Decision
5. Case
6. Incident
7. Event / Timeline
8. Evidence
9. Authority
10. Jurisdiction
11. Legal Source / Provenance
12. Procedure
13. Obligation / Compliance
14. Deadline
15. Response
16. Escalation / Appeal / Remedy
17. Resolution
18. Human Assistance
19. Authorization / Policy
20. Audit

## Track 2 — Persistence and execution

- provider-neutral transactional command boundary
- durable state adapters
- evidence storage adapters
- schema/version compatibility
- recovery semantics
- idempotency
- concurrency
- offline synchronization boundaries

## Track 3 — Platform services

- Capability Registry
- Tool Registry
- Tool Identity
- Tool Gateway
- Tool Runtime
- Workflow Engine
- Policy Engine
- Approval Engine
- notification capability
- integration capability
- async execution
- retries and resumability
- observability

## Track 4 — Intelligence

- document ingestion
- OCR
- extraction
- classification
- search
- retrieval
- knowledge graph
- RAG
- legal research
- document comparison
- reasoning
- evaluation
- bounded agents
- memory

Intelligence implementations must remain replaceable behind contracts.

## Track 5 — Domain packs

Initial architectural candidates include:

- Panchayat
- Municipality
- Police
- Land / Revenue
- Tax / GST
- Labour
- Consumer
- Environment
- Transport
- Utilities
- Education
- Health
- Welfare
- Business / Corporate
- Courts / specialist legal domains

Sequencing is determined by infrastructure readiness, authoritative source availability, legal review, user need and evidence—not arbitrary release numbering.

## Track 6 — Surfaces

- Web
- Android
- iOS
- Telegram
- WhatsApp
- Desktop
- CLI
- public/private APIs
- partner integrations

Every surface consumes shared capabilities through contracts.

## Track 7 — Ecosystem governance

Establish:

- capability lifecycle governance
- schema governance
- compatibility rules
- security review
- privacy review
- legal-source review
- release governance
- deprecation policy
- contributor governance
- evidence standards
- audit standards

## Strategic sequence

```text
Trust Kernel
    -> Shared Capabilities
    -> Platform Services
    -> Intelligence
    -> Domain Packs
    -> Surfaces
    -> Ecosystem Expansion
```

Multiple tracks may progress in parallel where dependencies are clear, but no track may bypass foundational contracts and safety boundaries.

## Long-term objective

SIDERETH should become an extensible legal and regulatory operating infrastructure in which new domains, tools, providers, intelligence systems and user surfaces can be added without rewriting the core.
