# SIDERETH — Canonical Read/Query Boundary

**Status:** DESIGNED  
**Authority:** Focused architecture specification  
**Scope:** Provider-neutral read/query semantics for canonical SIDERETH resources and derived projections

## 1. Purpose

SIDERETH needs one reusable read/query boundary through which surfaces, services, projections and adapters can retrieve canonical resources without creating domain-specific read repositories for every resource type.

The read/query boundary is infrastructure. It does not replace domain contracts, authorization policy, projection semantics, or persistence adapters.

## 2. Canonical principle

> **Query is discovery of canonical state; projection is composition of canonical state. Neither creates authority.**

The boundary MUST preserve canonical resource identity, schema version, revision and payload semantics. It MUST NOT reinterpret a payload into a stronger legal, epistemic, evidentiary or responsibility claim.

## 3. Existing foundation

The current provider-neutral persistence boundary already exposes `read_resource(ResourceRef)` inside `UnitOfWorkContext`. This remains the authoritative transactional read primitive for command workflows.

The query boundary defined here is the read-side extension for non-mutating retrieval and composition. It MUST reuse the existing `ResourceRef`, `ResourceRecord`, persistence error model, authorization boundary and provider adapter model rather than creating parallel domain repositories.

## 4. Query model

A canonical query consists of:

- explicit resource type or supported resource class;
- optional subject/resource scope;
- optional exact resource identity;
- optional relationship scope where supported;
- optional temporal bounds using canonical source timestamps;
- optional pagination/cursor;
- explicit result limit;
- authorization context;
- requested data classification/sensitivity scope;
- correlation identifier where applicable.

A query MUST be deterministic for the same canonical snapshot, authorization context and query parameters.

## 5. Result model

A query result SHOULD expose:

- canonical `ResourceRef`;
- schema version;
- revision where applicable;
- canonical payload;
- result ordering metadata when ordering is requested;
- continuation cursor when more results exist;
- projection/read-model freshness metadata when the result comes from a materialized projection.

The result MUST distinguish:

- no matching resources;
- resources unavailable because of authorization/policy;
- unavailable or stale read infrastructure;
- incomplete materialized projection.

A filtered result MUST NOT be interpreted as proof that excluded resources do not exist.

## 6. Identity and integrity

Queries MUST operate on typed `ResourceRef` identities. A bare ID without an explicit resource type is insufficient at the canonical boundary.

Readers MUST NOT silently coerce a resource into another resource type.

Returned schema versions MUST remain visible to callers. Reader compatibility with prior versions is an explicit implementation concern governed by versioning policy.

## 7. Authorization and privacy

Read access MUST pass through the canonical authorization/policy boundary before protected data is disclosed.

Cross-case access remains denied by default.

A query that combines otherwise individually accessible resources can create a more sensitive composite view. Authorization and data-classification policy therefore applies to the resulting query/projection scope, not merely to each source record independently.

Query infrastructure MUST NOT expose protected existence information through errors, counts, timing-sensitive behavior, cursors or relationship traversal.

## 8. Ordering and temporal semantics

Ordering is a read concern and MUST NOT rewrite source timestamps.

When multiple timestamps exist, the query contract MUST name the timestamp used for ordering. It MUST NOT silently substitute `recorded_at` for `observed_at`, or an event occurrence time for another temporal meaning.

Stable resource identity MUST be available as a deterministic tie-breaker when ordering values are equal.

## 9. Pagination

Large result sets SHOULD use bounded pagination.

Cursors MUST be opaque to callers and bound to the query semantics necessary to prevent accidental cross-query reuse. Cursor design MUST prevent a caller from changing authorization scope or filters while reusing a cursor created under different semantics.

Offset pagination may be used for implementation-specific low-volume cases but MUST NOT be treated as the canonical consistency mechanism.

## 10. Query versus projection

A direct query retrieves canonical resources.

A projection composes canonical resources into a derived read model such as `LongitudinalView`.

The two boundaries MUST remain distinct:

```text
Canonical resources
       │
       ├── Direct Query ───────► Resource results
       │
       └── Projection ─────────► Derived read model
                                  │
                                  └── may use Query internally
```

A projection MUST NOT become a write authority or alternative source of truth.

## 11. Relationship traversal

Relationship traversal MAY be supported as a query capability, but it MUST use canonical `ResourceLink` semantics and explicit relationship types.

Traversal MUST NOT infer `Strong` semantics from legacy/class-less links.

Traversal MUST NOT collapse distinct roles such as:

`Responsible Party != Nearest Authority != Geographic Authority != Asset Owner != Contractor != Contracting Officer`

Relationship candidates with inferred/probable status MUST retain that status.

## 12. Evidence and provenance

Query results involving Evidence MUST preserve the distinction between original evidence and derived artifacts.

A query or projection may expose references to evidence, but it MUST NOT manufacture provenance, upgrade integrity status, or replace an original artifact with a generated summary.

## 13. AI and agents

AI and agents may consume query results subject to authorization and data classification. They may use query capabilities for retrieval, extraction, classification, summarization, clustering and candidate linking.

Query infrastructure MUST NOT grant AI authority to establish truth, responsibility, legal effect, approval or execution rights.

AI-generated query parameters MUST be treated as untrusted input until validated by the same canonical query and policy boundaries as human-generated parameters.

## 14. Failure and consistency

A query failure MUST be distinguishable from an empty result.

A stale or unavailable projection MUST NOT be represented as absence of canonical records.

If a query spans multiple resource classes, implementations MUST document their consistency boundary. A result MUST NOT imply an atomic snapshot across providers when such a snapshot does not exist.

## 15. Provider neutrality

The canonical query boundary MUST NOT depend on PostgreSQL, a specific search engine, graph database, vector database, cloud provider, transport, framework or programming language.

Adapters may implement the boundary using SQL, indexes, embedded stores, graph traversal, local files or other mechanisms where justified, provided they preserve the canonical semantics.

## 16. Security requirements

At minimum, conformance MUST verify:

1. typed resource identity is enforced;
2. authorization is evaluated before protected disclosure;
3. cross-case access is denied by default;
4. data classification is respected;
5. filtered results do not leak protected existence information;
6. ordering does not rewrite source timestamps;
7. schema versions remain visible;
8. cursors cannot cross authorization/query scopes;
9. projection staleness is distinguishable from empty results;
10. canonical source records remain authoritative.

## 17. Implementation gate

This document defines architecture. It does not claim that a standalone read/query service or query API is implemented.

The first implementation should be the smallest reusable provider-neutral query capability that can support direct resource retrieval and the longitudinal projection without introducing domain-specific repositories.
