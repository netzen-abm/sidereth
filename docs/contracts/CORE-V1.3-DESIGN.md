# SIDERETH Core v1.3 — Domain Service & Policy Boundary

Status: DRAFT

## Purpose

Create the application/domain service boundary that prevents surfaces, agents and adapters from manipulating repositories directly.

## Canonical flow

Surface / Agent / API → Application Service → Authorization + Policy → Domain Invariants → Repository Contract → Persistence Adapter → Provider

## Design principles

1. Services own use-case orchestration, not persistence providers.
2. Authorization is evaluated before protected repository access.
3. Domain invariants remain deterministic and provider-neutral.
4. Services accept domain inputs and return domain/service results, not SQL, HTTP, cloud SDK or UI types.
5. Mutations carry an attributable actor identity and operation identity.
6. High-impact operations expose an explicit approval boundary rather than silently communicating or filing.
7. Read and write capabilities are explicit; deny-by-default applies to protected case data.
8. Idempotency is part of mutation semantics where repeated requests are possible.
9. Errors are typed and machine-readable.
10. Adapters remain replaceable without rewriting service logic.

## Initial scope

- Case creation and state transition.
- Incident creation and state transition.
- Event append through the service boundary.
- Authorization before protected access.
- Audit record generation after authorization and successful domain mutation.
- Explicit actor and operation identity.
- Deterministic service errors.

## Explicit non-goals

- HTTP or transport APIs.
- Authentication provider implementation.
- AI/agent implementation.
- Government submission.
- Autonomous legal judgment.
- Provider-specific transactions.
- Notification infrastructure.

## Architectural test

A service implementation fails this contract if replacing the repository implementation, storage provider, transport surface or AI runtime requires changing legal/domain rules.
