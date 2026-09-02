# SIDERETH Documentation Cleanup Plan V1

## Objective
Create one authoritative documentation system before production implementation begins.

## Cleanup principles
1. One concept, one canonical specification.
2. Preserve historical material before removal or replacement.
3. Separate decisions from architecture, contracts, plans and implementation evidence.
4. Never treat estimates, diagrams or prose as proof of implementation.
5. Every active document must identify status and scope.
6. Domain-specific documents must not redefine universal contracts.
7. New adapters must reference shared contracts rather than copy them.

## Duplicate/superseded documents identified

### `docs/ARCHITECTURE-BLUEPRINT.md`
Superseded by `docs/SIDERETH-ARCHITECTURE.md` and `docs/01-SIDERETH-MASTER-BLUEPRINT.md`.
Action: preserve in archive, remove from active specification path.

### `docs/05-REPOSITORY-CODE-MD-AUDIT.md`
Superseded by the more explicitly named repository audit already maintained as the migration/audit record.
Action: preserve in archive, remove from active path after cross-check.

## Documentation gaps to close before implementation
- canonical glossary
- documentation index
- security/privacy threat model
- legal-source lifecycle and versioning policy
- data classification and retention policy
- API/OpenAPI contract
- database model/migration policy
- test evidence index
- contribution/review policy
- release readiness checklist

## Completion rule
The repository is considered documentation-clean only when an engineer can determine from the index which document is authoritative for every major architectural or domain decision without relying on tribal knowledge.
