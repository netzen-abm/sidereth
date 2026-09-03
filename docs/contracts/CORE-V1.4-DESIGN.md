# SIDERETH Core v1.4 — Event & Workflow Command Boundary

**Status:** DRAFT

## Purpose

Introduce a canonical command execution boundary between an intent and its durable domain effects.

## Canonical flow

**Intent → Authorization → Validation → Mutation → Event → Audit → Result**

The command boundary owns orchestration. Domain invariants remain in the domain model. Persistence remains behind provider-neutral contracts.

## First implementation slice

The first slice covers case creation and case state transition.

A command carries:

- actor identity
- operation identity
- correlation identity
- explicit command intent
- expected revision for optimistic concurrency where applicable

The executor must:

1. identify the target aggregate and required access action;
2. authorize before claiming idempotency or mutating state;
3. claim the operation identity through the idempotency contract;
4. validate domain invariants;
5. mutate through the repository contract;
6. append an attributable domain event;
7. record an attributable audit entry;
8. return a deterministic command result.

## Failure semantics

v1.4 does **not** claim distributed exactly-once semantics. Case state, event storage, audit storage, and idempotency may be different providers.

If a mutation fails before a durable state change is confirmed, the service returns the typed persistence/service error. The idempotency claim is intentionally retained because the outcome may be ambiguous at the provider boundary; callers must reconcile the operation rather than blindly retrying the same mutation.

If state mutation succeeds but event append fails, the command returns a persistence error and the system may contain a committed state without its corresponding event. This is an explicit partial-commit condition, not a hidden success. A later transactional command boundary must close this gap.

If state and event succeed but audit recording fails, the domain mutation remains committed and the command returns `AuditFailure`. The operation must be reconciled from durable state/event records rather than retried blindly.

These semantics are deliberately conservative until a provider-neutral transaction contract has executable atomicity guarantees.

## Provider neutrality

The command layer must not import SQL, HTTP, cloud SDK, database ORM, hosted-service, AI-framework, or transport-specific types.

The architecture remains:

```text
Surface / Agent / API
        ↓
Command / Application Service
        ↓
Authorization + Policy
        ↓
Domain Invariants
        ↓
Repository Contracts
        ↓
Persistence Adapters
        ↓
Providers
```

## Explicit non-goals

- distributed transactions
- exactly-once delivery
- autonomous legal judgment
- government communication
- authentication providers
- transport APIs
- AI implementation
- notifications
- incident orchestration
- litigation strategy

## Next boundary

The next persistence evolution should provide a provider-neutral atomic command transaction capable of committing state mutation, event append, and idempotency outcome together where the selected provider supports it. Providers that cannot provide this guarantee must remain explicitly scoped rather than pretending to provide stronger semantics.
