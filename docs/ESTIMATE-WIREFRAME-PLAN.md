# SIDERETH — Pre-Implementation Estimate & UX/Figma Plan V1

## Purpose
No production feature implementation should begin until the product baseline, system architecture, effort estimate and UX prototype scope have been reviewed together.

## Engineering estimate
These are planning estimates, not commitments. They assume a small senior product/platform team plus part-time legal/privacy expertise.

| Workstream | Estimated hours |
|---|---:|
| Foundation/contracts/architecture | 100–160 |
| Universal Case/Incident core | 220–340 |
| Evidence + document/provenance layer | 140–220 |
| Legal source + jurisdiction/authority/procedure/deadline | 180–280 |
| Security/privacy/audit/observability | 150–250 |
| Web MVP + core UX | 180–260 |
| Agent/tool infrastructure | 250–400 |
| Panchayat/Municipality adapters | 180–280 |
| Application Guardian | 100–160 |
| Protect Now | 140–220 |
| QA, hardening, release engineering | 150–250 |
| **First production-grade milestone** | **1,790–2,820** |

A narrower deterministic MVP that excludes the full agent platform and advanced Protect Now scope is approximately **900–1,300 hours**. The full ecosystem remains materially larger and should be delivered in gated increments.

## Team assumption
Recommended core team: 1 architecture/technical lead, 2 backend/platform engineers, 1 frontend engineer, 1 security/privacy engineer, 1 QA/automation engineer; part-time legal subject-matter expert, UX/product support and DevOps/SRE.

## UX/Figma v0.1
Target 18–25 screens, prioritizing two complete journeys rather than a broad clickable shell.

### Journey A — Protect Now / live incident
Home → Protect Now → select event → identify authority → live incident → immediate guidance → record event → capture evidence → timeline → documents → deadline → response → human assistance.

### Journey B — Application readiness
Home → Prepare → service → jurisdiction → eligibility → document checklist → readiness → submission record → deadline → decision → rejection/response/escalation.

### Core screens
- Home
- Mode selection
- Prepare
- Check compliance
- Protect Now
- Incident setup
- Live incident
- Evidence capture
- Timeline
- Case overview
- Case readiness
- Documents
- Legal sources/citations
- Deadlines
- Response drafting/review
- Escalation/remedy
- Human assistance
- Settings/privacy
- Consent/AI controls

## UX principles
- User problem first; legal terminology second.
- Progressive disclosure.
- Clear distinction between verified law, official procedure, user facts, inference and uncertainty.
- AI controls are explicit and optional.
- High-impact actions visibly require approval.
- Emergency/live mode prioritizes speed, readability and offline continuity.
- Evidence capture must never be hidden behind AI.

## Prototype gate
The Figma/wireframe prototype should validate information architecture, trust/safety cues, approval boundaries and the two canonical journeys before production UI implementation. Visual polish is secondary to workflow correctness.
