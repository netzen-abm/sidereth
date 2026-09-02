# SIDERETH Core v0.1 — Domain Design

**Status:** CANONICAL DEVELOPMENT DESIGN
**Scope:** Universal Case, Incident, Event and Timeline foundation

## 1. Design principle

The core is deterministic legal/regulatory workflow infrastructure. It records and governs domain state; it does not independently decide legal outcomes.

The core must remain functional with AI, networking, decentralized protocols, external model providers and future capability adapters disabled.

## 2. Aggregate boundaries

### Case

A Case is the durable container for a legal/regulatory matter. It owns lifecycle state and references to related facts, events, documents, evidence, legal issues, deadlines, actions, decisions, appeals, escalations and assistance.

### Incident

An Incident records a real-world official interaction. It may be created independently and linked to a Case later. This is important for urgent/offline capture and preserves the distinction between an event that happened and a matter that is subsequently opened.

### Event

An Event is an immutable record of something that occurred or a domain state transition that must be auditable. Events identify the aggregate, actor, schema version, time, source references, correlation and causation.

### Timeline

A Timeline is a deterministic projection of events. It is derived from event records and must not become a second source of truth.

## 3. Identity and provenance

Every persisted object requires a stable identifier and schema version. System-generated legal propositions must carry source references. User-provided facts and system inferences must remain distinguishable.

Event provenance must answer:

- what happened;
- when it happened;
- to which aggregate;
- who or what recorded it;
- what caused it;
- which sources support it, where applicable;
- which schema version interpreted it.

## 4. Lifecycle

Case lifecycle currently supports:

`Draft → Active → WaitingUser / WaitingAuthority / ResponseDue / Escalated / Resolved → Closed`

Incident lifecycle currently supports:

`Prepared → Active ↔ Paused → Concluded → EvidenceReview → LinkedToCase`

Transitions are explicit and invalid transitions are rejected by the domain layer.

## 5. Mutation boundary

The domain layer owns invariants and state transitions. Service/application layers own authorization, policy, approval, persistence and audit integration. Adapters must not bypass those boundaries.

High-impact actions require an explicit approval record before execution.

## 6. Evidence boundary

Evidence originals are immutable. Derived artifacts—transcripts, OCR, summaries, classifications or redactions—are separate versioned artifacts linked to the original. Cryptographic hashes are integrity metadata for the preserved original, not a replacement for it.

## 7. Privacy boundary

The core domain model should carry references rather than unnecessary sensitive content. Sensitive storage, encryption, retention and access policy are shared infrastructure concerns governed by the storage and authorization contracts.

## 8. Future capability boundary

AI, MCP, Nostr, Nym, Reticulum, ZKP, blockchain, Freenet, WASM and other future technologies are adapters/capabilities. None is required for Case/Incident/Event semantics.

## 9. Implementation evidence for v0.1 increment

The current increment contains executable Rust primitives for:

- Case lifecycle transition validation;
- Incident lifecycle transition validation;
- Event envelope validation;
- deterministic Timeline ordering and duplicate-event rejection;
- EvidenceOriginal construction with SHA-256 content integrity metadata;
- DerivedArtifact source linkage validation.

Persistence, API exposure, authorization enforcement, durable audit storage and production evidence handling remain implementation work outside this increment.

## 10. Definition of done for this increment

The Case/Incident/Event/Timeline foundation is complete only when executable tests prove the documented invariants, including negative cases, and the documentation accurately describes what is implemented versus planned.
