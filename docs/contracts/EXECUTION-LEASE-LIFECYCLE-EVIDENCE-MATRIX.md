# SIDERETH — Execution Lease Lifecycle Evidence Matrix

**Status:** CANONICAL / IMPLEMENTATION GATE  
**Scope:** ExecutionLeaseRuntime lifecycle evidence and its integration with existing audit/provenance/persistence boundaries.

## Architectural rule

ExecutionLeaseRuntime owns runtime lifecycle orchestration and provider-handle ownership only. It must not become an audit service, persistence service, authorization engine, or provider implementation.

Lifecycle evidence must flow through the existing boundaries:

`ExecutionLeaseRuntime → AuditProvenanceSink → UnitOfWork → ResourceWrite → durable provider`

Provider handles, OS ownership, and process-local synchronization state are never durable facts.

## Evidence matrix

| ID | Scenario | Required runtime result | Required durable evidence | Safety invariant |
|---|---|---|---|---|
| EL-01 | Authorized activation succeeds | `Activated` | activation audit + provenance | provider handle exists only after provider activation succeeds |
| EL-02 | Normal release succeeds | `Released` | release audit + provenance | lease reaches `Completed → Released` only after provider release succeeds |
| EL-03 | Provider release fails | adapter error | failure evidence; lease remains active | active handle is retained and retry remains possible |
| EL-04 | Release retry succeeds | `Released` | successful release evidence | retained handle is cleared only after confirmed release |
| EL-05 | Revocation while active | `Revoked` or adapter error | revocation + release/failure evidence | future authorization is terminated before physical release |
| EL-06 | Cancellation while active | `Cancelled` or adapter error | cancellation + release/failure evidence | future authorization is terminated before physical release |
| EL-07 | Expiry while active | `Expired` or adapter error | expiry + release/failure evidence | expired authorization cannot be reused |
| EL-08 | Reauthorization after terminal state | fresh authorized lease | new authorization reference and lifecycle evidence | terminal lease is never reused |
| EL-09 | Reauthorization while active | `CannotReauthorize` | no replacement activation | active runtime state cannot be silently replaced |
| EL-10 | Process restart / lost provider handle | no assumption of ownership | durable lifecycle facts only | restart must not claim control of a non-durable provider handle |
| EL-11 | Audit persistence failure | explicit error | no false success | runtime success must not be represented as durable evidence unless the evidence boundary succeeds |
| EL-12 | Provenance persistence failure | explicit error | no partial audit/provenance pair | audit and provenance remain an atomic pair |

## Implementation sequence

1. Prove EL-01–EL-10 at the runtime/evidence boundary without introducing new lifecycle or persistence services.
2. Integrate lifecycle evidence through the existing `AuditProvenanceSink`.
3. Prove EL-11–EL-12 against the existing UnitOfWork rollback/commit contract and PostgreSQL proof matrix.
4. Only then make a production-readiness decision.

## Explicit non-goals

- No `RuntimeAuditService`.
- No `LeasePersistenceService`.
- No second lease model.
- No provider-specific runtime logic in the core.
- No persistence of provider handles.
- No new branch for this work; use the existing durable-gateway implementation lane.
