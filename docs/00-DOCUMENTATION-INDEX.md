# SIDERETH Documentation Index

**Status:** CANONICAL / DOCUMENTATION INDEX
**Purpose:** Single navigation and authority map for humans, developers and AI agents.

## 1. Read this first

The documentation system follows one rule:

> **One concept has one canonical authority. Supporting documents explain, test, plan or preserve history; they do not silently redefine the canonical concept.**

Before creating or modifying documentation, read:

1. this index;
2. `00-SIDERETH-MASTER-DECISIONS.md`;
3. `DECISION-REGISTER.md` when the change is architectural/product-level;
4. the relevant canonical contract before changing implementation;
5. implementation/tests before making implementation-status claims.

For the detailed documentation governance protocol, see `SIDERETH-DOCUMENTATION-GOVERNANCE.md`.

## 2. Authority hierarchy

1. **Locked decisions** — `00-SIDERETH-MASTER-DECISIONS.md`, `DECISION-REGISTER.md`.
2. **Canonical contracts** — `contracts/`.
3. **Canonical architecture** — master blueprint and architecture documents.
4. **Canonical domain/framework specifications** — capability model, data model, glossary and equivalent specifications.
5. **Conformance/evidence** — proves implementation behavior; does not redefine the contract.
6. **Planning** — roadmap, estimates and UX; not implementation proof.
7. **Migration/audit records** — historical or process evidence.
8. **Archive** — reference only; never an active authority.

A lower-level document cannot override a higher-level contract merely because it was edited later.

## 3. Governance and decisions

- `00-SIDERETH-MASTER-DECISIONS.md` — locked product and architecture decisions.
- `DECISION-REGISTER.md` — numbered decision history, consequences and review triggers.
- `DECISIONS/` — detailed decision records requiring context, alternatives and rationale.
- `SIDERETH-DOCUMENTATION-GOVERNANCE.md` — documentation authority, lifecycle, duplication and AI/developer reading rules.
- `SIDERETH-GLOSSARY.md` — canonical terminology.
- `DECISIONS/PLATFORM-RUNTIME-PRIVACY-LICENSING-2026-09-17.md` — canonical decision for polyglot runtime, privacy/safety defaults, optional hardware and layered licensing.

Do not create another general-purpose decision list.

## 4. Product and architecture

- `01-SIDERETH-MASTER-BLUEPRINT.md` — master product/system blueprint.
- `SIDERETH-ARCHITECTURE.md` — target system architecture, universal infrastructure and delivery boundaries.
- `SIDERETH-ECOSYSTEM-ARCHITECTURE.md` — ecosystem composition, shared capabilities, domain packs, surfaces, provider neutrality and Trust Kernel.
- `SIDERETH-CAPABILITY-MODEL.md` — capability/function/tool/resource/workflow abstraction.
- `SIDERETH-ECOSYSTEM-ROADMAP.md` — strategic capability-led roadmap.
- `ROADMAP.md` — delivery roadmap and execution sequencing; must remain consistent with the ecosystem roadmap.
- `SIDERETH-ECOSYSTEM-DISCUSSION-RECORD.md` — consolidated discussion context; not a contract.
- `brand/THE-PURPLE-FROG-BRAND-IDENTITY.md` — public brand/conservation identity.
- `08-MCP-ARCHITECTURE.md` — MCP interoperability boundary.
- `ESTIMATE-WIREFRAME-PLAN.md` — planning/UX estimate only.

### Architecture scope distinction

`SIDERETH-ARCHITECTURE.md`, `SIDERETH-ECOSYSTEM-ARCHITECTURE.md`, and focused architecture specifications under `docs/architecture/` are intentionally complementary:

- system architecture defines the target system boundary, lifecycle and universal infrastructure;
- ecosystem architecture defines composition, reusable capability semantics, domain packs, adapters, provider neutrality and Trust Kernel rules;
- focused architecture specifications define a bounded cross-cutting architectural concern without creating a competing master architecture.

Current focused architecture specifications include:

- `architecture/LONGITUDINAL-PROJECTION.md` — canonical architecture for the derived longitudinal read/projection layer; explicitly not a `LongitudinalRecord` domain aggregate.
- `architecture/READ-QUERY-BOUNDARY.md` — canonical architecture for provider-neutral direct resource discovery/query, distinct from mutation and derived projection.
- `architecture/PLATFORM-RUNTIME-AND-HARDWARE-STRATEGY.md` — canonical language/runtime, interoperability and hardware-independence strategy.
- `architecture/PRIVACY-AND-SAFETY-BY-DESIGN.md` — canonical privacy, safety, sensitive-capability and secure-default principles.

