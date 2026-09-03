# SIDERETH Core v0.4 — Durable Evidence & Data Security Contract

Status: DRAFT

## Purpose

v0.4 strengthens the evidence boundary without introducing a production
storage provider. It defines durable security semantics that every adapter
must preserve.

## Scope

1. Content integrity identity.
2. Immutable original evidence.
3. Case-scoped authorization at the repository boundary.
4. Audit linkage for sensitive mutations.
5. Retention and legal-hold policy boundaries.
6. Deterministic export and recovery contracts.
7. Encryption and key-provider abstraction.
8. Typed storage and integrity errors.

## Content identity

`content_hash` is the cryptographic integrity identity of evidence content.
A `storage_ref` is an opaque physical storage locator and is not itself a
content address. v0.4 therefore does not claim that arbitrary storage refs are
content-addressed storage.

An adapter may derive a storage address from the content hash, but the core
contract does not require a particular storage technology.

## Original evidence

An original evidence record is write-once by `evidence_id`. Its content hash,
provenance and aggregate attachment cannot be replaced in place.

Verification must distinguish:

- missing metadata;
- missing object;
- invalid content;
- successful verification.

A failed integrity check is not equivalent to an ordinary missing record.

## Authorization

Evidence reads and mutations are case-scoped. The repository boundary must
receive an authorization decision before exposing or mutating case evidence.

An adapter must not use direct object-store access to bypass this policy.

## Audit

Evidence mutations must be linkable to an actor and target evidence object.
The audit boundary records metadata needed for accountability without copying
sensitive evidence content into audit logs.

## Retention and legal hold

Retention is policy data, not an implicit storage-engine behavior.

The core contract represents:

- retention policy identifier;
- retention-until time where applicable;
- legal-hold state;
- deletion eligibility.

A legal hold prevents deletion while active. Production retention schedules
and regulatory rules remain outside the in-memory adapter.

## Export and recovery

Exports must be deterministic and identify the evidence metadata, content
hash and storage reference needed to reconstruct a case-scoped evidence set.

Recovery verification must recompute content hashes after restoration.

The core does not claim backup durability merely because an in-memory adapter
passes tests.

## Encryption boundary

The core defines an encryption/key-provider interface only. It does not
implement a cloud KMS, HSM, filesystem encryption scheme or key lifecycle.

Sensitive content must remain outside logs and AI/agent inputs unless an
explicit higher-level policy permits access.

## Adapter rule

Filesystem, database, object storage, encrypted local storage, cloud storage
and decentralized storage are replaceable adapters. None may weaken the
canonical integrity, authorization, audit or retention semantics.

## Non-goals

v0.4 does not provide:

- production KMS integration;
- production backup service;
- cloud deployment;
- autonomous deletion;
- AI access to evidence;
- legal conclusions;
- UI or API transport.
