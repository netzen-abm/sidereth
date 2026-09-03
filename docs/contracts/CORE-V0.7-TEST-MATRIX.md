# SIDERETH — Core v0.7 Procedure Test Matrix

| Area | Required coverage |
|---|---|
| Procedure identity | Valid procedure; empty ID; empty name |
| Procedure scope | Missing jurisdiction; missing authority |
| Provenance | Missing legal-source references |
| Step identity | Empty step ID; duplicate step ID |
| Step sequence | Zero sequence; duplicate sequence |
| Step ownership | Missing authority; unknown authority |
| Prerequisites | Unknown step; cross-procedure step; forward dependency; cycle |
| Registry | Duplicate procedure ID; deterministic procedure IDs |
| Step ordering | Stable sequence ordering |
| Atomicity | Failed insert leaves registry unchanged |
| Safety boundary | Model stores procedural structure without asserting lawfulness |

## Acceptance criteria

- Invalid procedure definitions are rejected deterministically.
- Invalid steps are rejected deterministically.
- Cross-procedure references are rejected.
- Dependency cycles are rejected.
- Failed registry insertion does not leave partial state.
- Registry output is stable across insertion order.
- No deadline calculation is introduced in v0.7.
- No AI, network, UI, or production data dependency is required.
