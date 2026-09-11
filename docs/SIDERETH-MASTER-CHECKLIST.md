# SIDERETH — Master Implementation Checklist

**Status:** CANONICAL / LIVE IMPLEMENTATION CHECKLIST
**Last audited baseline:** `main` at `ab36d34a90a9363044e4700576523a4a87259534`

> Checklist state is evidence-based. A checked item means the repository currently contains the stated foundation/evidence at the scope described; it does not automatically mean production-ready.

## Gate 0 — Repository safety
- [x] Create isolated foundation branch
- [x] Record current main baseline
- [x] Record existing branches
- [x] Preserve legacy material
- [x] Compare `decentralized-system` against `main`
- [x] Review unique legacy files and commits
- [x] Archive/document useful disposition before active-tree removal
- [x] Audit active programming and Markdown for SIDERETH alignment
- [ ] Verify workflow/deployment dependencies on final integration branch
- [ ] Decide branch deletion only after final evidence
- [x] Establish documentation authority and duplication governance

## Gate 1 — Product foundation
- [x] Define legal/regulatory OS boundary
- [x] Define neutrality principle
- [x] Define AI-optional principle
- [x] Define local-first privacy principle
- [x] Establish canonical Master Decisions baseline
- [x] Establish master architecture/roadmap baseline
- [x] Establish MCP interoperability decision
- [ ] Formal product constitution
- [x] Decision Register
- [x] Canonical terminology/glossary
- [x] Documentation Governance and Authority Map

## Gate 2 — Contracts before production code
- [x] domain model contract (draft)
- [x] database/storage boundary contract (draft)
- [ ] database schema implementation
- [ ] OpenAPI contract
- [x] capability contract (draft)
- [x] event contract baseline
- [x] state-machine baseline
- [x] authorization matrix baseline
- [x] encryption/storage model (draft)
- [x] audit model (draft)
- [x] error model (draft)
- [x] idempotency model (draft)
- [x] versioning policy (draft)
- [x] canonical Case JSON Schema
- [x] canonical Incident JSON Schema
- [x] persistence boundary audit
- [x] persistence contract hardening v1.2-C
- [x] contract review and consistency audit for persistence boundary

## Gate 3 — Universal core
- [x] Case Engine foundation
- [x] Incident Engine foundation
- [x] Event/Timeline Engine foundation
- [x] Party model foundation
- [x] Authority Engine foundation
- [x] Jurisdiction Engine foundation
- [ ] Document Engine
- [x] Evidence Vault foundation
- [x] Deadline Engine foundation
- [x] Action/Decision model foundation
- [x] Response Engine foundation
- [x] Escalation/Remedy Engine foundation
- [ ] Human Assistance Router

## Gate 4 — Legal knowledge
- [x] Legal Source Registry foundation
- [x] citation/provenance model foundation
- [x] effective-date handling foundation
- [x] version/supersession handling foundation
- [x] verification status foundation
- [ ] jurisdiction-aware retrieval
- [x] source confidence/uncertainty foundation

## Gate 5 — Security/privacy
- [ ] data classification
- [ ] local-first storage production layer
- [ ] encryption at rest
- [ ] encryption in transit
- [ ] key management
- [ ] minimisation/redaction
- [ ] consent boundaries
- [ ] retention/deletion
- [x] access control foundation
- [ ] threat model
- [ ] model/tool injection defenses
- [ ] security audit

## Gate 6 — Agent platform
- [x] Tool Registry contract/conformance design
- [x] Tool Gateway contract/conformance design
- [x] canonical Authorization Contract
- [x] Authorization Evaluator
- [x] Execution Gate
- [x] AuthorizationResult contextual binding
- [ ] Tool Gateway implementation — PR #73 in progress; not production-ready
- [ ] Tool Identity production implementation
- [ ] Tool Runtime production boundary
- [ ] Policy engine production layer
- [ ] workflow orchestration
- [ ] Memory Bank
- [ ] human approval production checkpoints
- [ ] asynchronous jobs
- [ ] retries/resume
- [ ] durable audit/observability
- [x] MCP adapter boundary

### Tool Gateway implementation gates — current
- [x] Exact-head Foundation CI at `7a028da2dd200b6cdf45950d69dff43b61ffadaf`
- [x] Exact-head security/supply-chain checks passed
- [ ] Trusted gateway clock / authorization expiry enforcement
- [ ] Returned authorization-constraint enforcement
- [ ] Registry-driven implementation/provider selection
- [ ] Rich implementation/provider/provenance audit
- [ ] Durable concurrent idempotency integration
- [ ] Direct adapter bypass evidence/tests
- [ ] TG-001–TG-060 evidence matrix completed
- [ ] Production-ready decision

## Gate 7 — UX
- [ ] Home
- [ ] Prepare
- [ ] Check
- [ ] Protect Now
- [ ] Incident workspace
- [ ] Case workspace
- [ ] Evidence capture
- [ ] Timeline
- [ ] Notice analysis
- [ ] Deadline view
- [ ] Response
- [ ] Escalation
- [ ] Human assistance
- [ ] privacy controls
- [ ] offline states
- [ ] accessibility
- [ ] mobile responsive

## Gate 8 — Domain packs
### Panchayat
- [ ] jurisdiction
- [ ] authorities
- [ ] services
- [ ] eligibility
- [ ] documents
- [ ] fees
- [ ] procedure
- [ ] deadlines
- [ ] decision/rejection
- [ ] appeal
- [ ] escalation
- [ ] authoritative sources

### Municipality
- [ ] jurisdiction
- [ ] authorities
- [ ] services
- [ ] eligibility
- [ ] documents
- [ ] fees
- [ ] procedure
- [ ] deadlines
- [ ] decision/rejection
- [ ] appeal
- [ ] escalation
- [ ] authoritative sources

## Gate 9 — Quality
- [x] unit tests for implemented core boundaries
- [ ] integration tests
- [x] contract tests for implemented persistence boundary
- [x] state-machine tests for implemented states
- [ ] security tests
- [ ] privacy tests
- [x] evidence integrity tests
- [x] legal-source verification tests for implemented registry rules
- [ ] offline tests
- [ ] accessibility tests
- [ ] load tests
- [x] recovery tests for local persistence reference scope
- [ ] end-to-end scenario suite

## Gate 10 — Production
- [x] CI/CD foundation validation
- [x] RustSec dependency audit foundation
- [x] secret scanning foundation gate
- [ ] SBOM
- [ ] release signing
- [ ] observability
- [ ] incident response
- [ ] backup/restore
- [ ] disaster recovery
- [ ] deployment runbook
- [ ] production approval

## Documentation governance gate
- [x] Documentation index authority map
- [x] Documentation governance rules
- [x] Archive-first disposition rule
- [x] AI-agent reading protocol
- [x] Developer reading protocol
- [x] Current Tool Gateway status documented honestly
- [ ] Full Markdown semantic-duplicate inventory
- [ ] Full internal-link audit after any physical moves
- [ ] Documentation consistency CI

## Definition of Done

A capability is not considered complete until its contract, implementation, tests, security controls, privacy controls, documentation and observability are present and verified.
