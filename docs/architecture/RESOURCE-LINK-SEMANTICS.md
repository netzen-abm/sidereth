# SIDERETH ResourceLink Semantics

Status: canonical semantic policy (wire-level class extension still pending)

## Purpose

`ResourceLink` expresses a relationship between two canonical `ResourceRef` values without assuming that every referenced resource is physically present in the same persistence provider.

## Link classes

The universal semantic model distinguishes three classes:

1. **Strong** — both endpoints are required to exist in the authoritative resource graph. A strong link is validated as part of the command transaction and failure prevents commit.
2. **Forward** — the target may be created later. The source remains valid while the target is unresolved; resolution is observable and can be validated when the target is materialized.
3. **External** — the target identifies a resource outside the authoritative SIDERETH store. The link is retained as a reference and must not be treated as a local foreign key.

The current `ResourceLink` wire contract has no class field. Therefore existing links MUST NOT be interpreted as strong references by inference. Until the wire-level class is added, they are compatibility references only.

## Duplicate semantics

Duplicate links are idempotent. Repeating the same `(source, relation, target)` tuple must not create multiple logical relationships. The PostgreSQL adapter implements this with a conflict-free insert.

## Transaction semantics

Links written by an authoritative command are part of the same Unit-of-Work write set as the resources whose relationships they establish. A strong-link validation failure must therefore roll back the complete command.

## Lifecycle / deletion

Deletion semantics are not yet part of the universal core. No implementation may silently cascade, detach, or reinterpret a link on resource deletion. Domain policies must explicitly define lifecycle behavior before destructive operations are introduced.

## Database integrity

The polymorphic link table deliberately does not use blanket SQL foreign keys. Forward and external links cannot safely be represented as ordinary local FKs. Strong-link enforcement belongs at the canonical command/policy boundary.

## Required gate work

Before ResourceLink semantics can be marked production-complete:

- add an explicit wire-level class (or equivalent typed contract);
- define backward-compatible decoding for existing links;
- enforce strong-link existence atomically;
- define forward-link resolution and lifecycle behavior;
- define external-reference validation and trust boundaries;
- add provider-neutral semantic tests and live PostgreSQL integration tests.

This policy is intentionally separate from the Decision Model. The Decision Model gate remains blocked until these semantics and the live transactional proof are complete.
