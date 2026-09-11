# SIDERETH — Tool Gateway Conformance Matrix

**Status:** DESIGNED / CONTRACTED
**Scope:** Implementation-independent conformance requirements for the canonical Tool Gateway
**Canonical contract:** `docs/contracts/TOOL-GATEWAY-CONTRACT.md`

## 1. Status vocabulary

This matrix uses the SIDERETH lifecycle vocabulary:

```text
VISION → DESIGNED → IMPLEMENTED → FUNCTIONAL → TESTED
        → SECURITY-VERIFIED → PRIVACY-VERIFIED → PRODUCTION-READY
```

A requirement is not considered implemented merely because the contract describes it. Until executable evidence exists, the requirement remains `DESIGNED` or `CONTRACTED`.

## 2. Conformance matrix

| ID | Requirement | Required proof | Status |
|---|---|---|---|
| TG-001 | Stable tool identity required | Invocation rejects missing/invalid tool identity | DESIGNED |
| TG-002 | Exact/compatible tool contract version enforced | Version mismatch test | DESIGNED |
| TG-003 | Retired tools cannot execute | Retired-tool negative test | DESIGNED |
| TG-004 | Active lifecycle required for execution | Lifecycle gate test | DESIGNED |
| TG-005 | Capability binding validated | Mismatched capability test | DESIGNED |
| TG-006 | Function binding validated where required | Mismatched function test | DESIGNED |
| TG-007 | Registry discovery is not authorization | Registry-only invocation negative test | DESIGNED |
| TG-008 | Registry metadata cannot grant permission | Metadata manipulation negative test | DESIGNED |
| TG-009 | Canonical AuthorizationResult required | Missing-result negative test | DESIGNED |
| TG-010 | `allow` is the only executable authorization decision | Allow/Deny/NotApplicable matrix | DESIGNED |
| TG-011 | `deny` fails closed | Deny execution test | DESIGNED |
| TG-012 | `not_applicable` fails closed | NotApplicable execution test | DESIGNED |
| TG-013 | Malformed authorization fails closed | Malformed-result test | DESIGNED |
| TG-014 | Stale/expired authorization fails closed | Freshness/expiry test | DESIGNED |
| TG-015 | Conflicting authorization fails closed | Conflict test | DESIGNED |
| TG-016 | Authorization reference correlates to invocation/action | Reference mismatch test | DESIGNED |
| TG-017 | Subject scope is enforced | Subject mismatch test | DESIGNED |
| TG-018 | Action/operation scope is enforced | Action mismatch test | DESIGNED |
| TG-019 | Resource/case scope is enforced | Resource mismatch test | DESIGNED |
| TG-020 | Declared purpose is enforced | Purpose mismatch test | DESIGNED |
| TG-021 | Jurisdiction constraints are enforced | Jurisdiction mismatch test | DESIGNED |
| TG-022 | Data-class constraints are enforced | Data-class mismatch test | DESIGNED |
| TG-023 | Returned authorization constraints are enforced | Constraint reduction/violation tests | DESIGNED |
| TG-024 | Caller flags cannot widen authorization | `authorized=true` style bypass test | DESIGNED |
| TG-025 | AI/agent output cannot grant authorization | AI assertion bypass test | DESIGNED |
| TG-026 | MCP metadata cannot grant authorization | MCP assertion bypass test | DESIGNED |
| TG-027 | Provider identity cannot grant authorization | Provider trust bypass test | DESIGNED |
| TG-028 | Approval remains distinct from authorization | Authorization-without-approval test | DESIGNED |
| TG-029 | Human approval required where contract requires it | Missing-approval negative test | DESIGNED |
| TG-030 | Non-human approval cannot satisfy human approval | System/AI approval negative test | DESIGNED |
| TG-031 | Revoked approval cannot execute | Revoked-approval negative test | DESIGNED |
| TG-032 | Mismatched approval cannot execute | Approval reference mismatch test | DESIGNED |
| TG-033 | Execution Gate remains authoritative for Actions | Action execution integration test | DESIGNED |
| TG-034 | Gateway does not duplicate Execution Gate semantics | Architectural/code-boundary review | DESIGNED |
| TG-035 | Provider selection is contract-compatible | Multi-provider compatibility test | DESIGNED |
| TG-036 | Provider replacement cannot widen authority | Provider substitution negative test | DESIGNED |
| TG-037 | Adapter cannot bypass gateway | Direct-adapter execution negative test | DESIGNED |
| TG-038 | External transport cannot bypass gateway | Transport isolation test | DESIGNED |
| TG-039 | Excess data is reduced or rejected | Data minimisation test | DESIGNED |
| TG-040 | Unauthorized protected data is never forwarded | Input capture/negative test | DESIGNED |
| TG-041 | Consequential operations carry idempotency identity | Missing-idempotency negative test | DESIGNED |
| TG-042 | Retry cannot duplicate consequential execution | Replay/idempotency test | DESIGNED |
| TG-043 | Validation remains side-effect free before permission | Pre-execution side-effect test | DESIGNED |
| TG-044 | Dispatch is not reported as successful completion | Outcome semantics test | DESIGNED |
| TG-045 | Invocation is attributable | Audit record test | DESIGNED |
| TG-046 | Authorization context is attributable | Audit correlation test | DESIGNED |
| TG-047 | Provider/implementation identity is attributable | Implementation audit test | DESIGNED |
| TG-048 | Evidence transformations preserve provenance linkage | Evidence provenance test | DESIGNED |
| TG-049 | Failure classes are deterministic and explicit | Failure taxonomy test | DESIGNED |
| TG-050 | Registry unavailability never becomes permission | Registry outage negative test | DESIGNED |
| TG-051 | Required dependency failure blocks execution | Dependency negative test | DESIGNED |
| TG-052 | Stale/conflicting registry state fails closed | Registry freshness/conflict test | DESIGNED |
| TG-053 | Provider/transport neutrality is preserved | Implementation-independent contract tests | DESIGNED |
| TG-054 | No direct implementation execution path exists | Repository/code-path audit | DESIGNED |
| TG-055 | Capability contract remains canonical | Contract binding/conformance review | DESIGNED |
| TG-056 | Tool Registry remains discovery/lifecycle authority only | Boundary review + negative test | DESIGNED |
| TG-057 | Authorization remains policy permission authority | Boundary review + integration test | DESIGNED |
| TG-058 | Human approval remains approval authority | Boundary review + integration test | DESIGNED |
| TG-059 | Execution Gate remains Action execution authority | Boundary review + integration test | DESIGNED |
| TG-060 | Gateway remains provider/transport neutral | Architecture review | DESIGNED |

