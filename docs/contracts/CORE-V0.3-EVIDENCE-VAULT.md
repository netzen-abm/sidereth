# SIDERETH Core v0.3 — Evidence Vault

Status: DRAFT / IMPLEMENTED INCREMENT
Version: 0.3

## Purpose

Define a storage boundary that preserves original evidence integrity without
coupling SIDERETH to a filesystem, database, object store, cloud provider,
or decentralized network.

## Evidence flow

Capture → Hash → Store → Retrieve → Verify → Derive → Audit

## Contracts

### EvidenceObjectStore

Stores opaque evidence bytes behind a storage reference.

Required behavior:

- reject empty storage references
- reject replacement of an existing object reference
- return stored bytes by reference
- return `None` for an unknown reference

The object store does not decide legal meaning or case authorization.

### EvidenceRepository

Stores canonical evidence metadata separately from raw bytes.

Required behavior:

- validate originals before persistence
- reject duplicate original identifiers
- validate derived artifacts before persistence
- reject duplicate artifact identifiers
- keep original and derived metadata distinct

## Original evidence

`EvidenceOriginal` remains the canonical metadata record. Its content hash is
computed from captured bytes before the record is constructed.

The domain record stores a storage reference and integrity hash rather than
raw content. This keeps the storage boundary explicit and limits unnecessary
copying of sensitive material.

Original evidence is append-only in this increment. Replacement requires a
new evidence identifier and an auditable workflow in a later service layer.

## Derived artifacts

Derived artifacts reference an original evidence identifier. They are not
substitutes for originals and may represent OCR, transcription, extraction,
redaction, classification, or other transformations.

Derived artifacts must remain separately identifiable and versionable.

## Verification boundary

Retrieval alone is not proof of integrity. A later service layer must compare
retrieved content against the canonical content hash before treating bytes as
verified evidence.

That verification operation must not mutate the original metadata.

## Security and privacy boundary

The v0.3 in-memory adapter is test infrastructure only. It is not a
production evidence vault.

Production storage must later define:

- encryption at rest
- key management
- authorization enforcement
- retention and deletion policy
- backup and recovery
- access logging
- secure export
- malware/content safety handling where applicable
- resource limits and large-object handling

AI and agents must never receive direct storage credentials or unrestricted
object-store access. Access must pass through the shared policy boundary.

## Implementation evidence

Implemented in this increment:

- `EvidenceObjectStore` interface
- `EvidenceRepository` interface
- in-memory object storage
- in-memory evidence metadata storage
- immutable-by-identifier original semantics
- derived artifact persistence boundary
- positive and negative tests

Not claimed by this increment:

- durable production storage
- encryption implementation
- production authorization enforcement
- production retention controls
- malware scanning
- cloud storage integration
- decentralized storage integration
