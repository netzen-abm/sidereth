# SIDERETH Documentation Index

## Purpose
This index defines the documentation hierarchy for the SIDERETH repository. It prevents duplicate specifications and separates decisions, architecture, contracts, plans, implementation evidence and historical material.

## Canonical hierarchy

### 00 — Governance and decisions
- `00-SIDERETH-MASTER-DECISIONS.md` — locked product and architectural decisions.
- `DECISION-REGISTER.md` — decision history and rationale.
- `DOCUMENTATION-INDEX.md` — this map.
- `SIDERETH-GLOSSARY.md` — canonical terminology.

### 01 — Product and architecture
- `01-SIDERETH-MASTER-BLUEPRINT.md` — master product/system blueprint.
- `SIDERETH-ARCHITECTURE.md` — current target architecture and boundaries.
- `ESTIMATE-WIREFRAME-PLAN.md` — planning estimate and UX/wireframe scope only.
- `ROADMAP.md` — phased delivery roadmap.
- `08-MCP-ARCHITECTURE.md` — MCP interoperability boundary.

### 02 — Contracts
`docs/contracts/` contains canonical domain, capability, lifecycle/event, authorization, API semantics, audit/storage and contract-test definitions.

Important capability contracts include:
- `OPTIONAL-CAPABILITY-CONTRACT.md` — plug-and-play policy for optional Nostr, Nym, Reticulum, ZKP, blockchain, Freenet and WASM capabilities.

### 03 — Migration and legacy
`docs/migration/` records repository migration and disposition decisions.
`docs/archive/` contains preserved historical material that is no longer active product specification.

Important historical audit records include:
- `docs/archive/historical-migration/DECENTRALIZED-SYSTEM-BRANCH-AUDIT.md` — evidence-based disposition of the legacy `decentralized-system` branch and its Freenet prototype.

## Documentation status vocabulary
- **LOCKED** — governing decision; changes require an explicit decision update.
- **CANONICAL** — authoritative specification for its scope.
- **DRAFT** — proposed design awaiting executable verification or decision.
- **PLANNING** — estimate/UX/roadmap material; not implementation evidence.
- **REFERENCE** — useful supporting material but not authoritative.
- **ARCHIVED** — retained for historical traceability; must not drive implementation.

## Non-negotiable documentation rule
Documentation must never imply implementation that is not supported by source code, automated tests, deployment evidence or other reproducible evidence.

## Change rule
Before creating a new specification, search this index and existing docs. Extend the canonical document when the subject already exists. Create a new document only when the scope is genuinely distinct.
