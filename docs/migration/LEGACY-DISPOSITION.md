# SIDERETH — Legacy Disposition Register

Status: Updated after code/Markdown audit on `sidereth-foundation`

## Preserve
Original Janavani/decentralized material remains recoverable through Git history and the preserved `decentralized-system` branch while final branch disposition is pending.

## Archived from active tree
The following artifacts were reviewed and removed from the active SIDERETH tree because they are Janavani-specific, superseded, overclaim implementation, or describe infrastructure not present in the repository:
- docs/01-JANAVANI-CONSTITUTION.md
- docs/02-REPRESENTATIVE-DATA-MODEL.md
- docs/03-OFFICIAL-DATA-SOURCE-REGISTRY.md
- docs/04-DATABASE-SCHEMA.md
- docs/CLI_INSTALLATION_GUIDELINES.md
- docs/DECENTRALIZED_INTEGRATION_GUIDELINES.md
- docs/FREENET_CRATE_GUIDELINES.md
- Complete Platform Architecture Blueprint Index.md
- Future Platform Engineering & Architectural Recommendations.md
- Hybrid Janavani WebSite.md
- Step-by-Step Production Launch Runbook.md
- The dynamic architecture.md
- root index.html

The archive manifest is `docs/archive/legacy-janavani/README.md`.

## Retained/adapted principles
- modular capability isolation
- evidence/source provenance
- privacy-by-design
- optional future transport adapters
- explicit human approval for high-impact actions
- auditability and versioning

## Active SIDERETH foundation
The active documentation and implementation now center on:
- SIDERETH Master Decisions
- SIDERETH Master Blueprint
- SIDERETH Architecture
- Case/Incident/Event contracts
- Authorization matrix
- MCP interoperability boundary
- repository migration/audit controls

## Branch decision
`decentralized-system` is NOT deleted at this stage. It contains unique files/commits and remains subject to the deletion gate:
1. compare with target integration branch;
2. review unique commits/files;
3. extract or archive useful assets;
4. verify release/deployment dependencies;
5. verify no open PR dependency;
6. record final decision;
7. delete only after evidence supports deletion.
