# SIDERETH — Master Decisions V1

Status: Canonical pre-implementation baseline

## Product
- **The Purple Frog** is the public product identity.
- **SIDERETH** remains the underlying Legal & Regulatory Infrastructure ecosystem and technical architecture identity unless a separate explicit platform-brand decision is made.
- North star: help people and businesses understand requirements, prepare correctly, protect lawful rights during official interactions, preserve evidence, meet deadlines, respond intelligently, and reach appropriate human legal help.
- Philosophy: navigate the rule of law; do not build a government-fighting tool.

## Architecture
- Shared, reusable infrastructure is the product foundation.
- Web, Android, iOS, Telegram, WhatsApp, future channels are independent adapters over shared capabilities.
- Domain logic must not be duplicated per surface.
- Universal Legal Infrastructure precedes domain packs.
- First engineering milestone: domain-independent Legal Case/Incident Engine.
- First domain adapters: Panchayat and Municipality; taxation follows.
- Advanced infrastructure technologies may be added as independent plug-and-play capability adapters, never as hidden core dependencies.
- The initial optional capability family is Nostr, Nym, Reticulum, Zero-Knowledge Proofs (ZKP), blockchain/distributed ledgers, Freenet, and WASM.
- Optional capabilities are user-choice features: users can enable or disable them according to need, threat model, jurisdiction, device capability and informed preference.
- Optional adapters must use the shared identity, policy, permission, data-minimisation, execution and audit infrastructure and must not create parallel authorization paths.
- WASM is treated primarily as an optional portable/sandboxed execution mechanism, not automatically as a networking, identity or trust layer.
- **Runtime decision (2026-09-17):** Rust is the canonical domain/security/runtime language; TypeScript, Python, Kotlin, Swift, SQL, WebAssembly/WIT and narrowly justified C/C++ FFI are supporting technologies selected by function. The ecosystem is intentionally polyglot at implementation boundaries while remaining single-semantic at the contract/domain boundary.
- **Hardware decision (2026-09-17):** dedicated hardware is optional. Hardware/OS/device integrations must remain adapters over canonical authorization, capability lease, evidence and provenance infrastructure and must never become hidden core dependencies.

## Trust & Safety
- Source verification, jurisdiction detection, uncertainty disclosure, human escalation and auditability are hard requirements.
- AI is optional and user-controlled; core case functionality must work without generative AI.
- Agents automate bounded workflows, not high-impact legal judgment.
- High-impact actions require explicit user approval and, where appropriate, qualified human/legal review.
- The system must not obstruct lawful official action or assume either side is correct.
- Evidence originals are immutable; derived analyses are separate artifacts.
- Optional decentralization, privacy, cryptographic or execution technologies must never be represented as implemented before code, tests, security review and deployment evidence exist.
- **Privacy and safety by design/by default (2026-09-17):** sensitive capabilities default to inactive; access is purpose-bound, least-privilege, explicitly authorized, time-bounded where appropriate, auditable and released after use. OS permission, SIDERETH authorization, capability lease and active resource remain distinct.

## Privacy & Security
- Local-first and privacy-by-default.
- Minimize collection; do not collect personal/sensitive data unless necessary.
- External processing requires minimization/redaction, encryption, authorization and audit.
- Identity, policy, permissions, tool gateway and audit are enforced centrally.
- Legal documents and external content are untrusted inputs and must not become agent instructions.
- Optional capabilities must declare data requirements, data egress, permissions, security assumptions, audit events and disable/failure behavior before activation.
- Sensitive device capabilities such as microphone, camera, location, contacts and protected storage must follow the same purpose-bound capability/lease model and must be automatically released after the bounded purpose completes.

## Legal reasoning
- Canonical flow: Facts → Issue → Jurisdiction → Authority → Rule → Procedure → Evidence → Deadline → Options → Risk → Escalation.
- Canonical procedural model: Authority → Power → Procedure → Document → Deadline → Decision → Appeal → Remedy.
- Legal propositions must expose provenance, effective dates/version, status and uncertainty.
- Cryptographic integrity, decentralized publication, network privacy or distributed anchoring does not by itself establish legal truth, legal authority, authenticity, admissibility or legal effect.

## Engineering governance
- Audit before modifying.
- Preserve useful history.
- Archive before delete.
- Never force-merge into main.
- Main becomes the integration branch only after evidence-based verification.
- Documentation must not claim implementation without code/test evidence.
- Decision Register records material architectural/product decisions and their rationale.
- Optional capability implementations must remain replaceable adapters over shared infrastructure and must not duplicate legal/domain logic.

## Brand
- **The Purple Frog** is the public product brand.
- The brand's conservation identity honors threatened species broadly.
- The canonical logo concept is a combined **Gaur / Indian Bison (*Bos gaurus*) + Purple Frog / Indian Purple Frog (*Nasikabatrachus sahyadrensis*)** emblem.
- Species identity and conservation status claims must be sourced independently; the logo itself is not a conservation-status claim.
- Trademark/legal clearance remains a separate gate; technical architecture must remain brand-independent.
- Brand assets and species-status claims are governed by `docs/brand/THE-PURPLE-FROG-BRAND-IDENTITY.md`.

## Licensing
- **Apache-2.0** remains the default license for the reusable open core and canonical infrastructure.
- **AGPL-3.0** may be applied to explicitly designated network-facing components where network copyleft is an intentional product boundary.
- **Proprietary/commercial licensing + custom EULA** applies to separately distributed commercial-only extensions/services where appropriate.
- Third-party code/data/model licenses remain applicable and must be inventoried.
- License boundaries follow component/distribution boundaries; they must not be used to create ambiguous or incompatible mixed-license modules.
- The licensing architecture is documented in `docs/legal/LICENSING-ARCHITECTURE.md` and the EULA framework in `docs/legal/COMMERCIAL-EULA-FRAMEWORK.md`.

## V1 non-goals
- Autonomous legal representation.
- Autonomous high-impact filings or government communication.
- Court-outcome prediction.
- Full litigation strategy as an MVP feature.
- Building separate infrastructure for every legal domain.
- Making any optional decentralized/privacy/execution technology mandatory for core operation.
- Making dedicated hardware mandatory for core operation.
