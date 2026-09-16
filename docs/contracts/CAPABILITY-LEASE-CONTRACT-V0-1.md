# SIDERETH Purpose-Bound Capability Lease Contract v0.1

**Status:** Contract proposal — implementation-gating  
**Baseline:** `67de39d14b67de251942c98bd4af87bd0b0536f4`  
**Related design:** Issue #104  
**Scope:** Shared SIDERETH infrastructure

## 1. Purpose

This contract defines the canonical semantics for temporary, purpose-bound activation of sensitive or otherwise explicitly leased capabilities and their underlying resources.

The capability lease is a control-plane primitive between canonical authorization/policy and a capability adapter or operating-system resource. It does not replace authorization, operating-system permissions, capability registration, execution approval, evidence provenance, or audit.

The canonical path is:

> **Canonical Authorization/Policy → Capability Lease → Capability Adapter / OS Resource → Protected Operation → Release / Expiry → Audit**

The contract is provider-neutral and surface-neutral. A web client, mobile client, desktop process, agent, hardware adapter, or future surface may implement an adapter, but none may redefine lease semantics.

## 2. Security model

SIDERETH distinguishes four states that must never be treated as equivalent:

| Layer | Meaning |
|---|---|
| OS / platform permission | The platform permits an application to request or use a resource under its own permission model. |
| SIDERETH authorization | The canonical policy boundary permits a specific subject to perform a specific operation in a specific context. |
| Capability lease | SIDERETH permits bounded activation/use of a named capability for a declared purpose, scope, subject, and session/context for a limited lifetime. |
| Active resource | The underlying resource is currently activated and being used by an adapter. |

**OS permission does not grant SIDERETH authorization.**  
**SIDERETH authorization does not by itself imply an active resource.**  
**A lease does not bypass the OS permission model.**  
**An active resource without a valid lease is an invalid protected state.**

## 3. Core invariant

> **No leased capability may be activated or used unless a valid, unrevoked, unexpired lease exactly authorizes the requested capability, purpose, scope, subject, and session/context.**

If any required lease condition is missing, malformed, expired, revoked, cancelled, purpose-mismatched, scope-broadened, or context-mismatched, the consuming boundary must fail closed.

## 4. Lease identity and canonical fields

A v0.1 lease is conceptually identified by:

- `lease_id` — globally unique lease identifier;
- `schema_version` — contract version;
- `capability_ref` — canonical capability being leased;
- `resource_ref` — optional exact underlying resource binding where applicable;
- `subject_ref` — subject for whom the capability is authorized;
- `actor_ref` — actor/process/surface requesting or exercising the lease, where distinct from the subject;
- `purpose` — explicit human/system-readable purpose for activation and use;
- `purpose_version` — optional version for materially governed purpose definitions;
- `authorization_ref` — canonical authorization reference that permits the leased operation;
- `incident_ref` / `session_ref` — optional bounded workflow/session binding;
- `scope` — exact data/resource/capability scope;
- `issued_at` — lease issuance time;
- `expires_at` — mandatory lease expiry;
- `state` — canonical lease lifecycle state;
- `revoked_at` — optional revocation time;
- `cancelled_at` — optional cancellation time;
- `activated_at` — optional activation time;
- `released_at` — optional release time;
- `adapter_ref` — adapter/provider responsible for resource activation, where applicable;
- `audit_ref` — audit/provenance reference for lifecycle transitions.

Implementations may carry additional metadata, but additional fields must not weaken the mandatory semantics above.

## 5. Lease lifecycle

The canonical lifecycle is:

`REQUESTED → AUTHORIZED → ACTIVE → COMPLETED → RELEASED`

`AUTHORIZED/ACTIVE → REVOKED`  
`ACTIVE → CANCELLED`  
`AUTHORIZED → EXPIRED`  
`ACTIVE → EXPIRED`

### State semantics

- **REQUESTED:** a lease has been requested but is not yet usable.
- **AUTHORIZED:** the lease has passed the required authorization boundary and may be activated if all adapter/OS preconditions are satisfied.
- **ACTIVE:** the underlying capability/resource may be in use within the exact lease bounds.
- **COMPLETED:** the declared purpose/session operation has completed and resource use must cease.
- **RELEASED:** the capability has been deactivated/released as far as the platform permits.
- **REVOKED:** authorization or lease validity has been withdrawn; use must cease.
- **CANCELLED:** the workflow/user/system explicitly cancelled the lease; use must cease.
- **EXPIRED:** the lease lifetime ended; use must cease.

A terminal state must not silently transition back to `ACTIVE`. A future use requires a new bounded authorization/lease decision.

## 6. Purpose binding

Purpose is a security boundary, not descriptive metadata.

