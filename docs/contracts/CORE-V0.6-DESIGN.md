# SIDERETH — Core v0.6 Jurisdiction & Authority

Status: DRAFT / CORE V0.6
Version: 1.0

## 1. Purpose

Core v0.6 establishes the deterministic boundary for answering two
foundational questions:

1. Which jurisdiction governs this matter?
2. Which authority may exercise which power within that jurisdiction?

It sits between the legal-source provenance layer and future procedure and
deadline engines.

## 2. Canonical chain

**Matter → Jurisdiction → Authority → Power → Competence → Procedure**

The core records relationships. It does not decide real-world legality from
incomplete facts.

## 3. Jurisdiction

A `Jurisdiction` identifies a legally meaningful territorial, subject-matter,
personal, institutional, or other scope.

It must have:

- stable identity
- jurisdiction type
- name
- parent jurisdiction where applicable
- status

A jurisdiction hierarchy must be deterministic and must reject self-parenting
and cycles.

## 4. Authority

An `Authority` identifies a government body, statutory office, court, tribunal,
or other legally recognised decision-maker.

It must have:

- stable identity
- name
- authority type
- jurisdiction reference
- status

An authority does not automatically possess every power associated with its
institutional name. Powers are explicit records.

## 5. Power

An `AuthorityPower` records a specific legal power or function that an
authority may exercise.

It must identify:

- stable identity
- authority reference
- jurisdiction reference
- power name
- legal-source references
- active status

The source references connect the power back to Core v0.5 provenance.

## 6. Competence

A `Competence` records whether a particular authority-power relationship is
within the declared jurisdictional scope.

Competence is represented as a bounded domain relationship, not as an
unqualified assertion that an official action was lawful.

## 7. Validation invariants

The v0.6 domain must reject:

- empty identifiers
- empty names
- unsupported jurisdiction types
- unsupported authority types
- missing jurisdiction references
- missing authority references
- authority powers without source references
- self-parenting jurisdiction relationships
- jurisdiction hierarchy cycles
- duplicate jurisdiction IDs
- duplicate authority IDs
- duplicate power IDs

## 8. Provenance boundary

Authority and power records reference legal-source IDs. They do not duplicate
legal-source metadata or embed unverified legal text.

A missing source reference is a validation failure for a legal power.

## 9. Determinism

Registry IDs are returned in stable identifier order. Hierarchy validation is
deterministic and independent of network access, AI models, UI, or databases.

## 10. Safety boundary

SIDERETH must not transform a missing record into a conclusion that an
authority acted unlawfully.

The infrastructure should support statements such as:

- authority relationship verified
- declared power found
- jurisdiction relationship not verified
- source unavailable
- professional review required

This preserves the distinction between missing evidence and a legal finding.

## 11. Scope exclusions

Core v0.6 does not implement:

- live government authority directories
- automatic jurisdiction discovery from arbitrary user text
- autonomous legality determinations
- legal advice
- procedure execution
- deadline calculation
- AI agent authority to mutate jurisdiction records
- production source ingestion

Those capabilities require separate contracts and verification.
