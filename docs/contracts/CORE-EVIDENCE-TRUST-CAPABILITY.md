# SIDERETH Core — Evidence Trust Capability

**Status:** CANONICAL / FOUNDATION DESIGN
**Scope:** Shared evidence capture, context, integrity, provenance and privacy metadata.

## Purpose

Evidence is a reusable SIDERETH capability, not an application-specific feature. This contract defines the minimum metadata required to preserve the distinction between an original artifact, its technical integrity, its capture context, later transformations and the privacy representation shown to another party.

## Trust model

SIDERETH makes bounded technical claims.

- A hash supports later integrity comparison; it does not prove legal authenticity.
- A digital signature supports cryptographic binding to a signing key; it does not prove that the recorded event was truthful.
- Hardware-backed attestation supports a stronger claim about where a key is protected when the platform attestation is independently verified; it does not prove the content itself is true.
- GPS records the device-reported location and its stated uncertainty; it does not prove the subject matter was located there.
- A capture timestamp records device/platform time; it does not by itself prove the real-world occurrence time.

## Evidence layers

```text
Original Artifact
      |
      +--> Immutable metadata record
      |
      +--> Capture Context
      |
      +--> Integrity
      |
      +--> Provenance
      |
      +--> Derived Artifacts
      |
      +--> Privacy-controlled representations
```

## Capture Context

Capture metadata MAY include:

- captured_at
- captured_by
- device_ref
- app_version
- capture_method
- location
- horizontal_accuracy_m
- altitude_m / altitude_accuracy_m where available
- heading_deg where available
- device_clock_observation where available

Location is optional by default. A GPS-first workflow may require location for that specific workflow, but must still provide a documented fallback where the design permits it.

## Media and origin classification

`media_origin` MUST distinguish at least:

- `live_capture`
- `imported_file`
- `derived_artifact`
- `unknown`

An imported media file MUST NOT be represented as a live capture merely because it was attached through a camera-style UI.

## Integrity status

Allowed logical states:

- `unverified`
- `hash_verified`
- `signature_verified`
- `hardware_attested`
- `integrity_failed`

A stronger state must not be inferred from a weaker state. For example, `hash_verified` does not imply `signature_verified`, and `signature_verified` does not imply `hardware_attested`.

## Hardware attestation

Hardware attestation is an optional evidence property. The canonical metadata MUST distinguish:

- `not_available`
- `not_requested`
- `software_fallback`
- `hardware_attested`
- `verification_failed`

A producer MUST NOT label evidence as hardware-backed unless a real platform-backed key and verifiable attestation have been used. Android hardware-backed attestation can provide evidence that a key is protected in a TEE or StrongBox when the attestation chain and security level are independently validated. See Android's key-attestation guidance. iOS Secure Enclave provides platform hardware protection for supported key operations, but SIDERETH must still record the exact implementation and verification state rather than assuming hardware backing from platform presence alone.

## Provenance

Every derived artifact MUST identify its immediate source evidence identifier. A transformation record SHOULD additionally include:

- transformation_type
- created_at
- created_by
- tool/provider identity
- tool/provider version
- input hash
- output hash
- policy/reference version where applicable

Examples: OCR, transcription, extraction, redaction, classification, summarization and resizing are derived operations and must not overwrite the original.

## Privacy representations

Original evidence and externally shared representations are distinct concepts. SIDERETH SHOULD support a policy-controlled representation such as:

- `exact`
- `reduced_precision`
- `redacted`
- `metadata_removed`
- `not_shareable`

A privacy transformation creates a derived artifact and must not mutate the original artifact.

## Evidence Passport

A read-only Evidence Passport projection SHOULD expose, as permitted by policy:

- evidence identifier
- media origin/type
- capture timestamp
- capture actor class
- location and accuracy, if disclosure is authorized
- original hash
- integrity status
- hardware-attestation status
- app/device reference as allowed
- source/derived relationships
- privacy representation status
- verification time and verifier identity when verification is external

The passport is a projection. It is not a second source of truth.

## Required invariants

1. Original bytes are write-once by evidence identifier.
2. Original metadata is write-once by evidence identifier.
3. A duplicate storage reference cannot silently replace existing bytes.
4. A derived artifact cannot become the original by mutation.
5. An imported file cannot be relabeled as live capture without an explicit new evidence object and provenance record.
6. Integrity verification never mutates the evidence record.
7. Location disclosure is policy-controlled and separate from capture.
8. Hardware-backed status cannot be asserted by a software fallback.
9. Every derived artifact retains a source reference.
10. A technical integrity result must never be presented as a legal authenticity conclusion.

## Extension boundary

This contract extends the existing `EvidenceOriginal`, `DerivedArtifact` and `EvidenceObjectStore`/`EvidenceRepository` boundaries. Existing fields retain their semantics. New metadata should be additive and versioned.

## Completion criteria

Implementation is not considered complete until the repository contains:

- typed models
- validation
- positive and negative tests
- persistence behavior tests
- provenance tests
- privacy transformation tests
- device integration evidence for any claimed hardware-backed implementation
- security/privacy review evidence
- updated documentation and capability registry state
