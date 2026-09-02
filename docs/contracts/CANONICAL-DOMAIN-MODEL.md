# SIDERETH — Canonical Domain Model

Status: Draft for Gate 2 verification

## 1. Purpose
This contract defines the minimum domain vocabulary shared by every SIDERETH surface and adapter. Domain-specific adapters may extend these objects but must not redefine their identity, lifecycle, provenance, authorization, or audit semantics.

## 2. Core aggregates

### Case
A Case is the universal matter container.

Required concepts:
- `case_id`
- `schema_version`
- `status`
- `created_at`
- `updated_at`
- `owner_subject_id`
- `jurisdiction_ref`
- `authority_ref` (nullable until identified)
- `matter_type`
- `title`
- `facts_ref`
- `event_refs`
- `document_refs`
- `evidence_refs`
- `legal_issue_refs`
- `deadline_refs`
- `action_refs`
- `decision_refs`
- `appeal_refs`
- `escalation_refs`
- `assistance_refs`
- `audit_ref`

### Incident
An Incident records a real-world official interaction.

Required concepts:
- `incident_id`
- `schema_version`
- `status`
- `started_at`
- `ended_at` (nullable)
- `location_ref` (privacy-minimised)
- `authority_ref`
- `officer_ref` (nullable and minimised)
- `purpose_stated`
- `legal_basis_ref` (nullable)
- `request_refs`
- `document_refs`
- `seizure_refs`
- `statement_refs`
- `witness_refs`
- `evidence_refs`
- `event_refs`
- `linked_case_id` (nullable)

### Evidence
Evidence is an immutable original plus separately versioned derived artifacts.

Minimum concepts:
- `evidence_id`
- `schema_version`
- `case_id` or `incident_id`
- `captured_at`
- `captured_by`
- `media_type`
- `content_hash`
- `storage_ref`
- `integrity_status`
- `provenance_ref`
- `derived_artifact_refs`
- `retention_policy_ref`

### Legal Source
A Legal Source is an authoritative or secondary source used to support a legal proposition.

Minimum concepts:
- `source_id`
- `source_type`
- `title`
- `issuing_authority`
- `jurisdiction`
- `effective_from`
- `effective_to` (nullable)
- `version`
- `retrieved_at`
- `citation`
- `location_ref`
- `supersession_status`
- `integrity_hash` where applicable

### Deadline
A Deadline is an actionable temporal constraint whose provenance is explicit.

Minimum concepts:
- `deadline_id`
- `case_id`
- `deadline_type`
- `due_at`
- `timezone`
- `basis_ref`
- `verification_status`
- `created_at`
- `status`

## 3. Supporting objects
Authority, Jurisdiction, Document, Legal Issue, Action, Decision, Appeal, Escalation, Assistance and Audit are first-class references. They must be independently addressable and versioned where their meaning can change.

## 4. Invariants
1. IDs are stable and globally unique within the SIDERETH deployment boundary.
2. Every aggregate carries a schema version.
3. System-generated legal propositions require source references.
4. User facts and system inferences are distinct fields/types.
5. Evidence originals are never overwritten.
6. State transitions are validated by the domain layer.
7. High-impact actions require an approval record.
8. Every mutating operation is attributable to a user, authorized service, or agent identity.
9. Cross-case access is denied by default.
10. Domain adapters cannot bypass policy, authorization or audit boundaries.

## 5. Extension rule
Adapters may add domain-specific fields under a namespaced extension object. They must not alter canonical semantics of identity, provenance, lifecycle, authorization, evidence integrity, deadlines, or audit.
