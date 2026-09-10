# SIDERETH — Tool Registry Contract

**Status:** CANONICAL / FOUNDATION CONTRACT
**Scope:** Identity, discovery, compatibility and lifecycle metadata for controlled executable tools

## 1. Purpose

The Tool Registry is the ecosystem index for executable tool interfaces exposed by SIDERETH capabilities. It makes controlled tools discoverable and resolvable without becoming the source of capability semantics, authorization, approval or execution authority.

A tool is an executable interface associated with a canonical SIDERETH capability/function. The registry describes the interface and its governance metadata; the Tool Gateway remains the execution-policy boundary.

The Tool Registry is infrastructure, not a second business-logic or permission layer.

## 2. Architectural boundary

```text
Capability Contract
       |
       v
Capability Registry
       |
       v
Tool Registry
       |
       +--> discover tool
       +--> resolve tool contract/version
       +--> inspect risk/data/lifecycle metadata
       +--> locate implementation/adapter references
       |
       v
Tool Gateway
       |
       +--> identity
       +--> policy
       +--> authorization
       +--> data minimisation
       +--> approval
       |
       v
Tool Runtime / Adapter
       |
       v
Canonical Capability
```

Registry membership never proves that a caller is authorized to invoke a tool.

## 3. Canonical source of truth

The canonical Capability Contract remains authoritative for capability semantics. The Tool Registry must reference the capability and function contracts it exposes and must not silently redefine them.

The registry may index or reference:

- `tool_id` and tool contract version;
- bound `capability_id` and capability contract version;
- function reference where applicable;
- purpose and description;
- lifecycle status;
- risk class;
- data classes accessed;
- jurisdiction scope;
- required permissions;
- approval requirements;
- input and output schemas;
- execution modes;
- implementation/provider references;
- resource dependencies;
- provenance requirements;
- audit/observability requirements;
- adapter references;
- documentation references.

Conflicting registry metadata must be rejected or fail closed. Registry metadata is not permission to override the canonical capability, policy or approval contracts.

## 4. Stable identity and versioning

A tool registry entry is uniquely identified by:

```text
tool_id + tool_contract_version
```

The `tool_id` remains stable across compatible implementations and providers. Tool contract versions describe the externally observable tool interface and semantics, not provider or implementation versions.

Breaking tool-interface changes require a new major tool contract version or an explicit documented migration path.

A provider or implementation upgrade must not silently change the meaning of the canonical tool contract.

Consumers must be able to request an exact tool version or an explicitly compatible version. Resolution must never silently cross a breaking boundary.

## 5. Capability and function binding

Every executable tool must declare the canonical capability it serves.

Where the tool maps to a specific function, that function reference must also be explicit. A tool must not create a parallel capability definition merely because it has a different transport, provider or interface.

The binding is informational and contractual; it does not grant access to the underlying capability.

## 6. Registration validation

Registration must validate, as applicable:

1. stable tool identity and valid version;
2. non-empty capability binding;
3. valid function binding where required;
4. purpose/description;
5. lifecycle status;
6. risk classification;
7. data-class declarations;
8. jurisdiction constraints;
9. permission requirements;
10. approval requirements;
11. input/output schema references;
12. execution modes;
13. implementation/provider references;
14. dependency declarations;
15. provenance requirements;
16. audit/observability requirements.

A `HIGH_IMPACT` or otherwise consequential tool must declare the applicable approval requirement. Registration must never weaken a stricter requirement inherited from the bound capability contract.

Registration does not mean the tool is operationally verified, trusted or authorized for a particular caller.

## 7. Lifecycle

Tool lifecycle follows the canonical ecosystem lifecycle:

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

Only tools satisfying applicable promotion gates may be represented as `Active`.

`Deprecated` tools may remain discoverable for migration. `Retired` tools must not be selected for new execution.

Material lifecycle changes must be attributable and auditable.

## 8. Discovery and resolution

Discovery must be deterministic for identical registry state and criteria.

Supported filters may include:

- tool ID;
- tool contract version/range;
- capability ID/version;
- function reference;
- lifecycle;
- risk class;
- data class;
- jurisdiction;
- execution mode;
- required feature/contract metadata.

Resolution must distinguish at least:

- tool not found;
- unsupported version;
- retired tool;
- invalid metadata;
- conflicting state;
- unavailable/stale registry state;
- dependency incompatibility.

