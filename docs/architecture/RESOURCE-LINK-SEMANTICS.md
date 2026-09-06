# SIDERETH ResourceLink Semantics

Status: canonical contract proposal

## Purpose

`ResourceLink` expresses a relationship between two canonical `ResourceRef` values without assuming that every referenced resource is physically present in the same persistence provider.

## Link classes

Every link has one of three semantic classes:

1. **Strong** — both endpoints are required to exist in the authoritative resource graph. A strong link is validated as part of the command transaction and failure prevents commit.
2. **Forward** — the target may be created later. The source remains valid while the target is unresolved; resolution is observable and can be validated when the target is materialized.
3. **External** — the target identifies a resource outside the authoritative SIDERETH store. The link is retained as a reference and must not be treated as a local foreign key.

The current wire contract does not yet add a class field. Until that field is introduced, existing `ResourceLink` instances are treated as **forward-compatible references**, and authoritative workflows must not infer strong referential integrity from their presence.

## Duplicate semantics

Duplicate links are idempotent. Repeating the same `(source, relation, target)` tuple must not create multiple logical relationships.

## Transaction semantics

Links written by an authoritative command are part of the same Unit-of-Work write set as the resources whose relationships they establish. A strong link failure therefore rolls back the complete command.

## Deletion / lifecycle

Deletion semantics are not yet part of the universal core. No implementation may silently cascade, detach, or reinterpret a link on resource deletion. Domain policies must explicitly define lifecycle behavior before destructive operations are introduced.

## Compatibility rule

The polymorphic link table deliberately does not use ordinary SQL foreign keys because targets can be forward or external references and may span resource types. Strong-link enforcement belongs at the canonical command/policy boundary, not in a blanket database FK.

## Required future contract extension

Before declaring ResourceLink semantics production-complete, add an explicit wire-level link class (or equivalent contract) and define validation, resolution, lifecycle, and authorization behavior for each class. Existing links must remain backward-readable during migration.
