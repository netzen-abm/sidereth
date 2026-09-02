# SIDERETH

**Legal & Regulatory Infrastructure**

SIDERETH is a privacy-first, modular legal and regulatory workflow platform designed to help people and businesses navigate the rule of law.

## What it does

- Prepare applications and compliance workflows
- Check jurisdiction, authority, procedure and documentary readiness
- Protect and record lawful government interactions
- Preserve evidence and build verifiable timelines
- Understand notices, orders and decisions
- Track deadlines and required actions
- Draft responses for user review
- Escalate unresolved matters
- Route high-stakes matters to qualified human professionals

## Architecture principle
The platform is built around shared, domain-independent capabilities. Web, mobile and messaging surfaces are independent adapters over the same contracts and engines. Domain logic must not be duplicated per surface.

## AI and agent principle
AI is optional and user-controlled. It may retrieve, classify, summarize, organize, draft and recommend within policy boundaries. Agents automate bounded workflows, not high-impact legal judgment. MCP is an interoperability boundary for approved tools; it does not replace SIDERETH policy, authorization, case, evidence or audit infrastructure.

## Privacy principle
Local-first by default. Sensitive case data should remain under user control. External processing is minimized, explicitly authorized, encrypted and auditable.

## Initial engineering milestone
Build the domain-independent Case and Incident Engine first, with evidence, events, source provenance and deterministic state transitions. Then add Panchayat and Municipality domain adapters without duplicating shared infrastructure.

## Repository status
This repository is undergoing a controlled migration from an earlier Janavani/decentralized prototype. Legacy artifacts have been audited and removed from the active tree where they no longer align; their history is preserved and the disposition is recorded.

## Current foundation documents
- `docs/00-SIDERETH-MASTER-DECISIONS.md`
- `docs/01-SIDERETH-MASTER-BLUEPRINT.md`
- `docs/SIDERETH-ARCHITECTURE.md`
- `docs/ESTIMATE-WIREFRAME-PLAN.md`
- `docs/08-MCP-ARCHITECTURE.md`
- `docs/05-REPOSITORY-CODE-MD-AUDIT.md`
- `docs/SIDERETH-MASTER-CHECKLIST.md`
- `docs/contracts/CASE-INCIDENT-EVENTS.md`
- `docs/contracts/AUTHORIZATION-MATRIX.md`
- `docs/migration/LEGACY-DISPOSITION.md`
- `docs/archive/legacy-janavani/README.md`

## Development rule
Do not infer implementation from architecture documents. A capability becomes complete only when its contract, implementation, tests, security controls, documentation and observability are present and verified.
