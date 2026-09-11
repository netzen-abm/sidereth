# SIDERETH — Tool Registry Conformance Matrix

**Status:** CANONICAL / FOUNDATION TEST DESIGN  
**Scope:** Implementation-independent conformance for the Tool Registry contract

## Purpose

This matrix defines the minimum behavioral proof required for any Tool Registry implementation. It deliberately separates registry behavior from execution authority.

A conforming implementation must satisfy these invariants regardless of storage technology, programming language, deployment topology, provider, framework, or discovery mechanism.

## Mandatory architectural invariants

1. **Registry membership is not authorization.** A registered tool is not thereby permitted to execute.
2. **Registry membership is not approval.** Registration cannot manufacture human approval.
3. **Registry does not own canonical semantics.** Capability Contract remains authoritative.
4. **Capability/function binding is explicit.** A tool cannot silently bind to unrelated semantics.
5. **Version resolution is deterministic.** Exact and compatible resolution produce reproducible results.
6. **Breaking interface changes require a major version.** Registry cannot silently reinterpret an incompatible contract.
7. **Retired tools are not selectable for new execution.** Historical metadata may remain auditable.
8. **Provider replacement preserves contract identity/semantics.** Provider metadata cannot redefine the capability.
9. **Discovery is side-effect free.** Discovery cannot mutate execution state or invoke a tool.
10. **Registry failure fails closed.** Missing, stale, corrupt or ambiguous metadata cannot authorize consequential invocation.
11. **Dependencies are explicit and cycle-safe.** Hidden dependency edges are non-conforming.
12. **Registry does not execute.** Runtime invocation remains behind Tool Gateway and the execution boundary.
13. **MCP is only an adapter.** MCP exposure cannot bypass registry, gateway, policy, authorization or approval.
14. **Provenance/audit metadata is preserved.** Registration and lifecycle changes remain attributable.

## Test matrix

