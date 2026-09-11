# SIDERETH Documentation Cleanup Plan V3

**Status:** ACTIVE MIGRATION PLAN  
**Scope:** Documentation deduplication, authority normalization, semantic audit and safe repository organization  
**Latest audit evidence:** `docs/evidence/DOCUMENTATION-AUDIT-2026-09-11.md`

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
10. Prefer extending an existing canonical document over creating a parallel specification.

## Completed findings

### Architecture blueprint

The former `docs/ARCHITECTURE-BLUEPRINT.md` is preserved under:

`docs/archive/superseded/ARCHITECTURE-BLUEPRINT.md`

It is no longer an active specification path.

### Documentation governance

The documentation index and governance authority map now define authority hierarchy, lifecycle vocabulary, canonical document families, AI-agent/developer reading protocols, duplicate detection rules and archive-first movement rules.

### Markdown audit evidence

A targeted semantic audit has confirmed that the principal architecture, decision, roadmap and contract families are complementary rather than safe candidates for blind consolidation. See `docs/evidence/DOCUMENTATION-AUDIT-2026-09-11.md`.

## Current canonical relationships

### Master Decisions vs Decision Register

Keep both:

- Master Decisions = concise locked baseline.
- Decision Register = numbered history, consequences and review protocol.

Do not create a third general decision specification.

### Architecture vs Ecosystem Architecture

Keep both:

- `SIDERETH-ARCHITECTURE.md` = target system architecture, universal infrastructure, lifecycle and delivery order.
- `SIDERETH-ECOSYSTEM-ARCHITECTURE.md` = ecosystem composition, capability abstraction, domain packs, surface independence, provider neutrality and Trust Kernel.

They are complementary and should not be merged merely to reduce file count.

### Capability Model vs Capability Contract

Keep both:

- `SIDERETH-CAPABILITY-MODEL.md` explains the ecosystem abstraction and composition model.
- `contracts/CAPABILITY-CONTRACT.md` is the implementation-facing normative boundary.

The explanatory model must not silently redefine the contract.

### Ecosystem roadmap vs delivery roadmap

Keep both, with explicit scope:

- `SIDERETH-ECOSYSTEM-ROADMAP.md` = strategic/capability-led roadmap.
- `ROADMAP.md` = delivery sequencing.

Any future convergence should be handled by explicit consolidation, not duplicated edits.

### Tool Gateway

Keep the contract and conformance matrix as the canonical specification in `docs/contracts/`. Implementation evidence belongs in source code, tests, CI and conformance evidence.

## Targeted semantic-audit queue

The next audit must inspect complete document contents for these families, not just filenames:

1. authorization / policy;
2. action / approval / execution;
3. capability / capability registry;
4. tool registry / tool gateway;
5. evidence / provenance / persistence;
6. API / error / idempotency / versioning;
7. intelligence;
8. domain model / Case / Incident / Party / Document.

For each family record canonical authority, supporting explanations, conformance/evidence documents, planning material, duplicated sections, unique material, proposed disposition and inbound/outbound references.

## Physical organization gate

Physical movement remains **BLOCKED** until the Markdown link/reference inventory is complete.

Do not create taxonomy folders simply for appearance. If physical migration is later justified: create the destination, migrate unique content, update every reference, archive the old source, verify links and CI, and delete only after evidence.

## Current Tool Gateway gate

PR #73 remains below production-ready. The implementation head previously audited was `7a028da2dd200b6cdf45950d69dff43b61ffadaf`.

Remaining semantic gates include trusted authorization expiry/freshness, returned authorization constraints, registry-driven implementation/provider selection, richer audit/provenance, durable/concurrent idempotency, direct adapter bypass evidence and TG-001–TG-060 conformance.

The conformance matrix explicitly requires executable evidence before requirements can be considered tested.

## Next cleanup phase

1. Complete the full active-Markdown inventory.
2. Build inbound/outbound Markdown link map.
3. Run family-by-family semantic duplicate analysis.
4. Consolidate only where authority is genuinely duplicated.
5. Rename/move only after reference impact is known.
6. Add lightweight documentation-consistency CI after taxonomy stabilizes.

## Completion rule

The repository is documentation-clean only when an engineer or AI agent can determine from the index which document is authoritative for every major architectural or domain decision without relying on tribal knowledge.
