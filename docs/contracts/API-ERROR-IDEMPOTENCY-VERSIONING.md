# SIDERETH — API Error, Idempotency and Versioning Contract

Status: Draft for Gate 2 verification

## 1. Error model
Every API error has a stable machine-readable code and a safe human-readable message.

Minimum envelope:
- `error.code`
- `error.message`
- `error.category`
- `error.retryable`
- `error.correlation_id`
- `error.details` (only non-sensitive, schema-defined details)

Categories:
- `VALIDATION_ERROR`
- `AUTHENTICATION_REQUIRED`
- `AUTHORIZATION_DENIED`
- `NOT_FOUND`
- `CONFLICT`
- `PRECONDITION_FAILED`
- `RATE_LIMITED`
- `DEPENDENCY_UNAVAILABLE`
- `SOURCE_UNVERIFIED`
- `POLICY_BLOCKED`
- `INTERNAL_ERROR`

Security rule: errors must not expose secrets, private case data, internal credentials, stack traces, or authorization-sensitive existence information.

## 2. Idempotency
Client-requested mutating operations must accept an idempotency key where replay could duplicate an action.

Required semantics:
1. Key is scoped to authenticated subject and operation class.
2. Same key + same request fingerprint returns the original result.
3. Same key + different request fingerprint returns `CONFLICT`.
4. An idempotency record has a bounded retention period appropriate to the operation.
5. High-impact operations additionally require an approval record and immutable audit event.

## 3. API versioning
Public API contracts use explicit major versions, beginning with `/v1`.

Rules:
- additive, backward-compatible fields may be introduced within a major version;
- breaking semantic or structural changes require a new major version;
- deprecated fields receive documented migration guidance and a retirement date;
- event schemas are independently versioned;
- persisted records retain the schema version used to create them;
- readers should support explicitly documented prior versions during migration windows.

## 4. Concurrency
Mutating aggregate operations should use optimistic concurrency through an aggregate version or equivalent precondition. Stale writes must fail deterministically rather than silently overwrite newer state.

## 5. Correlation
Requests, domain events, background jobs, tool invocations and audit records should propagate `correlation_id`; causally related operations should additionally carry `causation_id`.

## 6. No hidden legal decisions
An API error or policy denial must never be represented as a legal conclusion. Where a legal source cannot be verified, the API should expose an uncertainty/source-status condition rather than inventing authority.
