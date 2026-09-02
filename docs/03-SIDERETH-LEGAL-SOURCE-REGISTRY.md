# SIDERETH — Legal Source Registry

Status: CANONICAL FRAMEWORK / IMPLEMENTATION PENDING
Version: 1.0

## Purpose

Define the authoritative-source hierarchy and provenance requirements for legal and regulatory information used by SIDERETH.

This document is domain-independent and replaces the former project-specific public-representative data registry.

## Source hierarchy

1. Constitution and legislation
2. Rules and regulations
3. Notifications, orders and circulars
4. Official government procedures and portals
5. Judicial decisions and official court sources
6. Official guidance and authoritative publications
7. Reputable secondary sources

A lower-priority source must not silently override a higher-priority authoritative source.

## Required source metadata

Every legal or regulatory source record should carry, as applicable:

- source_id
- source_type
- title
- issuing_authority
- jurisdiction
- citation
- canonical_url or document reference
- publication_date
- effective_from
- effective_to, if known
- version
- retrieval_date
- verification_status
- supersession_status
- provenance/evidence reference

## Legal proposition rule

A system-generated legal proposition must be traceable to one or more authoritative source references.

If the authoritative source cannot be verified, SIDERETH must distinguish uncertainty rather than present the proposition as a verified rule.

## Required distinctions

The system must distinguish:

- verified rule
- official procedure
- authoritative interpretation
- user-provided fact
- inference
- uncertainty
- disputed interpretation
- professional review required

## AI rule

AI may assist with retrieval, explanation, comparison and summarisation.

AI output is not the authoritative legal source. The underlying source and provenance remain authoritative.

Sensitive case information must not be exposed to an AI or agent runtime unless the applicable authorization, purpose, minimisation and policy controls permit that processing.

## Lifecycle

Source discovery → verification → versioning → indexing → retrieval → supersession detection → archival.

No source becomes authoritative merely because an AI model retrieved or cited it.

## Implementation status

This is a canonical framework. A production legal-source registry, ingestion pipeline and automated supersession system are not claimed as implemented until executable code and verification evidence exist.
