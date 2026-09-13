# Longitudinal Projection

**Status:** DESIGNED  
**Authority:** Architecture specification  
**Depends on:** D-032 Observation Semantics; canonical Event, Evidence, ResourceLink, Action, Response, Resolution and Outcome semantics where implemented.

## 1. Purpose

SIDERETH needs a longitudinal way to understand how a case, subject, resource, or matter develops over time without introducing a second domain model that duplicates the canonical primitives.

The longitudinal view is therefore a **projection/composition**, not a new aggregate.

Canonical composition:

`Events + Observations + Evidence + Relationships + Actions + Responses + Resolutions + Outcomes -> Longitudinal View`

The projection may be materialized, queried dynamically, cached, or rebuilt. Its representation is an implementation concern. Its semantics must remain derived from canonical records.

## 2. Non-goals

This specification does **not** introduce:

- a `LongitudinalRecord` aggregate;
- a `LongitudinalRecordRepository`;
- a second persistence model for existing domain primitives;
- a universal timeline state machine;
- a new relationship system;
- AI authority over interpretation, truth, responsibility, or legal effect;
- automatic conflict resolution;
- automatic epistemic-status promotion;
- deletion or mutation of source records merely because a projection changes.

## 3. Canonical principle

A longitudinal view is a **derived read model**.

The source of truth remains the canonical domain records and their provenance/audit relationships. A projection must not become an alternative source of truth.

Where the projection is incomplete, stale, unavailable, or contradictory, that condition must remain distinguishable from absence of the underlying fact.

## 4. Source primitives

The initial projection is composed from existing canonical primitives:

| Primitive | Longitudinal role |
|---|---|
| Event | Recorded occurrence, change, or lifecycle transition |
| Observation | Epistemically bounded assertion/report/measurement about a subject/resource |
| Evidence | Preserved source artifact supporting an observation or other record |
| ResourceLink | Explicit typed relationship between resources |
| Action | Consequential operation and its approval/execution semantics |
| Response | Institutional or workflow response where implemented |
| Resolution | Resolution record where implemented |
| Outcome | Result/verification of an action or response where implemented |

Not every view needs every primitive. The projection must declare which source classes it includes.

## 5. Identity and references

Projection entries MUST retain references to their canonical source resources.

The projection MUST NOT replace canonical resource identity with a projection-local identity that obscures the source.

Where a relationship is shown, the projection MUST preserve the canonical relationship type and link semantics. In particular, it MUST NOT infer `Strong` semantics from a legacy or class-less relationship.

## 6. Time semantics

A longitudinal view may order entries for presentation, but it must not collapse distinct temporal meanings.

At minimum, Observation-derived entries must preserve:

- `observed_at`;
- `recorded_at`;
- provenance/source timing where available.

Event-derived entries must preserve their event occurrence time.

Ordering is therefore a presentation/read-model concern, not permission to rewrite source timestamps.

## 7. Epistemic integrity

The projection MUST preserve the source epistemic status of observations.

It MUST NOT:

- convert `INFERRED` to `OBSERVED`;
- convert `USER_REPORTED` to `EVIDENCE_SUPPORTED` without an explicit authoritative operation and basis;
- treat `UNVERIFIED` as verified fact;
- treat `CONTESTED` as resolved;
- treat `UNKNOWN` as `FALSE`.

A projection may calculate presentation metadata such as grouping or ordering, but such metadata is not itself an epistemic upgrade.

## 8. Contradiction, correction, and supersession

Lifecycle relationships remain visible in the longitudinal view.

If an Observation is corrected, the prior Observation remains addressable and its `corrects` relationship remains visible.

If an Observation is superseded, the prior Observation remains addressable and its `supersedes` relationship remains visible.

If Observations contradict one another, the projection MUST show the contradiction rather than selecting a winner unless a separate authoritative resolution mechanism exists.

The projection therefore represents history and relationships; it does not silently repair history.

## 9. Provenance and evidence

A longitudinal projection MUST retain enough source references to trace significant entries back to their canonical source.

Derived summaries, classifications, clustering, or AI-generated descriptions are projection artifacts. They are not replacements for original evidence or authoritative source records.

Where evidence is displayed, the distinction between original evidence and derived artifacts MUST remain explicit.

## 10. Responsibility and relationships

The projection may display responsibility-related relationships when canonical typed relationships exist.

It MUST NOT collapse distinct roles into one generic authority. In particular:

`Responsible Party != Nearest Authority != Geographic Authority != Asset Owner != Contractor != Contracting Officer`

Probable or inferred relationships must retain their epistemic status and must not be presented as established responsibility merely because they appear in a longitudinal view.

## 11. AI and intelligence

AI may assist with:

- extraction;
- classification;
- clustering;
- summarization;
- candidate linking;
- anomaly detection;
- navigation of the projection.

AI MUST remain bounded by the canonical Intelligence Contract and MUST NOT:

- promote epistemic status;
- manufacture provenance;
- establish legal authority;
- establish responsibility without authoritative source support;
- resolve contradictions merely by model preference;
- bypass authorization, approval, execution, or audit boundaries.

An AI-derived projection artifact must remain distinguishable from the canonical source records.

## 12. Consistency and rebuildability

The projection SHOULD be rebuildable from canonical source records.

If a materialized projection is used, its freshness and rebuild state should be observable. A projection failure must not corrupt the canonical records.

A stale projection is a read-model condition, not evidence that the underlying canonical records changed.

## 13. Persistence boundary

The longitudinal projection MUST reuse the existing generic persistence and transaction infrastructure.

This specification does not authorize a projection-specific persistence abstraction. If a future implementation needs materialization, it should be introduced as an implementation-specific read-model adapter behind the established architecture rather than as a new domain aggregate.

## 14. Security and privacy

Projection access MUST respect the authorization and data-classification semantics of the underlying resources.

A longitudinal view must not become a side channel that exposes data merely because multiple records can be correlated.

Composition can increase sensitivity even when individual records are separately permissible. Access policy must therefore consider the resulting view and applicable data classification.

Local-first and privacy-first principles remain applicable; projection materialization must not silently broaden data distribution.

## 15. Conformance direction

Future implementation conformance should verify at least:

1. source identity is preserved;
2. projection contains no authoritative-only facts not present in source records;
3. Observation epistemic status is preserved;
4. correction/supersession/contradiction relationships remain explicit;
5. timestamps are not rewritten;
6. evidence provenance remains traceable;
7. ResourceLink semantics are preserved;
8. responsibility roles are not collapsed;
9. AI-derived material remains distinguishable;
10. authorization/data classification is enforced;
11. canonical records remain independent of projection availability;
12. projection can be rebuilt without changing canonical source semantics.

## 16. Implementation gate

This document defines architecture only. It does **not** claim that a longitudinal projection has been implemented or is functional.

Implementation should proceed only after the source contracts required for the first projection slice are identified and their conformance is established.
