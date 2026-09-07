# SIDERETH — Future Evidence Trust Vision

**Date:** 2026-09-07  
**Status:** FUTURE / NOT FOR IMMEDIATE IMPLEMENTATION  
**Decision type:** Architecture research backlog / vision record

## Purpose

Capture useful ideas that emerge while building the Evidence Trust capability without prematurely expanding the current implementation scope.

This document is deliberately a parking place for validated architectural possibilities, research questions and future capabilities. Nothing here is an implementation commitment unless a later decision record promotes it.

## 1. Evidence Passport as a universal trust projection

Future SIDERETH surfaces may expose a common Evidence Passport for every evidence item. It should be possible to inspect, subject to authorization and privacy policy:

- evidence identity and type
- original capture/import status
- capture time and actor
- device/application context
- capture location and measurement accuracy
- integrity verification state
- hardware-attestation state
- original-preservation state
- transformation history
- source/derived relationships
- privacy/disclosure state
- provenance and verification history

The Passport should be a projection, not a second source of truth.

## 2. Three-layer evidence representation

Research a future model that clearly separates:

1. **Original evidence** — immutable source artifact retained under evidence policy.
2. **Protected evidence representation** — privacy-controlled storage/processing form.
3. **Presentation or submission artifact** — a derived form optimized for a recipient, workflow or public disclosure.

The layers should remain cryptographically/provenance-linked without requiring the original artifact to be publicly exposed.

## 3. Incident location versus capture location

Future evidence workflows should distinguish at least:

- where an incident is reported to have occurred,
- where the evidence was captured,
- where a device was located when capture occurred,
- and any location asserted by a document or external source.

These are different propositions and must not be silently collapsed into one GPS field.

## 4. Hardware-backed evidence attestation

Research provider-neutral adapters for mobile and edge devices that can verify hardware-backed keys and attestation evidence.

Potential future implementations include platform-specific verification for Android and Apple ecosystems. The Trust Kernel should record the verified result and its provenance rather than depending on a particular mobile platform.

A future implementation must distinguish:

- hardware capability exists,
- attestation was requested,
- attestation evidence was received,
- attestation chain was verified,
- policy accepted the verified result.

No software-only fallback should ever be represented as hardware-backed assurance.

## 5. Sensor and multimodal evidence

Explore a future evidence capture model for:

- photographs
- video
- audio
- scanned documents
- screen captures
- sensor observations
- structured device telemetry
- optional environmental measurements

Each modality should preserve source identity, acquisition context, integrity and transformation history.

## 6. Offline-first evidence capture

Research a future local-first capture pipeline in which evidence can be collected without network connectivity and synchronized later.

Important future questions:

- secure local queueing
- replay protection
- monotonic event sequencing
- conflict handling
- device clock uncertainty
- key rotation
- selective disclosure
- deletion/retention policy
- recovery after interrupted capture

## 7. Evidence chain / transformation graph

Future evidence processing may be represented as a directed provenance graph:

```text
Original
   |
   +--> normalized copy
   |
   +--> OCR text
   |
   +--> extracted facts
   |
   +--> redacted presentation copy
   |
   +--> submission package
```

Every derived node should point to its source evidence and record the transformation, actor/tool, version and relevant input/output integrity information.

## 8. Claim-to-evidence graph

Future legal intelligence could connect propositions to their support:

```text
Claim
  -> evidence item(s)
  -> source(s)
  -> transformation(s)
  -> verification state
  -> contradiction(s)
  -> human decision
```

This would allow SIDERETH to distinguish observed material, user-reported facts, source-supported propositions, system-derived results, inference and unresolved claims.

This is a research direction, not an AI-autonomy commitment.

## 9. Evidence challenge and verification workflow

Explore a future workflow in which important evidence can be challenged explicitly.

Possible states:

`unverified -> submitted -> verified -> contested -> re-verified -> accepted/rejected`

The workflow should preserve the history of challenges and decisions rather than overwriting prior states.

## 10. Privacy-preserving location disclosure

Research policy-controlled location transformations such as:

- exact coordinates for authorized investigators,
- reduced precision for ordinary sharing,
- redacted location for public presentation,
- non-shareable location retained only in protected evidence storage.

Future work should examine whether spatial proofs or other privacy-preserving mechanisms can establish relevant facts without exposing unnecessary precise location data.

## 11. Edge acceleration / Mojo research

Mojo may be evaluated in the future as an optional acceleration implementation for computationally intensive edge workloads such as image processing, OCR preprocessing, computer vision or specialized inference.

It must not become the canonical SIDERETH domain runtime merely because it is fast. Adoption should require measured benefit, maintainability, portability and a clean adapter boundary.

## 12. Evidence hardware kit

Longer-term research could explore a reference evidence-capture hardware kit with secure identity, GNSS, camera/microphone interfaces and tamper-aware capture metadata.

The kit should remain an optional adapter. SIDERETH must remain usable with ordinary phones and imported evidence.

## 13. Time integrity

Future research should treat device wall-clock time as an observation rather than unquestionable truth.

Potential future signals include:

- trusted time sources,
- monotonic clocks,
- server receipt time,
- secure hardware time where available,
- clock-drift measurements,
- sequence continuity.

The system should be able to state what is known about time rather than manufacturing certainty.

## 14. Evidence quality scoring

Explore a future deterministic quality profile that reports dimensions rather than producing a single opaque trust score.

Potential dimensions:

- source integrity
- provenance completeness
- capture-context completeness
- temporal confidence
- location confidence
- hardware assurance
- transformation transparency
- privacy/disclosure status
- contradiction status

Any future score must never be presented as legal authenticity or truth.

## 15. AI deception resistance

Future intelligence systems should be designed so that an AI model cannot upgrade an unsupported proposition into an authoritative fact merely by generating confident text.

Potential architectural controls:

- evidence-grounded generation
- explicit claim status
- source provenance
- contradiction detection
- challenge workflows
- deterministic policy gates
- human approval for high-impact actions
- audit of model/tool inputs and outputs

The model remains an analysis component; authoritative state remains outside the model.

## 16. Promotion rule

A future item may move from this document into active engineering only after a separate decision establishes:

1. the problem and user value,
2. the canonical contract,
3. its Trust Kernel / Platform / Intelligence / Domain Pack / Surface placement,
4. privacy and security implications,
5. provider-neutral boundaries,
6. test and verification requirements,
7. operational observability,
8. implementation sequencing.

Until then, ideas in this document are **not implementation requirements**.
