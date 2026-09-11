# SIDERETH — Documentation Governance and Authority Map

**Status:** CANONICAL / DOCUMENTATION GOVERNANCE
**Scope:** Human, developer, AI-agent and documentation-maintenance behavior across the SIDERETH repository
**Baseline audited:** `main` at `ab36d34a90a9363044e4700576523a4a87259534`

## 1. Purpose

This document defines how SIDERETH documentation is organized, interpreted, updated and verified.

It exists to prevent:

- duplicate specifications;
- conflicting architectural statements;
- stale implementation claims;
- AI-agent drift caused by reading an obsolete document as authoritative;
- developer implementation based on planning material rather than contracts;
- historical material being mistaken for current architecture;
- unnecessary document proliferation when an existing canonical document should be extended.

The repository is an ecosystem project. Documentation must therefore behave like a structured system of contracts and evidence rather than a collection of independent notes.

## 2. Governing documentation rule

> **One concept has one canonical authority. Supporting documents may explain, test, plan or record history, but they must not silently redefine the canonical concept.**

Before creating a new document:

1. search the documentation index;
2. search active documentation for the concept;
3. identify the canonical authority;
4. extend that authority if the new material belongs to the same scope;
5. create a separate document only when the scope is genuinely distinct;
6. update the index and cross-references in the same change.

## 3. Authority hierarchy

When documents appear to disagree, use this order:

1. **Locked decisions** — `docs/00-SIDERETH-MASTER-DECISIONS.md` and the Decision Register.
2. **Canonical contracts** — `docs/contracts/`.
3. **Canonical architecture** — master blueprint and architecture/ecosystem architecture documents.
4. **Canonical domain/framework documents** — data model, capability model, glossary and equivalent specifications.
5. **Conformance matrices and implementation evidence** — these prove what is implemented; they do not redefine the contract.
6. **Planning/roadmap/UX documents** — describe intended work only.
7. **Migration and audit records** — explain historical disposition and findings.
8. **Archive** — historical reference only; never an active implementation authority.

A lower-level document cannot override a higher-level contract merely because its wording is newer.

## 4. Lifecycle vocabulary

SIDERETH uses two related vocabularies.

### Decision/specification status

- **LOCKED** — governing decision; changes require an explicit decision update.
- **CANONICAL** — authoritative specification for the stated scope.
- **DRAFT** — proposed design awaiting decision or verification.
- **PLANNING** — intended work, estimate, UX or roadmap material.
- **REFERENCE** — supporting material that is not authoritative.
- **ARCHIVED** — historical material retained for traceability and excluded from active implementation authority.

### Implementation evidence status

```text
VISION
  -> DESIGNED
  -> IMPLEMENTED
  -> FUNCTIONAL
  -> TESTED
  -> SECURITY-VERIFIED
  -> PRIVACY-VERIFIED
  -> PRODUCTION-READY
```

Documentation must never advance an implementation status without reproducible evidence.

## 5. Current canonical document map

### Governance and decisions

- `docs/00-DOCUMENTATION-INDEX.md` — navigation and authority map.
- `docs/00-SIDERETH-MASTER-DECISIONS.md` — locked product and architecture decisions.
- `docs/DECISION-REGISTER.md` — numbered decision history and consequences.
- `docs/DECISIONS/` — detailed decision records where context, alternatives and rationale require more space.
- `docs/SIDERETH-GLOSSARY.md` — canonical terminology.
- `docs/SIDERETH-DOCUMENTATION-GOVERNANCE.md` — this document.

### Product and architecture

