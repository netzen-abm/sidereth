# Initial Case / Incident Event Contracts

Status: DESIGN BASELINE

## Case states
`draft → active → waiting_user → waiting_authority → response_due → escalated → resolved → closed`

A case may be reopened only through an explicit audited transition.

## Incident states
`prepared → active → paused → concluded → evidence_review → linked_to_case`

## Event envelope
```json
{
  "event_id": "uuid",
  "event_type": "CASE_CREATED",
  "aggregate_type": "case",
  "aggregate_id": "uuid",
  "occurred_at": "RFC3339",
  "actor_type": "user|system|agent|authority|professional",
  "actor_id": "opaque-id",
  "schema_version": 1,
  "payload": {},
  "source_refs": [],
  "correlation_id": "uuid",
  "causation_id": "uuid|null"
}
```

## Initial event catalogue
- CASE_CREATED
- CASE_UPDATED
- INCIDENT_CREATED
- INCIDENT_STARTED
- INCIDENT_ENDED
- AUTHORITY_IDENTIFIED
- LEGAL_BASIS_RECORDED
- REQUEST_RECORDED
- DOCUMENT_RECEIVED
- DOCUMENT_ISSUED
- EVIDENCE_CAPTURED
- EVIDENCE_LINKED
- DEADLINE_CREATED
- DEADLINE_VERIFIED
- ACTION_PROPOSED
- ACTION_APPROVED
- ACTION_REJECTED
- RESPONSE_DRAFTED
- HUMAN_REVIEW_REQUIRED
- RESPONSE_SUBMITTED
- DECISION_RECEIVED
- APPEAL_CREATED
- ESCALATION_CREATED
- CASE_RESOLVED
- CASE_CLOSED

## Rules
1. Events are append-only at the domain layer.
2. Derived views may be rebuilt from valid events.
3. Sensitive payloads must not be written to public ledgers.
4. Legal-source references are mandatory for system-generated legal propositions.
5. High-impact actions require explicit approval according to policy.
6. Every state transition must be deterministic and testable.
