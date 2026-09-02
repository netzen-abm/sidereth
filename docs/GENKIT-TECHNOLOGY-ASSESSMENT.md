# SIDERETH — Genkit Technology Assessment

**Status:** ARCHITECTURAL REFERENCE / DECISION PENDING
**Reviewed:** 2026-09-02
**Reference:** `genkit-ai/genkit`

## Executive recommendation

**Do not make Genkit a dependency of the SIDERETH universal core.**

**Retain it as a candidate AI/agent orchestration layer for a future implementation, subject to a focused proof-of-concept and security/privacy review.**

SIDERETH owns legal/regulatory semantics, provenance, authorization, evidence, workflow state and high-impact approval. Genkit may orchestrate model/tool workflows around those capabilities, but must not become the source of legal authority or the enforcement boundary.

## Integration boundary

```text
User / Surface
      |
Shared SIDERETH Capability API
      |
Identity -> Policy -> Permission -> Data Minimisation
      |
Tool Gateway
      |
Optional AI / Agent Runtime
      |    Genkit adapter / provider / RAG / tool workflow
      v
Validated SIDERETH capabilities
      |
Human approval where required
      |
Audit / Observability
```

**Genkit is downstream of SIDERETH policy, not upstream of it.**

## Candidate use cases

- Notice/document understanding into structured drafts for verification.
- Source-grounded explanation linked to SIDERETH provenance records.
- Proposed chronology construction for user confirmation.
- Procedure navigation using verified procedural data.
- Draft response assistance; submission remains outside autonomous agent authority.
- Bounded case triage and routing.

## Non-core responsibilities

Genkit must not own Case/Incident state, evidence integrity/storage, legal-source authority/provenance, jurisdiction/authority determination, authorization/policy enforcement, human approval, privacy enforcement, audit/event integrity, or final legal conclusions.

## Security requirements

- Tool Gateway remains mandatory.
- Model inputs are minimized/redacted before invocation.
- Untrusted documents are data, not instructions.
- Prompt-injection defenses surround the model runtime.
- Model output is untrusted until validated.
- Legal claims require provenance.
- High-impact actions require explicit human approval.
- Sensitive case data is not used for shared model training without explicit authorization.
- Model/tool workflows receive auditable identities without unnecessary sensitive-content logging.
- Provider/model selection is policy-controlled.

## Decision

**CANDIDATE — POC ONLY.** Do not add Genkit to `Cargo.toml` or couple it to the Rust universal core. If adopted, isolate it behind the future agent runtime as a replaceable adapter/service.

## POC acceptance criteria

- deterministic SIDERETH capability invocation;
- provenance-linked output;
- policy denial outside scope;
- human-approval denial when approval is absent;
- redaction/data-minimisation before model invocation;
- prompt-injection resilience tests;
- reproducible workflow/audit identifiers;
- AI-disabled operation;
- model/provider swap without changing legal-domain semantics.