| ID | Area | Required behavior | Adversarial condition |
|---|---|---|---|
| TR-001 | Identity | Reject missing tool identity | empty tool ID |
| TR-002 | Identity | Require stable tool ID + contract version | version omitted |
| TR-003 | Identity | Reject duplicate identity with conflicting definition | same ID/version, different schema |
| TR-004 | Binding | Require capability binding | no capability reference |
| TR-005 | Binding | Require explicit function binding where applicable | unrelated function |
| TR-006 | Binding | Reject binding to unknown capability | fabricated capability ID |
| TR-007 | Version | Resolve exact version deterministically | two exact candidates |
| TR-008 | Version | Resolve compatible major deterministically | multiple compatible patches |
| TR-009 | Version | Reject incompatible major | major mismatch |
| TR-010 | Version | Never silently downgrade a requested version | only older version exists |
| TR-011 | Lifecycle | Proposed is discoverable only as metadata, not active execution candidate | attempt invocation selection |
| TR-012 | Lifecycle | Designed is not active | selection request |
| TR-013 | Lifecycle | Contracted is not active | selection request |
| TR-014 | Lifecycle | Implemented is not active without required gates | selection request |
| TR-015 | Lifecycle | Tested is not active without security/operational gates | selection request |
| TR-016 | Lifecycle | Security Reviewed is not automatically active | missing operational verification |
| TR-017 | Lifecycle | Operationally Verified can become Active | valid promotion |
| TR-018 | Lifecycle | Deprecated remains auditable but is not preferred for new resolution | active alternative exists |
| TR-019 | Lifecycle | Retired cannot resolve for new execution | explicit resolution request |
| TR-020 | Discovery | Discovery is deterministic | repeated identical query |
| TR-021 | Discovery | Discovery has no execution side effects | tool invocation spy |
| TR-022 | Discovery | Criteria constrain returned tools | risk/data/jurisdiction filter |
| TR-023 | Discovery | Inactive tools are excluded from execution candidates | lifecycle mismatch |
| TR-024 | Discovery | Ambiguous matches fail closed | equal-ranked candidates |
| TR-025 | Provider | Multiple implementations may be registered | two providers, same contract |
| TR-026 | Provider | Provider metadata cannot alter capability semantics | conflicting provider schema |
| TR-027 | Provider | Provider replacement preserves stable tool identity where contract-compatible | provider swap |
| TR-028 | Provider | Registry does not choose execution provider | provider selection attempt |
| TR-029 | Risk | Risk class is explicit | missing risk class |
| TR-030 | Data | Data class is explicit | missing data classification |
| TR-031 | Jurisdiction | Jurisdiction scope is explicit when required | missing scope |
| TR-032 | Permissions | Permission metadata is descriptive, not an authorization grant | permission present but policy denies |
| TR-033 | Approval | Approval requirement is descriptive, not approval itself | `human_approval_required=true` without approval |
| TR-034 | Execution | Registry cannot execute a tool | invocation attempt against registry |
| TR-035 | Gateway | Registry resolution precedes Gateway execution | direct bypass attempt |
| TR-036 | MCP | MCP adapter resolves through canonical registry/gateway boundary | direct MCP invocation |
| TR-037 | Schema | Input schema is validated as registry metadata | malformed schema |
| TR-038 | Schema | Output schema is validated as registry metadata | malformed schema |
| TR-039 | Dependencies | Required dependencies are explicit | hidden dependency |
| TR-040 | Dependencies | Optional dependencies do not block registration when absent | optional dependency missing |
| TR-041 | Dependencies | Missing required dependency blocks activation | dependency unavailable |
| TR-042 | Dependencies | Dependency cycles are rejected | A→B→A |
| TR-043 | Provenance | Registration records attributable provenance | missing provenance |
| TR-044 | Audit | Lifecycle changes produce audit records | promotion without audit |
| TR-045 | Audit | Audit records do not become execution permission | forged audit record |
| TR-046 | Failure | Registry unavailable fails closed | backend unavailable |
| TR-047 | Failure | Corrupt metadata fails closed | invalid persisted entry |
| TR-048 | Failure | Stale metadata cannot authorize consequential invocation | stale version/permission data |
| TR-049 | Isolation | One malformed tool does not corrupt unrelated registry entries | invalid neighboring entry |
| TR-050 | Isolation | Discovery results are isolated from caller mutation | mutate returned collection |
| TR-051 | Security | Untrusted tool metadata cannot escalate privilege | forged permission/approval metadata |
| TR-052 | Security | Registry cannot create human approval | synthetic intelligence/system approval |
| TR-053 | Semantics | Canonical capability contract remains source of truth | provider-defined semantics |
| TR-054 | Compatibility | Breaking contract change requires major version | same major, incompatible schema |
| TR-055 | Auditability | Registration and resolution context is reproducible | missing timestamp/actor/context |

## Conformance method

Each implementation should run the same semantic suite against a provider-neutral test harness. The harness should use deterministic fake capability/tool definitions and adversarial metadata rather than real external providers.

The implementation may use in-memory, relational, document, distributed or other storage later, but storage-specific tests must be additional to this matrix rather than substitutes for it.

## Promotion gates

A Tool Registry implementation is not considered conforming until:

- all mandatory invariants are demonstrated;
- all TR-001–TR-055 tests pass;
- failure-path behavior is demonstrated;
- security/privacy review appropriate to registry metadata is complete;
- no test relies on a particular provider or transport;
- documentation and implementation agree;
- CI passes on the exact PR head.

## Explicit non-goals

This matrix does not define or implement:

- Tool Gateway policy execution;
- authorization semantics;
- human approval semantics;
- Tool Runtime execution;
- MCP server/client implementation;
- AI/model registry;
- external distributed registry infrastructure;
- autonomous agents;
- legal reasoning or legal authority.

Those boundaries remain governed by their respective canonical contracts.
