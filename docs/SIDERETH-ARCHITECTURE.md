# SIDERETH — Target Architecture

## Status
FOUNDATION / PRE-PRODUCTION

## Product boundary
SIDERETH is a legal and regulatory workflow infrastructure. It helps users navigate lawful procedures, prepare correctly, preserve evidence, meet deadlines, understand notices and decisions, respond, escalate and reach qualified human assistance.

It is not an autonomous lawyer and does not obstruct lawful government action.

## Core lifecycle
Discover → Classify → Verify → Prepare → Act → Record → Monitor → Respond → Escalate → Resolve → Learn

## Universal shared infrastructure
- Case Engine
- Incident Engine
- Event/Timeline Engine
- Evidence Vault
- Document Engine
- Legal Source Registry
- Citation/Provenance Engine
- Jurisdiction Engine
- Authority Engine
- Procedure Engine
- Compliance Engine
- Application Engine
- Deadline Engine
- Response Engine
- Escalation/Remedy Engine
- Human Assistance Router
- Identity/Policy Gateway
- Tool Registry/Runtime
- Memory Bank
- Audit/Observability
- Privacy/Security/Model Armor

## Domain adapters
First: Panchayat and Municipality.
Later: GST, Income Tax, Labour, Land/Revenue, Environment, Consumer, Police/Criminal Procedure, Utilities, Transport, Welfare and specialist legal research.

## Canonical reasoning pipeline
Facts → Issue → Jurisdiction → Authority → Rule → Procedure → Evidence → Deadline → Options → Risk → Escalation

## Core case aggregate
Case
- parties
- authority
- jurisdiction
- matter
- incidents
- events
- documents
- evidence
- legal issues
- sources
- procedures
- deadlines
- actions
- decisions
- responses
- appeals
- escalations
- remedies
- human assistance
- audit

## Incident aggregate
Incident
- type
- authority
- actors
- location
- started_at / ended_at
- stated purpose
- stated legal basis
- actions
- requests
- documents
- items
- statements
- witnesses
- observations

## State principles
All material state changes are represented by validated events. Sensitive case data is local-first. Cloud processing is optional, explicit, minimized, encrypted and auditable.

## AI/agent principle
AI is optional intelligence, not the source of authority. Agents automate bounded workflows; high-impact legal actions require user and/or qualified-human approval.

## Repository migration principle
Existing Janavani/decentralized prototype material is preserved as legacy/reference until verified extraction or archive. No historical material is deleted merely because it is no longer active.
