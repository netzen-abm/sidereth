# SIDERETH — Core v0.6 Test Matrix

Status: DRAFT / CORE V0.6

| Area | Positive case | Negative case |
| --- | --- | --- |
| Jurisdiction identity | stable ID accepted | empty ID rejected |
| Jurisdiction type | supported type accepted | unsupported type impossible by enum |
| Jurisdiction name | non-empty name accepted | empty name rejected |
| Parent relationship | valid parent accepted | self-parent rejected |
| Hierarchy | acyclic hierarchy accepted | cycle rejected |
| Authority identity | stable ID accepted | empty ID rejected |
| Authority type | supported type accepted | unsupported type impossible by enum |
| Authority jurisdiction | valid reference accepted | missing reference rejected |
| Power identity | unique ID accepted | duplicate ID rejected |
| Power ownership | authority reference accepted | missing authority rejected |
| Power scope | jurisdiction reference accepted | missing jurisdiction rejected |
| Power provenance | source reference accepted | no source reference rejected |
| Registry | unique records accepted | duplicate records rejected |
| Ordering | IDs returned sorted | nondeterministic order not permitted |
| Separation | jurisdiction and authority remain distinct | authority does not become jurisdiction implicitly |
| Safety | incomplete relationship remains unresolved | missing record does not imply illegality |

## Interpretation boundary

Passing this matrix proves deterministic validation of domain primitives only.
It does not prove that a real-world authority exists, that a source is authentic,
that a power is currently in force, or that an official action was lawful.
