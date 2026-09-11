# SIDERETH — Authorization & Policy Conformance Matrix

**Status:** CANONICAL / FOUNDATION TEST DESIGN  
**Scope:** Implementation-independent conformance for authorization and policy evaluation

A conforming implementation must satisfy the Authorization & Policy Decision Contract regardless of policy engine, storage technology, programming language, provider, deployment topology or identity system.

## Mandatory invariants

1. Deny by default.
2. Authorization failure cannot become allow.
3. `not_applicable` is not an authorization grant.
4. Evaluation is side-effect free.
5. Registry membership cannot authorize execution.
6. Authorization cannot manufacture human approval.
7. Human approval cannot substitute for missing authorization.
8. Legal authority is not inferred from authorization.
9. Purpose, resource and case scope are explicit.
10. Constraints are preserved and enforceable downstream.
11. Policy references/version context are attributable.
12. Conflicts require deterministic explicit resolution or fail closed.
13. Stale/invalid/unavailable policy cannot silently downgrade.
14. AI/agent output cannot self-authorize.
15. Consequential execution cannot proceed from an unverified authorization state.

## Test matrix

| ID | Area | Required behavior | Adversarial condition |
|---|---|---|---|
| AP-001 | Request | Reject missing subject | empty subject reference |
| AP-002 | Request | Reject missing action | empty action reference |
| AP-003 | Request | Reject missing resource | empty resource reference |
| AP-004 | Request | Require purpose | empty purpose |
| AP-005 | Request | Reject malformed references | invalid resource reference |
| AP-006 | Default | Deny when no applicable policy exists | empty policy set |
| AP-007 | Default | Never treat `not_applicable` as allow | evaluator returns not_applicable |
| AP-008 | Policy | Reject malformed policy | invalid policy document |
| AP-009 | Policy | Fail closed when policy unavailable | provider timeout |
| AP-010 | Policy | Fail closed on stale required policy | expired policy |
| AP-011 | Policy | Do not silently downgrade | newer required version unavailable |
| AP-012 | Conflict | Resolve conflict deterministically | allow + deny conflict |
| AP-013 | Conflict | Fail closed without precedence rule | conflicting equal-priority policies |
| AP-014 | Scope | Enforce case scope | request for unrelated case |
| AP-015 | Scope | Enforce resource scope | resource outside granted scope |
| AP-016 | Purpose | Enforce purpose limitation | different purpose from grant |
| AP-017 | Jurisdiction | Enforce jurisdiction constraint | mismatched jurisdiction |
| AP-018 | Data class | Enforce data-class restriction | restricted data requested |
| AP-019 | Time | Reject not-yet-effective policy | future validity window |
| AP-020 | Time | Reject expired grant | expired validity window |
| AP-021 | Constraints | Preserve read-only constraint | update requested under read-only grant |
| AP-022 | Constraints | Preserve field restriction | restricted field requested |
| AP-023 | Constraints | Preserve destination restriction | unauthorized destination |
| AP-024 | Identity | Do not infer authority from role label alone | forged role metadata |
| AP-025 | Registry | Registry membership does not authorize | registered tool invoked without grant |
| AP-026 | Approval | Authorization does not manufacture approval | allow without required approval |
| AP-027 | Approval | Approval does not bypass authorization | approval present, authorization denied |
| AP-028 | AI | AI-generated permission is non-authorizing | model outputs allow |
| AP-029 | Agent | Agent cannot rewrite policy context | altered purpose/scope |
| AP-030 | Side effects | Evaluation performs no mutation | evaluation causes state change |
| AP-031 | Side effects | Evaluation performs no tool invocation | evaluator calls tool |
| AP-032 | Provenance | Preserve policy references | decision lacks policy provenance |
| AP-033 | Determinism | Same inputs produce same decision | repeated identical evaluation |
| AP-034 | Audit | Consequential decision is attributable | missing correlation/audit context |
| AP-035 | Error | Distinguish policy failure from deny where contract requires | provider error collapsed ambiguously |
| AP-036 | Execution | Denied decision blocks execution | downstream gate receives deny |
| AP-037 | Execution | Not-applicable blocks execution | downstream gate receives not_applicable |
| AP-038 | Execution | Unknown authorization state blocks execution | missing decision |
| AP-039 | Provider | Provider replacement preserves semantics | alternate evaluator implementation |
| AP-040 | Concurrency | Policy snapshot is consistent for one evaluation | policy changes mid-evaluation |

## Required end-to-end proofs

A future integration/conformance suite must demonstrate at minimum:

```text
ALLOW + required approval  → execution may proceed
ALLOW + missing approval   → execution blocked
DENY                       → execution blocked
NOT_APPLICABLE             → execution blocked
policy unavailable         → execution blocked
policy stale               → execution blocked
policy conflict            → deterministic resolution or blocked
purpose mismatch           → blocked
scope mismatch             → blocked
data-class violation       → blocked
AI self-authorization      → blocked
registry-only permission   → blocked
```

Passing this document is a conformance gate, not a claim that a deployment is security-verified or production-ready.
