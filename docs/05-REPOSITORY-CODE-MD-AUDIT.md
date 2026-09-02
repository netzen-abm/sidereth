# SIDERETH — Repository Code & Markdown Audit

Status: Executed on `sidereth-foundation`

## Executive result
The repository contains a SIDERETH foundation plus legacy Janavani/decentralized prototype material. The legacy programming and documentation does not align with the locked SIDERETH product boundary and must not remain on the active product path.

## Programming audit

### `main` baseline
- `Cargo.toml` identifies the package as `janavani` and contains optional decentralized protocol dependencies. This is legacy architecture, not SIDERETH core.
- `src/lib.rs` contains protocol mock modules whose tests mainly assert `Ok()` responses; the ZKP implementation returns a dummy proof byte array. These are not production legal/regulatory capabilities.
- `index.html` is a Janavani decentralized-protocol toggle dashboard and is not a SIDERETH application surface.
- The legacy GitHub Actions workflows are empty or decentralized/Janavani-specific and are not suitable SIDERETH validation.

### Foundation replacement
- `Cargo.toml` now defines `sidereth-core` with minimal deterministic dependencies.
- `src/lib.rs` now contains deterministic Case, Incident and Event primitives with no AI, network transport or autonomous legal action.
- The foundation workflow validates documentation, obsolete active-code references, formatting, compilation, tests and Clippy.

## Markdown audit

### Retain as authoritative SIDERETH material
- `docs/00-SIDERETH-MASTER-DECISIONS.md`
- `docs/SIDERETH-ARCHITECTURE.md`
- `docs/SIDERETH-REPOSITORY-MIGRATION.md`
- `docs/SIDERETH-MASTER-CHECKLIST.md`
- `docs/ESTIMATE-WIREFRAME-PLAN.md`
- `docs/08-MCP-ARCHITECTURE.md`
- `docs/contracts/CASE-INCIDENT-EVENTS.md`
- `docs/contracts/AUTHORIZATION-MATRIX.md`
- `docs/migration/LEGACY-DISPOSITION.md`

### Archive from active product path
The audited Janavani/decentralized files are preserved historically and are removed from the active tree:
- Janavani constitution and representative/governance schemas
- Janavani source registry and database schema
- Janavani CLI/Freenet/decentralized integration guides
- Janavani architecture/website/runbook documents
- root decentralized dashboard

## Corrections required by the audit
1. Do not claim components are implemented unless code/tests/deployment evidence exists.
2. Do not retain Janavani product identity in active SIDERETH implementation.
3. Do not make Nostr, Nym, Reticulum, Freenet, blockchain or ZKP dependencies of SIDERETH core.
4. Keep future transports as optional adapters behind shared contracts and policy controls.
5. Do not use the old representative/governance database as the SIDERETH legal/regulatory domain model.
6. Do not claim production database, OpenAPI, AI-agent, deployment or government-integration capabilities before their gates are implemented and verified.
7. MCP remains an interoperability boundary, never a replacement for SIDERETH authorization, policy, evidence, case or workflow infrastructure.

## Disposition rule
A legacy artifact is removed from the active tree only after its purpose, reuse value and dependencies are reviewed and historical availability is preserved. The archive manifest is at `docs/archive/legacy-janavani/README.md`.

## Gate status
- Programming alignment: COMPLETE for foundation scope.
- Markdown alignment: COMPLETE for foundation scope.
- Production implementation authorization: NOT YET GRANTED; contract design remains the next engineering gate.
