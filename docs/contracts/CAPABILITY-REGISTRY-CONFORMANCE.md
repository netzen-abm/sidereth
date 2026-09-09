# SIDERETH — Capability Registry Conformance Contract

**Status:** CANONICAL TEST DESIGN / PRE-IMPLEMENTATION
**Contract:** `docs/contracts/CAPABILITY-REGISTRY-CONTRACT.md`
**Scope:** Provider-neutral registry boundary only.

This document defines the conformance boundary that every Capability Registry implementation must satisfy before it can be treated as a SIDERETH registry implementation. It does not select a database, service-discovery system, cache, cloud provider, programming language or framework.

## 1. Test principles

1. The Capability Registry Contract is the authoritative behavioral boundary.
2. Registry membership never grants authorization, approval or execution authority.
3. Capability semantics remain owned by the canonical Capability Contract.
4. Registry implementations and storage providers are replaceable.
5. Tests must verify rejection and failure semantics as rigorously as successful discovery.
6. Version resolution must never silently cross a breaking contract boundary.
7. Retired capabilities must never be selected for new execution.
8. Registry metadata conflicts must fail closed rather than silently redefining capability semantics.
9. Contract tests must be deterministic and provider-neutral.
10. The initial conformance suite must be executable without an external registry service.

## 2. Conformance matrix

| ID | Area | Requirement | Test | Expected result | Gate |
|---|---|---|---|---|---|
| REG-001 | Identity | Stable capability identity | Register capability with ID/version | Entry is addressable by canonical identity | Contract |
| REG-002 | Identity | Duplicate identity rejection | Register same ID + version twice | Duplicate is rejected deterministically | Integrity |
| REG-003 | Versioning | Exact version lookup | Register v1 and request v1 | v1 is returned | Contract |
| REG-004 | Versioning | Unsupported version | Request unregistered version | Explicit unsupported-version failure | Compatibility |
| REG-005 | Versioning | No silent major upgrade | Request v1 when only v2 exists | Registry does not silently return v2 | Safety |
| REG-006 | Versioning | Implementation/provider separation | Change implementation version only | Capability identity remains stable | Compatibility |
| REG-007 | Metadata | Required metadata | Register incomplete entry | Registration is rejected | Contract |
| REG-008 | Metadata | Risk class integrity | Register declared risk | Risk metadata is preserved | Safety |
| REG-009 | Metadata | Data-class integrity | Register data classes | Data classes remain unchanged | Privacy |
| REG-010 | Metadata | Jurisdiction integrity | Register jurisdiction scope | Scope remains explicit | Legal correctness |
| REG-011 | Metadata | Approval requirement integrity | Register high-impact capability | Approval requirement cannot be weakened | Safety |
| REG-012 | Contract | Contract reference | Register contract/schema reference | Reference is preserved and validated | Contract |
| REG-013 | Lifecycle | Proposed | Register Proposed capability | Discoverable only according to lifecycle policy | Lifecycle |
| REG-014 | Lifecycle | Active promotion | Attempt Active without required gates | Promotion is rejected | Governance |
| REG-015 | Lifecycle | Deprecated | Discover Deprecated capability | Returned with deprecated status, not silently treated as Active | Governance |
| REG-016 | Lifecycle | Retired | Discover Retired capability for execution | Registry refuses execution selection | Safety |
| REG-017 | Lifecycle | History | Change lifecycle status | Material transition is attributable/traceable | Audit |
| REG-018 | Discovery | Determinism | Repeat identical query | Same result ordering/content for same state | Contract |
| REG-019 | Discovery | ID filter | Query by capability ID | Matching versions only | Contract |
| REG-020 | Discovery | Version filter | Query by supported version | Only compatible requested version is returned | Compatibility |
| REG-021 | Discovery | Risk filter | Query by risk class | Only matching risk class returned | Contract |
| REG-022 | Discovery | Data-class filter | Query by data class | Only matching declarations returned | Privacy |
| REG-023 | Discovery | Jurisdiction filter | Query by jurisdiction | Out-of-scope capability excluded | Legal correctness |
| REG-024 | Discovery | Execution-mode filter | Query for supported execution mode | Unsupported mode excluded | Contract |
| REG-025 | Discovery | No authorization inference | Discover protected capability | Discovery does not grant access | Security |
| REG-026 | Security | Registry is not authority | Treat registry membership as permission | Authorization is still required elsewhere | Security |
| REG-027 | Security | Approval isolation | Discover high-impact capability | Human approval remains required | Safety |
| REG-028 | Security | Metadata tampering | Alter risk/permission metadata | Conflict is rejected or fails closed | Security |
| REG-029 | Security | Downgrade protection | Prefer older unsafe version | Policy prevents unauthorized downgrade | Security |
| REG-030 | Security | Stale state | Resolve stale registry state | Explicit stale-state behavior | Reliability |
| REG-031 | Dependencies | Required dependency | Register valid required dependency | Dependency reference is preserved | Contract |
| REG-032 | Dependencies | Missing dependency | Register absent required dependency | Registration/resolution fails deterministically | Integrity |
| REG-033 | Dependencies | Optional dependency | Register absent optional dependency | Capability remains valid when policy permits | Contract |
| REG-034 | Dependencies | Hidden optional dependency | Implementation requires undeclared optional dependency | Conformance fails | Integrity |
| REG-035 | Dependencies | Cycle detection | Create dependency cycle | Cycle is detected/rejected where required | Integrity |
| REG-036 | Dependencies | Version compatibility | Dependency requests incompatible version | Resolution fails deterministically | Compatibility |
| REG-037 | Provider | Multiple implementations | Register two implementations | Both remain replaceable references | Provider neutrality |
| REG-038 | Provider | First-provider bias | Register provider A then B | Registration order does not make A canonical | Provider neutrality |
| REG-039 | Provider | Provider removal | Remove one implementation | Capability remains resolvable if another valid implementation exists | Reliability |
| REG-040 | Provider | No provider lock-in | Replace implementation | Contract consumer semantics remain unchanged | Architecture |
| REG-041 | Failure | Not found | Request unknown capability | Explicit not-found failure | Reliability |
| REG-042 | Failure | Invalid metadata | Register malformed metadata | Explicit validation failure | Reliability |
| REG-043 | Failure | Registry unavailable | Registry backend unavailable | Consumer does not assume authorization/existence | Safety |
| REG-044 | Failure | Conflicting state | Two incompatible registry records | Conflict is surfaced/fails closed | Integrity |
| REG-045 | Audit | Registration audit | Register capability | Required mutation audit context exists | Audit |
| REG-046 | Audit | Lifecycle audit | Change lifecycle state | Actor/time/old/new state are attributable | Audit |
| REG-047 | Audit | Deregistration audit | Remove registry entry | Removal is attributable | Audit |
| REG-048 | Provenance | Contract provenance | Reference canonical contract | Contract reference is traceable | Provenance |
| REG-049 | Provenance | External resource provenance | Registry references legal/evidentiary resource metadata | Applicable provenance remains intact | Provenance |
| REG-050 | Isolation | Registry cannot mutate domain | Registry lookup occurs | Case/Party/Evidence/Action state is unchanged | Integrity |
| REG-051 | Isolation | Registry cannot execute | Consumer discovers capability | No execution occurs from discovery alone | Safety |
| REG-052 | Isolation | Registry cannot approve | Consumer discovers high-impact capability | No approval is manufactured | Safety |
| REG-053 | Compatibility | Contract change | Breaking capability contract introduced | New major version/migration path is required | Governance |
| REG-054 | Compatibility | Consumer rejection | Consumer rejects unsupported version | Registry returns deterministic incompatibility | Compatibility |
| REG-055 | Conformance | Provider-neutral tests | Run suite against two registry implementations | Canonical behavior remains equivalent | Conformance |

