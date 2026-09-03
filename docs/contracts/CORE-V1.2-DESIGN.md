# SIDERETH Core v1.2 — Durable Persistence Contract

Status: DRAFT

## 1. Objective

Define a durable persistence contract without coupling SIDERETH to any database, object store, filesystem, cloud vendor, hosted service, or deployment model.

The core must remain portable, independently deployable, and replaceable at every storage boundary.

## 2. Non-Negotiable Architecture Principle

**Storage is an adapter, not a dependency of the domain.**

PostgreSQL, SQLite, Supabase, S3-compatible storage, local filesystem, encrypted device storage, embedded databases, self-hosted services, or future storage technologies may implement the contracts independently.

No storage provider may become part of the canonical domain model.

## 3. Canonical Boundary

Application / Adapter
→ Domain Service
→ Authorization + Policy
→ Domain Repository Contract
→ Storage Adapter
→ Provider

Evidence follows a separate boundary:

Domain Evidence Contract
→ Evidence Storage Adapter
→ Local / Object / Encrypted Provider

Audit follows its own append-oriented contract and must remain independently replaceable.

## 4. Repository Contract

Repository interfaces must define domain operations, not SQL, HTTP, SDK, ORM, vendor-specific types, or storage paths.

Minimum semantics:
- get by stable ID
- create without accidental replacement
- update through validated domain transitions
- deterministic not-found behavior
- conflict detection
- version/concurrency information
- transaction participation where required
- authorization before retrieval or mutation

## 5. Aggregate and Transaction Boundaries

A transaction boundary must be explicit.

A persistence implementation must not partially commit a logically atomic domain operation.

Cross-aggregate operations must declare whether they are:
- atomic transaction
- independently committed workflow
- event-driven eventual consistency
- human-approved multi-step operation

The domain contract must not assume a particular transaction technology.

## 6. Concurrency Contract

Durable implementations must prevent silent lost updates.

The contract should support an opaque revision/version token or equivalent optimistic-concurrency mechanism.

A stale mutation must fail deterministically rather than overwrite a newer state.

## 7. Serialization and Versioning

Persisted representations must be versioned independently from implementation language structures.

Requirements:
- explicit schema version
- deterministic field semantics
- forward/backward compatibility policy
- migration identifier where applicable
- unknown-field policy
- no provider-specific serialization leaking into domain contracts

## 8. Reference Integrity

Stable references between Case, Incident, Event, Evidence, Legal Source, Jurisdiction, Authority, Procedure, Deadline, Compliance Requirement, Response, Escalation, Remedy and Resolution must be validated at the appropriate domain/persistence boundary.

Dangling references must not be silently created.

## 9. Evidence Preservation

Evidence originals remain immutable.

Persistence may relocate an evidence object, but must preserve:
- stable evidence identity
- content hash
- provenance
- immutability semantics
- legal-hold semantics
- retention semantics
- access authorization

A storage provider must never redefine evidence identity.

## 10. Audit Independence

Audit persistence must be replaceable independently of domain-state storage.

A security-relevant or mutating operation must not be considered complete when required audit persistence has failed, unless an explicitly defined durable failure policy applies.

Sensitive payloads should remain outside audit records.

## 11. Failure and Recovery

Adapters must expose deterministic failure classes for at least:
- unavailable
- timeout
- conflict
- integrity failure
- authorization failure
- validation failure
- serialization/version failure
- not found
- retention/legal-hold restriction

Recovery must not invent or duplicate domain mutations.

Idempotent operations require explicit idempotency semantics.

## 12. Portability and Freedom

SIDERETH must be able to operate without a mandatory proprietary infrastructure provider.

A deployment may choose:
- fully local
- self-hosted
- private cloud
- public cloud
- hybrid
- offline-first device storage
- multiple providers for different data classes

Switching providers must be an adapter/migration concern, not a domain rewrite.

## 13. Data-Class Independence

Canonical domain state, immutable evidence, derived artifacts, legal-source data, audit records and ephemeral workflow state may use different storage implementations.

There must be no assumption that one provider stores everything.

## 14. Security Boundary

Authorization and policy remain above storage.

Storage credentials, encryption keys and provider APIs must never become user-facing domain capabilities.

A storage adapter cannot grant access that SIDERETH policy denies.

## 15. Explicit Non-Goals for v1.2-A

This contract does not select or deploy:
- PostgreSQL
- Supabase
- SQLite
- S3
- a cloud vendor
- a particular ORM
- a particular KMS/HSM
- a production backup service
- a production replication topology

Provider selection belongs to v1.2-B or later deployment architecture.
