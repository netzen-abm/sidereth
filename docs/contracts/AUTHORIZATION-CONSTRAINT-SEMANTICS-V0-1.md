# SIDERETH Authorization Constraint Semantics v0.1

**Status:** Contract proposal — implementation-gating
**Baseline:** `05e83ba561553bbaa802e92cb316fdfe5733d99d`
**Issue:** #101

## 1. Purpose

This document defines the provider-neutral semantics of authorization constraints before changing the current Rust representation from generic string key/value pairs to a typed model.

The canonical consumer-side authorization boundary remains the only generic authorization-enforcement boundary. This document does not introduce a second evaluator.

## 2. Core rule

An `Allow` decision is conditional on every applicable constraint being satisfied.

Therefore:

> **Allow + unsatisfied, missing, unknown, malformed, conflicting, or unbound restrictive constraint = deny at the consuming boundary.**

Constraints are restrictive by default. A consumer must never interpret an unknown constraint optimistically.

## 3. Current closed vocabulary

The current accepted vocabulary is deliberately small:

| Constraint | Canonical value | Semantic effect |
|---|---|---|
| `scope` | `exact_resource` | Authorization applies only to the exact bound resource in the authorization context. |
| `scope` | `exact_evidence` | Authorization applies only to the exact bound evidence object in the authorization context. |
| `access_mode` | `read_only` | Authorization permits observation/retrieval semantics only; it does not itself grant mutation or consequential execution authority. |

Unknown keys and unknown values fail closed.

## 4. Exact resource scope

`scope=exact_resource` requires the consuming operation to bind its protected resource to the authorization result's exact `resource_ref`.

An adapter may not:

- substitute another resource;
- widen a case/resource scope;
- reinterpret a parent resource as authorization for an unrelated child resource;
- treat registry membership as authorization.

Any required contextual binding must be performed before the authoritative protected operation.

## 5. Exact evidence scope

`scope=exact_evidence` requires the consuming operation to bind the requested evidence object to the exact evidence reference authorized by the result.

An adapter may not:

- substitute another evidence object;
- widen authorization from one evidence object to an evidence collection;
- infer authorization from evidence provenance, storage location, registry membership, or metadata alone.

## 6. Read-only semantics

`access_mode=read_only` is a restrictive authorization condition, not an approval to execute arbitrary actions.

It means the authorization result is compatible only with operations whose declared semantics are observational/read-only. It does **not** define read-only status by parsing arbitrary action-name strings.

Until a canonical typed action-operation classification exists, consumers must not invent compatibility rules such as string prefixes, substrings, or adapter-local action lists.

A future typed contract must provide an implementation-independent operation semantic that can be evaluated against `read_only` deterministically.

## 7. Consequential execution remains separate

Authorization does not replace the Action/Approval/Execution Gate.

A valid authorization result may establish permission for an operation, but consequential Actions remain subject to their existing approval and execution requirements.

AI/system output, tool metadata, registry membership, or capability registration cannot manufacture authorization or human approval.

## 8. Duplicate and conflicting constraints

Constraints are conjunctive by default.

- Duplicate identical constraints are semantically idempotent only when the canonical contract explicitly defines them as such.
- Conflicting values for the same constraint key fail closed.
- Unknown or malformed constraints fail closed.
- A consumer must not select the most permissive duplicate value.

For v0.1, conflicting `scope` values and conflicting `access_mode` values are invalid.

## 9. Missing constraints

A consuming boundary must enforce every restrictive constraint returned by the authorization result.

Where a protected operation requires a particular constraint as part of its declared contract, an `Allow` result that omits that required constraint is insufficient and must fail closed.

This does not mean every operation universally requires every vocabulary item. Required constraints are operation-contract properties and must be declared explicitly rather than inferred from names.

## 10. Context immutability

Constraints cannot alter the identity of the evaluated authorization context.

They cannot widen or rewrite:

- subject;
- action/operation;
- resource;
- purpose;
- jurisdiction;
- data class;
- authorization reference.

An adapter must consume the canonical result as evaluated; it cannot manufacture a broader context after authorization.

## 11. Temporal semantics

Constraint enforcement does not weaken authorization freshness or expiry.

The existing exact expiry rule remains:

`now >= expires_at` => expired => fail closed.

Future-dated evaluation results are invalid where the canonical validator already rejects them.

## 12. Determinism and provider neutrality

The same normalized request, policy inputs, authorization result, and current time must produce the same conformance outcome regardless of policy provider or adapter implementation.

Provider replacement may change how a policy decision is produced, but it must not change the meaning of the canonical constraint vocabulary.

## 13. Migration rule

The current serialized representation remains unchanged until a typed representation has an explicit compatibility and migration contract.

The migration must:

1. preserve existing serialized field names and values where wire compatibility is required;
2. reject values that cannot be mapped unambiguously to the typed vocabulary;
3. avoid silently converting unknown constraints into permissive behavior;
4. preserve fail-closed behavior during mixed-version operation;
5. include round-trip tests for every supported v0.1 constraint.

A typed Rust enum/structure is an implementation mechanism only after these semantics are accepted; it is not itself the policy contract.

## 14. Conformance matrix

| Case | Expected result |
|---|---|
| known constraint, valid binding | permit if all other checks pass |
| unknown key | fail closed |
| unknown value | fail closed |
| malformed constraint | fail closed |
| conflicting constraints | fail closed |
| exact resource with different resource | fail closed |
| exact evidence with different evidence | fail closed |
| read-only against an operation not declared read-only compatible | fail closed |
| missing operation-required constraint | fail closed |
| expired authorization | fail closed |
| future evaluation | fail closed |
| registry/tool metadata used as permission | fail closed / no authorization effect |
| consequential action without required human approval | fail at Execution Gate |
| AI/system output presented as approval | fail at approval/execution boundary |

## 15. Explicit non-goals

This contract does not authorize implementation of:

- Tool Gateway;
- MCP;
- external policy engines;
- distributed authorization services;
- production identity providers;
- autonomous agents;
- provider-specific authorization semantics;
- a replacement for Action/Approval/Execution Gate.

## 16. Implementation gate

Do not migrate `AuthorizationConstraint` to a richer executable type until this semantic contract and the corresponding conformance matrix have been reviewed against the current protected consumers.

The next implementation should be a bounded conformance-test change, followed by the smallest typed-model migration that preserves the established wire contract.
