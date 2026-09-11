# SIDERETH — Tool Gateway Contract

**Status:** CANONICAL / FOUNDATION CONTRACT
**Scope:** Provider-neutral enforcement boundary for controlled tool invocation

## 1. Purpose

The Tool Gateway is the enforcement boundary through which an invocation of a registered SIDERETH tool must pass before a bounded implementation may execute.

The gateway composes existing contracts. It does not replace them and does not create parallel authority.

The Tool Gateway must enforce, as applicable:

- caller and subject identity;
- canonical tool identity and version;
- capability/function binding;
- authorization decision and returned constraints;
- jurisdiction and resource scope;
- data minimisation;
- risk controls;
- human approval requirements;
- provenance and audit requirements;
- execution-mode and lifecycle constraints;
- implementation/provider selection rules;
- bounded invocation and failure semantics.

The gateway is an enforcement boundary, not a business-logic layer, policy-definition layer, AI authority layer or registry authority.

## 2. Canonical execution path

```text
Caller / Surface / Agent / MCP Adapter
              |
              v
        Tool Resolution
        (Tool Registry)
              |
              v
    Identity + Authorization
       + Policy Evaluation
              |
              v
        Tool Gateway
              |
      +-------+--------+
      |                |
      v                v
Data / Scope      Human Approval
Enforcement        where required
      |                |
      +-------+--------+
              |
              v
      Execution Gate
              |
              v
   Tool Runtime / Adapter
              |
              v
    Canonical Capability
```

The exact internal ordering may vary where a control is safely performed earlier, but no implementation may bypass the canonical authorization and execution boundaries.

For consequential Actions, the Action/Approval/Execution Gate remains authoritative for execution. The Tool Gateway must not manufacture approval or substitute registry metadata for authorization.

## 3. Authority boundaries

### Tool Registry

The Tool Registry provides identity, contract/version resolution, discovery and lifecycle metadata. Registry membership is never authorization.

### Authorization

The canonical Authorization Contract determines whether the subject may perform the requested operation against the scoped resource for the declared purpose. An `allow` result remains subject to its enforceable constraints.

### Approval

Human approval is distinct from authorization. Where explicit approval is required, the gateway must require the applicable canonical approval state and must not accept AI, system or caller assertions as human approval.

### Execution Gate

The Execution Gate is the final domain execution authorization boundary for Actions. The Tool Gateway must enter this boundary rather than creating an alternate execution permission mechanism.

### Tool Runtime / Adapter

The runtime performs the bounded implementation after gateway and execution controls succeed. Runtime/provider behavior must not widen authority.

## 4. Canonical invocation

A gateway invocation must carry sufficient context to bind the request to the registered tool and the authorization decision, including as applicable:

- invocation identity;
- subject/actor reference;
- tool identity and contract version;
- capability/function reference;
- action/operation reference;
- resource/case scope;
- declared purpose;
- authorization result reference;
- requested data class;
- jurisdiction context;
- input payload or canonical input reference;
- requested execution mode;
- correlation/operation identity;
- provenance context.

Transport-specific envelopes may contain additional metadata, but adapters must not silently widen any security-relevant field.

## 5. Resolution requirements

Before invocation, the gateway must establish a valid registered tool contract.

At minimum it must reject:

- unknown tool identity;
- unsupported or incompatible contract version;
- retired tool;
- invalid/conflicting registry metadata;
- unavailable required implementation;
- unsatisfied required dependency;
- stale registry state where freshness is required;
- capability/function binding inconsistent with the canonical contract.

Registry failure or uncertainty is not permission to execute.

## 6. Authorization requirements

The gateway must consume the canonical `AuthorizationResult` produced by the authorization boundary.

It must not accept any caller-supplied boolean, flag, role claim, registry membership, tool metadata or AI-generated statement as a substitute for the canonical decision.

At minimum:

- `allow` may proceed only within its returned constraints;
- `deny` must not execute;
- `not_applicable` must not execute;
- missing authorization must not execute;
- malformed authorization must not execute;
- stale/expired authorization must not execute;
- conflicting authorization must not execute;
- mismatched authorization/tool/action/resource references must not execute.

A downstream implementation may reduce scope but may never expand the authorized scope.

## 7. Data minimisation

The gateway must enforce the narrowest applicable authorized data scope.

A tool input must not contain protected data outside the authorized resource, purpose or data class merely because the implementation can technically accept it.

The gateway must support reduction or rejection of excess data before invocation where the canonical contract permits such enforcement.

Secrets, credentials, protected evidence and sensitive personal data must not be forwarded to an implementation unless explicitly authorized and required by the canonical tool contract.

## 8. Risk and approval

Tool risk metadata is a control input, not an authority grant.

For tools or Actions requiring explicit approval, the gateway must require the applicable canonical human approval and execution state.

