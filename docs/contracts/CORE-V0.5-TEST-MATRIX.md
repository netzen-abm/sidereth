# SIDERETH — Core v0.5 Test Matrix

Status: DRAFT / CORE V0.5

## Required validation

- Source identity is non-empty and stable.
- Source type is supported.
- Jurisdiction and issuing authority are present.
- Citation is present.
- Effective dates form a valid interval.
- Version and retrieval time are present.
- A proposition has at least one source reference.
- Proposition type remains within the canonical provenance taxonomy.
- Verification status and confidence are independently validated.
- A source cannot supersede itself.
- Supersession relationships cannot form a cycle.
- Duplicate source identifiers are rejected.
- Canonical reference ordering is deterministic.

## Provenance distinctions

Tests must preserve these as distinct values:

- verified rule
- official procedure
- authoritative interpretation
- user-provided fact
- inference
- uncertainty
- disputed interpretation
- professional review required

## Boundary

Passing these tests proves deterministic domain validation only. It does not
prove real-world source authenticity, completeness, current legal force, or
correct legal interpretation.
