# SIDERETH — Universal Core Cross-Contract Audit

**Date:** 2026-09-04  
**Scope:** Case, Incident, Event, Evidence, Authority, Jurisdiction, Party, Document, Action  
**Baseline:** `main` after PR #25 (Universal Action Model)  
**Audit branch:** `sidereth-universal-core-audit`  
**Status:** AUDIT COMPLETE — REMEDIATION REQUIRED BEFORE DECISION MODEL

## 1. Executive decision

The universal-core direction is structurally sound, but the cross-contract audit identifies several contract-level inconsistencies that should be corrected before adding the Universal Decision Model.

**Decision:** Do not implement Decision yet. First harden the universal reference, serialization, provenance, authorization, and lifecycle contracts.

The most important finding is that the Rust models and JSON Schemas do not yet form a single canonical wire contract. Several enums use Rust's default externally-tagged string representation while schemas use lowercase values or unconstrained strings. This creates interoperability risk for every future surface, adapter, database serializer, AI tool, and domain pack.

## 2. Contract matrix

| Primitive | Identity | Context refs | Provenance | Privacy | Lifecycle | Versioning | Integrity | Authorization boundary |
|---|---|---|---|---|---|---|---|---|
| Case | `case_id` | native matter | implicit/not modeled in `Case` | not modeled | yes | no explicit schema version | no | external policy baseline |
| Incident | `incident_id` | native matter | implicit/not modeled in `Incident` | not modeled | yes | no explicit schema version | no | external policy baseline |
| Event | `event_id` + aggregate | aggregate refs | source refs/causation | not modeled | occurrence | yes | no | actor is recorded, authorization is external |
| Evidence | `evidence_id` | case OR incident | capture metadata | not modeled | capture/derived | yes | content hash | external policy |
| Authority | `authority_id` / power | jurisdiction | source refs on power | not modeled | active/inactive | no explicit schema version | no | authority semantics are contextual |
| Jurisdiction | `jurisdiction_id` | parent | not modeled | not modeled | active/inactive | no explicit schema version | no | governing scope |
| Party | `party_id` | jurisdiction/context relationships | optional | yes | active/inactive/etc. | yes | no | relationship authorization ref; access policy external |
| Document | `document_id` + versions | case/incident/jurisdiction | optional | yes | yes | yes | version hash/status | external policy |
| Action | `action_id` | context/target | required | not modeled | yes | yes | references/effects | explicit independent authorization boundary |

## 3. Findings

### CORE-001 — Canonical ID type is too weak

**Severity:** P1  
**Area:** Identity / references

`crate::Id` is currently `String`. Every reference therefore has identical compile-time semantics even when it points to a Party, Document, Case, Authority, Evidence, Action, or other primitive.

This is acceptable as an initial implementation baseline but is insufficient as a mature universal contract because it permits accidental cross-primitive references and makes validation dependent entirely on registries/policy layers.

**Recommendation:** retain stable opaque IDs, but introduce a canonical typed-reference contract at the boundary (without prematurely creating incompatible Rust wrapper types). The contract should distinguish object type from object ID where cross-primitive validation requires it.

### CORE-002 — Rust enum serialization and JSON Schema values are inconsistent

**Severity:** P0  
**Area:** Interoperability / serialization

The Rust enums currently rely on default Serde serialization. For example, `PartyKind::Person` serializes as `"Person"`, while `schemas/party.schema.json` defines `"person"`; similarly Party status and Document status schemas use lowercase values. Action's schema currently permits any string for `kind` and `status` rather than constraining the canonical values.

This violates the requirement that implementation and schema represent one canonical contract.

**Recommendation:** establish one canonical wire representation, preferably explicit `snake_case` serialization for public JSON, and align every schema and test to it. Do not rely on default enum serialization for public contracts.

### CORE-003 — Schema extension policy is inconsistent with Rust models

**Severity:** P1  
**Area:** Schema evolution

`party.schema.json` and `document.schema.json` declare an `extension` property, but the corresponding Rust structs do not expose an `extension` field. Document and Party schemas also use `additionalProperties: false`, meaning arbitrary fields are prohibited while a named extension object is allowed.

Action's schema does not explicitly declare `additionalProperties` behavior.

**Recommendation:** standardize the universal extension policy across all canonical schemas and implementations. Prefer a typed `extension` envelope with explicit semantics rather than unbounded top-level fields.

### CORE-004 — Document current-version invariant is under-specified

**Severity:** P1  
**Area:** Versioning

`Document.current_version_id` is mandatory, but `DocumentRegistry::insert_document` does not verify that the referenced version exists, and version insertion does not update or validate the document's current version pointer. The initial-version logic also permits a document state whose current-version relationship is dependent on caller discipline.

**Recommendation:** define the invariant explicitly: a persisted active document must point to an existing immutable version belonging to that document. Version registration and current-version changes should be atomic and policy-controlled.

### CORE-005 — Provenance is not yet universal at the implementation boundary

**Severity:** P1  
**Area:** Provenance

The relationship contract states that provenance is preserved across transformations, but provenance is optional or absent in several primitives. Case and Incident currently have no provenance field; Authority and Jurisdiction also lack a universal provenance reference; Party and Document make provenance optional.

This does not invalidate the baseline models, but it means the universal provenance invariant is currently architectural intent rather than an enforced cross-contract guarantee.

**Recommendation:** define a common provenance contract and specify when provenance is mandatory, optional, inherited, or generated. Derived transformations must carry a traceable provenance chain.

### CORE-006 — Authorization is correctly independent, but the baseline is not universal enough