Discovery and resolution are side-effect free. They must not execute a tool or mutate canonical domain state.

## 9. Provider and implementation neutrality

A tool may have zero, one or multiple implementations/providers.

The registry may reference implementations and providers but must not make a provider authoritative merely because it was registered first, is currently preferred, or happens to be available.

Provider selection and execution are Tool Gateway/runtime concerns unless a tool contract explicitly requires a mechanism for semantic correctness.

Replacing an implementation/provider must preserve the tool contract for consumers that depend only on the canonical contract.

Duplicate implementation identities must be rejected deterministically.

## 10. Security and authority boundary

The Tool Registry is not an authority grant.

A consumer must never infer solely from registry membership:

- caller authorization;
- user consent;
- human approval;
- legal validity;
- evidence authenticity;
- provider trustworthiness;
- runtime availability;
- permission to access protected data;
- permission to execute consequential operations.

The Tool Gateway remains the execution-policy boundary. Every invocation must pass through the applicable identity, policy, authorization, data-minimisation, risk and approval controls.

AI/agent discovery of a tool must not manufacture permission or approval.

## 11. MCP boundary

MCP is an optional interoperability adapter for selected tools. MCP discovery metadata must not become canonical SIDERETH tool identity, authorization or execution state.

An MCP adapter must map to a registered SIDERETH tool and then enter through the Tool Gateway. MCP must never bypass the registry, policy, authorization or approval boundaries.

Tool descriptions, schemas and external resources received through MCP remain untrusted input until validated by the applicable security boundary.

## 12. Data, provenance and audit

Tool metadata must declare applicable data classes and jurisdiction constraints.

Where a tool consumes or produces legal, regulatory or evidentiary information, provenance requirements must remain explicit and must resolve to the canonical provenance systems.

Material registry mutations must preserve, at minimum:

- tool identity/version;
- change type;
- actor/system identity;
- authorization context;
- timestamp;
- previous/new lifecycle where applicable;
- bound capability/function references.

Operational audit of actual invocations belongs to the Tool Gateway/runtime and must not be confused with registry mutation audit.

## 13. Dependencies

Tool dependencies must be explicit and typed where the contract supports them.

Required dependencies must be resolvable before a tool is considered executable. Optional dependencies must not become hidden mandatory dependencies through registration or discovery behavior.

Dependency references should use canonical capability/tool contract identities rather than private provider implementation details where practical.

Dependency cycles that prevent deterministic composition or lifecycle management must be detected and rejected.

## 14. Failure and stale-state semantics

If registry state cannot establish a trustworthy tool resolution, the caller must receive an explicit failure or uncertainty result.

Registry unavailability must never be interpreted as permission to execute.

Cached tool metadata must carry freshness/version information and obey an explicit stale-data policy. A stale or conflicting record must not silently authorize a consequential invocation.

## 15. Conformance requirements

A conformant Tool Registry implementation must demonstrate:

1. stable tool identity/version resolution;
2. explicit capability/function binding;
3. deterministic discovery;
4. registration validation;
5. lifecycle enforcement;
6. provider-neutral multi-implementation handling;
7. dependency validation;
8. retired-tool exclusion from new execution;
9. no authorization/approval bypass;
10. MCP/Gateway isolation;
11. provenance and mutation auditability;
12. deterministic failure semantics;
13. compatibility with the canonical Capability Contract;
14. security/privacy checks appropriate to registry metadata.

Contract tests must remain implementation- and provider-neutral.

## 16. Non-goals

This contract does not define:

- a specific database or service-discovery technology;
- Tool Gateway implementation;
- Tool Runtime implementation;
- authorization or policy services;
- human approval services;
- MCP server implementation;
- an AI/model registry;
- autonomous tool execution;
- autonomous consequential legal action;
- a provider-specific tool protocol as the canonical contract.

These concerns consume the Tool Registry contract rather than redefining it.

## 17. Immediate implementation rule

The first implementation should be the minimum provider-neutral registry needed to prove this contract:

- typed tool registry entry;
- capability/function binding;
- registration validation;
- deterministic exact-version lookup;
- lifecycle-aware discovery;
- dependency/reference validation;
- provider-neutral implementation references;
- deterministic conformance tests;
- no external registry dependency unless later justified.

Persistence, distributed discovery, caching and runtime execution are not prerequisites for the initial conformance proof.
