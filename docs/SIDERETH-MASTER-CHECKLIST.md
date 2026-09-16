# SIDERETH — Master Implementation Checklist

**Status:** CANONICAL / LIVE IMPLEMENTATION CHECKLIST
**Last audited baseline:** `main` at `0ac5cdb8e8da545e1e5276af510ba4e8677558b9`
**Audit date:** 2026-09-16

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
- [x] Establish repository `.gitignore` baseline
- [x] Archive and remove transient root-level capability-registry diagnostic

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
- [x] capability registry contract/conformance baseline
- [x] purpose-bound capability lease contract/conformance baseline
- [x] event contract baseline
- [x] state-machine baseline
- [x] authorization matrix baseline
- [x] canonical authorization policy contract
- [x] typed authorization constraint semantics/conformance baseline
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
- [x] Observation contract and conformance baseline

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
- [x] Longitudinal observation/projection foundation

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
- [x] canonical consumer-side authorization enforcement foundation
- [x] purpose-bound capability lease lifecycle contract/model/conformance foundation
- [ ] threat model
- [ ] model/tool injection defenses
- [ ] security audit

## Gate 6 — Agent platform
- [x] Tool Registry contract/conformance design
- [x] Tool Gateway contract/conformance design
- [x] canonical Authorization Contract
- [x] Authorization Evaluator
- [x] canonical consumer-side Authorization Enforcement
- [x] Execution Gate
- [x] AuthorizationResult contextual binding
- [x] typed AuthorizationConstraint semantics/conformance
- [ ] Tool Gateway implementation — implementation exists, but production readiness remains deferred
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
- [ ] Trusted gateway clock / authorization expiry enforcement proven at gateway boundary
- [ ] Returned authorization-constraint enforcement proven at gateway boundary
- [ ] Registry-driven implementation/provider selection proven
- [ ] Rich implementation/provider/provenance audit proven
- [ ] Durable concurrent idempotency integration proven
- [ ] Direct adapter bypass evidence/tests completed
- [ ] TG-001–TG-060 evidence matrix completed
- [ ] Production-ready decision

> The Tool Gateway contract remains canonical, but gateway expansion is deliberately sequenced after the shared authorization, constraint and capability-lease foundations and a fresh protected-operation convergence audit.

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
- [x] GitHub Actions security validation foundation
- [x] secret-pattern scanning foundation gate
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
- [x] GitHub repository governance baseline documented
- [ ] GitHub `main` ruleset activated
- [ ] Live GitHub governance configuration independently verified
- [ ] Full Markdown semantic-duplicate inventory
- [ ] Full internal-link audit after any physical moves
- [ ] Documentation consistency CI

## Definition of Done

A capability is not considered complete until its contract, implementation, tests, security controls, privacy controls, documentation and observability are present and verified.