## 3. Required adversarial test families

The implementation PR must include negative tests for at least these bypass classes:

### Authorization bypass

- registry entry exists but no authorization result;
- caller sets an authorization/permission flag;
- authorization result is `deny`;
- authorization result is `not_applicable`;
- authorization reference belongs to another action/resource;
- authorization is stale, malformed or conflicting.

### Approval bypass

- tool metadata says approval is optional while canonical action requires it;
- caller supplies an approval flag;
- AI/agent supplies an approval assertion;
- system approval is presented where human approval is required;
- approval is revoked or references another action.

### Scope/data bypass

- different subject;
- different resource/case;
- different purpose;
- different jurisdiction;
- broader data class;
- implementation receives data outside the authorized scope.

### Registry/provider bypass

- retired tool;
- stale registry state;
- conflicting registry metadata;
- provider marked preferred without authorization;
- implementation invoked directly without gateway.

### Transport/AI bypass

- MCP tool invocation bypasses gateway;
- agent invokes provider directly;
- model-generated tool instruction is treated as authority;
- external tool description changes canonical permission semantics.

## 4. Evidence requirements

Before an implementation can claim `TESTED`, evidence should include:

1. deterministic unit tests;
2. integration tests across Registry → AuthorizationResult → Gateway → Execution Gate where applicable;
3. adversarial negative tests;
4. audit/provenance assertions;
5. idempotency/replay assertions for consequential operations;
6. repository/code-path audit showing no bypass route;
7. provider-neutral contract tests.

`SECURITY-VERIFIED` requires security-focused review of the complete invocation boundary, not only successful-path tests.

`PRIVACY-VERIFIED` requires evidence that data minimisation, scope and protected-data handling are enforced at the gateway boundary.

## 5. Merge gate

The Tool Gateway implementation must not be merged as production-ready merely because the happy path works.

Minimum merge evidence:

- all mandatory TG requirements implemented;
- all mandatory adversarial tests green;
- exact-head CI green;
- security/supply-chain checks green;
- no direct execution bypass discovered;
- documentation status updated only to the highest evidenced lifecycle state.

## 6. Non-conformance rule

If implementation behavior conflicts with the canonical Tool Gateway Contract, the implementation is non-conformant even if a particular provider, transport, model or test harness accepts the behavior.

The contract is the source of truth; implementation convenience does not override the boundary.
