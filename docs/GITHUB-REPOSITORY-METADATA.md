# GitHub Repository Metadata

**Status:** CANONICAL REPOSITORY-METADATA REFERENCE

This document records the recommended public GitHub repository positioning for SIDERETH. It is intentionally kept separate from the repository's legal/product architecture so GitHub metadata can change without changing the product contract.

## Recommended repository description

> Privacy-first legal & regulatory infrastructure for cases, incidents, evidence, deadlines, compliance, rights protection, and human legal assistance — with optional AI and plug-and-play decentralized/privacy capabilities under user control.

**Character count:** 263 characters.

## Recommended GitHub topics

Use lowercase GitHub topics; avoid trademark claims and avoid implying technologies are implemented when they are only planned.

### Primary topics

- `legal-tech`
- `legal-infrastructure`
- `regtech`
- `compliance`
- `legal-workflow`
- `case-management`
- `evidence-management`
- `privacy`
- `security`
- `open-source`

### Ecosystem / future-capability topics

Use these only if the repository's public positioning is intended to signal future interoperability rather than current implementation:

- `decentralized`
- `privacy-preserving`
- `zero-knowledge-proofs`
- `wasm`
- `nostr`
- `reticulum`
- `freenet`
- `blockchain`

**Recommendation:** For the current foundation, keep the primary topic set and add future-capability topics only after corresponding adapter specifications or implementations are sufficiently mature. GitHub topics should not imply production support.

## Metadata governance

- The repository description must not identify SIDERETH as Janavani or as a decentralized-system prototype.
- The description must not claim an AI lawyer, autonomous legal representation, autonomous government communication, or production decentralized integrations.
- Topics must describe the repository accurately at its current maturity level.
- When a capability moves from PLANNING/CANDIDATE to implemented, topics may be expanded as appropriate.
- Product identity and legal/trademark clearance remain separate from GitHub metadata.

## Current architecture alignment

SIDERETH is a standalone Legal & Regulatory Infrastructure product. Shared infrastructure is the foundation; user-facing surfaces are independent adapters. AI is optional and user-controlled. Future Nostr, Nym, Reticulum, ZKP, blockchain, Freenet and WASM capabilities are plug-and-play options, not mandatory core dependencies.

See `docs/contracts/OPTIONAL-CAPABILITY-CONTRACT.md` for the governing capability policy.
