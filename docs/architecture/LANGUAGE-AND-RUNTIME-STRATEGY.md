# SIDERETH Language and Runtime Strategy

**Status:** Accepted architectural direction

## Decision

SIDERETH is not defined by a programming language. It is defined by its canonical domain model, contracts, invariants, security policies, and observable behavior.

**Canonical Domain Runtime: Rust.**

Rust is the canonical implementation runtime for the deterministic domain core and shared infrastructure where strong correctness, integrity, and security guarantees are required.

**Application & Integration Layer: Polyglot.**

Application, integration, automation, AI, interface, and specialized service layers may use other languages and runtimes where a justified technical advantage exists.

## Language neutrality principle

> SIDERETH has one canonical domain and contract system. Rust is the canonical runtime for the deterministic domain core. Other languages may implement application and integration capabilities where justified. Every implementation must conform to the canonical SIDERETH contracts and must not create a competing interpretation of the domain.

## Responsibility boundaries

| Layer | Responsibility | Runtime guidance |
|---|---|---|
| Canonical Domain Runtime | Domain entities, invariants, deterministic rules, core services | Rust |
| Canonical Contract Layer | Domain, wire, policy, authorization, security and interoperability contracts | Language-neutral contracts; Rust reference implementation |
| Application / Integration | APIs, orchestration, adapters, automation, external integrations | Polyglot: TypeScript, Python, Django, etc. where justified |
| AI / Intelligence | RAG, OCR/NLP, research tooling, model orchestration, agent services | Primarily Python or other suitable runtimes; never domain authority |
| Web | User-facing web application, progressive enhancement, browser execution | Hybrid architecture; runtime selected per capability |
| WASM | Browser/edge execution target for suitable deterministic capabilities | Rust-to-WASM where justified; WASM is a target, not a competing domain |
| Surfaces | Web, mobile, messaging, desktop and other channels | Independent adapters over shared contracts |

## Web platform: Hybrid by design

The SIDERETH Web Platform should be a **hybrid web architecture**, not a single-runtime commitment.

A practical target architecture is:

```text
                         SIDERETH WEB PLATFORM
                                  |
             +--------------------+--------------------+
             |                                         |
       Web UI / UX                              Browser capabilities
             |                                         |
      +------+-------+                         +-------+-------+
      |              |                         |               |
 Server-rendered   Interactive              WASM          Web APIs
 / application     client UI             capabilities   (where useful)
      |              |                         |
      +------+-------+-------------------------+
             |
       Contract/API boundary
             |
      Application / Integration layer
             |
       Canonical SIDERETH contracts
             |
       Rust canonical domain core
```

The hybrid model allows the web platform to combine:

- server-side rendering and conventional web delivery where it improves accessibility, SEO, first-load performance, and operational simplicity;
- client-side interactivity where rich workflows require it;
- Rust/WASM for deterministic, computation-heavy, security-sensitive, or offline-capable browser capabilities where justified;
- polyglot backend services for AI, ingestion, integrations, notifications, search, and other specialized capabilities;
- shared contract-driven APIs so the browser never becomes a second domain implementation.

## Non-negotiable constraints

1. The web application MUST NOT become the canonical source of domain semantics.
2. Client-side code MUST NOT independently redefine domain rules that belong to the canonical domain runtime.
3. WASM MUST be treated as an execution/deployment target, not as a competing domain architecture.
4. Python, Django, TypeScript, JavaScript, and other runtimes are allowed where justified, but domain forks are forbidden.
5. All application implementations MUST conform to canonical SIDERETH wire and policy contracts.
6. Security, authorization, provenance, privacy, auditability, and lifecycle invariants MUST remain enforced at trusted boundaries, not merely in UI code.
7. Web, mobile, messaging, and other surfaces MUST remain independently deployable and failure-isolated where practical.
8. A capability should be implemented once at the appropriate shared infrastructure layer and consumed by multiple surfaces rather than reimplemented per surface.

## Recommended web composition

For the initial platform, prefer a **progressive hybrid** rather than committing to a framework ideology:

- server-rendered shell and accessible navigation;
- interactive application islands/components for complex workflows;
- API/contract boundary between UI and application services;
- Rust/WASM only for capabilities that demonstrate a measurable benefit;
- specialized polyglot services behind stable contracts;
- local/offline capability where privacy, resilience, or user control materially benefits from it.

This keeps the architecture open to Dioxus, React/Next.js, vanilla web components, or other technologies without changing the SIDERETH domain architecture.

## Forbidden architecture patterns

- a TypeScript-only or Python-only reimplementation of the canonical domain;
- separate domain semantics for web, mobile, Telegram, or other surfaces;
- browser code being treated as an authority for authorization or legal decisions;
- framework choice becoming a permanent architectural dependency of the domain core;
- adding WASM merely because it is available, without a concrete capability-level justification.

## Architectural rule

**One canonical domain. One canonical contract system. Multiple implementation runtimes where justified. Independent surfaces. Shared capabilities.**
