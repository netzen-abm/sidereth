# SIDERETH Core v0.2 — Persistence, Authorization and Audit

Status: IMPLEMENTATION

## Purpose

Core v0.2 adds explicit boundaries for persistence, case access control, and audit recording without coupling the domain to a database or transport layer.

## Architecture

```text
Domain objects
    |
    +--> Repository traits --> storage adapters
    |
    +--> Authorization policy
    |
    +--> Audit sink --> durable audit adapter
```

The domain remains deterministic. Database choice, API transport, AI, and network adapters remain outside the core domain.

## Repository boundary

Repositories provide minimal read/write contracts for Case, Incident, and Event.
The v0.2 in-memory adapter is a test/reference implementation only.

Rules:

- missing records return `None` rather than synthetic objects;
- duplicate Case and Incident identities are rejected;
- duplicate Event identities are rejected;
- events are validated before append;
- storage implementation is replaceable without changing domain types.

## Authorization boundary

Every case-scoped operation must pass an explicit authorization policy before the operation reaches a storage adapter.

The current policy demonstrates owner isolation. It is deliberately small and does not claim to implement the production role, delegation, professional-access, or service-to-service model.

Production requirements remain:

- deny by default;
- explicit actor identity;
- case/resource scope;
- least privilege;
- service and agent identities;
- delegation and expiry where applicable;
- policy decision audit;
- no adapter bypass.

## Audit boundary

Mutations must produce attributable audit records. The v0.2 in-memory sink establishes the contract and duplicate audit identity protection.

Production audit storage must additionally address append-only semantics, durable persistence, integrity protection, access control, retention, privacy minimisation, and operational recovery.

## Approval boundary

High-impact legal actions remain outside these low-level primitives. They require an explicit approval record and policy enforcement before execution. v0.2 does not expose autonomous legal action APIs.

## Scope limit

This increment does not claim production persistence, encryption, API authentication, durable audit storage, role-based authorization, or regulatory/legal correctness. Those require dedicated implementation and verification gates.
