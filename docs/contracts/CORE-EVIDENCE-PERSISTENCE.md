# SIDERETH Core — Evidence Persistence Contract

**Status:** CANONICAL / FOUNDATION DESIGN
**Scope:** Provider-neutral durable persistence of Evidence Trust state.

## Purpose

The Evidence Trust capability requires durable persistence without coupling the Trust Kernel to PostgreSQL, object storage, filesystem storage, mobile databases or any other provider.

This contract therefore treats evidence persistence as a provider-neutral boundary. Providers implement the contract; the domain model remains authoritative.

## Persisted evidence unit

A persisted Evidence Trust record contains:

- the existing `EvidenceOriginal` metadata;
- additive `EvidenceTrustMetadata`;
- zero or more `EvidenceTransformation` records;
- an explicit schema version.

The original artifact remains immutable. Trust metadata and provenance are part of the authoritative evidence state and must not be silently detached from the evidence identifier.

## Atomicity boundary

When a persistence provider supports transactions, creation of the evidence metadata record and its trust/provenance metadata MUST commit atomically.

The contract does **not** require every object-storage provider to provide database-style transactions over bytes. Instead, the persisted record references the immutable content through the existing `storage_ref` and content hash.

A provider MUST reject a record whose referenced original bytes are unavailable when the provider's evidence boundary requires object verification before metadata commit.

## Provider neutrality

The canonical contract MUST NOT depend on:

- PostgreSQL
- Supabase
- SQLite
- S3-compatible storage
- a filesystem
- a mobile database
- a cloud provider
- a particular ORM

These may implement adapters independently.

## Invariants

1. Evidence identifiers are unique within the persistence namespace.
2. Original evidence cannot be replaced by an update operation.
3. Trust metadata cannot assert a stronger integrity state than its attestation state permits.
4. Every transformation retains its immediate source evidence identifier.
5. A persisted record must validate before commit.
6. Schema version zero is invalid.
7. Reads return the authoritative persisted record, not a reconstructed approximation from a presentation artifact.
8. Persistence failures must not be represented as successful evidence creation.
9. Provider revision/CAS mechanisms MAY be used for concurrency, but provider-specific revision semantics must remain behind the adapter boundary.
10. Technical integrity remains distinct from legal authenticity.

## Repository boundary

The provider-neutral repository SHOULD provide at least:

- create immutable evidence record;
- read evidence record;
- verify the referenced original content when supported;
- enumerate transformation metadata for an evidence identifier;
- expose provider-neutral persistence errors.

Updates to original evidence are prohibited. Future mutable trust-state changes, such as external verification events, should use explicit append-only verification records rather than mutating the original capture record.

## Transaction integration

Where SIDERETH's existing `UnitOfWork` is available, evidence creation SHOULD participate in the same authoritative command transaction as related Case, Incident, Event, provenance or audit writes.

The evidence contract must not create a parallel transaction abstraction that competes with the canonical Unit-of-Work boundary.

## Completion gate

This contract is considered implemented only when there is:

- a provider-neutral repository contract;
- typed validation;
- immutable-create tests;
- persistence round-trip tests;
- duplicate/replacement rejection tests;
- transformation/provenance persistence tests;
- transactional integration tests for supported transactional providers;
- provider-specific adapter tests without leaking provider semantics into the Trust Kernel.
