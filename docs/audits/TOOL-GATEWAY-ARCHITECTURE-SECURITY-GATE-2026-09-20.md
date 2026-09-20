# SIDERETH Tool Gateway Architecture & Security Gate Audit — 2026-09-20

## Scope
Audit of the current bounded Tool Gateway implementation on branch architecture/durable-tool-gateway-idempotency-v0-1-2026-09-19 against GitHub Issue #111 and the canonical TOOL-GATEWAY-CONTRACT.md.

## Foundation evidence
Foundation Validation run #753 passed at commit 71fba979d716ebfb943beb85a0e685f47d0cdef4.
Verified gates: required foundation documents; SIDERETH-only active documentation identity; phantom/inherited CLI claim rejection; obsolete legacy protocol reference scan; direct secret-handling scan; repository structure; rustfmt; compilation; tests; live PostgreSQL proof; Clippy with -D warnings.

The live PostgreSQL proof matrix passed 7/7: duplicate insert atomicity; case/evidence trust rollback atomicity; case/evidence trust commit atomicity; CAS race single-writer behavior; Tool Gateway idempotency concurrent claim single winner; Tool Gateway idempotency claim survives restart; failed command rollback of resource writes.

Security & Supply Chain run #441 also passed.

## Audit conclusion
The current implementation is a sound bounded foundation, but it is not yet conformant with Issue #111 / the v0.1 Tool Gateway contract. The Foundation green result proves build/test quality; it does not prove the complete Tool Gateway security contract.

Do not merge the Tool Gateway as production-ready yet.

## Confirmed strengths
1. Canonical authorization consumption — ToolGateway::validate calls canonical authorization enforcement and does not create a second policy evaluator.
2. Exact request/result binding — request identity, authorization reference, subject, action, resource, purpose, jurisdiction and data class are compared before execution.
3. Constraint enforcement — returned authorization constraints are validated and unsupported constraint vocabulary fails closed.
4. Canonical expiry — authorization expiry is checked at now >= expires_at.
5. Capability lease validation — required leases are required and validated separately.
6. Registry lifecycle/version enforcement — tool identity/version, lifecycle, execution mode, data class and jurisdiction are enforced.
7. Durable idempotency foundation — PostgreSQL proves concurrent claim single-winner and restart persistence.
8. Side-effect ordering — validation occurs before the idempotency claim in execute.

## Blocking conformance gaps

### TG-01 — Capability/function binding is incomplete (P0)
ToolGatewayInvocation carries no canonical capability_ref or function_ref. The registry contains these fields, but the invocation does not explicitly bind to them.
Required: carry canonical capability reference; carry function reference where applicable; compare invocation context against registry contract; fail closed on mismatch.

### TG-02 — Provider/implementation identity is not bound (P0)
ToolGatewayProvider has no canonical implementation/provider identity. execute receives an arbitrary provider object while the registry may contain multiple implementations.
Required: provider identity contract; implementation identity/version contract; registry-to-provider compatibility check; auditable selected implementation identity.

### TG-03 — Human approval is not integrated with the canonical Execution Gate (P0)
When tool.approval_required is true, the gateway currently returns ApprovalRequired unconditionally. It does not consume the canonical Action/ApprovalRecord/ExecutionGate boundary.
This is fail-closed and therefore safer than accepting a caller flag, but it is incomplete against Issue #111 and the contract.
Required: consume canonical Action/approval context where applicable; invoke ExecutionGate::permit; require human approval where the Action contract requires it; never manufacture approval.

### TG-04 — Invocation audit/provenance is absent (P0)
Tool Registry mutation audit exists, but Tool Gateway invocation audit is not yet emitted.
Required operational record: invocation identity; actor/subject; tool/version; capability/function; authorization context; action/operation; approval; implementation/provider; relevant input/output hashes or references; scope/data controls; outcome/failure; provenance.

### TG-05 — Data minimisation is under-specified (P1)
Current invocation contains data class and requested scope but no canonical input payload/reference or explicit minimisation/reduction operation.
Required: canonical input/reference; enforce resource/purpose/data-class boundary; support reduction or reject excess data before provider execution; never forward protected data merely because provider accepts it.

### TG-06 — Public claim() permits phase bypass (P1)
ToolGateway::claim is public and can be called independently of validate. The comment recommends execute, but the API does not enforce the invariant.
Preferred fix: make claim internal/private, or require an unforgeable validated-phase token/type.

### TG-07 — Unknown-state semantics are incomplete (P1)
The lifecycle store has Unknown, but ToolGateway::execute never records Unknown. A process failure after dispatch can leave InProgress without an explicit indeterminate outcome.
Required: define crash/timeout/indeterminate semantics; preserve Unknown when outcome cannot safely be classified; define recovery/reconciliation.

### TG-08 — Provider failure and lifecycle-persistence failure need separation (P1)
If provider execution succeeds but mark_completed fails, the gateway returns an idempotency error even though the provider may already have performed the side effect. This is an indeterminate outcome and must not be represented as a simple execution failure.
Required: distinguish provider outcome from lifecycle persistence outcome; preserve Unknown when final outcome cannot be established; define retry semantics.

## Non-blocking observations
AuthorizationPolicy/AccessRequest is a legacy/simple policy abstraction retained alongside the canonical AuthorizationEvaluator. It is not used by the current Gateway path, but future convergence should prevent accidental reuse for protected Tool Gateway operations.
ObservationCommand and ObservationLifecycle have local contextual authorization validators. They correctly call canonical enforcement but duplicate contextual binding logic. This is a future convergence target.
CaseService already uses the authoritative Unit-of-Work command path.
Intelligence is explicitly modeled as untrusted computation and does not itself grant authority.

## Decision
Foundation: GREEN.
Tool Gateway Issue #111 merge/security gate: NOT YET GREEN.

Next implementation order:
1. capability/function binding;
2. provider/implementation identity binding;
3. canonical Action/Execution Gate integration;
4. invocation audit/provenance;
5. private validated-to-claim transition;
6. indeterminate/Unknown lifecycle semantics;
7. data-minimisation contract.

Only after these changes have implementation-independent adversarial tests and a fresh exact-head Foundation + Security/Supply Chain run should Issue #111 be considered ready for merge.

## Programming-language decision
No additional programming language is justified by this gate. The current work remains entirely within the canonical Rust domain/control-plane runtime. External languages remain adapter candidates only when a concrete capability requirement demonstrates material benefit.