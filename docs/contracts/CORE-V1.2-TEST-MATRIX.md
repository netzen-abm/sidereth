# SIDERETH Core v1.2 — Persistence Contract Test Matrix

Status: DRAFT

## Contract tests

1. Repository contracts contain no provider-specific types.
2. Create rejects duplicate stable IDs.
3. Retrieval returns deterministic not-found behavior.
4. Valid updates require the expected revision/version.
5. Stale updates are rejected without overwriting newer state.
6. Invalid domain transitions are rejected before persistence.
7. Required atomic operations cannot partially commit.
8. Serialization includes an explicit schema version.
9. Unsupported schema versions fail deterministically.
10. Cross-entity references cannot silently become dangling.
11. Evidence originals cannot be replaced through persistence.
12. Evidence content hashes remain stable across storage relocation.
13. Legal-hold restrictions cannot be bypassed by storage adapters.
14. Audit persistence remains independently replaceable.
15. Required audit failure does not silently report successful mutation.
16. Idempotent operations produce one logical mutation.
17. Retry after provider timeout does not create duplicate mutation.
18. Provider outage produces a typed failure rather than domain corruption.
19. Authorization is evaluated before protected retrieval.
20. Authorization is evaluated before mutation.
21. Storage provider identifiers do not become canonical domain identifiers.
22. The same domain contract can be implemented by more than one storage adapter.
23. Local/offline storage remains contract-compatible with remote storage.
24. Migration/export/import preserves canonical identities and integrity metadata.
25. Derived artifacts remain replaceable without modifying original evidence.

## Exit criteria

- Contract tests pass without a mandatory database vendor.
- Domain modules contain no storage SDK imports.
- Persistence interfaces are technology-neutral.
- Concurrency behavior is explicit and deterministic.
- Failure/retry semantics are explicit.
- Evidence and audit boundaries remain independent.
- Provider choice remains a deployment decision.
