# SIDERETH — Repository Migration & Disposition

## Audit baseline
The repository was inherited from an earlier civic/decentralized proof of concept. It contained a small Rust crate, a static protocol dashboard, project-specific schemas and documentation, and experimental decentralized integrations.

## Disposition
### Retain and adapt
- modular capability isolation pattern
- evidence/source provenance principles
- privacy-by-design principles
- protocol adapters as optional future transports
- existing license unless a legal review requires change

### Archive / quarantine
- former civic constitution and governance material
- former representative/public-data model
- former official-data registry
- former representative-data database schema
- former website/dashboard documentation
- former production runbook
- former CLI installation guidance
- experimental decentralized-only documentation

Historical material remains recoverable through Git history. It is not part of the active SIDERETH specification.

### Replace from active product path
- inherited package identity
- inherited protocol mock implementation
- root decentralized dashboard
- inherited project README
- obsolete project-specific CI

## Branch strategy
- `main`: production integration target
- `sidereth-foundation`: architecture and migration work
- `decentralized-system`: preserve until all unique files/commits are audited and archived; do not delete yet

## Deletion gate for branches
A branch may be deleted only after:
1. compare against main
2. unique commits reviewed
3. unique files reviewed
4. useful code/docs extracted or archived
5. no release/deployment dependency found
6. no open PR depends on it
7. final verification recorded in migration log

## Active repository target
```text
sidereth/
├── .github/workflows/
├── docs/
│   ├── architecture/
│   ├── contracts/
│   ├── security/
│   ├── testing/
│   └── migration/
├── crates/
│   ├── sidereth-core/
│   ├── sidereth-legal/
│   └── sidereth-platform/
├── domains/
│   ├── panchayat/
│   └── municipality/
├── apps/
│   └── web/
├── schemas/
├── migrations/
└── README.md
```
