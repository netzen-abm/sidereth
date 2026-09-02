# SIDERETH — Master Blueprint V1

## 1. Product identity
SIDERETH is Legal & Regulatory Infrastructure: a digital operating system for navigating legal and regulatory obligations, official interactions, evidence, deadlines, responses, escalation and human assistance.

## 2. User modes
1. PREPARE — I want to do something.
2. CHECK — Am I compliant?
3. PROTECT — Something is happening now.
4. UNDERSTAND — I received this.
5. RESPOND — Help me deal with it.
6. ESCALATE — This has not been resolved.
7. ASSIST — I need a human professional.

## 3. Universal lifecycle
Discover → Classify → Verify → Prepare → Act → Record → Monitor → Respond → Escalate → Resolve → Learn.

## 4. Core legal models
Canonical procedural model: Authority → Power → Procedure → Document → Deadline → Decision → Appeal → Remedy.

Canonical reasoning model: Facts → Issue → Jurisdiction → Authority → Rule → Procedure → Evidence → Deadline → Options → Risk → Escalation.

## 5. Platform architecture
Shared engines are exposed through independent adapters. Target shared capabilities include Identity, Policy, Authorization, Jurisdiction, Authority, Legal Source, Procedure, Case, Incident, Evidence, Deadline, Compliance, Application, Document/Notice Analysis, Response, Escalation/Remedy, Human Assistance, Audit, Observability, Storage, and bounded AI/agent infrastructure.

## 6. Case/Incident foundation
Case is the universal matter container. Incident captures real-world official interactions and may later link to a Case. Domain events are append-only; derived views are rebuildable. Sensitive case data is not placed on a public ledger.

## 7. Legal knowledge
Source priority: Constitution/legislation; rules/regulations; notifications/orders/circulars; official procedures; judicial decisions; official guidance; reputable secondary sources. Every legal proposition should carry provenance, jurisdiction, effective date/version, retrieval date, authority, citation, confidence and supersession status.

## 8. Evidence
Evidence is preserved before analysis. Originals remain immutable; derived OCR, summaries, classifications and AI analyses are separate artifacts. Evidence is linked to cases, incidents and events.

## 9. Application Guardian
Checks administrative/documentary completeness, technical compliance, eligibility, jurisdiction, fees, timing, consistency, procedural sequence and proof of submission. Product promise is reduced avoidable rejection, not guaranteed approval.

## 10. Protect Now
Live workflows cover inspection, search, seizure, raid, questioning, notice and other official interactions. The system identifies authority/officer, records chronology and stated legal basis, provides conservative lawful guidance, preserves evidence and routes to human help. It must never obstruct lawful action or automatically accuse officials of wrongdoing.

## 11. AI and agents
AI remains optional/user-controlled. The deterministic core must operate without generative AI. Agents may perform bounded retrieval, organization, chronology, deadline checks, summarization and drafting. High-impact submissions, legal representation, consequential government communication and final legal judgments require explicit human control/approval.

Agent platform: Tool Registry → Tool Identity → Tool Gateway → Policy → Permission → Data Minimisation → Tool Runtime → Audit. Memory is scoped to device, case, public legal knowledge or workflow. Model Armor protects against prompt injection, malicious documents, poisoned sources/tools, exfiltration and unauthorized actions.

## 12. Privacy/security
Local-first, privacy-by-default, encryption in transit/at rest, minimization, redaction, explicit authorization, retention/deletion/export controls, case isolation and auditable access. Sensitive data must not silently become shared training data.

## 13. Offline-first
Incident recording, evidence capture, timeline, basic checklists, cached guidance, emergency contacts and local case creation should remain usable without connectivity where technically feasible.

## 14. Domain strategy
Universal core first. Tier 1 includes civic/administrative and public-service domains. Tier 2 covers tax, financial, corporate and regulated industries. Tier 3 covers advanced case-law, litigation support, contracts, detailed legal research and legal strategy. First adapters: Panchayat and Municipality.

## 15. UX blueprint
Primary home actions: Prepare something; Check compliance; Something is happening now; I received a notice; Track an existing matter; I need legal help.

Canonical journey A: Home → Protect Now → Authority → Incident → Evidence → Timeline → Documents → Deadline → Response → Human Assistance.

Canonical journey B: Home → Prepare → Service → Jurisdiction → Eligibility → Documents → Readiness → Submit → Deadline → Decision.

## 16. Engineering sequence
Phase 0: decision/documentation baseline.
Phase 1: domain model, database, events, API, authorization, errors/idempotency/versioning, audit, storage/encryption contracts.
Phase 2: universal Case/Incident/Evidence/Source/Jurisdiction/Authority/Procedure/Deadline/Response/Escalation core.
Phase 3: trust, privacy, security and verification.
Phase 4: agent runtime and human approval infrastructure.
Phase 5: Panchayat/Municipality complete journeys.
Phase 6: Application Guardian.
Phase 7: Protect Now.
Phase 8: domain expansion.
Phase 9: advanced legal intelligence.

## 17. Definition of done
A milestone is complete only when architecture, contracts, implementation, tests, security/privacy controls, source provenance, observability, documentation and operational evidence agree. No merge is justified solely by compilation.

## 18. Current repository state
`sidereth-foundation` is the controlled migration branch. `main` remains untouched until foundation verification. Legacy/decentralized material remains subject to evidence-based disposition.
