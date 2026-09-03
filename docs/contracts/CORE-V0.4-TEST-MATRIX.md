# SIDERETH Core v0.4 — Test Matrix

Status: DRAFT

| Area | Required verification |
| --- | --- |
| Content identity | Same bytes produce the same hash; changed bytes produce a different hash |
| Original immutability | Duplicate evidence ID is rejected |
| Integrity | Hash mismatch is rejected and distinguishable from missing data |
| Object boundary | Duplicate storage reference is rejected |
| Authorization | Authorized actor can access; unrelated actor is denied |
| Case isolation | Evidence attached to one case cannot be accessed through another case authorization context |
| Audit | Evidence mutation requires an attributable audit record |
| Retention | Active legal hold prevents deletion |
| Retention | Expired retention can become deletion-eligible only when no hold applies |
| Export | Export ordering is deterministic |
| Export | Export contains identity and integrity metadata needed for recovery |
| Recovery | Restored content is hash-verified |
| Encryption boundary | Key-provider abstraction does not expose key material through evidence metadata |
| Errors | Missing, unauthorized, duplicate and integrity failures remain distinguishable |
| Derived artifacts | Artifact references an existing original before persistence |

Passing unit tests does not prove production encryption, backup durability,
regulatory retention compliance or deployment readiness.
