# SIDERETH — Semantic Documentation Audit: Domain & Storage

**Audit date:** 2026-09-11  
**Baseline:** `main` at `10c3804be25b6cf1d0252f21f35d280618daabd5`  
**Scope:** Domain-model and storage documentation family  
**Disposition:** Semantic audit; no physical moves or deletions

## Executive finding

The audit found **real semantic overlap** between the older framework-level documents and the newer contract-level documents, but not enough evidence to justify deleting either family immediately.

The correct distinction is:

- `docs/04-SIDERETH-DATA-MODEL.md` is a concise **canonical framework vocabulary**.
- `docs/contracts/CANONICAL-DOMAIN-MODEL.md` is the more detailed **implementation-facing domain contract**.
- `docs/05-SIDERETH-DATA-STORAGE-CONTRACT.md` is a concise **storage framework boundary**.
- `docs/contracts/CORE-V1.2-DESIGN.md` is a detailed **durable persistence design** with transaction, concurrency, serialization, evidence and recovery semantics.

The framework documents should not redefine details owned by the contracts. Conversely, the detailed contracts should remain the normative implementation boundary.

## 1. Domain model family

### `docs/04-SIDERETH-DATA-MODEL.md`

Strengths:
- concise shared vocabulary;
- explicitly domain-independent;
- distinguishes user facts, system inferences, legal propositions and authoritative sources;
- establishes privacy and versioning principles;
- explicitly says production persistence/API/validation are separate work.

Risk:
- it uses the word **Canonical** and could be mistaken for the normative implementation contract when read without the index.

### `docs/contracts/CANONICAL-DOMAIN-MODEL.md`

Strengths:
- defines minimum required concepts and fields for major aggregates;
- establishes explicit invariants;
- defines adapter extension rules;
- provides the implementation-facing boundary.

Risk:
- its current status is `Draft for Gate 2 verification`, while the framework document is labelled canonical framework. This is acceptable only if the documentation index makes the authority distinction explicit.

### Decision

**Retain both.** Do not delete or merge in this phase.

Recommended authority rule:

> The framework data model explains the shared conceptual vocabulary; `contracts/CANONICAL-DOMAIN-MODEL.md` governs implementation conformance until a future promoted canonical contract supersedes it.

Future improvement: add an explicit cross-reference from the framework document to the contract and state that implementation conformance is governed by the contract.

## 2. Storage family

### `docs/05-SIDERETH-DATA-STORAGE-CONTRACT.md`

This is a compact storage boundary defining storage domains, required properties, privacy and the rule that AI/agent access passes through shared controls.

### `docs/contracts/CORE-V1.2-DESIGN.md`

This is a substantially more detailed persistence design covering repository semantics, transaction boundaries, concurrency, serialization/versioning, reference integrity, evidence preservation, audit independence, failure/recovery and provider portability.

### Decision

**Retain both.** The compact framework document is useful as an architectural overview; the v1.2 document is the detailed design boundary.

However, the word `CONTRACT` in `05-SIDERETH-DATA-STORAGE-CONTRACT.md` creates authority ambiguity because the newer detailed document is also contract-like. This should be resolved through naming/status normalization in a later controlled documentation PR rather than by deleting content.

## 3. Versioned Core design family

The `CORE-V0.x` through `CORE-V1.4` documents represent an implementation-history/design evolution rather than one flat set of competing canonical contracts.

The audit confirms that v1.3 and v1.4 are related but not interchangeable:

- v1.3 establishes the service/policy boundary;
- v1.4 extends the command boundary with event/audit/idempotency sequencing and explicit partial-commit semantics.

Therefore they should not be mechanically merged solely because they share the `CORE` prefix.

A later audit should determine which versions are historical design records and which remain active planning authorities, then mark superseded versions explicitly where appropriate.

## 4. High-priority normalization recommendations

1. Add explicit cross-links between framework documents and their implementation-facing contracts.
2. Normalize terminology so `canonical framework`, `contract`, `design`, and `conformance` are never used interchangeably.
3. Review the full CORE version family as a lifecycle/history family before any physical move.
4. Keep contract documents in `docs/contracts/` as the normative implementation namespace.
5. Do not delete older design records until their unique information is shown to be preserved elsewhere.

## 5. No-delete / no-move decision

No document in this family is authorized for deletion or physical relocation from this audit alone.

The repository rule remains:

**Read → classify → preserve unique content → update authority references → archive → verify → delete only after evidence.**

## 6. Next audit family

Proceed next to the **authorization / policy + action / approval / execution** family because these boundaries directly affect the Tool Gateway implementation and are higher risk than cosmetic documentation consolidation.
