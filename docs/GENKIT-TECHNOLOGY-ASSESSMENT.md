# SIDERETH — Genkit Technology Assessment

**Status:** ARCHITECTURAL REFERENCE / DECISION PENDING
**Reviewed:** 2026-09-02
**Reference:** `genkit-ai/genkit`

## Executive recommendation

**Do not make Genkit a dependency of the SIDERETH universal core.**

**Do retain it as a candidate AI/agent orchestration layer for a future implementation, subject to a focused proof-of-concept and security/privacy review.**

The separation is deliberate: SIDERETH owns legal/regulatory semantics, provenance, authorization, evidence, workflow state and high-impact approval. Genkit may help orchestrate model/tool workflows around those capabilities, but must not become the source of legal authority or the enforcement boundary.

## What was verified

The referenced repository is public, active, Apache-2.0 licensed, and describes itself as an open-source framework for building agentic applications in JavaScript, Go, Dart and Python. Its repository topics include agents, AI, RAG, multimodal and vector databases.

The current repository structure includes JavaScript/TypeScript AI/core/genkit packages and plugins, documentation, tests and development tooling. The project homepage is `https://genkit.dev`.

## Where Genkit fits SIDERETH

### Strong fit
- Agent/flow orchestration around bounded SIDERETH capabilities.
- Model/provider abstraction where a workflow needs an LLM.
- Tool invocation orchestration.
- Retrieval/RAG workflows for legal-source analysis, provided SIDERETH controls source authority and provenance.
- Evaluation/observability patterns that can complement, but not replace, SIDERETH audit infrastructure.
- Rapid experimentation for AI-assisted document understanding, classification and drafting.

### Poor fit as a core dependency
Genkit should not own:
- Case or Incident canonical state.
- Evidence integrity or original evidence storage.
- Legal-source authority/provenance registry.
- Jurisdiction or authority determination.
- Authorization and policy enforcement.
- Human-approval gates for high-impact actions.
- Privacy/data-minimisation enforcement.
- SIDERETH audit/event integrity.
- Final legal conclusions.

## Recommended integration boundary

```text
                 SIDERETH

User / Surface
      |
      v
Shared Capability API
      |
Case / Incident / Evidence / Legal Source / Procedure
      |
Identity -> Policy -> Permission -> Data Minimisation
      |
Tool Gateway
      |
+-------------------------------+
| Optional AI / Agent Runtime   |
|                               |
| Genkit adapter / provider     |
| RAG / model / tool workflow   |
+-------------------------------+
      |
      v
Validated SIDERETH capabilities
      |
Human approval where required
      |
Audit / Observability
```

The key rule is **Genkit is downstream of SIDERETH policy, not upstream of it**.

## Legal/regulatory AI use cases worth testing

1. **Notice/document understanding** — extract dates, parties, authority, stated legal basis and requested actions into a structured draft for verification.
2. **Source-grounded explanation** — retrieve approved legal sources and generate an explanation whose claims remain linked to SIDERETH provenance records.
3. **Chronology construction** — turn case documents and user-recorded events into a proposed timeline for user confirmation.
4. **Procedure navigation** — propose next workflow steps from verified procedural data, with uncertainty surfaced.
5. **Draft response assistance** — produce a draft only; submission remains outside autonomous agent authority.
6. **Bounded case triage** — classify workflow type and route to the appropriate deterministic engine or human assistance pathway.

## Security requirements before adoption

A Genkit integration must pass SIDERETH's existing trust boundaries and must not receive unrestricted case data or tool access.

Minimum controls:
- Tool Gateway remains mandatory.
- Model inputs are minimized/redacted where possible.
- Untrusted documents are treated as data, not instructions.
- Prompt-injection defenses remain outside and around the model runtime.
- Model output is treated as untrusted until validated.
- Legal claims require source provenance.
- High-impact actions require explicit human approval.
- Sensitive case data is not used for shared model training without explicit authorization.
- Every model/tool workflow receives a correlation/workflow identity and auditable outcome record without unnecessarily storing sensitive content.
- Provider/model selection must be policy-controlled and jurisdiction/data-residency aware where required.

## Architecture decision

**Decision: CANDIDATE — POC ONLY.**

Do not add Genkit to `Cargo.toml` or otherwise couple it to the Rust universal core. If adopted, introduce it as an isolated AI/agent adapter under the future agent runtime, likely in a separate application/service boundary.

## Proof-of-concept acceptance criteria

The POC should demonstrate:

- deterministic SIDERETH capability invocation;
- source-grounded output with provenance IDs;
- denial when a tool is outside policy scope;
- denial when required human approval is missing;
- redaction/data-minimisation before model invocation;
- prompt-injection resilience tests;
- reproducible workflow/audit identifiers;
- graceful operation with AI disabled;
- ability to swap the model/provider without changing legal-domain semantics.

## Final opinion

Genkit is potentially valuable **technology infrastructure**, but it is not SIDERETH's architectural center of gravity. SIDERETH should own the legal/regulatory operating model; Genkit can be one replaceable engine used by the AI/agent layer.

This preserves the project's vendor/model neutrality and keeps the universal legal infrastructure usable without generative AI.
