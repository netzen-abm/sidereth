# SIDERETH — Authorization & Policy Conformance Matrix

**Status:** CANONICAL / FOUNDATION TEST DESIGN  
**Scope:** Implementation-independent conformance for authorization/policy evaluation

## Mandatory architectural invariants

1. Authorization is distinct from capability/tool registration.
2. Authorization is distinct from human approval.
3. Authorization does not create legal authority.
4. `allow` never overrides explicit constraints.
5. Protected operations fail closed on missing, stale, malformed or ambiguous authorization context.
6. Adapters cannot widen subject, resource, purpose, jurisdiction or data scope.
7. AI/system output cannot manufacture authorization or human approval.
8. Consequential Actions still require the canonical Action/Approval/Execution Gate.
9. Evaluation is deterministic for identical request/policy/context state.
10. Provider replacement preserves authorization semantics and failure behavior.
11. Policy references and freshness/version context remain attributable.
12. Authorization evaluation does not execute tools or mutate canonical domain state.

## Test matrix

| ID | Area | Required behavior | Adversarial condition |
|---|---|---|---|
| AUTH-001 | Identity | Subject reference is required | missing subject |
| AUTH-002 | Identity | Action reference is required | missing action |
| AUTH-003 | Scope | Resource/case scope is explicit where required | unrestricted resource |
| AUTH-004 | Purpose | Purpose is explicit for protected access | empty purpose |
| AUTH-005 | Policy | Applicable policy references are attributable | missing policy context |
| AUTH-006 | Decision | Only allow/deny/not_applicable are canonical decisions | unknown decision |
| AUTH-007 | Fail closed | Missing required authorization fails closed | absent result |
| AUTH-008 | Fail closed | Provider failure cannot become allow | provider unavailable |
| AUTH-009 | Fail closed | Malformed result cannot become allow | corrupt result |
| AUTH-010 | Fail closed | Stale/expired result fails closed when freshness is required | expired decision |
| AUTH-011 | Fail closed | Conflicting policy results fail closed | contradictory allow/deny |
| AUTH-012 | Scope | Subject scope cannot be widened | alternate actor injected |
| AUTH-013 | Scope | Resource scope cannot be widened | different case/resource |
| AUTH-014 | Scope | Purpose cannot be widened | different purpose |
| AUTH-015 | Scope | Jurisdiction cannot be widened | out-of-scope jurisdiction |
| AUTH-016 | Data | Data-class restriction is enforceable | restricted data requested under public scope |
| AUTH-017 | Data | Data minimisation survives downstream translation | adapter requests full record |
| AUTH-018 | Constraint | Returned constraints are enforceable | gateway ignores constraint |
| AUTH-019 | Constraint | Downstream may narrow but never expand scope | provider expands result |
| AUTH-020 | Registry | Capability registration is not authorization | registered capability but policy denies |
| AUTH-021 | Registry | Tool registration is not authorization | active tool but policy denies |
| AUTH-022 | Tool | Authorization does not execute a tool | evaluation side-effect spy |
| AUTH-023 | Approval | Allow does not imply human approval | consequential action without approval |
| AUTH-024 | Approval | Human approval cannot override authorization denial | approved but denied |
| AUTH-025 | Intelligence | AI cannot manufacture allow | forged model result |
| AUTH-026 | Intelligence | AI cannot manufacture approval | synthetic intelligence approval |
| AUTH-027 | Action | Consequential execution remains behind Action gate | direct execution attempt |
| AUTH-028 | Determinism | Same request/context yields same decision | repeated evaluation |
| AUTH-029 | Versioning | Policy version/freshness is reproducible | unrecorded policy revision |
| AUTH-030 | Audit | Decision attribution is preserved | missing actor/policy context |
| AUTH-031 | Audit | Authorization record is not itself legal authority | forged authorization record |
| AUTH-032 | Provider | Provider replacement preserves semantics | two providers, same policy |
| AUTH-033 | Provider | Provider-specific metadata cannot redefine scope | conflicting provider constraint |
| AUTH-034 | Adapter | Transport adapter cannot widen identity | forged transport actor |
| AUTH-035 | Adapter | MCP cannot bypass authorization | direct MCP invocation |
| AUTH-036 | Adapter | UI cannot bypass authorization | client-side allow flag |
| AUTH-037 | Jurisdiction | Out-of-scope operation is denied | boundary mismatch |
| AUTH-038 | Purpose | Reuse for a different purpose is denied/re-evaluated | purpose substitution |
| AUTH-039 | Freshness | Expiry is enforced before protected execution | old allow reused |
| AUTH-040 | Error | NotApplicable is not treated as allow | no applicable policy |
| AUTH-041 | Error | Empty constraints do not erase required scope | omitted restriction |
| AUTH-042 | Security | Tool metadata cannot escalate privilege | forged permission metadata |
| AUTH-043 | Security | External content cannot alter authorization semantics | prompt/tool injection |
| AUTH-044 | Security | Authorization cannot be bypassed by alternate provider | provider failover escalation |
| AUTH-045 | Isolation | One malformed policy cannot corrupt unrelated evaluations | poisoned neighboring policy |
| AUTH-046 | Isolation | Caller cannot mutate authoritative result semantics | returned object mutation |
| AUTH-047 | Concurrency | Concurrent evaluations preserve deterministic policy state | simultaneous policy update |
| AUTH-048 | Replay | Expired authorization cannot be replayed for protected execution | replayed allow |
| AUTH-049 | Provenance | Policy source/version is retained where required | missing policy provenance |
| AUTH-050 | Boundary | Authorization remains a decision boundary, not an execution boundary | evaluator invokes implementation |

## Promotion gates

An authorization implementation is not conforming until:

- all mandatory invariants are demonstrated;
- AUTH-001–AUTH-050 are satisfied or explicitly marked not applicable with rationale;
- failure-path behavior is demonstrated;
- scope and data-minimisation enforcement is tested at the consuming boundary;
- provider and transport independence is demonstrated;
- compatibility with Action/Approval/Execution Gate is demonstrated;
- security/privacy review is complete for the implementation scope;
- CI passes on the exact PR head.

## Explicit non-goals

This matrix does not define a particular identity provider, policy language, distributed authorization service, Tool Gateway implementation, Tool Runtime, MCP implementation, human approval service, legal reasoning engine or autonomous agent.
