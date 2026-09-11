# SIDERETH ResourceLink Semantic Conformance

**Status:** CONTRACTED / implementation gate
**Scope:** Universal Core persistence and cross-resource relationship semantics
**Authority:** `docs/architecture/RESOURCE-LINK-SEMANTICS.md`

## 1. Purpose

This document turns the canonical ResourceLink semantic policy into an implementation and verification gate. It does not create a second relationship model and does not change ownership of Party, Evidence, Action, Event, Case, or other canonical resources.

`ResourceLink` remains the universal reference primitive between canonical `ResourceRef` values.

## 2. Canonical link classes

Every new persisted link MUST have exactly one semantic class:

- **Strong** — both endpoints are authoritative SIDERETH resources and MUST exist when the link is committed.
- **Forward** — the target MAY be unresolved when the source is committed. Resolution must remain observable and must not silently become a strong link.
- **External** — the target is outside the authoritative SIDERETH store. The reference MUST NOT be interpreted as a local foreign key.

No implementation may infer a class from resource type, relation name, or whether a target currently exists.

## 3. Wire compatibility

The current deployed link shape contains:

`source_ref + relation + target_ref`

The current shape predates the explicit class field. Therefore:

1. Existing links MUST remain decodable.
2. Existing links MUST NOT silently acquire Strong semantics merely because their endpoints exist locally.
3. The compatibility representation MUST be explicitly documented and versioned before a class-bearing wire representation is persisted.
4. New class-bearing links MUST round-trip without loss of class semantics.
5. Migration MUST NOT rewrite historical link meaning by inference.

## 4. Validation invariants

All links MUST satisfy:

1. source reference is valid;
2. target reference is valid;
3. relation is non-empty after trimming;
4. class is explicit for the new representation;
5. Strong links require authoritative endpoint existence at the command transaction boundary;
6. Forward links may reference an unresolved target;
7. External links retain their external boundary and MUST NOT be treated as local foreign keys;
8. duplicate `(source, relation, target, class)` writes are idempotent;
9. conflicting semantic-class writes for the same logical relationship are rejected rather than silently merged;
10. link provenance and authorization remain governed by the calling command/policy boundary.

## 5. Transaction semantics

Strong-link existence validation MUST occur atomically with the authoritative command that creates the relationship.

A failed Strong-link validation MUST roll back the complete command write set.

Forward and External links MUST NOT require local foreign-key enforcement.

The PostgreSQL adapter is an implementation of the provider-neutral contract; SQL schema details MUST NOT redefine the domain semantics.

## 6. Lifecycle semantics

Resource deletion semantics remain outside the initial implementation gate.

Until a separate lifecycle policy exists:

- no automatic cascade;
- no automatic detach;
- no automatic conversion between Strong, Forward, and External;
- no deletion-time reinterpretation of a link.

## 7. Required conformance tests

### RL-001 — Explicit class

New links carry an explicit semantic class.

### RL-002 — Round trip

Strong, Forward, and External links serialize and deserialize without semantic loss.

### RL-003 — Legacy compatibility

A pre-class link remains decodable without being inferred as Strong.

### RL-004 — Strong endpoint existence

A Strong link to a missing endpoint is rejected atomically.

### RL-005 — Forward unresolved target

A Forward link may be committed while its target is unresolved.

### RL-006 — External boundary

An External link does not require a local target record and is never validated as a local foreign key.

### RL-007 — Duplicate idempotency

Repeating the same link does not create multiple logical relationships.

### RL-008 — Class conflict

The same logical relationship cannot silently change semantic class through a duplicate write.

### RL-009 — Transaction rollback

A Strong-link validation failure leaves the complete authoritative Unit-of-Work write set unchanged.

### RL-010 — Provider neutrality

Semantic behavior is identical at the provider-neutral contract boundary and the PostgreSQL adapter boundary.

### RL-011 — Authorization boundary

ResourceLink persistence does not itself grant authorization or execution permission.

### RL-012 — Provenance boundary

Link persistence does not manufacture provenance; provenance remains supplied and validated by the authoritative command/policy boundary.

## 8. Implementation boundary

The next implementation MUST be limited to:

- the canonical ResourceLink type/serialization contract;
- backward-compatible decoding;
- provider-neutral validation;
- PostgreSQL persistence required by the new semantics;
- bounded conformance tests;
- live PostgreSQL proof for Strong/Forward/External behavior and duplicate/conflict semantics.

It MUST NOT introduce:

- a new generic relationship entity;
- a Longitudinal Record entity;
- domain-specific responsibility graphs;
- AI/agent execution logic;
- MCP/provider-specific semantics;
- a second authorization engine.

## 9. Gate

ResourceLink semantic work is complete only when RL-001 through RL-012 have evidence, the exact-head Foundation workflow is green, Security/Supply Chain is green, and the live PostgreSQL proof is green.

Until then, the Decision Model gate remains blocked for cross-resource reference semantics.
