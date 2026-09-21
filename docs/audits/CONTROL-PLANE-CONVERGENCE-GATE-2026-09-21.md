# SIDERETH Control-Plane Convergence Gate — 2026-09-21

## Purpose

This gate records the current state after the September 21 repository convergence work. It supersedes stale assumptions in earlier Tool Gateway audit prose where the implementation has since advanced.

## Current verified state

### Repository hygiene

- Active branches: exactly 9.
- Open pull requests: 0 after PR #130 merge.
- Nine-branch enforcement workflow: passing on current `main`.
- Direct changes to protected `main` remain prohibited; changes proceed through pull requests.

### Authorization

The canonical runtime path is:

`AuthorizationRequest → AuthorizationEvaluator → AuthorizationResult → validate_authorization → contextual binding → authoritative execution/UoW → audit/provenance`

Repository search found the canonical evaluator and enforcement boundary, with no active `VaultAuthorizer` implementation.

The legacy AuthorizationPolicy/AccessRequest API has been retired from active exports/implementation and archived as historical disposition material.

### Tool Gateway P0 convergence

Current `src/tool_gateway.rs` now contains:

1. explicit `capability_ref` binding;
2. optional `function_ref` binding;
3. registry-selected `implementation_id`;
4. registry-selected `provider_id`;
5. implementation-version binding;
6. provider-object identity validation;
7. canonical Action/Approval/ExecutionGate enforcement;
8. invocation audit/provenance emission;
9. validated-phase internal claim transition;
10. durable Unknown fallback when audit/lifecycle completion cannot be established;
11. invocation-bound data-access grant.

Adversarial tests in the same module cover forged capability/provider/version contexts, provider-object mismatch, missing canonical execution context, non-human approval, duplicate operation claims, lease omission, unsupported constraints, and scope mismatch.

Therefore the earlier September 20 Tool Gateway audit items TG-01 through TG-04 are **implemented in current main**, although the complete Tool Gateway production gate remains subject to the contract's full conformance matrix and exact-head security evidence.

## Remaining P0/P1 gates

### P0 — ResourceLink wire convergence

The semantic model is canonical, but the wire-level semantic-class extension remains an explicit implementation gate.

Required before production completion:

- Strong / Forward / External wire representation;
- backward-compatible decoding;
- no inferred Strong semantics for legacy links;
- atomic Strong endpoint validation;
- duplicate/class-conflict semantics;
- live PostgreSQL proof;
- provider-neutral conformance tests.

### P1 — Tool Gateway lifecycle proof

The implementation now records Unknown on audit/lifecycle ambiguity. Remaining proof should cover:

- provider success + lifecycle persistence failure;
- provider failure + audit failure;
- restart/reconciliation of Unknown;
- retry semantics that cannot duplicate external side effects;
- complete TG-001–TG-060 evidence matrix.

### P1 — Independent consumer proof

The shared infrastructure is not considered extraction-ready merely because contracts exist.

An independent consumer must be able to consume the canonical control-plane primitives without importing SIDERETH application-specific authorization or policy semantics.

## Canonical architecture

```text
Identity / Subject Context
        ↓
Authorization / Policy
        ↓
Capability + Purpose-Bound Lease
        ↓
Capability → Function → Tool
        ↓
Action / Approval
        ↓
Execution Gate
        ↓
Tool Data Access Grant
        ↓
Durable Idempotency
        ↓
Provider / Implementation Adapter
        ↓
Canonical State / Unit of Work
        ↓
Evidence / Provenance / Audit
        ↓
Independent Surfaces
```

No adapter, AI model, MCP server, database, provider, or surface may become a competing semantic authority.

## Extraction rule

Do not create SDKs/packages or duplicate frameworks until:

1. the control-plane conformance gate is green;
2. ResourceLink wire semantics are stable;
3. Tool Gateway conformance evidence is complete;
4. transactional lifecycle semantics are proven;
5. an independent consumer passes the same security/conformance suite.

## Decision

The next implementation target is **not broader ecosystem functionality**.

It is:

**ResourceLink wire/conformance closure + Tool Gateway lifecycle adversarial proof + independent-consumer conformance harness.**

This preserves the architecture's central objective: build reusable infrastructure once, then allow the wider ecosystem to compose it without reproducing its authority, security, evidence, or persistence semantics.