A lease may be used only for the exact declared purpose and any explicitly defined narrower operation permitted by the contract for that purpose.

A consumer must reject:

- a different purpose;
- an absent purpose where purpose is required;
- a purpose version mismatch where versioning is required;
- a request that materially broadens the declared purpose;
- reuse of an old lease for a materially different task.

A client, adapter, agent, or provider must not rewrite the purpose after authorization.

## 7. Scope binding

Lease scope is restrictive.

A request may not broaden:

- the protected resource set;
- the data class;
- the capability's operation set;
- the incident/session scope;
- the geographic or temporal scope, where applicable.

Exact-resource and exact-evidence semantics must remain compatible with the canonical authorization constraint contract. Registry membership, provider metadata, OS permission, or possession of a capability handle cannot widen lease scope.

## 8. Authorization dependency

Every lease that enables a protected capability must reference the canonical authorization decision that permits it.

The lease is not an alternative authorization evaluator.

The authorization boundary remains responsible for deciding whether the subject/action/resource/purpose/context is permitted. The lease binds that permission to a bounded capability activation lifecycle.

If the underlying authorization becomes invalid or is revoked, dependent lease use must fail closed or terminate according to the revocation semantics available to the adapter.

A lease must never manufacture, upgrade, or broaden authorization.

## 9. Activation semantics

Activation means obtaining or enabling access to the underlying resource through an authorized adapter or platform mechanism.

Before activation, the consumer must verify at minimum:

1. lease identity and schema are valid;
2. lease state permits activation;
3. authorization reference is valid and contextually bound;
4. capability and resource match the request;
5. subject/actor binding is valid;
6. purpose and purpose version match;
7. scope is not broader than authorized;
8. the lease is not expired, revoked, or cancelled;
9. the workflow/session binding matches where required;
10. required OS/platform permission is present or can be obtained through an explicit platform flow.

Activation failure must not be represented as successful resource use.

## 10. Automatic deactivation and release

When the declared purpose is completed, cancelled, or otherwise ends, the capability must automatically transition out of active use and the adapter must release/deactivate the underlying resource where technically possible.

At minimum:

`purpose complete → stop use → release/deactivate → record lifecycle transition`

If an operating system does not expose a programmatic permission-revocation primitive, SIDERETH must still:

- stop using the resource;
- release/close the active handle or stream where possible;
- mark the lease completed/released;
- prevent reuse under the old lease;
- require a fresh SIDERETH authorization and a new lease for subsequent use.

SIDERETH must not claim that the OS permission itself was revoked when it was only the SIDERETH lease that ended.

## 11. Expiry semantics

Expiry is mandatory and fail-closed.

The canonical rule is:

`now >= expires_at` → lease expired → resource use denied.

Expiry must be evaluated at the consuming boundary, not only when the lease is created.

A long-lived or effectively perpetual lease is incompatible with this v0.1 contract for sensitive capabilities.

Clock uncertainty must be handled conservatively. Implementations must not extend a lease merely because a local clock appears favorable.

## 12. Revocation and cancellation

Revocation invalidates further use immediately where the system can enforce it.

Cancellation represents an explicit end of the requesting workflow or user/system intent.

For active resources:

- stop new protected operations;
- terminate/release the resource where technically possible;
- record the transition;
- prevent silent restart under the old lease.

If immediate physical termination is impossible, the system must fail closed at the next protected operation boundary and must not represent the capability as fully released until the adapter confirms release or records the platform limitation.

## 13. Emergency continuation

An emergency workflow may require bounded continuation after an ordinary completion boundary only when interruption would materially increase the relevant safety risk.

Such continuation is not an implicit exception to the lease model. It requires an explicit policy and bounded lease semantics including:

- declared emergency purpose;
- exact capability/scope;
- short explicit expiry;
- workflow/incident binding;
- visibility to the user where technically possible;
- audit of activation, continuation, and termination;
- automatic termination at expiry or emergency-policy completion.

Emergency continuation must not become a hidden persistent lease.

## 14. User consent and purpose disclosure

Where user authorization is required, the activation flow should expose the capability and purpose in a form appropriate to the surface and risk.

Consent/approval is not itself proof of canonical authorization. Likewise, canonical authorization does not erase a required user-consent or platform-permission requirement.

For sensitive resources, the UI should make clear:

- what capability is being activated;
- why it is needed;
- what scope is involved;
- how long it will remain active;
- what event ends it;
- what happens if the user cancels.

## 15. Offline and interrupted operation

Lease semantics must remain safe during network loss, process interruption, device restart, or adapter failure.

An implementation may support bounded offline operation only when the lease was already validly issued and its expiry/scope can be enforced locally.

