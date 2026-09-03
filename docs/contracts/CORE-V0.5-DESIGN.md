# SIDERETH — Core v0.5 Legal Source & Provenance

Status: DRAFT / CORE V0.5
Version: 1.0

## 1. Purpose

Core v0.5 establishes the deterministic provenance boundary for legal and
regulatory knowledge.

It answers one question:

> Where does a legal proposition come from, and was that source in force for
the relevant time and jurisdiction?

This increment defines provenance primitives. It does not build a national
legal corpus, crawler, AI legal reasoner, or production source-ingestion
pipeline.

## 2. Canonical chain

SIDERETH represents legal provenance as:

**Source → Authority → Version → Effective Date → Citation → Proposition →
Confidence → Supersession**

A proposition is never authoritative merely because a model produced it.

## 3. Legal source

A `LegalSource` is a versioned record describing a legal or regulatory source.
It contains:

- `source_id`
- `source_type`
- `title`
- `issuing_authority`
- `jurisdiction`
- `citation`
- `published_at`
- `effective_from`
- `effective_to` (optional)
- `version`
- `retrieved_at`
- `verification_status`
- `supersession_status`

The core object stores metadata and provenance. Source content storage is a
separate concern.

## 4. Source types

The initial source taxonomy follows the canonical registry:

1. Constitution or legislation
2. Rule or regulation
3. Notification, order or circular
4. Official procedure
5. Judicial decision
6. Official guidance
7. Secondary source

Unsupported source types must be rejected rather than silently mapped to a
nearby category.

## 5. Proposition

A `LegalProposition` is a bounded statement derived from one or more source
references.

It must identify:

- proposition identity
- schema version
- proposition type
- statement
- source references
- verification status
- confidence

The implementation must preserve the distinction between:

- verified rule
- official procedure
- authoritative interpretation
- user-provided fact
- inference
- uncertainty
- disputed interpretation
- professional review required

A user fact or model inference is not converted into a legal source by type
casting or omission of provenance.

## 6. Citation and provenance

Each proposition must have at least one source citation reference.

A citation identifies the location within a source sufficiently to support
later review. The v0.5 core does not require a specific external citation
format beyond a non-empty citation value.

Source references are explicit objects/IDs rather than free-form text embedded
in the proposition statement.

## 7. Effective time

A source has an inclusive `effective_from` instant/date and optional
`effective_to` instant/date.

The interval is valid only when `effective_to` is not earlier than
`effective_from`.

Unknown end dates remain open intervals; they must not be fabricated.

## 8. Verification and confidence

Verification status describes provenance state, not legal correctness beyond
the evidence available to SIDERETH.

Confidence describes confidence in the provenance/proposition record. It must
not be used as a substitute for source authority.

The core therefore keeps verification status and confidence as separate
fields.

## 9. Supersession

A source may supersede another source. A source must not supersede itself.

The v0.5 domain boundary rejects direct self-reference and cycles detectable
from the supplied supersession graph. It does not claim automated discovery
of all real-world supersession relationships.

## 10. Determinism

Validation must be deterministic. Source and proposition references are
ordered by stable identifiers when a canonical ordering is required.

No network retrieval or model call is required to construct or validate these
objects.

## 11. Separation from evidence

A legal source is not the same object as case evidence.

Evidence may preserve a copy or capture of a source document, but legal-source
provenance identifies the legal authority and version independently. Evidence
integrity is governed by the Evidence Vault contracts.

## 12. Scope exclusions

Core v0.5 does not implement:

- production government-source crawling
- a national legal database
- automatic legal interpretation
- autonomous legal conclusions
- AI access to private case data
- cloud-provider-specific source storage
- production source signing infrastructure
- automatic real-world supersession discovery

These are separate capabilities requiring their own contracts and verification.
