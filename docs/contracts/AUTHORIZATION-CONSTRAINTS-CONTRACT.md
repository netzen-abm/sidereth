# Authorization Constraints Contract

**Status:** Proposed v0.1
**Scope:** SIDERETH canonical authorization contract

## 1. Purpose

This contract defines the canonical vocabulary and semantics for constraints returned by the SIDERETH authorization evaluator.

An authorization decision answers whether an operation is authorized in the evaluated context. A constraint narrows the authority granted by that decision. A consuming boundary MUST enforce every constraint that applies to the operation.

Constraints MUST NOT be interpreted as a second authorization evaluator, human approval, legal authority, evidence authenticity, execution result, or provider-trust decision.

## 2. Design rule

The canonical model MUST prefer typed, closed semantics over arbitrary string key/value interpretation.

The current wire representation is a string key/value pair. Until a typed wire representation is introduced, consumers MUST use the closed vocabulary defined here and MUST fail closed for an unknown key or value.

## 3. Canonical v0.1 vocabulary

### 3.1 Resource scope

`scope=exact_resource`

The authorization applies only to the exact `resource_ref` carried by the evaluated authorization request/result. A consumer MUST NOT substitute another resource merely because it is related to, contained by, or accessible from the authorized resource.

`scope=exact_evidence`

The authorization applies only to the exact evidence resource bound by the consuming evidence operation. A consumer MUST NOT substitute another evidence object or artifact.

### 3.2 Access mode

`access_mode=read_only`

The authorized operation MUST NOT mutate the protected resource, append state-changing events, or otherwise produce a protected write as part of the operation.

## 4. Fail-closed requirements

A consuming boundary MUST reject the authorization when:

- a constraint key is unknown;
- a known key has an unknown value;
- incompatible duplicate constraints are present;
- a required constraint cannot be applied to the actual invocation;
- the invocation would exceed the constrained resource or operation scope.

An `Allow` decision MUST therefore never be treated as unconditional permission.

## 5. Duplicate and conflict semantics

Constraints are conjunctive by default: every returned constraint applies.

For a single constraint family, duplicate values MUST be semantically compatible. If compatibility cannot be established deterministically, the consuming boundary MUST fail closed.

Examples:

- `scope=exact_resource` + `scope=exact_evidence` is invalid when the operation has one scope family and cannot satisfy both meanings.
- `access_mode=read_only` cannot be combined with a write requirement.

The canonical enforcement layer owns generic validation of the constraint vocabulary. Domain consumers own only the binding of a validated constraint to their domain operation.

## 6. Validity

Constraints are valid only within the authorization result's existing request context and validity interval. They do not extend `expires_at_epoch_seconds`, alter freshness, or authorize a different subject, action, resource, purpose, jurisdiction, or data class.

At exact expiry (`now == expires_at_epoch_seconds`), the authorization MUST be rejected.

## 7. Evolution

New constraint families MUST be added to this contract before production consumers depend on them.

A future typed Rust representation may replace the current string pair, for example an enum-backed constraint family with structured values. Such a change MUST preserve the fail-closed rule and explicit serialization compatibility.

Do not introduce tool-specific, provider-specific, agent-specific, or surface-specific authorization semantics into this contract.

## 8. Relationship to Tool Gateway

A future Tool Gateway MUST consume this canonical constraint contract. It MUST NOT create a parallel constraint vocabulary or reinterpret `Allow` as unconditional permission.

## 9. Non-goals

This contract does not define:

- policy authoring or policy storage;
- human approval workflows;
- legal authority;
- evidence authenticity or provenance;
- execution semantics;
- provider trust;
- tool registration or capability registration.
