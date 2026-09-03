# SIDERETH Core v1.3 — Test Matrix

Status: DRAFT

| Area | Required proof |
|---|---|
| Case service | create, retrieve and valid transition |
| Incident service | create, retrieve and valid transition |
| Event service | valid append and duplicate rejection |
| Authorization | protected access denied without authorization |
| Actor identity | every mutation identifies actor and operation |
| Audit | successful protected mutation produces attributable audit record |
| Domain invariant | invalid state transition cannot persist |
| Idempotency | repeated mutation does not duplicate the operation |
| Error boundary | typed errors contain no provider-specific type |
| Provider neutrality | service compiles against repository contracts only |
| High-impact boundary | approval-required action cannot silently execute |
| Determinism | same valid input yields same domain result |
