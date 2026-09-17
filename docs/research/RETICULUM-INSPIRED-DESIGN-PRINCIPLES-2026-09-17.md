# Reticulum-Inspired Design Principles — Research Note

**Date:** 2026-09-17  
**Status:** Research / design input — not an implementation decision  
**Scope:** SIDERETH only

## 1. Purpose

This note records design principles worth studying from Reticulum and the supplied research material. It does **not** adopt Reticulum as SIDERETH's transport, protocol, license, or runtime. Any future adoption requires a separate architecture and dependency gate.

## 2. Principles worth adopting

### 2.1 Cost-of-a-byte discipline

Treat transmission as a resource with explicit cost rather than assuming abundant bandwidth.

Potential SIDERETH rules:
- Prefer compact, typed, context-referential messages where the receiver already possesses the required context.
- Do not transmit duplicated identity, policy, provenance, or metadata when a stable scoped reference is sufficient.
- Separate canonical state from derived/presentation payloads.
- Define size budgets for constrained channels and evidence envelopes.
- Measure bytes transmitted, stored, decoded, and retained where that measurement materially affects energy, latency, or storage cost.

This is an efficiency principle, not a license to remove information required for auditability, provenance, safety, or legal preservation.

### 2.2 Asynchronous-first / delay-tolerant thinking

Design workflows so that temporary disconnection does not automatically become system failure.

Potential SIDERETH rules:
- Commands/events should have explicit lifecycle state rather than assuming synchronous completion.
- Operations should distinguish `accepted`, `queued`, `in-progress`, `completed`, `failed`, `expired`, and `unknown` where relevant.
- Retry must be idempotent and bound to the authorized operation.
- Offline operation must never broaden authorization, purpose, scope, or lease lifetime.
- Stale authorization must fail closed even when transport is intermittent.
- Evidence should preserve capture time, processing time, delivery time, and uncertainty separately when relevant.

### 2.3 Disconnection as a supported operating condition

The system should tolerate loss of individual transports or providers without treating one network path as the whole system.

Potential SIDERETH rules:
- Keep domain semantics transport-neutral.
- Keep surfaces/adapters independently replaceable.
- Allow local work where policy permits it.
- Queue bounded work for later delivery rather than requiring permanent connectivity.
- Preserve provenance across transport changes.
- Never treat connectivity or provider availability as authorization.

### 2.4 Intent over payload

The supplied research emphasizes asking for the minimum information required to convey intent. SIDERETH can formalize this as **minimum sufficient representation**.

A request should carry enough information to establish:
- subject/actor,
- action,
- resource or capability,
- purpose,
- scope,
- relevant context,
- authorization reference,
- integrity/provenance references,
- lifecycle/idempotency identity.

Everything else should be justified by a concrete consumer need.

### 2.5 Local autonomy with bounded authority

Reticulum's resilience model suggests a useful SIDERETH distinction:

> Local execution can be autonomous in operation while remaining bounded in authority.

This fits the existing authorization + capability-lease architecture. A device may continue an already-authorized, bounded operation during disconnection, but it must not manufacture new authority, extend a lease, widen scope, or infer permission from possession of a capability.

### 2.6 Safety as an architectural property

The supplied Harm Principle is useful as a design prompt: powerful infrastructure should make harmful or high-impact use difficult to authorize, execute, and conceal.

For SIDERETH, the compatible architectural form is:
- canonical authorization;
- purpose and scope binding;
- capability leases;
- human approval for consequential actions where required;
- provenance and auditability;
- fail-closed behavior;
- explicit classification of uncertainty and provider failure;
- separation of evidence originals from derived outputs.

This is deliberately expressed as technical architecture rather than copying another project's license restriction.

## 3. What SIDERETH should NOT adopt automatically

### 3.1 Reticulum as the canonical transport

Not justified yet. SIDERETH should remain transport-neutral. A Reticulum adapter may become useful later for constrained/off-grid deployments, but only after a concrete use case and security/provenance analysis.

### 3.2 Reticulum's license model

Do not copy or imply compatibility with Reticulum's licensing choices. SIDERETH already has a layered licensing architecture. Third-party components remain under their own licenses and must be inventoried before distribution.

### 3.3 Public-domain protocol assumptions

Protocol openness, implementation licensing, trademark rights, data rights, and dependency licenses are separate questions. SIDERETH should preserve those distinctions.

### 3.4 Extreme compression at the expense of evidence

A smaller message is not inherently better if it destroys context needed to reproduce, verify, authorize, or audit an operation.

### 3.5 Permanent disconnected autonomy

Disconnection tolerance must not become an excuse for indefinite local authority. Offline authority must remain bounded by pre-issued, locally enforceable constraints and leases.

## 4. Proposed SIDERETH design doctrine

### Minimum Sufficient Information

Transmit, store, compute, and retain the minimum information necessary to perform the authorized operation and preserve required safety, provenance, audit, and legal guarantees.

### Delay Is A State

Latency and disconnection are explicit operating states, not exceptional assumptions. State machines and retry semantics must represent them without weakening authorization.

### Authority Is Not Connectivity

Network access, provider availability, possession of a credential, capability registration, or local execution does not itself grant permission.

### Local Is Not Unbounded

Local/offline execution is permitted only within pre-established authorization, purpose, scope, time, and lease constraints.

### Provenance Survives Transport

Changing transport, provider, device, or synchronization path must not silently sever the provenance chain.

## 5. Research directions for future evaluation

1. Evaluate Reticulum as an optional constrained/off-grid transport adapter rather than a platform foundation.
2. Evaluate LXMF/store-and-forward concepts for bounded asynchronous message delivery.
3. Evaluate RNode/microcontroller ecosystems only against concrete SIDERETH edge/evidence requirements.
4. Study Sideband/NomadNet as examples of user-facing operation over intermittent networks, without importing their product architecture.
5. Study forensics/evidence-related projects for provenance and capture-chain lessons, with independent security review.
6. Compare compact binary encodings against JSON/CBOR and existing SIDERETH contracts using measured size, CPU, energy, debuggability, and auditability rather than ideology.

## 6. Relationship to Tool Gateway

These research principles do **not** change Issue #111's current scope. The Tool Gateway must first establish its canonical contract, authorization consumption, constraint enforcement, durable idempotency, provenance, and provider boundary. A constrained transport can later be an adapter underneath that boundary.

## 7. Source notes

Primary research supplied for this note:
- Reticulum "Zen of Reticulum" material supplied in the SIDERETH project conversation.
- Reticulum project and ecosystem repositories/links supplied in the same research material.

External verification consulted during preparation:
- Official Reticulum documentation and website.
- Official Reticulum GitHub repository.

This document records design lessons and research hypotheses; it is not a claim that every supplied technical or philosophical statement has been independently verified.