Offline operation must not:

- create a new authorization implicitly;
- extend expiry;
- broaden purpose/scope;
- convert an interrupted lease into a perpetual lease;
- erase lifecycle history.

After interruption, restoration must conservatively revalidate lease state before protected use resumes.

## 16. Capability handles and adapter isolation

A capability handle is an implementation artifact, not authorization.

Possession of a handle must never be sufficient to activate a capability outside the current valid lease.

Adapters must be replaceable without changing canonical lease semantics.

Provider-specific behavior belongs below the lease boundary:

`canonical lease → adapter contract → provider/OS resource`

An adapter must not introduce a second purpose, expiry, or permission vocabulary that changes the meaning of the canonical contract.

## 17. Audit and provenance

Every material lifecycle transition must be auditable, including as applicable:

- request;
- authorization binding;
- issuance;
- activation;
- protected use;
- completion;
- release/deactivation;
- expiry;
- revocation;
- cancellation;
- failed activation;
- platform limitation preventing full release.

Audit records should bind the transition to lease identity, authorization reference, capability/resource, purpose, scope, actor/subject, relevant session/incident, time, adapter, and outcome.

Audit/provenance records must not contain more sensitive payload than necessary to establish lifecycle accountability.

## 18. Relationship to evidence

A capability lease governs access to a capability; it does not make resulting data evidence by itself.

When a leased capability produces evidence, the evidence subsystem must apply the canonical evidence/provenance contract independently.

For safety or investigative workflows, the expected conceptual path is:

`Authorization/Policy → Capability Lease → Adapter/OS Resource → Protected Operation → Original Evidence → Provenance/Audit`

Derived OCR, transcription, classification, biometric matching, summaries, or other analysis remain derived artifacts and cannot replace original evidence.

## 19. Failure-closed matrix

| Condition | Required outcome |
|---|---|
| No valid lease | deny activation/use |
| OS permission present but no SIDERETH lease | deny activation/use |
| Lease expired (`now >= expires_at`) | deny use; terminate/release where possible |
| Lease revoked | deny use; terminate/release where possible |
| Lease cancelled | deny use; terminate/release where possible |
| Purpose mismatch | deny |
| Purpose version mismatch where required | deny |
| Scope widening | deny |
| Subject/actor mismatch | deny |
| Session/incident mismatch where required | deny |
| Invalid authorization reference/context | deny |
| Capability/resource mismatch | deny |
| Unknown lease state | deny |
| Adapter claims activation without valid lease | reject at boundary |
| Old released lease reused | deny; require fresh authorization/lease |
| Network loss with still-valid bounded offline lease | permit only within locally enforceable bounds |
| Network loss requiring new authorization | deny until authorization/lease can be established |
| Emergency continuation without explicit bounded policy | deny |

## 20. Conformance requirements

A conforming implementation must test at least:

1. valid activation with exact purpose/scope;
2. OS permission present but SIDERETH lease absent;
3. exact expiry at `now == expires_at`;
4. expired lease after process restart;
5. revocation while authorized;
6. cancellation while active;
7. purpose mismatch;
8. purpose-version mismatch where applicable;
9. scope widening;
10. subject/actor mismatch;
11. session/incident mismatch;
12. capability/resource mismatch;
13. reuse after release;
14. automatic release after normal completion;
15. adapter failure during release;
16. offline use within a pre-issued bounded lease;
17. interrupted operation and conservative revalidation;
18. emergency continuation with explicit bounded expiry;
19. emergency continuation without policy;
20. duplicate/conflicting lease context;
21. audit completeness for lifecycle transitions;
22. provider/adapter replacement without semantic change.

## 21. Non-goals

This contract does not authorize implementation of:

- a new authorization evaluator;
- a general consent-management replacement;
- automatic OS permission revocation where the platform does not support it;
- persistent background access to sensitive resources;
- unrestricted autonomous agent access;
- biometric identification;
- emergency-service integration;
- police/FIR transmission;
- Tool Gateway;
- MCP;
- provider-specific permission semantics.

Those may consume this contract later through bounded adapters and separate contracts.

## 22. Implementation gate

Before implementing platform adapters or sensitive-resource integrations, SIDERETH must establish:

1. canonical lease data model and lifecycle state machine;
2. consuming-boundary validation semantics;
3. exact expiry/revocation/cancellation behavior;
4. purpose/scope/session binding;
5. automatic release semantics;
6. conformance tests;
7. audit/provenance requirements;
8. compatibility with the canonical authorization and constraint contracts.

The first implementation should remain provider-neutral and test the lifecycle at the SIDERETH boundary before adding mobile, browser, desktop, hardware, or external-provider adapters.
