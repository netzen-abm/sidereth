# SIDERETH — Markdown Documentation Audit

**Audit date:** 2026-09-11  
**Baseline:** `main` at `ab36d34a90a9363044e4700576523a4a87259534`  
**Purpose:** Evidence record for documentation normalization before physical moves/renames

## Result

The repository has a substantial and generally coherent documentation system. The principal risk is not simply duplicate filenames; it is **semantic overlap without an explicit authority relationship**.

The audit therefore recommends logical normalization first and physical reorganization second.

## Canonical families identified

### Governance / decisions

- Master Decisions — locked concise baseline.
- Decision Register — numbered decisions, consequences and review protocol.
- Decision records — detailed rationale for individual material decisions.
- Documentation Governance — rules for authority, lifecycle, duplication and maintenance.

**Disposition:** retain as complementary authorities. Do not create another general decision list.

### Architecture

- Master Blueprint — master product/system blueprint.
- System Architecture — target system architecture and universal infrastructure boundary.
- Ecosystem Architecture — ecosystem composition, capabilities, domain packs, adapters, provider neutrality and Trust Kernel.
- Capability Model — capability/function/tool/resource/workflow abstraction.

**Disposition:** retain. Their scopes overlap conceptually but are not interchangeable.

### Roadmaps

- Ecosystem Roadmap — strategic/capability-led direction.
- Roadmap — delivery sequencing.

**Disposition:** retain with explicit distinction; future divergence must be prevented.

### Contracts

`docs/contracts/` is the canonical implementation-facing contract namespace. Contract and conformance files should be maintained as families: contract = normative boundary; conformance = evidence requirements/status.

Priority families for deeper semantic audit:

1. authorization/policy;
2. action/approval/execution;
3. capability/registry;
4. tool registry/gateway;
5. evidence/provenance/persistence;
6. API/error/idempotency/versioning;
7. intelligence;
8. domain model/Case/Incident/Party/Document.

### Future research

`docs/DECISIONS/FUTURE-EVIDENCE-VISION.md` is correctly scoped as future research rather than an implementation commitment. It already contains the Mojo and hardware-backed evidence directions, so additional standalone documents for those topics should not be created without a distinct canonical scope.

## Duplicate-risk observations

1. Similar subject matter does not automatically mean duplication.
2. Filename similarity is insufficient for consolidation decisions.
3. Planning documents must not repeat normative contract language unless explicitly marked explanatory.
4. Conformance matrices must not become alternative contracts.
5. README should remain an entry point, not a second architecture specification.
6. Historical/archive material must never be cited as current authority.

## Physical organization decision

**No mass physical move is authorized by this audit.**

Before any move/rename:

- enumerate all Markdown files;
- map inbound and outbound links;
- compare complete document contents;
- identify unique sections;
- migrate unique content to the surviving authority;
- archive the superseded source;
- update links and index;
- run documentation consistency validation.

## Current known historical disposition

`docs/ARCHITECTURE-BLUEPRINT.md` is already preserved under `docs/archive/superseded/ARCHITECTURE-BLUEPRINT.md` and is not an active specification.

## Tool Gateway evidence boundary

The Tool Gateway Contract and Conformance Matrix remain the canonical specification. The implementation PR is evidence against them. The conformance matrix explicitly requires evidence for stale authorization, returned constraints, provider selection, adapter bypass, data minimisation, idempotency, audit/provenance and registry failure boundaries.

At this audit baseline these requirements remain `DESIGNED`, not automatically `TESTED`, because the matrix itself states that contract prose is not executable evidence.

## Conclusion

The correct next repository action is a **complete machine-assisted Markdown inventory and link audit**, followed by targeted consolidation. The repository should not optimize for fewer files; it should optimize for unambiguous authority, traceability and maintainability.
