# SIDERETH — Execution Lease Reconciliation and Ambiguous Outcome Contract v0.1

**Status:** CANONICAL / IMPLEMENTATION SAFETY CONTRACT  
**Scope:** ExecutionLeaseRuntime when provider-side ownership and local runtime/evidence state can become disconnected.

## Purpose

Define the fail-closed boundary for execution leases when a provider operation may have occurred but the process cannot prove the final local runtime or evidence state.

This contract does **not** introduce a recovery service, lease persistence service, provider-specific logic, or a second lease model.

## Authority boundary

| Fact | Authoritative boundary |
|---|---|
| Authorization, purpose, scope, subject, actor | canonical CapabilityLease / authorization boundary |
| Physical provider handle ownership during a live process | ExecutionLeaseRuntime + provider adapter |
| Durable audit/provenance | existing AuditProvenanceSink + UnitOfWork |
| Provider-side outcome after local uncertainty | provider's authoritative reconciliation mechanism, when one exists |

A durable lifecycle record MUST NOT be treated as proof that a process still owns a provider handle.

## Ambiguous-outcome rule

If the provider operation may have taken effect but the local process cannot establish the final outcome, the outcome is **indeterminate**.

The platform MUST:
1. fail closed;
2. preserve the last durable facts without inventing provider ownership;
3. avoid blind retry;
4. avoid converting uncertainty into `Completed` or `Failed` without authoritative evidence;
5. require provider-authoritative reconciliation before treating the external resource as resolved, when such reconciliation is available.

`Unknown` is an epistemic state, not a permission state.

## Required transitions

### Provider release fails while the process is alive

`Active → release attempt fails → Active`

The live provider handle remains retained so a controlled retry can occur.

### Provider release succeeds but local completion/evidence cannot be durably established

The external provider outcome MAY be successful, but the platform MUST NOT infer that fact from local state alone after restart.

If the process restarts without a live provider handle, an orphaned persisted `Active` lease MUST be rejected by `ExecutionLeaseRuntime::new()`.

This is a deliberate fail-closed invariant.

### Provider outcome is authoritative and externally queryable

Reconciliation may classify the outcome as:

`Unknown → Completed`

or

`Unknown → Failed`

Only authoritative provider evidence may make that classification.

### Provider outcome remains unresolved

`Unknown → Unknown`

No automatic retry is permitted merely because the outcome is unresolved.

## Evidence boundary

Lifecycle evidence remains a separate responsibility:

`ExecutionLeaseRuntime → AuditProvenanceLifecycleSink → AuditProvenanceSink → UnitOfWork`

Evidence persistence failure MUST NOT mutate runtime state or imply that provider execution and persistence were atomic.

## Retry boundary

Retry is not part of lease recovery.

A retry is allowed only when the provider contract establishes that the operation is safe to repeat, or authoritative reconciliation establishes that the prior operation did not take effect.

A fresh authorization/lease is required for a new execution attempt; a new lease MUST NOT be used to conceal an unresolved external outcome.

## Recovery non-goals

This contract intentionally prohibits:
- `ExecutionLeaseRecoveryService`
- `LeasePersistenceService`
- provider handles in durable storage
- reconstructing provider ownership from an `Active` durable lease
- blind retry of ambiguous operations
- model/AI inference as execution evidence
- coupling runtime state transitions to durable persistence transactions

## Verification gate

Production readiness for execution leases requires evidence for:
- provider release failure retains the live handle;
- release retry succeeds only with the retained handle;
- lifecycle evidence persistence failure does not create false runtime success;
- restart rejects orphaned persisted `Active` lease state;
- authoritative provider reconciliation is explicit where a provider supports it;
- no unsafe automatic retry path exists.

