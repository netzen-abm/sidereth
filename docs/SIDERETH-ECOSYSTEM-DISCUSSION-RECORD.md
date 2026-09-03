# SIDERETH — Ecosystem Discussion Record

**Date recorded:** 2026-09-03
**Status:** REFERENCE / CONSOLIDATED DISCUSSION RECORD

This document records the strategic and architectural conclusions reached while reviewing the SIDERETH architecture, repository evolution, uploaded platform/legal-tool material, prototype evidence and the transition to a full-fledged ecosystem strategy.

## 1. Product identity

SIDERETH is **Legal & Regulatory Infrastructure**.

It is not an AI lawyer, legal chatbot, complaint application, legal marketplace or collection of unrelated legal features.

North-star:

> A digital legal and regulatory operating system that helps people and businesses understand what the law requires, prepare correctly, protect their rights during official interactions, preserve evidence, meet deadlines, respond intelligently, and reach appropriate human legal assistance when needed.

## 2. Governing philosophy

> Do not build a tool that helps people fight government officers. Build a system that helps people navigate the rule of law.

SIDERETH should protect people when authorities act improperly while also helping them comply when authorities act correctly.

It must not obstruct lawful government action or automatically accuse authorities of illegality or corruption without adequate verified basis.

## 3. Core modes

1. PREPARE — I want to do something.
2. CHECK — Am I compliant?
3. PROTECT — Something is happening now.
4. UNDERSTAND — I received this.
5. RESPOND — Help me deal with it.
6. ESCALATE — This has not been resolved.
7. ASSIST — I need a human professional.

## 4. Canonical lifecycle

Discover -> Classify -> Verify -> Prepare -> Act -> Record -> Monitor -> Respond -> Escalate -> Resolve -> Learn

## 5. Canonical legal model

Authority -> Power -> Procedure -> Document -> Deadline -> Decision -> Appeal -> Remedy

## 6. Canonical reasoning

Facts -> Issue -> Jurisdiction -> Authority -> Rule -> Procedure -> Evidence -> Deadline -> Options -> Risk -> Escalation

## 7. Full ecosystem decision

The project is not governed by prototype/MVP-first thinking. Prototypes are evidence and validation instruments. The strategic objective is a long-lived ecosystem of shared infrastructure.

The primary reusable unit is the capability.

The ecosystem must share:

- infrastructure
- capabilities
- functions
- tools
- resources
- knowledge
- workflows
- policies
- schemas
- integrations
- intelligence services

## 8. Shared capability principle

Build a capability once and expose it everywhere.

Examples:

- Evidence is shared by incidents, notices and cases.
- Deadline is shared by applications, compliance, notices, appeals and courts.
- Document is shared by applications, notices, orders, evidence and contracts.
- Party is shared by citizens, businesses, authorities, lawyers, witnesses and organizations.
- Authority and Jurisdiction are universal legal infrastructure.
- Provenance and Audit apply across the ecosystem.

## 9. Five-part architecture vocabulary

Capability = what SIDERETH can do.

Function = reusable operation inside a capability.

Tool = executable interface for a function or capability.

Resource = reusable/consumable asset such as a legal source, dataset, template, schema, model or knowledge collection.

Workflow = composition of capabilities into an outcome.

## 10. Domain architecture

Domains are domain packs, not independent products.

A tax pack, land pack, police pack, municipality pack or court pack should contribute specialized rules and resources while using the same universal infrastructure.

## 11. Surface architecture

Web, Android, iOS, Telegram, WhatsApp, desktop, CLI and partner integrations are independent adapters.

A failure of one surface must not break another surface or the shared infrastructure.

## 12. Trust Kernel

Rust is currently the chosen implementation technology for the deterministic trust kernel because it is well suited to explicit domain invariants and integrity-sensitive infrastructure.

However, Rust does not own the architecture. Contracts and invariants do.

The Trust Kernel contains authoritative state, invariants, evidence integrity, provenance, authorization, policy and persistence contracts.

