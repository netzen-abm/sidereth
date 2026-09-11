# SIDERETH Documentation Cleanup Plan V2

**Status:** ACTIVE MIGRATION PLAN
**Scope:** Documentation deduplication, authority normalization and safe repository organization

## Objective

Maintain one authoritative documentation system for SIDERETH before and during production implementation.

## Cleanup principles

1. One concept, one canonical specification.
2. Preserve historical material before removal or replacement.
3. Separate decisions from architecture, contracts, plans, implementation evidence and history.
4. Never treat estimates, diagrams or prose as proof of implementation.
5. Every active document must identify status and scope.
6. Domain-specific documents must not redefine universal contracts.
7. New adapters must reference shared contracts rather than copy them.
8. Do not perform cosmetic mass moves when link/reference impact has not been audited.
9. Archive first; delete only after evidence.

## Completed findings

### Architecture blueprint

The former `docs/ARCHITECTURE-BLUEPRINT.md` is already preserved under:

`docs/archive/superseded/ARCHITECTURE-BLUEPRINT.md`

It is no longer an active specification path. No additional deletion is required for that file.

### Documentation index

The index now explicitly defines:

- authority hierarchy;
- status vocabulary;
- canonical architecture scope distinctions;
- Tool Gateway status;
- AI/developer reading rules;
- archive-first movement rules.

See `docs/00-DOCUMENTATION-INDEX.md` and `docs/SIDERETH-DOCUMENTATION-GOVERNANCE.md`.

## Audited active-document relationships

### Master Decisions vs Decision Register

Keep both. They serve different purposes:

- Master Decisions = concise locked baseline.
- Decision Register = numbered history, consequences and review protocol.

Do not create a third general decision specification.

### Architecture vs Ecosystem Architecture

Keep both for now.

- `SIDERETH-ARCHITECTURE.md` = target system architecture, universal infrastructure, lifecycle and delivery order.
- `SIDERETH-ECOSYSTEM-ARCHITECTURE.md` = ecosystem composition, capability abstraction, domain packs, surface independence, provider neutrality and Trust Kernel.

They are complementary, not duplicate enough to justify a risky move at this stage.

### Ecosystem roadmap vs delivery roadmap

Keep both, but prevent divergence:

- `SIDERETH-ECOSYSTEM-ROADMAP.md` = strategic/capability-led roadmap.
- `ROADMAP.md` = phased delivery sequencing.

Any future overlap should be consolidated rather than copied.

### Tool Gateway

Keep the contract and conformance matrix as the canonical specification in `docs/contracts/`. Implementation evidence belongs in code, tests, CI and the conformance evidence process.

## Documents requiring later targeted review

The repository contains a large contract surface. Before renaming or physically moving contract documents, audit them as families rather than individually by filename.

Priority families:

1. authorization / policy;
2. action / approval / execution;
3. capability / capability registry;
4. tool registry / tool gateway;
5. evidence / provenance / persistence;
6. API / error / idempotency / versioning;
7. intelligence;
8. domain model / Case / Incident / Party / Document.

The objective is not to reduce file count artificially. The objective is to ensure each contract has one authority and each conformance document clearly points to it.

## Future documentation structure

The repository may eventually use:

```text
docs/
├── governance/
├── architecture/
├── contracts/
├── evidence/
├── planning/
├── migration/
└── archive/
```

However, physical migration is deferred until a link/reference inventory and CI validation strategy exist. The current normalization through `00-DOCUMENTATION-INDEX.md` and `SIDERETH-DOCUMENTATION-GOVERNANCE.md` is safer and sufficient for this bounded phase.

## Next cleanup phase

1. Inventory every active Markdown file and classify it as decision, architecture, contract, conformance/evidence, planning, migration, reference or archive.
2. Detect semantic duplicates using headings/content, not filenames alone.
3. For each duplicate, identify the surviving authority and migrate unique content before archiving/removing the superseded file.
4. Audit all internal Markdown links after any rename/move.
5. Add a lightweight documentation-consistency CI check when the final taxonomy is stable.

## Completion rule

The repository is documentation-clean only when an engineer or AI agent can determine from the index which document is authoritative for every major architectural or domain decision without relying on tribal knowledge.