## 3. Mandatory architectural invariants

### I-01 — Registry membership is not authorization

A capability appearing in the registry must never be sufficient to access protected data or execute protected operations.

### I-02 — Registry membership is not approval

A registered HIGH_IMPACT capability cannot bypass the canonical Action/Approval/Execution Gate.

### I-03 — Registry does not own capability semantics

If registry metadata conflicts with the canonical Capability Contract, the canonical contract remains authoritative and the conflict must not be silently resolved in favor of the registry.

### I-04 — No breaking-version downgrade or upgrade

Exact or compatible version resolution must not cross a breaking contract boundary without an explicit consumer-approved migration path.

### I-05 — Retired means non-selectable for new execution

Retired capabilities may remain discoverable for historical or migration purposes but cannot be selected for new execution.

### I-06 — Provider replacement preserves contract semantics

Replacing one implementation/provider with another must not require changes to consumers that depend only on the canonical capability contract.

### I-07 — Registry failure fails closed

If registry state cannot establish a trustworthy resolution, the consumer must not infer that a capability is authorized, active or executable.

### I-08 — Registry is side-effect free for discovery

Discovery and resolution operations must not mutate canonical Case, Party, Document, Evidence, Action, Authorization, Approval or other domain state.

### I-09 — Dependency graph is explicit

Required, optional and implementation-specific dependencies must remain distinguishable; undeclared mandatory dependencies are a conformance failure.

### I-10 — Deterministic resolution

For identical registry state and identical resolution criteria, a registry implementation must produce equivalent results and failure semantics.

## 4. Deterministic fake registry strategy

The first conformance suite should use an in-memory deterministic implementation and fixture registry entries.

### Required fixtures

At minimum:

- one valid Proposed capability;
- one valid Active capability;
- one Deprecated capability;
- one Retired capability;
- two contract versions of the same capability;
- two implementations of one capability;
- a capability with a required dependency;
- a capability with an optional dependency;
- an invalid/malformed entry;
- a dependency cycle;
- a conflicting metadata entry.

The fixture set must not depend on external services, network availability or provider-specific behavior.

## 5. Conformance execution model

The suite should be executable against any implementation through a small registry-facing interface equivalent to:

```text
register(entry)
get(capability_id, version)
discover(criteria)
resolve(capability_id, version_requirement)
validate(entry)
```

The exact programming-language interface is intentionally implementation-specific. The behavioral contract is not.

## 6. Promotion gates

| Stage | Minimum evidence |
|---|---|
| Registry contract | Contract + conformance matrix accepted |
| Reference implementation | Typed in-memory registry + deterministic tests |
| Provider conformance | Full applicable REG suite passes |
| Security review | Authority bypass, downgrade, tampering and stale-state tests pass |
| Integration verification | Capability resolution integrates without bypassing authorization/approval |
| Production readiness | Persistence, availability, audit and operational evidence appropriate to deployment |

## 7. Explicit non-goals

This conformance contract does not authorize or require:

- PostgreSQL registry persistence;
- Redis or distributed caching;
- Kubernetes/service discovery;
- cloud-managed service registry;
- Tool Registry implementation;
- Tool Gateway implementation;
- Workflow Engine implementation;
- AI/model registry;
- autonomous capability selection with execution authority;
- domain-specific capability proliferation.
