# Tool Gateway Conformance Matrix v0.1

**Status:** Required test specification for Issue #111.

**Companion contract:** `docs/contracts/TOOL-GATEWAY-CONTRACT-V0-1.md`

The matrix is intentionally implementation-neutral. A bounded gateway implementation must demonstrate every applicable invariant before production readiness is claimed.

| ID | Scenario | Required invariant | Expected outcome |
|---|---|---|---|
| TG-01 | Exact context match | Invocation and AuthorizationResult are exactly bound | Proceed to next gate |
| TG-02 | Subject mismatch | Subject cannot change after authorization | Denied |
| TG-03 | Actor mismatch | Actor cannot change where actor binding is present | Denied |
| TG-04 | Action mismatch | Authorized action is immutable for execution | Denied |
| TG-05 | Resource mismatch | Authorized resource is immutable for execution | Denied |
| TG-06 | Purpose mismatch | Authorized purpose is immutable | Denied |
| TG-07 | Jurisdiction mismatch | Jurisdiction binding cannot broaden/change silently | Denied |
| TG-08 | Data-class mismatch | Data-class binding cannot broaden silently | Denied |
| TG-09 | Stale authorization | Freshness requirement must be satisfied | Denied / stale |
| TG-10 | Exact expiry | `now >= expires_at` means expired | Expired |
| TG-11 | Returned constraint satisfied | All applicable constraints are enforced | Proceed |
| TG-12 | Unsupported constraint | Unknown/unsupported semantic value fails closed | ConstraintFailed |
| TG-13 | Conflicting constraint | Conflicting same-key constraints fail closed | ConstraintFailed |
| TG-14 | Missing constraint context | Required execution context must be present | ConstraintFailed |
| TG-15 | Scope widening | Execution scope cannot exceed authorized scope | ConstraintFailed |
| TG-16 | Required lease missing | Leased capability requires valid lease | LeaseInvalid |
| TG-17 | Lease purpose/version mismatch | Lease must bind to authorization purpose/version | LeaseInvalid |
| TG-18 | Lease scope widening | Lease cannot authorize more than permitted scope | LeaseInvalid |
| TG-19 | Lease expired | Expired lease cannot activate/use capability | LeaseInvalid / Expired |
| TG-20 | Lease revoked/cancelled | Terminal invalid lease cannot be reused | LeaseInvalid |
| TG-21 | Registry-only authority attempt | Tool/provider registry metadata cannot grant authority | Denied |
| TG-22 | Provider identity captured | Provider/adapter identity is preserved in provenance | Pass with provenance |
| TG-23 | Provider failure | Provider failure is distinct from authorization denial | ProviderFailed |
| TG-24 | Timeout / uncertain completion | Unknown completion remains distinguishable | Unknown |
| TG-25 | Durable duplicate | Same authorized operation cannot execute twice after durable commit | Existing outcome / no duplicate |
| TG-26 | Concurrent duplicate | Concurrent identical authorized operation is serialized/suppressed | One execution |
| TG-27 | Restart recovery | Process restart cannot erase durable idempotency state | No unsafe duplicate |
| TG-28 | Queued operation expires | Authorization/lease expiry before execution prevents use | Expired |
| TG-29 | Offline bounded execution | Offline mode cannot extend authority, scope, purpose, or lease | Execute only while still valid; otherwise deny |
| TG-30 | Transport replacement | Changing transport does not change authorization semantics | Same decision semantics |
| TG-31 | Provenance continuity | Transport/provider changes do not erase execution lineage | Complete lineage |
| TG-32 | Audit completeness | Authorization-to-execution chain is reconstructable | Required audit present |
| TG-33 | Direct bypass | Protected gateway-mediated operation cannot bypass canonical checks | Blocked / explicitly classified |
| TG-34 | MCP adapter boundary | MCP cannot create or bypass policy semantics | Same canonical gateway checks |

## Evidence required per test

For each executed test, retain enough machine-readable evidence to establish:

- implementation revision / commit SHA;
- test identifier;
- input authorization context;
- relevant lease context;
- trusted clock value where expiry matters;
- decision/error state;
- whether provider execution occurred;
- idempotency lifecycle state where applicable;
- provenance/audit references;
- pass/fail result.

Sensitive test fixtures must use synthetic/minimized data. Production data must not be required to prove these invariants.

## Failure interpretation

A failing test is a contract failure, not a reason to weaken the test. If an implementation intentionally differs, the exception must be documented with scope, threat analysis, compensating control, and explicit approval before production-readiness claims.

## Release gate

The matrix is complete only when every applicable row has an automated or otherwise reproducible verification. Rows marked not applicable require a written rationale tied to the bounded implementation scope.
