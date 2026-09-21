# SIDERETH Shareable Infrastructure Architecture

## Purpose

This document defines the reusable infrastructure boundary for SIDERETH. It is an architectural contract, not a feature roadmap.

## Core rule

**One canonical domain. One canonical control plane. Multiple implementation technologies only behind explicit contracts.**

The reusable layer owns semantics. Applications and technology adapters consume those semantics.

## Canonical shared primitives

1. Identity and subject context
2. Authorization request/result/evaluation
3. Capability and capability lease
4. Policy decision boundary
5. Action and Execution Gate
6. Tool Registry and Tool Gateway
7. Bounded Tool Data Access
8. Idempotency and execution lifecycle
9. Unit of Work and transactional persistence
10. Audit and provenance
11. Event/correlation identity
12. Conformance and security invariants

## Dependency direction

```
Application / Surface
        |
        v
Application Contract
        |
        v
Canonical Control Plane
        |
        +--> Authorization / Policy
        +--> Capability / Lease
        +--> Action / Execution Gate
        +--> Tool Gateway / Data Access
        +--> Audit / Provenance
        +--> Idempotency / Lifecycle
        |
        v
Canonical Domain Services
        |
        v
Persistence / Transport / AI / Decentralized adapters
```

Adapters must not redefine identity, authorization, lifecycle, provenance, audit, or policy semantics.

## Security invariants

- No consequential execution without canonical authorization and the canonical Execution Gate.
- No provider execution before registry/provider binding validation.
- No provider access beyond the invocation-bound data-access grant.
- No external caller can directly claim durable idempotency.
- Provider outcomes that cannot be durably reconciled are represented as indeterminate/Unknown.
- Audit and provenance describe what happened; they do not create authority.
- Sensitive data is minimized before provider execution.
- AI and intelligence components never create authority or approval.

## Shareability boundary

A future external application should be able to consume the reusable layer without importing SIDERETH-specific legal/case business logic.

Target conceptual packages:

```
sidereth-core
sidereth-control-plane
sidereth-domain-services
sidereth-persistence
sidereth-adapters
sidereth-conformance
```

This is a target decomposition, not a mandate to split the repository immediately.

## Technology policy

Rust remains the canonical domain/runtime implementation.

Other languages or runtimes are permitted only when a concrete adapter requirement demonstrates material benefit. They must sit behind stable contracts and must not become alternate semantic authorities.

## Extraction gate

Do not extract packages merely because code can be moved. Extract only after:

1. control-plane convergence is green;
2. protected-data and consequential-execution bypass audit is green;
3. conformance tests cover public contracts;
4. persistence and lifecycle semantics are stable;
5. at least one independent consumer can use the boundary without importing application-specific policy.

## Ecosystem readiness test

SIDERETH is infrastructure-ready when a second application can reuse authorization, capability, execution, provenance, audit, persistence, and conformance without duplicating those primitives.

Until that test is passed, feature growth remains subordinate to contract stabilization.