Do not merge focused architecture documents merely to reduce file count unless a future audit proves their scopes can be consolidated without loss of authority clarity.

## 5. Implementation control

- `SIDERETH-MASTER-CHECKLIST.md` — master implementation gates and completion state.
- `contracts/` — implementation-facing contracts and conformance matrices.
- `migration/` — migration and repository audit records.
- `archive/` — historical material excluded from active implementation authority.

## 6. Contracts

`docs/contracts/` is the authoritative home for implementation-facing contracts and their conformance matrices.

Key contract families include:

- canonical domain model;
- Case/Incident/Event;
- Observation and Observation Conformance;
- Party, Document, Evidence and Evidence Trust;
- capability and Capability Registry;
- Tool Registry;
- authorization/policy;
- Action/Approval/Execution Gate;
- intelligence;
- persistence/storage;
- API/error/idempotency/versioning;
- Tool Gateway.

A contract defines a boundary. Code must conform to it. A convenient implementation must not redefine the boundary.

## 7. Legal, licensing and brand

- `legal/LICENSING-ARCHITECTURE.md` — layered Apache-2.0 / AGPL / proprietary-commercial licensing strategy.
- `legal/COMMERCIAL-EULA-FRAMEWORK.md` — non-final EULA structure requiring legal review.
- `brand/THE-PURPLE-FROG-BRAND-IDENTITY.md` — brand and combined Gaur + Purple Frog emblem governance.
- `conservation/THE-PURPLE-FROG-CONSERVATION-AWARENESS.md` — conservation-awareness principles and sourced-claim rules.

Software licensing does not grant trademark or brand-use rights.

## 8. Current Tool Gateway status

The Tool Gateway contract is canonical in:

- `contracts/TOOL-GATEWAY-CONTRACT.md`
- `contracts/TOOL-GATEWAY-CONFORMANCE.md`

The implementation must consume the canonical authorization/policy boundary and is **not production-ready** until the bounded readiness/conformance evidence is complete. See the current implementation issue/PR state rather than relying on historical PR numbers in this index.

## 9. Migration and archive

- `migration/` — active repository cleanup, migration and audit plans/results.
- `archive/` — historical/superseded material; never use it as current implementation authority.

The former `docs/ARCHITECTURE-BLUEPRINT.md` is preserved under `docs/archive/superseded/`.

## 10. Documentation status vocabulary

### Specification status

- **LOCKED** — governing decision.
- **CANONICAL** — authoritative specification for its scope.
- **DRAFT** — proposed design awaiting decision/verification.
- **PLANNING** — estimate, UX or roadmap material.
- **REFERENCE** — supporting material, not authoritative.
- **ARCHIVED** — historical reference only.

### Implementation evidence status

```text
VISION → DESIGNED → IMPLEMENTED → FUNCTIONAL → TESTED
        → SECURITY-VERIFIED → PRIVACY-VERIFIED → PRODUCTION-READY
```

Never infer an implementation status from prose alone.

## 11. Change rules

Before creating a new specification:

1. search this index and active docs;
2. identify the existing canonical authority;
3. extend it if the subject belongs to the same scope;
4. create a new document only for a genuinely distinct scope;
5. update this index and all cross-references in the same change.

Before moving or deleting a document:

1. read the complete document;
2. identify unique content and references;
3. merge unique content into the surviving authority where appropriate;
4. create/update the destination;
5. update links;
6. verify CI and documentation consistency;
7. archive first;
8. delete only after evidence.

## 12. Repository structure

The active repository organization now distinguishes implementation, assets, documentation domains and historical material:

```text
docs/
├── 00-DOCUMENTATION-INDEX.md
├── 00-SIDERETH-MASTER-DECISIONS.md
├── 01-SIDERETH-MASTER-BLUEPRINT.md
├── SIDERETH-DOCUMENTATION-GOVERNANCE.md
├── SIDERETH-GLOSSARY.md
├── DECISION-REGISTER.md
├── DECISIONS/
├── architecture/
├── brand/
├── conservation/
├── contracts/
├── legal/
├── migration/
├── evidence/
├── audits/
└── archive/
```

This taxonomy is semantic rather than cosmetic: a document should move only when its scope and reference impact are understood.

## 13. Non-negotiable documentation rule

Documentation must never imply implementation that is not supported by source code, automated tests, deployment evidence or other reproducible evidence.

If a human, developer or AI agent cannot determine the authoritative document for a major decision without tribal knowledge, the documentation system is not complete.
