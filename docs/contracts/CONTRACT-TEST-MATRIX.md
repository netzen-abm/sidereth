# SIDERETH — Contract Test Matrix V1

## Purpose
Define the executable verification required before the Case/Incident Engine becomes a production foundation.

| ID | Contract | Test | Pass condition |
|---|---|---|---|
| CT-001 | Case lifecycle | valid transitions | only declared transitions succeed |
| CT-002 | Case lifecycle | invalid transitions | undeclared transitions fail deterministically |
| CT-003 | Incident lifecycle | valid transitions | only declared transitions succeed |
| CT-004 | Incident lifecycle | invalid transitions | undeclared transitions fail deterministically |
| CT-005 | Event envelope | required fields | malformed envelopes are rejected |
| CT-006 | Event replay | reconstruction | same valid event sequence yields same state |
| CT-007 | Schema | Case JSON | valid fixture validates; malformed fixture fails |
| CT-008 | Schema | Incident JSON | valid fixture validates; malformed fixture fails |
| CT-009 | Authorization | own case | user access is allowed only within scope |
| CT-010 | Authorization | cross-case access | unauthorized access is denied |
| CT-011 | High-impact action | approval gate | action cannot execute without required approval |
| CT-012 | Idempotency | duplicate command | retry does not create duplicate side effect |
| CT-013 | Concurrency | stale version | conflicting update is rejected |
| CT-014 | Evidence | original immutability | original evidence cannot be overwritten |
| CT-015 | Provenance | legal proposition | missing source reference is rejected where required |
| CT-016 | Audit | sensitive minimization | audit record contains required metadata without unnecessary sensitive payload |

## Execution rule
A test marked passing requires executable evidence in CI or an equivalent reproducible local test run. Documentation alone never marks a contract test complete.
