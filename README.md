# The Purple Frog

**Privacy-first Legal & Regulatory Infrastructure**

**The Purple Frog** is the public product identity for a privacy-first, modular legal and regulatory infrastructure platform. The underlying ecosystem and technical architecture remain **SIDERETH** unless a separate explicit platform-brand decision is made.

> **The Purple Frog — one product, shared infrastructure, many capabilities.**

The canonical visual identity combines the **Gaur / Indian Bison (*Bos gaurus*)** and the **Purple Frog / Indian Purple Frog (*Nasikabatrachus sahyadrensis*)**. The emblem is a brand symbol; species taxonomy and conservation-status claims remain independently sourced.

## Conservation mission

> **Awareness of a Vulnerable species. Protection of threatened species. Respect for every species.**

Conservation-status claims must remain sourced, scoped and current. The brand must not turn scientific status into decorative marketing language.

## What it does

- Prepare applications and compliance workflows
- Check jurisdiction, authority, procedure and documentary readiness
- Protect and record lawful government interactions
- Preserve evidence and build verifiable timelines
- Understand notices, orders and decisions
- Track deadlines and required actions
- Draft responses for user review
- Escalate unresolved matters
- Route high-stakes matters to qualified human professionals

## Architecture principle

The platform is built around shared, domain-independent capabilities. Web, mobile, messaging, AI and future hardware surfaces are independent adapters over shared contracts and engines. Domain logic must not be duplicated per surface.

## Privacy and safety by design/by default

Sensitive capabilities are inactive by default. Microphone, camera, location, contacts, protected storage, external data egress and other sensitive resources require a bounded authorization path and purpose-bound capability lease where applicable. After the bounded purpose completes, active use is released; later use requires fresh activation under policy.

Local-first processing, data minimisation, explicit purpose, least privilege, encryption, provenance and auditability are architectural requirements rather than optional add-ons.

## AI, agent and MCP boundary

AI is optional and user-controlled. AI/agent systems receive only the minimum authorized, policy-filtered data required for a task. Personal or sensitive case information must never be exposed directly to an AI or agent merely because it is available in the system. Redaction, data minimisation, purpose limitation, authorization, audit and provider/model policy are mandatory boundaries.

Agents automate bounded workflows, not high-impact legal judgment. MCP is an interoperability boundary for approved tools; it does not replace SIDERETH policy, authorization, case, evidence or audit infrastructure.

## Runtime and language strategy

Rust is the canonical domain/security/runtime language. TypeScript, Python, Kotlin, Swift and SQL are selected by function; WebAssembly/WIT provides an optional portable interoperability boundary; C/C++ is limited to justified vendor/hardware FFI; Mojo remains benchmark-gated rather than a core dependency.

This makes SIDERETH intentionally **polyglot without becoming poly-semantic**: the domain contracts are language-neutral and implementations are replaceable adapters.

## Hardware independence

Dedicated hardware is optional. Future cameras, microphones, GNSS, wearables, sensors, evidence-capture devices and embedded systems must integrate through adapters over the same authorization, capability lease, evidence and provenance infrastructure. No device vendor or hardware family is a canonical dependency.

## Licensing

SIDERETH uses a layered licensing strategy rather than one blanket ecosystem license:

- **Apache-2.0** — default for reusable open core/infrastructure;
- **AGPL-3.0** — explicitly designated network-facing components where strong network copyleft is intentional;
- **Proprietary/commercial + custom EULA** — separately distributed commercial-only extensions/services;
- third-party code/data/models — their applicable licenses/terms;
- **The Purple Frog** name and logo — separate trademark/copyright/brand-use controls.

See `docs/legal/LICENSING-ARCHITECTURE.md`.

## Documentation entry point

**Before changing architecture, contracts, implementation boundaries or documentation, read `docs/00-DOCUMENTATION-INDEX.md` and `docs/SIDERETH-DOCUMENTATION-GOVERNANCE.md`.**

The documentation system uses one canonical authority per concept, separates decisions from contracts and plans, and requires evidence before implementation-status claims.

## Current foundation documents

- `docs/00-DOCUMENTATION-INDEX.md`
- `docs/00-SIDERETH-MASTER-DECISIONS.md`
- `docs/01-SIDERETH-MASTER-BLUEPRINT.md`
- `docs/SIDERETH-DOCUMENTATION-GOVERNANCE.md`
- `docs/SIDERETH-ARCHITECTURE.md`
- `docs/SIDERETH-ECOSYSTEM-ARCHITECTURE.md`
- `docs/architecture/PLATFORM-RUNTIME-AND-HARDWARE-STRATEGY.md`
- `docs/architecture/PRIVACY-AND-SAFETY-BY-DESIGN.md`
- `docs/legal/LICENSING-ARCHITECTURE.md`
- `docs/brand/THE-PURPLE-FROG-BRAND-IDENTITY.md`
- `docs/contracts/`
- `docs/migration/`
- `docs/archive/`

## Development rule

Do not infer implementation from architecture documents. A capability becomes complete only when its contract, implementation, tests, security controls, documentation and observability are present and verified.

## Current implementation priority

The current bounded platform priority is governed by the Tool Gateway readiness gate. The gateway must consume the canonical authorization/policy boundary and is not production-ready until its required conformance evidence is complete. Current implementation status must be verified from source, tests, CI and the active issue/PR rather than from historical PR numbers in README text.
