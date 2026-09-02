# SIDERETH — Master Decisions V1

Status: Canonical pre-implementation baseline

## Product
- SIDERETH is a standalone Legal & Regulatory Infrastructure product.
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

## Trust & Safety
- Source verification, jurisdiction detection, uncertainty disclosure, human escalation and auditability are hard requirements.
- AI is optional and user-controlled; core case functionality must work without generative AI.
- Agents automate bounded workflows, not high-impact legal judgment.
- High-impact actions require explicit user approval and, where appropriate, qualified human/legal review.
- The system must not obstruct lawful official action or assume either side is correct.
- Evidence originals are immutable; derived analyses are separate artifacts.
- Optional decentralization, privacy, cryptographic or execution technologies must never be represented as implemented before code, tests, security review and deployment evidence exist.

## Privacy & Security
- Local-first and privacy-by-default.
- Minimize collection; do not collect personal/sensitive data unless necessary.
- External processing requires minimization/redaction, encryption, authorization and audit.
- Identity, policy, permissions, tool gateway and audit are enforced centrally.
- Legal documents and external content are untrusted inputs and must not become agent instructions.
- Optional capabilities must declare data requirements, data egress, permissions, security assumptions, audit events and disable/failure behavior before activation.

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
- SIDERETH is the current working master brand.
- Trademark/legal clearance remains a separate gate; technical architecture must remain brand-independent.

## V1 non-goals
- Autonomous legal representation.
- Autonomous high-impact filings or government communication.
- Court-outcome prediction.
- Full litigation strategy as an MVP feature.
- Building separate infrastructure for every legal domain.
- Making any optional decentralized/privacy/execution technology mandatory for core SIDERETH operation.
