# SIDERETH — Privacy & Safety by Design and by Default

**Status:** CANONICAL CROSS-CUTTING ARCHITECTURE PRINCIPLE  
**Decision date:** 2026-09-17

## 1. Core rule

> **Privacy and safety are architectural properties, not optional features added after implementation.**

SIDERETH must be privacy-preserving and safety-conscious **by design and by default**.

A capability that can expose sensitive data, activate a sensitive resource, create a consequential action, or materially affect a person must require an explicit bounded path before use.

## 2. Default-deny capability activation

The default state for sensitive capabilities is:

```text
NOT REQUESTED
→ NOT AUTHORIZED
→ NOT ACTIVE
```

Examples include:

- microphone;
- camera;
- location/GNSS;
- address book/contacts;
- files or protected storage;
- biometric or sensitive sensors;
- external data transmission;
- privileged device APIs.

Availability of an OS permission, API handle, provider credential or registry entry does not itself grant SIDERETH authorization.

## 3. Just-in-time permission

Sensitive capability activation must be:

1. purpose-bound;
2. scope-bound;
3. subject/actor-bound;
4. context-bound;
5. time-bounded;
6. auditable;
7. revocable where technically possible;
8. automatically released when the bounded operation ends.

If the capability is needed again later, it is reactivated through the applicable authorization and lease path.

## 4. Data minimisation

Collect only the minimum data required for the stated purpose.

Prefer:

- local processing;
- selective disclosure;
- redaction;
- pseudonymous references;
- derived summaries where originals are not required;
- scoped queries;
- short retention periods;
- user-controlled export/delete mechanisms where legally and technically applicable.

Do not collect sensitive data merely because a device or provider makes it available.

## 5. Local-first architecture

Where functionality can be safely completed locally, local processing should be preferred over unnecessary external processing.

External processing requires explicit policy evaluation covering:

- data class;
- purpose;
- destination/provider;
- jurisdiction;
- retention;
- security controls;
- user authorization/consent where required;
- auditability;
- failure behavior.

## 6. AI and agent safety boundary

AI and agents are untrusted consumers of protected data and capabilities.

They must not:

- bypass canonical authorization;
- obtain data merely because a user session can access it;
- invent provenance;
- silently broaden purpose or scope;
- activate hardware outside the capability-lease path;
- turn inference into authoritative fact;
- perform high-impact actions without the required approval boundary.

AI-disabled operation must remain possible for core deterministic workflows.

## 7. Evidence safety

Original evidence is preserved separately from derived artifacts.

OCR, transcription, summarisation, classification, matching, extraction and other AI/algorithmic outputs must never overwrite or replace the original evidence artifact.

Every derived artifact must preserve provenance and relationship to its source.

## 8. Safety-by-default execution

Consequential actions must fail closed when required authorization, context, constraints, human approval, capability lease or required evidence is missing.

The system must distinguish:

- authorization denial;
- expired capability;
- policy/constraint failure;
- human-approval failure;
- provider/device failure;
- uncertain state;
- completed operation.

A provider failure must never be misrepresented as a successful action.

## 9. User control

Users should be able to understand, where appropriate:

- what sensitive capability is being requested;
- why it is needed;
- what data will be accessed;
- whether external processing occurs;
- when access begins;
- when access ends;
- what was recorded/audited;
- how access can be revoked or stopped.

High-risk capabilities should provide visible state where technically and contextually appropriate.

## 10. Retention and lifecycle

Sensitive data must have an explicit lifecycle:

```text
collect
  ↓
use for stated purpose
  ↓
release / minimise
  ↓
retain only when justified
  ↓
expire / delete / anonymise according to policy
```

Legal preservation requirements, evidence preservation and regulatory retention obligations may override ordinary deletion schedules; such exceptions must be explicit and auditable.

## 11. Security engineering

Privacy controls do not replace security controls.

SIDERETH must apply, as appropriate:

- least privilege;
- secure defaults;
- encryption in transit and at rest;
- secret isolation;
- input validation;
- provenance;
- tamper evidence;
- dependency/supply-chain controls;
- audit trails;
- idempotency;
- bounded timeouts;
- safe failure;
- recovery testing.

## 12. Hardware safety

Hardware adapters must inherit the same privacy and safety rules.

A camera, microphone, location source, contact database or sensor must never remain active simply because the operating system permits it.

SIDERETH controls its own use through purpose-bound authorization and capability leases. If an operating system cannot revoke a permission programmatically, SIDERETH must stop using/release its active handle and must not falsely claim that the OS permission itself was revoked.

## 13. Threat-model rule

Every new sensitive capability requires a bounded threat model covering:

- assets;
- actors;
- trust boundaries;
- abuse cases;
- data flows;
- permissions;
- external providers;
- failure modes;
- recovery;
- audit requirements;
- residual risk.

## 14. Verification status

A privacy/safety requirement is not considered implemented merely because this document exists.

Required evidence may include:

- code;
- automated tests;
- conformance tests;
- security review;
- privacy review;
- threat model;
- device/provider tests;
- operational evidence.
