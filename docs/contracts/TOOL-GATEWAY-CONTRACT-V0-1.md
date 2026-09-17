# Tool Gateway Contract v0.1

**Status:** Normative implementation contract for the bounded Tool Gateway gate (Issue #111).

**Scope:** Define the boundary between SIDERETH's canonical control plane and tool/provider execution without introducing a second authorization model.

## 1. Architectural position

The Tool Gateway is a **consumer-side execution boundary**, not a policy engine.

Canonical path:

```text
Tool Invocation Request
        ↓
Canonical Authorization Evaluator
        ↓
Canonical AuthorizationResult
        ↓
Canonical Authorization Enforcement
        ↓
Typed Constraint Enforcement
        ↓
Capability Lease Validation (when required)
        ↓
Tool Registry Resolution
        ↓
Durable Idempotency
        ↓
Provider Adapter
        ↓
Execution / Human Approval Boundary (when required)
        ↓
Result + Provenance + Audit
```

The gateway must not reinterpret or replace authorization semantics established upstream.

## 2. Gateway input contract

A gateway invocation MUST carry, directly or through an immutable bound request object, the context needed to prove that the execution request is the same operation that was authorized:

- `request_id`
- `authorization_ref`
- `subject_ref`
- `actor_ref` where applicable
- `action`
- `resource_ref`
- `purpose`
- `jurisdiction_ref` where applicable
- `data_class` where applicable
- `tool_ref`
- provider/implementation selection metadata
- `idempotency_ref`
- lease reference when a leased capability is required
- relevant incident/session/context references

Omission is permitted only for fields that the canonical authorization contract itself defines as optional. The gateway MUST NOT invent missing authority from tool metadata, provider identity, transport state, session presence, or capability possession.

## 3. Exact authorization binding

Before provider execution, the gateway MUST invoke the canonical authorization-enforcement boundary and require exact contextual compatibility between the invocation and the `AuthorizationResult`.

At minimum, the following must not silently change between authorization and execution:

- subject
- actor, when present
- action
- resource
- purpose
- jurisdiction, when present
- data class, when present
- authorization reference
- relevant session/incident/context binding

A mismatch MUST fail closed.

The gateway is prohibited from constructing a substitute `AuthorizationResult` merely to make a request executable.

## 4. Constraint enforcement

All returned authorization constraints are execution prerequisites.

The gateway MUST enforce the canonical constraint semantics before invoking a provider. This includes:

- unsupported constraint values → fail closed;
- conflicting values for the same semantic key → fail closed;
- required constraint absent from the execution context → fail closed;
- constraint scope broader than authorized scope → fail closed;
- equivalent supported constraints may be treated idempotently according to the canonical constraint contract.

The gateway MUST NOT silently discard, weaken, or reinterpret a returned constraint.

## 5. Expiry and trusted time

Authorization freshness and expiry use the canonical trusted execution clock.

The invariant is:

```text
now >= expires_at  ⇒  authorization is expired  ⇒  execution denied
```

The gateway MUST NOT extend, refresh, or infer a new expiry locally.

A queued or offline request does not gain authority merely because it was authorized earlier. If execution occurs after expiry, it is denied unless a new canonical authorization has been obtained.

## 6. Capability Lease separation

A capability lease is a separate control-plane object from authorization.

When a tool requires a leased capability, the gateway MUST validate the lease independently for:

- capability identity
- resource identity
- subject/actor binding
- purpose and purpose version
- authorization reference
- scope
- session/incident/context where required
- state
- expiry
- revocation/cancellation

An OS permission, provider token, adapter handle, or tool registry entry MUST NOT substitute for a valid SIDERETH capability lease where a lease is required.

A lease MUST NOT create authority that the canonical authorization did not grant.

## 7. Tool and provider registry

Registry metadata is discovery and implementation metadata, not authorization.

The gateway MAY use registry information to resolve:

- tool identity and version
- provider/implementation identity
- supported capabilities
- protocol/adapter information
- provenance metadata
- operational compatibility

Registry membership, ranking, availability, provider preference, or adapter identity MUST NOT grant execution authority.

## 8. Provider boundary

Providers and adapters are untrusted execution dependencies from the gateway's authorization perspective.

The gateway MUST pass only the data and authority necessary for the already-authorized operation. Provider output MUST NOT be treated as proof of authorization.

Provider failures, malformed responses, timeouts, unavailable dependencies, and uncertain completion MUST remain distinguishable from authorization denial.

Provider identity and implementation version MUST be captured in execution provenance where available.

## 9. Durable concurrent idempotency

Idempotency is an execution-safety control, not an authorization mechanism.

For operations that can produce side effects, the gateway MUST bind idempotency to the authorized operation context rather than to an untrusted caller-provided key alone.

The durable idempotency record MUST prevent concurrent duplicate execution for the same authorized operation according to the selected policy.

At minimum, the binding MUST cover:

- operation/request identity
- authorization reference
- subject/actor context
- action/resource/tool identity
- relevant purpose/scope
- idempotency reference
- lifecycle state
- result or terminal outcome where retained

A process restart MUST NOT silently permit a previously committed operation to execute again.

The bounded implementation MUST use a durable store or transactional primitive appropriate to the deployment; an in-memory guard alone is not production-compliant.

## 10. Error semantics

The gateway MUST preserve materially distinct terminal/error states. At minimum:

- `Denied` — canonical authorization decision does not permit execution;
- `Expired` — authorization or required lease is no longer valid;
- `ConstraintFailed` — an authorized constraint cannot be satisfied by the proposed execution context;
- `LeaseInvalid` — required capability lease is absent, mismatched, revoked, cancelled, or otherwise invalid;
- `ProviderFailed` — provider/adapter execution failed after gateway prerequisites passed;
- `Unknown` — completion state cannot safely be established.

These states MUST NOT be collapsed into a generic success/failure result where doing so would obscure whether execution occurred or whether authority was present.

## 11. Delay, offline, and constrained transport

The gateway is transport-neutral.

It MUST support explicit asynchronous lifecycle states where the implementation needs queueing or delayed execution, including at least:

```text
ACCEPTED → QUEUED → IN_PROGRESS → COMPLETED
                         ↘ FAILED
                         ↘ UNKNOWN
```

Expiration, cancellation, revocation, or constraint failure may terminate queued work before execution.

Offline or delayed execution MAY preserve a previously authorized bounded operation only while all applicable authorization, constraint, lease, scope, and expiry requirements remain valid.

Connectivity MUST NOT be treated as authority.

Transport replacement MUST NOT change authorization semantics or erase provenance.

## 12. Minimum Sufficient Information

The gateway should minimize duplicated payload metadata while retaining every field required for authorization, safety, idempotency, provenance, audit, legal preservation, and correct execution.

Stable canonical references SHOULD be preferred over repeatedly copying large metadata structures when the reference remains resolvable and its semantics are immutable for the operation.

Compression or compact encoding MUST NOT remove information required to establish:

- who/what is acting;
- what operation is authorized;
- what resource is affected;
- why it is permitted;
- what constraints apply;
- when it is valid;
- which capability lease applies;
- what actually executed;
- what provider was used;
- what evidence/provenance must be retained.

## 13. Provenance and audit

A gateway execution record MUST preserve enough context to reconstruct the authorization-to-execution chain, subject to the applicable privacy and retention policy.

Where applicable, record:

- request and authorization references;
- subject/actor references;
- action/resource/tool;
- purpose and scope;
- lease reference;
- provider/adapter identity and version;
- execution timestamps;
- idempotency reference and lifecycle state;
- result/error state;
- evidence/provenance references;
- uncertainty or interrupted-completion state.

Audit is evidence of what the system recorded; it is not an alternate source of execution authority.

## 14. MCP boundary

MCP, if adopted, is an interoperability/adapter surface.

An MCP client/server/adapter MUST terminate at the same gateway/control-plane contracts. MCP messages, tool declarations, server identity, or transport connections MUST NOT become a second policy boundary or authorization vocabulary.

MCP adoption is therefore implementation work downstream of this contract, not a reason to alter canonical authorization semantics.

## 15. Direct bypass prohibition

No production provider or adapter path may execute a protected operation by bypassing the gateway's required control-plane checks when that operation is designated as gateway-mediated.

Any intentionally direct path MUST be explicitly classified, documented, and governed by an equivalent canonical control boundary; it MUST NOT be an undocumented exception.

## 16. Non-goals

This contract does not introduce:

- a new policy engine;
- new authorization semantics;
- a replacement for human approval;
- a replacement for capability leases;
- provider-derived authority;
- autonomous unrestricted agent authority;
- external provider integrations;
- hardware/edge implementations;
- MCP server expansion;
- automatic OS permission revocation.

## 17. Required conformance matrix

The bounded implementation is not production-ready until tests cover at least:

1. exact request/result context match;
2. subject mismatch;
3. actor mismatch;
4. action mismatch;
5. resource mismatch;
6. purpose mismatch;
7. jurisdiction/data-class mismatch where applicable;
8. stale authorization;
9. exact expiry boundary (`now == expires_at`);
10. returned constraint enforcement;
11. unsupported constraint;
12. conflicting constraint;
13. missing required constraint context;
14. scope widening;
15. missing required lease;
16. lease purpose/version mismatch;
17. lease scope widening;
18. lease expiry/revocation/cancellation;
19. registry metadata cannot grant authority;
20. provider failure after authorization;
21. timeout/unknown completion;
22. durable duplicate suppression;
23. concurrent duplicate suppression;
24. restart after an accepted/committed operation;
25. queued operation expires before execution;
26. offline execution cannot extend authority;
27. transport/provider replacement preserves semantics;
28. provenance/audit completeness;
29. direct bypass classification;
30. MCP adapter cannot bypass the gateway.

## 18. Production-readiness rule

A Tool Gateway implementation may be called **production-ready** only when:

- this contract is implemented or an explicit bounded exception is documented;
- the conformance matrix is complete for the implemented scope;
- exact-head Foundation and Security/Supply Chain gates are green;
- durable concurrent idempotency is demonstrated for the deployment's persistence model;
- direct bypass analysis is complete;
- final security/diff review finds no unresolved critical or high-severity boundary defect.

Until then, the gateway remains an implementation under bounded development.