- `docs/01-SIDERETH-MASTER-BLUEPRINT.md` — master product/system blueprint.
- `docs/SIDERETH-ARCHITECTURE.md` — target system architecture and universal infrastructure boundaries.
- `docs/SIDERETH-ECOSYSTEM-ARCHITECTURE.md` — ecosystem-wide composition, shared capability model, surfaces and provider neutrality.
- `docs/SIDERETH-CAPABILITY-MODEL.md` — canonical capability/function/tool/resource/workflow model.
- `docs/SIDERETH-ECOSYSTEM-ROADMAP.md` — capability-led strategic roadmap.
- `docs/ROADMAP.md` — delivery roadmap; must remain consistent with the ecosystem roadmap.
- `docs/SIDERETH-ECOSYSTEM-DISCUSSION-RECORD.md` — consolidated discussion record; it is supporting context, not a contract.
- `docs/THE-PURPLE-FROG-BRAND-IDENTITY.md` — public brand and conservation identity.
- `docs/08-MCP-ARCHITECTURE.md` — MCP as an adapter/interoperability boundary, never as canonical authority.
- `docs/ESTIMATE-WIREFRAME-PLAN.md` — UX/estimate planning only.

### Contracts

`docs/contracts/` is the authoritative home for implementation-facing contracts and conformance matrices. Examples include:

- canonical domain model;
- capability contract and capability registry;
- tool registry;
- authorization/policy;
- action/approval/execution gate;
- intelligence;
- evidence/provenance;
- persistence/storage;
- API/error/idempotency/versioning;
- audit/storage/security;
- Tool Gateway contract and conformance.

A contract defines a boundary. Its implementation must conform to it; the implementation does not redefine the contract by convenience.

### Migration and archive

- `docs/migration/` — active migration, audit and cleanup records.
- `docs/archive/` — historical material that must not drive current implementation.

## 6. Audited duplication findings

The documentation audit found that the repository had already started a cleanup process and that several apparent duplicates have different intended scopes.

### Architecture documents

`SIDERETH-ARCHITECTURE.md` and `SIDERETH-ECOSYSTEM-ARCHITECTURE.md` are related but not identical:

- `SIDERETH-ARCHITECTURE.md` defines the target system boundary, universal infrastructure, lifecycle and delivery order.
- `SIDERETH-ECOSYSTEM-ARCHITECTURE.md` defines ecosystem composition, capability abstraction, domain packs, surface independence, provider neutrality and the Trust Kernel.

They should remain separate unless a future audit demonstrates that their scopes can be consolidated without losing authority clarity.

The historical `ARCHITECTURE-BLUEPRINT.md` has already been removed from the active path and preserved under `docs/archive/superseded/`. The migration cleanup record should be updated to reflect that this disposition has been completed.

### Decision documents

`00-SIDERETH-MASTER-DECISIONS.md` and `DECISION-REGISTER.md` are complementary:

- Master Decisions is the concise locked baseline.
- Decision Register provides numbered decision history and consequences.

Do not create a third general-purpose decision list.

### Roadmaps

`SIDERETH-ECOSYSTEM-ROADMAP.md` and `ROADMAP.md` must not evolve into competing roadmaps. The ecosystem roadmap is strategic/capability-led; the delivery roadmap is execution sequencing. If their scopes converge, consolidate rather than duplicate.

### Tool Gateway

The Tool Gateway contract and conformance matrix are canonical in `docs/contracts/`. The implementation PR is evidence against those documents and must not silently weaken them.

## 7. Tool Gateway documentation state as of this baseline

PR #73 implements the first provider-neutral gateway kernel, but it is not production-ready.

Verified at exact implementation head:

`7a028da2dd200b6cdf45950d69dff43b61ffadaf`

Current evidence:

- Foundation validation: PASS.
- Formatting: PASS.
- Compilation: PASS.
- Test suite: PASS.
- Live PostgreSQL proof matrix: PASS.
- Clippy: PASS.
- GitHub Actions security analysis: PASS.
- RustSec audit: PASS.

Remaining semantic work includes:

1. trusted/current-time authorization expiry enforcement;
2. returned authorization-constraint enforcement;
3. registry-driven implementation/provider selection;
4. richer invocation, implementation and provenance audit;
5. canonical durable/concurrent idempotency integration;
6. explicit evidence against direct adapter execution bypasses;
7. the TG-001–TG-060 conformance evidence matrix.

Green CI is necessary evidence but is not equivalent to Tool Gateway contract conformance or production readiness.

## 8. Canonical Tool Gateway execution boundary