AI/agent output, MCP metadata, provider claims, registry membership or a caller-provided `approved=true` style flag must never satisfy a human approval requirement.

High-impact execution must fail closed when required approval context is absent, stale, revoked, mismatched or otherwise invalid.

## 9. Implementation and provider neutrality

A registered tool may have multiple implementations/providers.

Provider selection must preserve the canonical tool contract and must not change authorization semantics. Selection may consider explicit implementation compatibility, availability and operational policy, but provider preference must never itself grant permission.

The gateway must not expose provider-specific authority concepts as canonical SIDERETH authorization.

An implementation adapter must be treated as an execution mechanism, not as a policy authority.

## 10. MCP and external adapters

MCP and other transports are optional adapters.

An adapter may discover or request a registered tool, but it must enter the Tool Gateway before execution.

External metadata, schemas, instructions and tool descriptions are untrusted input until validated by the applicable boundary.

No MCP server, agent, model, external service or transport may invoke an implementation by bypassing the gateway.

## 11. Provenance and audit

Every material invocation decision must be attributable to its canonical context.

At minimum, operational audit should preserve, where applicable:

- invocation identity;
- actor/subject;
- tool identity/version;
- capability/function binding;
- authorization reference and decision context;
- action/operation reference;
- approval reference where applicable;
- implementation/provider identity and version;
- relevant input/output hashes or canonical references;
- data/scope controls applied;
- timestamps;
- outcome/failure state;
- provenance references.

Tool invocation audit is distinct from Tool Registry mutation audit.

For evidence-producing or evidence-transforming tools, derived outputs must retain linkage to source evidence and transformation provenance.

## 12. Failure semantics

The gateway must fail closed for protected or consequential execution when required enforcement context cannot be established.

Failures must distinguish, where practical:

- tool not found;
- version incompatibility;
- lifecycle/retirement failure;
- dependency failure;
- authorization required/denied/not applicable;
- authorization scope mismatch;
- approval required/invalid;
- data minimisation failure;
- policy conflict;
- implementation unavailable;
- execution failure.

A generic transport error must never be converted into a successful execution decision.

Retries must preserve idempotency and must not accidentally repeat consequential operations.

## 13. Side effects and idempotency

Gateway validation and decision checks should be side-effect free until execution is explicitly permitted.

Consequential invocation must carry an operation/correlation identity suitable for the canonical idempotency infrastructure.

The gateway must not claim successful execution merely because an implementation request was dispatched.

Execution outcome must be recorded according to the applicable command/action and audit contracts.

## 14. Security invariants

A conformant gateway must preserve these invariants:

1. Tool discovery never grants authorization.
2. Tool Registry metadata never grants authorization.
3. AI/agent output never grants authorization.
4. MCP metadata never grants authorization.
5. Provider identity never grants authorization.
6. Caller-supplied authorization flags never grant authorization.
7. Human approval never overrides a denied authorization decision.
8. Authorization never manufactures human approval.
9. Downstream implementations cannot widen authorized scope.
10. Retired or unresolved tools cannot execute.
11. Stale/conflicting security context fails closed.
12. Every consequential execution is attributable and auditable.
13. Gateway behavior remains provider- and transport-neutral.

## 15. Conformance requirements

A conformant implementation must demonstrate, with implementation-independent tests:

1. exact tool identity/version binding;
2. capability/function binding validation;
3. lifecycle enforcement;
4. canonical AuthorizationResult enforcement;
5. denial of caller-supplied authorization assertions;
6. authorization reference correlation;
7. subject/action/resource/purpose scope enforcement;
8. jurisdiction/data-class enforcement;
9. approval enforcement;
10. human-versus-system/AI approval separation;
11. provider-neutral implementation selection;
12. retired/stale/conflicting state rejection;
13. MCP/transport isolation;
14. provenance and invocation auditability;
15. idempotent consequential execution semantics;
16. deterministic failure behavior;
17. no direct execution path outside the gateway;
18. compatibility with the canonical Capability, Tool Registry, Authorization and Action/Execution contracts.

## 16. Non-goals

This contract does not define:

- a specific HTTP/RPC/MCP transport;
- a specific policy language;
- an identity provider;
- a human approval UI;
- a model/agent framework;
- a particular tool runtime;
- a database or message broker;
- provider-specific trust as canonical authority;
- autonomous consequential legal action.

## 17. Implementation rule

The first implementation should be the smallest provider-neutral gateway capable of proving this contract.

It should initially provide:

- typed invocation and decision-boundary structures;
- Tool Registry resolution;
- canonical AuthorizationResult enforcement;
- Action/Execution Gate integration where applicable;
- explicit approval handling;
- deterministic scope/data checks;
- provider-neutral adapter invocation;
- audit/provenance hooks;
- idempotency correlation;
- adversarial conformance tests.

Distributed execution, remote registries, MCP transport, model-specific adapters and autonomous agents are not prerequisites for the initial conformance proof.
