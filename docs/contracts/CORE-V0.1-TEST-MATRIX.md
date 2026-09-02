# SIDERETH Core v0.1 — Executable Test Matrix

**Status:** CANONICAL DEVELOPMENT GATE
**Scope:** Case → Incident → Event → Timeline foundation

## Objective

Prove the deterministic domain foundation before adding storage, adapters, AI, networking, or domain-specific legal modules.

## Invariants under test

| ID | Invariant | Required evidence |
|---|---|---|
| C01 | A Case starts in a valid initial state | Unit test |
| C02 | Case lifecycle rejects invalid transitions | Unit test |
| C03 | An Incident may exist before case linkage | Unit test |
| C04 | Incident lifecycle rejects skipped transitions | Unit test |
| E01 | Events identify aggregate, actor, schema and causation | Type-level contract |
| E02 | Event timestamps and IDs are explicit fields | Type-level contract |
| E03 | Source references are explicit and separate from payload | Type-level contract |
| T01 | Timeline ordering is deterministic | Planned unit test |
| T02 | Timeline preserves original event identity | Planned unit test |
| P01 | Mutations are attributable | Contract + future integration test |
| P02 | Cross-case access is denied by default | Policy integration test |
| A01 | High-impact mutations require approval | Authorization integration test |
| V01 | Domain objects carry schema versioning | Contract/schema test |

## Current implementation gate

The current Rust foundation implements C01–C04 and provides the `EventEnvelope` contract needed for the next increment. It does not yet claim persistence, cryptographic evidence integrity, authorization enforcement, or a materialized timeline service.

## Required sequence

1. Strengthen domain types and transition tests.
2. Add explicit event construction/validation.
3. Add deterministic timeline projection.
4. Add evidence/provenance references without overwriting originals.
5. Add policy/authorization enforcement at the service boundary.
6. Add persistence only after domain behavior is stable.

## Negative testing requirements

The implementation must test at minimum:

- invalid state transitions;
- duplicate event IDs;
- missing aggregate identity;
- unsupported schema versions;
- events with missing actor identity;
- cross-case reference attempts;
- mutation without required approval;
- attempts to replace an immutable evidence original.

## Definition of done

Core v0.1 is not complete merely because `cargo test` passes. Completion requires:

- deterministic domain behavior;
- explicit invariants;
- executable positive and negative tests;
- no AI/network dependency in the core;
- no legal conclusion engine in the core;
- provenance-ready event model;
- authorization-ready mutation boundaries;
- documentation matching the implemented code.
