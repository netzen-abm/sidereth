# SIDERETH ResourceLink Semantics

Status: canonical semantic policy with compatibility-safe wire-level class extension

## Purpose

`ResourceLink` expresses a relationship between two canonical `ResourceRef` values without assuming that every referenced resource is physically present in the same persistence provider.

## Link classes

The universal semantic model distinguishes three classes:

1. **Strong** — an internally authoritative relationship whose target is expected to be resolvable when the applicable command policy requires it.
2. **Forward** — the target may be created later. The source remains valid while the target is unresolved; resolution is observable and can be validated when the target is materialized.
3. **External** — the relationship is to a target outside the authoritative SIDERETH resource graph and therefore receives no internal referential-integrity guarantee.

`ResourceLinkClass` is serialized as `strong`, `forward`, or `external`. The compatibility constructor `ResourceLink::new(...)` defaults to `strong`, while `ResourceLink::new_with_class(...)` and the `strong`, `forward`, and `external` helpers make intent explicit.

Legacy serialized links without `class` decode as `strong` to preserve source and wire compatibility. This default is a compatibility interpretation of existing links, not evidence that their targets were historically validated.

## Duplicate semantics

The logical identity of a link is `(source, relation, target)`. Repeating the same tuple is idempotent. A class is semantic metadata, not part of link identity; the same logical link must not exist simultaneously with conflicting classes.

The PostgreSQL adapter preserves this identity with a primary key and conflict-free insert. A later hardening step must make conflicting-class attempts observable rather than silently accepted.

## Transaction semantics

Links written by an authoritative command are part of the same Unit-of-Work write set as the resources whose relationships they establish. A policy-level strong-link validation failure must therefore roll back the complete command.

## Lifecycle / deletion

Deletion semantics are not yet part of the universal core. No implementation may silently cascade, detach, or reinterpret a link on resource deletion. Domain policies must explicitly define lifecycle behavior before destructive operations are introduced.

## Database integrity

The polymorphic link table deliberately does not use blanket SQL foreign keys. Forward and external links cannot safely be represented as ordinary local FKs. Strong-link enforcement belongs at the canonical command/policy boundary.

## External references

`ResourceRef` currently represents SIDERETH resource types. Therefore `External` is presently a relationship class, not a new URL/URI target type. A future external-reference contract may introduce a dedicated opaque URI/reference target without changing the internal `ResourceRef` identity model.

## Remaining gate work

ResourceLink semantics are now represented in the universal wire contract and persisted by PostgreSQL. Before marking the semantics production-complete, the ecosystem still needs:

- atomic strong-link target validation;
- explicit forward-link resolution policy;
- explicit external-reference validation and trust boundaries;
- provider-neutral semantic tests for duplicate/conflicting classes;
- live PostgreSQL tests for link persistence and transactional rollback.

This policy is intentionally separate from the Decision Model. The Decision Model gate remains blocked until the remaining transactional and semantic proof work is complete.