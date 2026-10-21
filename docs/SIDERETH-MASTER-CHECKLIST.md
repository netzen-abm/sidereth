# SIDERETH — Master Implementation Checklist

**Status:** CANONICAL / LIVE IMPLEMENTATION CHECKLIST  
**Last verified main baseline:** `73b17427b0c5fdc6bac9e01a612e0f75b3196a9c`  
**Audit date:** 2026-10-21

> Checklist state is evidence-based. A checked item means the repository contains the stated implementation or proof at the described scope; it does not automatically mean production-ready.

## Current verified baseline — 2026-10-21

**Main:** `73b17427b0c5fdc6bac9e01a612e0f75b3196a9c`  
**Active branches:** exactly **9**, verified after PR #172 merged.  
**Engineering posture:** close concrete trust-boundary bypasses and prove behavior. Add abstractions only where distinct trust, persistence, provider, change, or reuse boundaries justify them.

### Latest verified GitHub evidence

- PR [#170](https://github.com/netzen-abm/sidereth/pull/170) merged the generic Evidence write guard as `e7f901ccc75d7d59a87c6f5009cc98721dfd327f`. Exact-head Foundation Validation [#39610409084](https://github.com/netzen-abm/sidereth/actions/runs/39610409084) and Security/Supply Chain [#39610409076](https://github.com/netzen-abm/sidereth/actions/runs/39610409076) passed, including tests, the live PostgreSQL proof matrix, Clippy, RustSec Audit, and GitHub Actions Security.
- PR [#171](https://github.com/netzen-abm/sidereth/pull/171) rebaselined the checklist as `8b75998098f60abf67457a132da9866c39d31a31`; main Foundation Validation [#39612724805](https://github.com/netzen-abm/sidereth/actions/runs/39612724805), Security/Supply Chain [#39612724707](https://github.com/netzen-abm/sidereth/actions/runs/39612724707), branch enforcement [#39612724728](https://github.com/netzen-abm/sidereth/actions/runs/39612724728), and push orchestration [#39612771384](https://github.com/netzen-abm/sidereth/actions/runs/39612771384) passed on that head.
- PR [#172](https://github.com/netzen-abm/sidereth/pull/172) merged as `73b17427b0c5fdc6bac9e01a612e0f75b3196a9c`. On exact PR head `2d022eb2325de0c7975eb251b1e47e90ba99ef7e`, Foundation Validation [#39619637009](https://github.com/netzen-abm/sidereth/actions/runs/39619637009), Security/Supply Chain [#39619637116](https://github.com/netzen-abm/sidereth/actions/runs/39619637116), and CodeQL [#39619627604](https://github.com/netzen-abm/sidereth/actions/runs/39619627604) all passed. Foundation included format, compile, test suite, live PostgreSQL proof matrix, and Clippy; Security passed RustSec and GitHub Actions Security.
- Current working change on the existing approved architecture branch adds a trust-state gate to `AuthorizedEvidenceQuery`. It is intended to close the follow-up finding that the protected read path verified authorization and content hashes without requiring the canonical `PersistedEvidence` aggregate. Treat the implementation as **pending exact-head CI** until a PR is open and its final-head checks pass.
- The live branch endpoint reports exactly nine branches. Do not create a tenth branch; reuse the approved branch or deliberately disposition an existing branch first. Post-merge workflows for current main `73b17427b0c5fdc6bac9e01a612e0f75b3196a9c` still need to be checked independently.

### Nine active branches — verified after PR #172

1. `main`
2. `architecture/durable-tool-gateway-idempotency-v0-1-2026-09-19`
3. `architecture/repo-organization-privacy-runtime-licensing-2026-09-17`
4. `architecture/reticulum-inspired-cost-delay-sovereignty-research-2026-09-17`
5. `governance/ruleset-baseline-2026-09-16`
6. `docs/documentation-governance-2026-09-11`
7. `docs/semantic-audit-domain-model-2026-09-11`
8. `sidereth-ecosystem-architecture`
9. `feat/knowledge-graph-next`

The enforcement workflow keeps an explicit keep/retire policy, checks open PRs before deletion, rejects unexpected branches, and verifies the final count.

### Dependency, cost, and governance policy

- `.github/dependabot.yml` names Cargo and GitHub Actions but sets `open-pull-requests-limit: 0` for both; routine version-update PRs are disabled to preserve the nine-branch invariant. Dependencies must be changed through an approved branch and required CI.
- **Separate GitHub security settings remain unverified through the available connector.** Dependabot security-update PRs and vulnerability-alert settings are distinct from routine update limits; the owner must inspect [Settings → Code security and analysis](https://github.com/netzen-abm/sidereth/settings/security_analysis). Never delete a security update without evaluating the vulnerability and retaining its fix.
- Active dependencies are Rust `serde`, `serde_json`, `sha2`, and optional `postgres`. The repository source/config scan did not identify a paid external AI/provider integration or paid deployment/billing configuration; this is not an external-account billing audit.
- CI uses GitHub-hosted `ubuntu-latest` and PostgreSQL 16 service tests. Metered services must remain opt-in and have an owner, data boundary, budget/limit, and shutoff procedure.
- The `SIDERETH Main Protection` ruleset requires a pull request and `Validate Foundation`, `RustSec Audit`, and `GitHub Actions Security`, and prohibits force-push and branch deletion. The previously observed gap remains: zero approving reviews and no resolved-conversation requirement. The connector can read but not administer this setting; keep issue #100 open until the owner changes and verifies it.

### Verified implementation and current work

- PR #161 binds each persisted transformation to one provenance record and validates source, operation, and actor consistency.
- PR #168 moves Tool Gateway authorization and trusted time into the gateway boundary. Complete gateway contract/conformance and lifecycle/idempotency proof remain open.
- PR #170 validates Evidence aggregates/resource IDs/schema at construction and again immediately before PostgreSQL SQL, rejects generic Evidence upserts, and proves mutation rejection with unit and live PostgreSQL tests.
- PR #172 moves Knowledge Graph and Observation Command/Lifecycle behind canonical evaluator and trusted-time boundaries. The graph binds exact `graph.read`/`graph.write` actions and each target; unauthorized traversal targets are filtered. Edge mutations validate canonical fields before state change and enforce source-node existence (Strong links also require an existing target). Observation lifecycle evaluates authorization for both the new and prior Observation before opening the Unit of Work. Exact-head Foundation, Security/Supply Chain, and CodeQL passed.
- The current follow-up gates `AuthorizedEvidenceQuery` on the existing `EvidenceTrustRepository` contract. Before returning bytes, it must validate the trust aggregate and prove that its embedded `EvidenceOriginal` exactly matches the stored original metadata, then verify the bytes' hash. Regressions cover missing trust state, legacy transformations without provenance, original-metadata mismatch, and tampered content. **These newest changes are pending final-head CI and have not yet merged.**
- `docs/contracts/LEGACY-EVIDENCE-QUARANTINE-POLICY.md` proposes preservation-first quarantine and recovery only from independent evidence. It is a policy draft, not an implemented migration job. No production migration or quarantine run has occurred.

### Remaining security and portability gates

1. **Issue #159 — legacy evidence disposition:** inventory historical records; verify backups and original hashes; produce and review a dry-run classification report; implement/test quarantine across every normal read/export/RAG/graph route; prove idempotent recovery/rollback; and record an explicit policy decision. Missing attribution must never be silently reconstructed. Keep the issue open.
2. **Current trust-state read gate — exact-head proof:** require Foundation Validation (format, compile, tests, live PostgreSQL, Clippy), Security/Supply Chain (RustSec and zizmor), and relevant CodeQL checks before merging the pending follow-up.
3. **Remaining authorization consumers:** audit other protected operations and AI/tool paths for evaluator origin, exact action/resource/purpose/data-class binding, current time, expiry, and lease enforcement.
4. **Storage boundary:** keep raw `EvidenceObjectStore` and original-metadata repositories inside trusted adapters. Provider interfaces are authorization-neutral and must not be handed to untrusted surfaces or agents.
5. **Persistence conformance:** finish ResourceLink endpoint/atomicity, duplicate/class-conflict, backward-compatibility, and provider-neutral adapter conformance.
6. **Portability:** verify trusted time on every intended runtime, especially WASM.
7. **Repository governance:** address the approving-review/review-thread-resolution gap and verify security-update settings without weakening CI or violating the exact-nine-branch invariant.

---

**Historical note:** The prior audits below are retained as historical records. The verified baseline above is authoritative for `main` at `712b13fcf624a6d14b531bcc2d01d03f2f3bde31`.

## Historical post-merge security audit — 2026-10-15

**Verified main baseline:** `bcaeda9329990982cade7d29bb726790e0a5fefc`  
**Active branches:** exactly 9 observed after merge.  
**Current engineering posture:** close operation-specific authorization gaps, then continue the protected-data bypass audit. No new orchestration abstraction is justified by this finding.

### Verified security finding and fix

- PR #163, `fix: bind evidence retrieval to read action`, was squash-merged to `main` as `bcaeda9329990982cade7d29bb726790e0a5fefc`.
- Finding: `AuthorizedEvidenceQuery` validated that an authorization result matched the supplied request but did not bind the request action to the read operation. A matching Allow result for a different action could pass the context-consistency check.
- Fix: the retrieval boundary now requires `ResourceType::Action` with action ID `evidence.read`.
- Regression test: a matching request and Allow result for `evidence.delete` must be rejected.
- Exact-head Foundation Validation passed: format, compile, test suite, live PostgreSQL proof matrix, and Clippy.
- Exact-head Security and Supply Chain passed; RustSec Audit and GitHub Actions Security passed. OpenSSF Scorecard was skipped and is not counted as passing.
- Post-merge CI for `main` is queued/in progress at the time of this checklist update and must be confirmed independently.

### Next proof obligations

1. Add negative tests for operation-specific authorization binding across every protected-data consumer. A consumer must require its own action; request/result equality alone is insufficient.
2. Audit each protected read/write/export/transform path for action binding, exact resource binding, purpose and data-class requirements, freshness/expiry, and capability lease requirements where relevant.
3. Inventory historical evidence transformations without trustworthy provenance; define fail-closed migration/quarantine, with no fabricated history.
4. Test direct-store access paths, tampered content, storage failures, rollback, and metadata/content mismatch.
5. Retire stale branches only after unique commits, PRs, protections, and workflow/deployment references are verified. Keep exactly nine active branches.

---

**Historical note:** this section records the baseline immediately after PR #163 and before PRs #165–#166. The current verified baseline above supersedes it.

## Historical repository and engineering-cycle audit — 2026-10-15

**Verified main baseline:** `230e97a504a07cbb60dfbff9c089a166b39ba4ef`  
**Active branch count:** exactly 9 observed. No branches were created or deleted in this cycle.  
**Current implementation priority:** prove provenance integrity, authorization, protected-data access, lease lifecycle, persistence semantics, and independent-consumer conformance. Avoid new abstraction layers unless a demonstrated invariant or independent boundary requires them.

### Verified changes and CI evidence

- PR #161, `fix: enforce evidence transformation provenance binding`, was squash-merged to `main` as `230e97a504a07cbb60dfbff9c089a166b39ba4ef`.
- Latest PR head: `c625baaf7f7041a8b6402cee9cc6f8a3f6c6b1e6`.
- Foundation Validation passed on the exact final PR head, including format check, compile check, test suite, live PostgreSQL proof matrix, and Clippy.
- Security and Supply Chain passed on the exact final PR head; RustSec Audit and GitHub Actions Security passed.
- OpenSSF Scorecard was skipped by the workflow and is not counted as a pass.
- The persisted evidence model now binds each transformation ID to exactly one provenance record. Validation checks source-evidence binding, input reference, operation, actor identity consistency, and missing/duplicate/orphaned bindings before persistence.
- PostgreSQL resource-type serialization/deserialization now maps `Tool` and `Service` explicitly.
- Backward-compatibility risk remains: legacy records containing transformations without verifiable provenance bindings fail closed. Historical-record inventory and a trustworthy migration/quarantine plan remain open.

### Current branch governance

The live branch listing contains exactly nine branches. No branch was deleted in this cycle. Do not remove stale branches merely because they are behind: verify branch protection, open PRs, workflow/deployment references, and unique material first. Preserve the exact-nine invariant when an authorized branch-retirement mechanism is available.

### Next engineering cycle — evidence, not abstractions

1. Inventory legacy persisted transformations and design a fail-closed migration/quarantine path that never fabricates provenance.
2. Complete the line-referenced protected-data and consequential-execution bypass audit, including direct evidence-store access and every authorization/lease consumer.
3. Close ResourceLink endpoint/atomicity, duplicate/class-conflict, PostgreSQL, backward-compatibility, and provider-neutral conformance tests.
4. Continue durable idempotency and execution-lease lifecycle failure testing; investigate skipped security jobs rather than counting them as passes.
5. Retire stale branches only after the required governance and archival checks; keep exactly nine active branches.

---

**Historical snapshot:** the 2026-10-11 audit below is retained as history. Its branch comparisons and PR statuses are not current and are superseded by the 2026-10-15 baseline above.

## Historical repository and engineering-cycle audit — 2026-10-11

**Verified main baseline:** `25b09849b925b388f51389a5f408dae4302ab4b0`  
**Active branch count:** exactly 9 observed. No branch was created or deleted during this audit.  
**Current implementation priority:** prove authorization, protected-data access, lease lifecycle, persistence semantics, and independent-consumer conformance; do not add new abstraction layers unless a demonstrated invariant or independent boundary requires them.

### Branch disposition evidence

| Branch | Current comparison with `main` | Disposition |
|---|---:|---|
| `main` | baseline | Protected integration target |
| `architecture/durable-tool-gateway-idempotency-v0-1-2026-09-19` | 9 ahead / 2 behind | PR #156 merged as `25b09849b925b388f51389a5f408dae4302ab4b0`; branch remains active and should be retired only when branch governance can preserve the exact-nine count |
| `feat/knowledge-graph-next` | 18 ahead / 11 behind | PR #157 open; current diff is limited to graph authorization implementation, exports, and conformance tests. Requires current-head CI and merge-conflict/base revalidation |
| `architecture/repo-organization-privacy-runtime-licensing-2026-09-17` | 0 ahead / 38 behind | Stale retirement candidate; no unique commits ahead, but verify branch protections and references before retirement |
| `architecture/reticulum-inspired-cost-delay-sovereignty-research-2026-09-17` | 0 ahead / 33 behind | Stale retirement candidate; no unique commits ahead, but verify branch protections and references before retirement |
| `docs/documentation-governance-2026-09-11` | 0 ahead / 236 behind | Stale retirement candidate; no unique commits ahead, but verify branch protections and references before retirement |
| `docs/semantic-audit-domain-model-2026-09-11` | 0 ahead / 232 behind | Stale retirement candidate; no unique commits ahead, but verify branch protections and references before retirement |
| `governance/ruleset-baseline-2026-09-16` | 0 ahead / 46 behind | Stale retirement candidate; no unique commits ahead, but verify branch protections and references before retirement |
| `sidereth-ecosystem-architecture` | 0 ahead / 543 behind | Stale retirement candidate; no unique commits ahead, but verify branch protections and references before retirement |

**Branch-control rule:** exactly nine active branches were observed after PR #156 merged. Do not delete a branch until the approved disposition, branch protections, open PRs, workflow references, and unique material have been verified. Since the connected GitHub actions do not expose a branch-deletion operation, no deletion is claimed. Keep the exact-nine invariant when the repository's authorized branch-retirement mechanism is available.

### Verified architecture and open proof gates

- Canonical authorization enforcement exists in `src/authorization_enforcement.rs`; it binds request/result context, enforces freshness and expiry, requires `Allow`, and fails closed on unknown or conflicting constraints.
- Protected evidence retrieval is routed through `AuthorizedEvidenceQuery` in `src/evidence_query.rs`, which validates authorization before metadata/object reads and verifies the content hash.
- `src/tool_gateway.rs` composes canonical authorization, registry/provider binding, capability leases, execution-gate checks, idempotency, invocation-bound data access and audit/provenance. Production readiness is **not** established by module presence alone.
- `src/execution_lease.rs` keeps provider-handle ownership in the runtime and lifecycle evidence at the separate persistence boundary. The current main commit adds a fail-closed restart test for orphaned persisted Active leases.
- `ResourceLinkClass` exists in `src/persistence/resource.rs`; legacy links decode without being inferred as Strong. The remaining gate is end-to-end proof of Strong-link endpoint validation/atomicity, duplicate and class-conflict behavior, PostgreSQL behavior, backward compatibility and provider-neutral conformance.
- The `AuthorizedAudit` helper in `src/security.rs` checks only that the supplied result is `Allow`; it does not itself validate request/result contextual binding or expiry. No production call site was found by the current code search. Keep it non-authoritative and ensure any future protected operation enters through the canonical validation boundary.
- PR #156 was verified at head `d070520ce96f9a52eb1473b5a439ef0e69321ea0`; Foundation Validation and Security/RustSec passed. It was squash-merged to `main` as `25b09849b925b388f51389a5f408dae4302ab4b0`.

### Next engineering cycle — evidence, not abstractions

1. Verify PR #156's exact head, mergeability, required checks and current-head CI; integrate the latest main fail-closed restart change without losing unique work.
2. Complete a line-referenced protected-data/consequential-execution bypass audit, including direct evidence-store access and every authorization/lease consumer.
3. Close ResourceLink wire/PostgreSQL/conformance tests and durable idempotency/lifecycle failure tests.
4. Run the independent-consumer conformance suite; only then decide whether any shared component is extraction-ready.


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
- [x] execution lease runtime boundary — provider-neutral activation/release/revocation/cancellation/expiry/reauthorization foundation
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
- [x] Execution Lease Runtime boundary
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
- [x] runtime lifecycle audit integration
- [x] provider-neutral audit/provenance persistence adapter through UnitOfWork
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

## Execution Lease Runtime Gate — current
- [x] Provider-neutral runtime boundary
- [x] Exact-context activation through canonical CapabilityLease validation
- [x] Provider/OS adapter isolation
- [x] Release after normal completion
- [x] Fail-closed expiry
- [x] Revocation and cancellation terminate future use before adapter release
- [x] Fresh authorized lease required for reauthorization
- [x] Explicit synchronization requirement for concurrent callers
- [x] Adapter failure / partial-release evidence matrix defined
- [x] Runtime audit/provenance integration
- [ ] Durable runtime lease persistence
- [ ] Production readiness decision


## Follow-up execution update — 2026-10-11

### PR #157 — canonical graph authorization

- Latest reviewed head: `b4353417bd668930f0b65738f39a1be69930dce2`.
- PR diff is now focused on `src/knowledge_graph.rs`, `src/lib.rs`, and `tests/knowledge_graph_conformance.rs`. The unused evidence-provenance validator was removed from this PR rather than shipping a validator with no authoritative persistence caller.
- Initial Foundation Validation failed at `cargo fmt --all -- --check`; the reported formatting differences were corrected in the latest file versions.
- Security and Supply Chain passed on the earlier reviewed head; Foundation Validation failed before compile/tests because formatting stopped the job.
- **Latest-head CI is still unverified.** The available workflow query returned no run for the latest content-API commit. Do not infer success from the earlier run, and do not merge until a normal PR-head workflow run passes.
- GitHub currently reports PR #157 as open; mergeability is transient and not a substitute for required checks.

### Immediate acceptance gate

1. Trigger CI on the latest PR head through a normal Git push/PR synchronization or repository workflow mechanism.
2. Require formatting, compile, unit/conformance tests, Clippy, security/supply-chain checks and relevant PostgreSQL proof gates to pass; investigate skipped jobs rather than counting them as passes.
3. Keep the PR unmerged until the current head is verified. Keep the graph storage/traversal cohesive; do not add a separate graph authorization service.
4. Continue with protected evidence access and provider/lease consumers only after the graph gate is resolved.



## Follow-up execution update — 2026-10-11 (after PR #156 merge)

- PR #156 squash-merged successfully as `25b09849b925b388f51389a5f408dae4302ab4b0` after Foundation Validation and Security/RustSec passed at its exact head.
- Exactly nine active branches remain. The merged source branch still exists; no branch deletion was performed.
- PR #157 was hardened to bind authorization to action as well as exact resource: `graph.read` for reads/traversal and `graph.write` for mutations. A regression test verifies read authorization cannot mutate the graph.
- PR #157 was merged as `236f1bd0bff7bcd44f9fb0a37642d61eab139c68` after Foundation Validation and Security/Supply Chain passed on exact head `84fa6f2f17649b8946321b75fcc52cfcbc84cf3f`.
- The P1 graph traversal finding was resolved after verifying unauthorized traversal targets are filtered in `src/knowledge_graph.rs` and CI passed. Evidence provenance findings were explicitly scoped out and tracked in issue #159; they remain open security work, not resolved controls.
- Next: execute the evidence-provenance persistence audit under issue #159, then continue line-referenced protected-data and execution-lease consumer review. Keep branch hygiene archive-first and do not create abstractions without a demonstrated independent boundary.



## Follow-up execution update — 2026-10-11 (graph authorization merged)

- PR #157 merged to `main` as `236f1bd0bff7bcd44f9fb0a37642d61eab139c68`.
- Exact reviewed head: `84fa6f2f17649b8946321b75fcc52cfcbc84cf3f`. Foundation Validation and Security/Supply Chain workflows passed on this head. OpenSSF Scorecard was skipped by workflow and is not counted as a pass.
- Graph reads/traversal require `graph.read`; mutations require `graph.write`; exact-resource binding and dual endpoint authorization for edge creation remain in the cohesive graph boundary. Unauthorized traversal targets are filtered.
- Exactly nine active branches were verified after merge.
- Open issue #159 tracks the separate evidence trust/persistence findings: non-Party provenance actors and enforcing provenance binding before authoritative persistence. These are open and must not be marked done until integration tests prove them.
- Next two steps: (1) trace `PersistedEvidence` through all mutation/persistence paths and bind provenance validation before commit; (2) run a full authorization/direct-protected-data access search with file/line/test evidence and refresh this checklist from verified results.

