# SIDERETH Core v0.2 — Test Matrix

Status: IMPLEMENTATION

| Area | Required checks |
| --- | --- |
| Case repository | save, load, missing, duplicate, empty ID |
| Incident repository | save, load, missing, duplicate, empty ID |
| Event repository | validate before append, load, missing, duplicate |
| Authorization | owner allowed, other actor denied, missing case ID denied |
| Audit | valid record stored, missing identity rejected, duplicate ID rejected |
| Isolation | authorization is evaluated before case-scoped storage access |
| Boundary | core compiles without database, network, AI, or transport dependency |

Passing unit tests does not prove production readiness. Production verification must separately test durable storage, concurrent writes, transaction boundaries, cryptographic integrity, authentication, authorization policy composition, audit durability, recovery, and security controls.
