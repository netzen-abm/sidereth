# Decision — Platform Runtime, Privacy/Safety, Hardware and Layered Licensing

**Decision date:** 2026-09-17  
**Status:** CANONICAL DECISION  
**Decision scope:** SIDERETH / The Purple Frog ecosystem

## Decision

SIDERETH will use a **polyglot implementation architecture around one canonical domain/runtime language**.

### Runtime

- **Rust** is the canonical domain/security/runtime language.
- **TypeScript** is preferred for web/browser surfaces.
- **Python** is preferred for AI/ML/science/research workloads.
- **Kotlin** is preferred for Android adapters.
- **Swift** is preferred for Apple adapters.
- **SQL** is used for persistence/migration boundaries.
- **WebAssembly + WIT** may provide portable/sandboxed components and language-neutral interfaces.
- **C/C++** may be used only at justified vendor/hardware/FFI boundaries.
- **Mojo** remains benchmark-gated and is not a core dependency.

The ecosystem is polyglot at implementation boundaries but single-semantic at the domain/contract level.

## Privacy and safety

Privacy and safety are **by design and by default**.

Sensitive capabilities default to inactive. Access is purpose-bound, least-privilege, explicitly authorized, time-bounded where appropriate, auditable and released after use.

OS permission, SIDERETH authorization, capability lease and active resource remain distinct concepts.

## Hardware

Hardware is optional and must never be a hidden dependency of the canonical core.

Future device integrations are adapters over the same authorization, capability, lease, evidence and provenance infrastructure.

No vendor, device family or hardware architecture may become a canonical domain dependency.

## Licensing

The ecosystem will not be governed by one blanket software license.

- **Apache-2.0** remains the default license for the reusable open core and canonical infrastructure.
- **AGPL-3.0** may be used for explicitly designated network-facing components where strong network copyleft is an intentional boundary.
- **Proprietary/commercial licensing + custom EULA** applies to separately distributed commercial-only extensions/services where appropriate.
- Third-party dependencies retain their own licenses.
- The Purple Frog brand, name and logo are governed separately under trademark/copyright/brand-use rules.

## Structural consequence

Licensing follows **component boundaries**, not arbitrary files chosen after the fact.

A component must not be made AGPL or proprietary if doing so would create an unclear or legally incompatible boundary with its dependencies. License changes require dependency/provenance review.

## Rationale

This preserves:

- broad reuse of the canonical infrastructure;
- optional network-copyleft boundaries;
- a commercial path without relicensing open-source code;
- freedom to use different implementation languages for different engineering functions;
- future hardware support without hardware lock-in;
- privacy and safety as system invariants rather than UI features.

## Required follow-up

1. Maintain the platform/runtime strategy in `docs/architecture/PLATFORM-RUNTIME-AND-HARDWARE-STRATEGY.md`.
2. Maintain privacy/safety requirements in `docs/architecture/PRIVACY-AND-SAFETY-BY-DESIGN.md`.
3. Maintain licensing boundaries in `docs/legal/LICENSING-ARCHITECTURE.md`.
4. Finalize any commercial EULA only after legal review.
5. Add AGPL license text and component-specific SPDX markers when the first AGPL component is actually introduced.
6. Add license inventory/SBOM checks to CI before the first mixed-license production distribution.
