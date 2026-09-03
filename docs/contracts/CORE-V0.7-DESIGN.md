# SIDERETH — Core v0.7 Procedure Infrastructure

Status: DRAFT / CORE V0.7
Version: 1.0

## 1. Purpose

Core v0.7 establishes the deterministic procedure boundary between legal
competence and deadlines.

The procedure layer answers:

- what procedural step is expected
- who may perform it
- what prerequisite must be satisfied
- what follows next
- what evidence or document may prove completion

It records procedural structure. It does not decide legal outcomes.

## 2. Canonical chain

**Matter → Jurisdiction → Authority → Power → Competence → Procedure → Deadline**

## 3. Procedure

A `Procedure` is a versioned workflow definition within a declared
jurisdiction and authority scope.

It contains:

- stable identity
- name
- jurisdiction reference
- authority reference
- status
- ordered step references
- legal-source references

A procedure must not silently change its step sequence after creation.
Future versions create new definitions.

## 4. Procedure step

A `ProcedureStep` represents one expected procedural action or checkpoint.

It contains:

- stable identity
- procedure reference
- sequence number
- step name
- responsible authority reference
- prerequisite step references
- required evidence references where applicable
- legal-source references

Sequence numbers must be positive and unique within a procedure.

## 5. Preconditions

A step cannot be represented as complete merely because it exists.
Prerequisites are explicit references to earlier steps.

The v0.7 domain validates that:

- referenced procedures exist
- referenced authorities exist
- step IDs are unique
- sequence numbers are unique
- prerequisite steps belong to the same procedure
- prerequisite cycles are rejected
- required source references are non-empty

## 6. Execution boundary

The core does not execute government procedures in v0.7.

Execution belongs to a future workflow/application layer that will enforce
identity, authorization, policy, evidence capture, audit, and human approval.

The domain only defines and validates the procedure model.

## 7. Provenance

Procedure definitions and steps reference Core v0.5 legal-source IDs.
They do not duplicate source metadata or assert that a source is currently in
force merely because a reference exists.

## 8. Safety

SIDERETH must distinguish:

- procedure defined
- procedure applicability verified
- step recorded
- step completion evidenced
- source verified
- applicability uncertain
- professional review required

The existence of a procedure record must never be treated as proof that a
specific user is legally required to follow it without applicability analysis.

## 9. Determinism

Registry identifiers and procedure steps are returned in stable order.
Validation is independent of network access, AI models, UI, or database choice.

## 10. Scope exclusions

Core v0.7 does not implement:

- live government procedure ingestion
- automatic procedure discovery from arbitrary text
- deadline calculation
- legal advice
- government submission
- autonomous procedural execution
- AI agent authority to alter procedure definitions
- production source verification
