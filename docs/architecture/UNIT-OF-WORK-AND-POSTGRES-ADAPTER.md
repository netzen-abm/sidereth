# SIDERETH Unit-of-Work and PostgreSQL Adapter

## Status

**Implemented on branch:** `sidereth-p1-unit-of-work`

## Decision

SIDERETH now defines a provider-neutral **Unit-of-Work (UoW)** boundary in the canonical Rust domain contract layer. A UoW is the atomic infrastructure boundary for a multi-resource workflow.

The contract is intentionally provider-neutral:

```text
Application / Domain Service
        |
        v
  UnitOfWorkFactory
        |
        v
     UnitOfWork
        |
        +--> UnitOfWorkContext
        |      +--> write_resource(ResourceWrite)
        |      +--> link_resources(ResourceLink)
        |
        +--> commit / rollback
        |
        +--------------------+
        |                    |
   PostgreSQL             future adapters
   adapter                (other stores)
```

## Why this boundary exists

The earlier document invariant work established **logical atomicity** inside the provider-neutral `DocumentRegistry`. That does not by itself guarantee database transaction atomicity.

The UoW separates the two guarantees:

1. **Domain atomicity:** all domain preconditions must pass before the logical state changes.
2. **Persistence atomicity:** all writes participating in one workflow commit together at the storage provider boundary.

This is required for workflows such as:

```text
Case
  +-- references --> Document
  +-- references --> Evidence
  +-- emits ------> Event
```

A caller can stage the Case resource, Document/Evidence resources, and their references through one UoW and commit once. A failed operation causes rollback rather than a partially committed workflow.

## Contract rules

### `UnitOfWorkContext`

The context exposes only provider-neutral operations:

- `write_resource(ResourceWrite)`
- `link_resources(ResourceLink)`

It does not expose SQL, ORM types, HTTP, connection pools, or vendor-specific transaction objects.

### `ResourceWrite`

A resource write contains:

- canonical `ResourceRef`
- positive schema version
- serialized payload
- `Insert` or `Upsert` mode

This keeps the cross-resource persistence boundary aligned with the existing typed `ResourceRef` contract while preserving source compatibility for existing domain IDs.

### `ResourceLink`

A link records:

- source resource
- explicit relation
- target resource

Relations are explicit rather than inferred from identifier names.

### `UnitOfWork`

The UoW provides:

- an operation scope through `execute`
- explicit `commit`
- explicit `rollback`
- automatic rollback by the PostgreSQL adapter when an operation fails or the UoW is dropped while active

## PostgreSQL adapter

The PostgreSQL adapter is feature-gated behind `postgres` so the canonical core does not acquire a mandatory database-provider dependency.

The adapter uses the Rust `postgres` client and a real database transaction. Resource payloads are stored as JSONB; PostgreSQL supplies durability and transactionality while Rust remains authoritative for domain semantics.

The adapter does **not** make PostgreSQL schemas the domain model. The SQL tables are an infrastructure projection of the canonical resource contract.

## Database schema

Migration:

`migrations/0001_resource_unit_of_work.sql`

Tables:

- `sidereth_resource_records` — durable canonical resource payloads keyed by `(resource_type, resource_id)`.
- `sidereth_resource_links` — explicit resource-to-resource relations.

The link table deliberately avoids hard foreign keys to resource tables. SIDERETH resources are polymorphic and may be supplied by different domain packs or future storage adapters; referential policy therefore remains a contract-level concern rather than being hard-coded into one provider schema.

## Security and correctness boundary

The PostgreSQL adapter:

- uses parameterized statements for user/resource data;
- never constructs SQL by concatenating resource identifiers or payloads;
- does not receive application secrets directly through source code;
- does not grant authorization or legal authority;
- does not interpret AI output;
- does not bypass domain validation.

Provider errors are currently normalized into the existing `PersistenceError` vocabulary. Provider-specific diagnostics should be added later without leaking provider types into the canonical contract.

## Testing policy

CI now runs `cargo test --all-targets --all-features` and `cargo clippy --all-targets --all-features -- -D warnings`, ensuring the feature-gated PostgreSQL adapter is compiled and tested even when no PostgreSQL server is present.

A live integration test against PostgreSQL is intentionally deferred until the repository has an explicit database-service/integration-test policy. It should verify:

1. Case + Document + Evidence links commit atomically.
2. An error after one successful write rolls back every write in the UoW.
3. Duplicate `Insert` is rejected.
4. `Upsert` advances the stored representation safely.
5. Link uniqueness is deterministic.
6. connection/transaction failures map to the canonical persistence error contract.

## Architectural consequence

The next persistence work should **not** introduce provider-specific transaction semantics into domain services. Domain services should depend on `UnitOfWorkFactory`/`UnitOfWork` and continue to operate against canonical contracts.

The PostgreSQL adapter is therefore an implementation of the SIDERETH persistence contract, not the contract itself.