## 13. Intelligence layer

Python is the preferred environment for intelligence capabilities such as OCR, extraction, retrieval, RAG, research, evaluation and bounded agents.

AI is optional and user-controlled.

AI cannot become the legal source of truth or bypass authorization, policy, data minimisation, audit or approval boundaries.

MCP is an interoperability adapter, not the core.

Langflow, OpenRAG, Genkit and other frameworks are candidate implementations that must remain replaceable.

## 14. Provider neutrality

SIDERETH depends on contracts, not providers.

Storage, search, queues, identity, AI providers, OCR, workflow runtimes and cloud services are replaceable adapters.

## 15. Privacy and security

Local-first is the default.

Sensitive case data must be minimized and protected. External processing requires appropriate authorization, purpose limitation, encryption and audit.

No AI or agent receives unrestricted case data simply because the data exists.

## 16. Agent architecture

Agents automate bounded workflows rather than autonomous high-impact legal judgment.

Tool access must pass:

```text
Identity -> Policy -> Permission -> Data Minimisation
         -> Risk -> Approval where required -> Execution -> Audit
```

High-impact actions require explicit human approval and qualified professional review where required.

## 17. Legal knowledge architecture

Legal propositions must be traceable to classified sources with jurisdiction, effective date/version, citation, retrieval context, verification status and uncertainty information.

Source hierarchy:

1. Constitution / legislation
2. rules / regulations
3. notifications / orders / circulars
4. official procedures
5. judicial decisions
6. official guidance
7. reputable secondary sources

AI output is not itself legal authority.

## 18. Evidence architecture

Original evidence is immutable at the domain level and must be separated from derived artifacts.

Derived artifacts include OCR, extraction, summaries and analysis.

Prototype testing showed that hash verification detects later modification but does not by itself prevent filesystem overwrite. Production architecture must therefore distinguish tamper detection from tamper prevention and implement both where feasible.

## 19. Event architecture

Material state changes should be represented by validated events. Derived state should be reproducible from valid event history.

Prototype testing exposed an authority-state reconstruction issue. That finding reinforces the requirement that important mutable state be reconstructible deterministically from the canonical event history rather than relying on an opaque snapshot.

## 20. Command boundary

Canonical command execution:

Intent -> Authorization -> Validation -> Mutation -> Event -> Audit -> Result

The current boundary intentionally does not claim distributed exactly-once semantics. Provider-neutral transactional atomicity remains a distinct infrastructure capability to be implemented and verified.

## 21. Prototype architecture disposition

The Kivy/Python prototype successfully demonstrated the interaction flow and provided valuable failure evidence.

It is not the production mobile architecture.

Production mobile should use native platform technologies while reusing SIDERETH contracts and the deterministic kernel. Prototype code and artifacts remain historical/reference evidence and must not silently become architectural dependencies.

## 22. Product/application relationship

Applications are compositions of workflows.

Workflows are compositions of capabilities.

Capabilities are implemented behind contracts.

Surfaces adapt to those contracts.

Providers implement adapters behind those contracts.

This hierarchy prevents feature duplication and vendor lock-in.

## 23. Engineering decision rule

Before implementing anything, ask:

- Is this already a shared capability?
- Can an existing capability be extended rather than duplicated?
- What is its contract?
- What data does it access?
- What risk class applies?
- What permissions apply?
- Can multiple surfaces consume it?
- Can its provider be replaced?
- How is it audited?
- How is it tested?
- Where does it belong architecturally?

## 24. Development philosophy

Do not advance merely because a version number is available.

Advance when the next shared dependency is understood, contracted, implemented and verified.

Avoid framework proliferation and unnecessary architectural expansion.

Preserve history. Archive before deletion. Never force a merge. Never weaken a validation gate simply to move faster.

## 25. Definition of done

A capability is complete only when its contract, implementation, tests, security/privacy controls, documentation and observability are present and verified.

Architecture documents describe intent; executable evidence proves implementation.
