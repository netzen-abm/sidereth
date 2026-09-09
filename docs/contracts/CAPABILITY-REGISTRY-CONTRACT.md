# SIDERETH — Capability Registry Contract

**Status:** CANONICAL / FOUNDATION CONTRACT
**Scope:** Discovery, identity, version resolution and lifecycle metadata for reusable SIDERETH capabilities

## 1. Purpose

The Capability Registry is the ecosystem index for canonical SIDERETH capabilities. It makes capabilities discoverable and resolvable without becoming the source of capability semantics, authorization, execution, or implementation truth.

The registry exists so that independent surfaces, workflows, tools and platform services can discover the same shared capability by stable identity and contract version.

The registry is infrastructure, not a second business-logic layer.

## 2. Architectural boundary

The registry owns **metadata and discovery**. It does not own capability execution.

```text
Capability Contract
       |
       v
Capability Registry
       |
       +--> discover capability
       +--> resolve contract version
       +--> inspect lifecycle / risk metadata
       +--> locate implementations / adapters
       |
       v
Execution Boundary
       |
       +--> identity
       +--> authorization / policy
       +--> approval
       +--> capability implementation
```

The registry must never be treated as evidence that an invocation is authorized or safe to execute.

## 3. Canonical source of truth

The Capability Contract remains authoritative for capability semantics.

The registry may index or reference:

- capability identity and version;
- name and purpose;
- lifecycle status;
- risk class;
- data classes;
- jurisdiction scope;
- source requirements;
- approval requirements;
- execution modes;
- contract/schema references;
- implementation references;
- adapter references;
- dependency references;
- documentation references;
- observability requirements.

Registry metadata must not silently redefine these properties. Conflicting registry metadata must fail closed or be rejected during registration/validation.

## 4. Stable identity and version resolution

A registry entry is uniquely identified by:

```text
capability_id + contract_version
```

`capability_id` is stable across compatible implementations and providers.

Contract versioning follows the canonical Capability Contract. Implementation/provider versions are separate metadata and must not create new capability identities unless contract semantics change.

Consumers must be able to request:

- an exact capability version;
- a compatible version range, where version-range semantics are explicitly defined;
- the active/default version only when the consumer explicitly permits that behavior.

A registry must never silently upgrade a consumer across a breaking contract boundary.

## 5. Registration

A capability may be registered only when its required contract metadata is present and structurally valid.

Registration must validate, as applicable:

1. stable capability identity;
2. valid contract version;
3. lifecycle status;
4. risk classification;
5. required permissions and data classes;
6. jurisdiction/source requirements;
7. approval requirements;
8. execution semantics;
9. referenced contract/schema;
10. implementation and adapter references;
11. dependency references;
12. documentation and observability requirements.

Registration does not imply that an implementation is operationally verified or active.

## 6. Lifecycle and promotion

Registry lifecycle metadata follows the canonical capability lifecycle:

```text
Proposed
 -> Designed
 -> Contracted
 -> Implemented
 -> Tested
 -> Security Reviewed
 -> Operationally Verified
 -> Active
 -> Deprecated
 -> Retired
```

Only capabilities satisfying the applicable promotion gates may be represented as `Active`.

`Deprecated` capabilities may remain discoverable for migration purposes. `Retired` capabilities must not be selected for new execution.

The registry must preserve lifecycle history sufficient to explain material status transitions.

## 7. Discovery semantics

Discovery must be deterministic for the same registry state and query criteria.

A discovery result should provide enough metadata for a consumer to decide whether the capability is suitable before invoking it.

Discovery may support filters such as:

- capability ID;
- contract version/range;
- lifecycle status;
- risk class;
- data class;
- jurisdiction;
- execution mode;
- required feature/contract metadata.

Discovery is not authorization. A discovered capability remains subject to identity, policy, authorization and approval at execution time.

## 8. Implementation and provider neutrality

One capability may have zero, one or multiple implementations.

The registry may reference implementations and providers, but must not make a provider implementation canonical merely because it is registered first or currently preferred.

Implementation selection is a separate execution concern unless a capability contract explicitly requires a particular mechanism.

A registry implementation must itself be replaceable without changing capability semantics.

## 9. Dependencies

Capability dependencies must reference canonical capability identities and compatible contract versions rather than private implementation details where practical.

Dependency cycles must be detectable and rejected where they would prevent deterministic composition or lifecycle management.

A registry must distinguish:

- required capability dependencies;
- optional capability dependencies;
- implementation-specific dependencies.

Optional dependencies must not become hidden mandatory dependencies through registry behavior.

## 10. Security and trust boundary

The registry is not a trust grant.

A consumer must not infer any of the following solely from registry membership:

- authorization;
- user consent;
- human approval;
- legal validity;
- evidence authenticity;
- provider trustworthiness;
- operational availability.

Registry mutation must itself be authenticated, authorized and audited.

Registry data integrity must be protected against unauthorized modification, stale resolution and downgrade attacks.

## 11. Provenance and audit

Material registry changes must be attributable and auditable, including at minimum:

- capability identity/version affected;
- change type;
- actor or system identity;
- authorization context;
- timestamp;
- previous and new lifecycle state where applicable;
- relevant contract/schema reference.

Registry metadata that describes externally sourced legal or evidentiary resources must retain the applicable provenance references; the registry does not replace the canonical provenance systems.

## 12. Availability and failure semantics

Consumers must be able to distinguish at least:

- capability not found;
- unsupported version;
- retired capability;
- invalid registry metadata;
- dependency resolution failure;
- registry unavailable;
- stale/conflicting registry state.

Registry unavailability must not be converted into an unsafe assumption that a capability exists, is authorized, or is executable.

Cached registry data must carry freshness/version metadata and must obey an explicit stale-data policy.

## 13. Conformance requirements

A registry implementation is conformant only when it demonstrates:

1. stable identity/version resolution;
2. deterministic discovery;
3. registration validation;
4. lifecycle enforcement;
5. no authorization/approval bypass;
6. provider-neutral resolution;
7. dependency validation;
8. stale/conflict handling;
9. auditability of registry mutations;
10. deterministic failure semantics;
11. compatibility with the canonical Capability Contract;
12. security/privacy checks appropriate to the registry data.

Contract tests must be implementation/provider-neutral.

## 14. Non-goals

This contract does not define:

- a specific database;
- a specific service-discovery technology;
- a specific programming language or framework;
- a Tool Registry;
- a Workflow Engine;
- an authorization service;
- an approval service;
- an execution runtime;
- an AI/model registry;
- autonomous capability selection with authority to execute consequential actions.

Those are separate concerns and must consume this contract rather than redefine it.

## 15. Evolution

Changes to registry semantics that affect capability identity, version resolution, lifecycle promotion, risk, authorization, approval, provenance or consequential execution require architectural review.

The registry must evolve by extending the canonical contract boundary rather than creating parallel capability catalogs.

## 16. Immediate implementation rule

The first implementation should be the **minimum provider-neutral registry needed to prove this contract**:

- typed registry entry;
- registration validation;
- deterministic exact-version lookup;
- lifecycle-aware discovery;
- dependency/reference validation;
- conformance tests;
- no external registry dependency unless later justified.

Persistence, distributed registry replication, caching, service discovery and advanced query infrastructure are not prerequisites for the initial conformance proof.
