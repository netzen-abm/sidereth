# Tool Gateway Lifecycle and Reconciliation Contract v0.1

## Purpose

Define the minimum durable lifecycle semantics required for SIDERETH Tool Gateway execution when provider outcome and lifecycle persistence can become disconnected.

This contract is deliberately provider-neutral. It does not grant authorization, approve actions, or define provider-specific retry semantics.

## Canonical lifecycle

An invocation progresses through:

`Claimed → InProgress → Completed | Failed | Unknown`

`Unknown` means the platform cannot safely establish the external outcome from the evidence currently available.

## Safety rules

1. **Unknown is not success.**
2. **Unknown is not failure.**
3. **Unknown MUST NOT be automatically retried merely because the caller wants another attempt.**
4. A provider-side retry is permitted only when the provider contract explicitly establishes that repeating the operation cannot duplicate an external side effect, or when reconciliation establishes that the prior operation did not take effect.
5. Reconciliation MUST use authoritative provider evidence where available; an AI/model inference is not authoritative execution evidence.
6. A restart MUST preserve the durable lifecycle state. Restart itself MUST NOT convert Unknown into a new execution attempt.
7. Failure to durably persist the final outcome after provider dispatch MUST resolve to Unknown, not to a confident Completed or Failed state.
8. Audit/provenance persistence failure after provider dispatch MUST preserve uncertainty.
9. A new idempotency key does not make an unsafe retry of an Unknown operation safe; retry safety is a property of the underlying provider operation.
10. Authorization, capability leases, Action/Approval, and Execution Gate remain independent authority boundaries. Reconciliation does not grant authority.

## Reconciliation model

A reconciliation operation may classify an Unknown invocation only from explicit evidence:

- **Completed** — authoritative evidence establishes that the original operation took effect.
- **Failed** — authoritative evidence establishes that the original operation did not take effect.
- **Unknown** — evidence remains insufficient or contradictory.

Reconciliation MUST NOT infer an outcome from absence of local records alone.

## Retry model

Retry is a separate decision from reconciliation:

`Unknown → reconcile → Completed | Failed | Unknown`

Only an explicitly retry-safe operation may take:

`Unknown → controlled retry`

A controlled retry MUST retain linkage to the original invocation/idempotency identity and MUST be auditable.

## Current implementation evidence

The Tool Gateway test suite proves the critical uncertainty transitions:

- provider success + lifecycle completion persistence failure → Unknown;
- provider failure + audit persistence failure → Unknown;
- provider failure + lifecycle failure persistence → Unknown.

The durable stores already preserve `Unknown` across lifecycle state reads.

## Gate

The lifecycle gate is not complete until:

- restart preserves Unknown;
- reconciliation has explicit evidence-based semantics;
- unsafe automatic retry is rejected;
- retry-safe operations have explicit provider contracts;
- PostgreSQL lifecycle behavior is proven at the exact implementation head;
- Foundation and Security/Supply Chain checks are green.

This contract intentionally does not add a generic automatic retry mechanism. Such a mechanism would create a distributed-systems safety hazard before provider idempotency/reconciliation semantics are explicit.