```text
Caller / Surface / Agent / MCP Adapter
              |
              v
        Tool Registry
       discovery/version
              |
              v
   Canonical Authorization
              |
              v
        Tool Gateway
              |
       +------+------+
       |             |
   Scope/Data     Approval
       |             |
       +------+------+
              |
              v
        Execution Gate
              |
              v
     Tool Runtime/Adapter
```

The following are never authority sources:

- registry membership;
- tool metadata;
- caller booleans or flags;
- AI/model output;
- MCP metadata;
- provider identity;
- adapter implementation preference.

## 9. AI-agent reading protocol

Any AI agent working on SIDERETH must:

1. read `docs/00-DOCUMENTATION-INDEX.md`;
2. read `docs/00-SIDERETH-MASTER-DECISIONS.md`;
3. read `docs/DECISION-REGISTER.md` for relevant decisions;
4. read the relevant canonical contract before modifying implementation;
5. inspect implementation and tests before claiming status;
6. treat planning documents as non-evidence;
7. treat archive material as historical only;
8. search before creating a new document;
9. preserve existing useful content unless a verified conflict requires change;
10. record material architectural decisions explicitly;
11. never infer authorization, legal authority or implementation maturity from prose alone;
12. report uncertainty instead of inventing missing evidence.

## 10. Developer reading protocol

A developer should normally work in this order:

```text
Decision
  -> Architecture
  -> Contract
  -> Existing implementation
  -> Tests
  -> Change
  -> Exact-head CI
  -> Evidence update
```

For security-sensitive or consequential execution paths:

```text
Identity
  -> Authorization
  -> Scope/Data Controls
  -> Human Approval when required
  -> Execution Gate
  -> Runtime
  -> Audit/Provenance
```

No surface, provider, transport or agent may create a parallel path.

## 11. Human review protocol

A human reviewer should be able to answer:

- What is the canonical decision?
- What contract governs this code?
- What implementation is actually present?
- What tests prove the behavior?
- What security/privacy evidence exists?
- What remains unverified?
- Is the document claiming more than the repository proves?

If any answer requires tribal knowledge, the documentation system is incomplete.

## 12. Repository organization policy

The repository should converge toward these logical documentation domains:

```text
docs/
├── 00-DOCUMENTATION-INDEX.md
├── 00-SIDERETH-MASTER-DECISIONS.md
├── 01-SIDERETH-MASTER-BLUEPRINT.md
├── SIDERETH-DOCUMENTATION-GOVERNANCE.md
├── SIDERETH-GLOSSARY.md
├── DECISION-REGISTER.md
├── DECISIONS/
├── contracts/
├── architecture/        # future physical consolidation only when links are migrated safely
├── planning/            # future physical consolidation only when links are migrated safely
├── evidence/            # implementation/security/privacy evidence indexes
├── migration/
└── archive/
```

The current repository should not perform a mass physical move merely for cosmetic naming. Existing links, CI checks, cross-references and external references must be inventoried first.

The first organization step is therefore **logical normalization through the index and governance rules**. Physical moves are a separate bounded migration task with archive-first evidence and link verification.

## 13. Rename/move criteria

Rename or move a document only when at least one is true:

- its name materially misrepresents its scope;
- it is in the wrong authority class;
- it duplicates an existing canonical document and can be safely merged;
- it belongs to an archive/migration area but remains in the active path;
- its current location causes recurring implementation ambiguity.

Before moving/removing:

1. read the complete source document;
2. identify unique content;
3. identify references to it;
4. merge unique content into the surviving authority if required;
5. create the destination;
6. update all references and the documentation index;
7. verify CI and links;
8. archive before deletion;
9. delete only after evidence.

## 14. Documentation change checklist

- [ ] Existing docs searched.
- [ ] Canonical authority identified.
- [ ] Duplicate/superseded content assessed.
- [ ] Status and scope declared.
- [ ] Cross-references updated.
- [ ] Implementation claims backed by evidence.
- [ ] Archive disposition recorded where applicable.
- [ ] CI/documentation validation passed.
- [ ] Index updated.

## 15. Non-negotiable principle

> **Documentation is part of SIDERETH's architecture. A document is not authoritative because it exists; it is authoritative because its scope, status, relationship to the contracts and evidence chain are explicit.**