**Severity:** P1  
**Area:** Authorization

Action correctly references an independent authorization rather than granting authority itself. However, the current `AuthorizationPolicy` is specifically case-access oriented (`AccessRequest` with `case_id`) and does not yet provide a universal authorization decision contract for Action, Document, Evidence, Party relationship, external sharing, or other resources.

**Recommendation:** introduce a resource/action/purpose-scoped authorization decision contract before effectful Action execution. Keep access authorization separate from legal authority and from user consent.

### CORE-007 — Lifecycle semantics are inconsistent across primitives

**Severity:** P1  
**Area:** State machines

Case, Incident, Action, Party, Document, Authority, and Jurisdiction each have different lifecycle models. That is legitimate, but there is no common lifecycle contract defining terminal states, reopening rules, cancellation semantics, supersession semantics, or audit requirements.

Action's `Approved → Executing` boundary is intentionally policy-dependent, which is correct. The same pattern should be made explicit across other effectful state changes.

**Recommendation:** keep primitive-specific state machines but establish a shared lifecycle meta-contract: valid transition, actor, authorization, timestamp, causation/event, audit record, and reason where applicable.

### CORE-008 — Event model is not yet the universal audit spine

**Severity:** P1  
**Area:** Events / audit

`EventEnvelope` has strong correlation/causation fields, but primitive lifecycle methods do not themselves emit events. The architecture therefore relies on higher layers to remember to produce events.

**Recommendation:** define an event emission contract around state-changing commands/services rather than coupling low-level data structures directly to an event bus. Every auditable state transition should have a deterministic event/audit path.

### CORE-009 — Evidence and Document derived-artifact semantics need consolidation

**Severity:** P1  
**Area:** Evidence / Document

There are two `DerivedArtifact` concepts: one under Evidence and another under Document. This can be valid if they represent distinct bounded semantics, but their names and fields overlap substantially.

The universal relationship contract says Document owns derived information artifacts while Evidence owns capture/integrity semantics. The implementation should make that distinction unmistakable.

**Recommendation:** retain separate bounded models only if the contract explicitly defines them as different artifact families; otherwise introduce a universal `DerivedArtifact` base/reference contract with specialized metadata.

### CORE-010 — Privacy classification is not yet universal

**Severity:** P1  
**Area:** Privacy

Party and Document have privacy classification fields. Evidence, Event, Authority, Jurisdiction, Case, Incident, and Action do not have equivalent data-classification metadata.

The security roadmap separately lists data classification as unfinished, so this is an intentional maturity gap rather than an implementation defect.

**Recommendation:** define a universal data-classification/handling contract and permit inherited classification from containing context while allowing stricter classification at the object level.

### CORE-011 — Timestamp validation is syntactic, not semantic

**Severity:** P2  
**Area:** Temporal correctness

Most Rust validators only check that timestamps are non-empty strings. Schemas sometimes specify `format: date-time`. This creates another implementation/schema divergence and permits malformed timestamps in the Rust boundary.

**Recommendation:** adopt one canonical timestamp representation and validate it at the contract boundary.

### CORE-012 — Action approval semantics need refinement before execution infrastructure

**Severity:** P1  
**Area:** Action / authorization

The current Action model requires an authorization reference when `requires_explicit_approval` is true, which is a useful baseline. However, the model does not yet distinguish authorization decision, human approval, policy satisfaction, and legal authority as separate concepts.

**Recommendation:** keep Action semantically neutral. Model authorization, approval, legal authority, and policy evaluation as independent references/results. `Executing` should require the governing policy engine to establish all required conditions rather than treating the presence of `authorization_ref` as sufficient.

## 4. Architectural invariants that passed

1. Party identity is separated from contextual role/relationship semantics.
2. Document logical identity is separated from immutable versions and derived artifacts.
3. Evidence preserves original capture material and content hashing.
4. Authority is separated from Jurisdiction.
5. Action is separated from Event, Decision, Authorization, Evidence, and Workflow.
6. Action does not grant authority merely by existing.
7. AI/agents are explicitly bounded from acquiring legal authority through Action.
8. Domain packs are defined as consumers/specializers of universal primitives rather than replacements.
9. The architecture explicitly favors stable references over copying canonical identity.
10. The repository's Definition of Done requires contract, implementation, tests, security, documentation, and observability before a capability is considered complete.

## 5. Required remediation order

### P0 — Must resolve before Decision Model

1. Canonical JSON enum serialization and schema alignment.
2. Canonical cross-primitive reference/wire contract.

### P1 — Resolve before production-grade Action execution

3. Document current-version invariant and atomic version/current-pointer semantics.
4. Universal authorization decision contract.
5. Universal provenance policy.
6. Lifecycle meta-contract and event/audit transition contract.
7. Derived-artifact boundary clarification.
8. Universal privacy/data-classification contract.
9. Action approval/authorization semantic separation.

### P2 — Harden subsequently

10. Timestamp parsing/validation.
11. Schema extension standardization.

## 6. Decision on next capability

**Decision Model is deferred.**

The next implementation slice should be a small **Universal Contract Hardening** change set addressing P0 findings first, followed by the highest-value P1 invariants. Only after those changes pass CI and contract tests should the Universal Decision Model be implemented.

## 7. Definition-of-Done impact

Party, Document, and Action remain implementation baselines rather than fully complete capabilities because the repository's own Definition of Done requires security controls, documentation, and observability in addition to implementation and tests.

The master checklist should therefore not mark these capabilities complete merely because their source files exist.
