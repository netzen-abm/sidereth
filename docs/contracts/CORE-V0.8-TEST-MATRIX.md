# SIDERETH — Core v0.8 Test Matrix

| Area | Required test |
|---|---|
| Identity | Empty deadline/obligation IDs rejected |
| Procedure | Missing procedure reference rejected |
| Source | Missing legal-source references rejected |
| Duration | Negative/overflow duration rejected |
| Date | Due date inconsistent with anchor + duration rejected |
| Status | Invalid deadline transitions rejected |
| Obligation | Invalid applicability state rejected |
| Deadline | Duplicate deadline IDs rejected |
| Obligation | Duplicate obligation IDs rejected |
| Ordering | Registry IDs returned deterministically |
| Safety | Unverified applicability remains non-conclusive |
| Atomicity | Failed registration leaves registry unchanged |
