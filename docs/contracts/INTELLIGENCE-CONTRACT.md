# SIDERETH — Intelligence Contract

Status: Draft — architecture baseline

## 1. Purpose

The SIDERETH Intelligence layer provides bounded, replaceable computational assistance for retrieval, extraction, reasoning, drafting, classification and related intelligence workloads.

Intelligence is **not authoritative legal state**. Canonical legal state, evidence integrity, provenance, authorization, policy and consequential actions remain governed by SIDERETH core contracts.

## 2. Architectural boundary

```text
Surface / Workflow
        |
        v
Capability Contract
        |
        v
Policy + Authorization + Risk
        |
        v
Intelligence Contract
        |
        +---- Model Provider Adapter
        +---- Retrieval Adapter
        +---- Tool Adapter
        +---- Evaluation Adapter
        |
        v
Candidate Output
        |
        v
Claims / Sources / Uncertainty / Provenance
        |
        v
Human approval when required
```

No intelligence provider may bypass the capability, authorization, policy, provenance, audit or approval boundaries.

## 3. Intelligence request

An implementation should receive a structured request containing, as applicable:

- `request_id`
- `capability_id`
- `contract_version`
- `model_provider`
- `model_id`
- `model_version`
- `task_type`
- bounded input/context references
- jurisdiction
- data classification
- required output schema
- tool permissions
- retrieval requirements
- provenance requirements
- risk class
- approval policy

The contract must avoid passing unrestricted application state to a model provider.

## 4. Intelligence response

An implementation returns a structured result containing, as applicable:

- `request_id`
- model/provider identity
- generated or extracted content
- structured outputs
- source references
- evidence references
- claim status
- uncertainty indicators
- tool-use records
- warnings/errors
- provenance references
- execution metadata

A natural-language answer alone is not a sufficient authoritative result.

## 5. Epistemic boundary

Generated claims must be distinguishable from authoritative facts.

Recommended statuses:

- `OBSERVED`
- `USER_REPORTED`
- `EVIDENCE_SUPPORTED`
- `SOURCE_SUPPORTED`
- `SYSTEM_DERIVED`
- `INFERRED`
- `UNVERIFIED`
- `CONTESTED`
- `UNKNOWN`

The intelligence layer must not silently upgrade `INFERRED`, `UNVERIFIED`, `CONTESTED` or `UNKNOWN` information into authoritative fact.

## 6. Provider neutrality

SIDERETH does not depend on a particular model runtime or vendor.

Candidate implementations may include:

- local model runtimes
- Ollama
- hosted model providers
- institutional/private model services
- future runtimes

A provider adapter must be replaceable without changing canonical domain contracts.

### Ollama status

Ollama is an **optional future provider adapter**. This contract does not make Ollama a SIDERETH dependency.

## 7. Retrieval boundary

Retrieval systems are indexes and discovery mechanisms. They are not automatically canonical legal truth.

Where legal or regulatory assertions are consequential, the system should preserve:

- source identity
- source version/date where available
- jurisdiction
- retrieval provenance
- relevant citation/location
- verification status
- contradiction information where detected

## 8. Tool boundary

Intelligence may propose tool calls. It does not grant itself permission to execute them.

Tool execution must pass through the SIDERETH Tool Gateway, which evaluates identity, authorization, policy, scope and risk.

High-impact operations require the applicable human-approval contract.

## 9. Prompt and context provenance

Where practical and proportionate to the risk class, retain:

- prompt/template identity and version
- relevant context references
- model/provider identity
- model version
- retrieval references
- tool calls
- policy decision
- approval decision
- output/provenance references

Sensitive raw prompts or context must remain subject to data-classification and retention policy.

## 10. Data boundary

Local inference is a deployment option, not a universal privacy guarantee.

Every intelligence provider must declare:

- data sent to the provider
- data retained by the provider where known
- network requirements
- external dependencies
- supported data classes
- jurisdiction constraints
- failure behavior

Providers that cannot satisfy a capability's data policy must be rejected deterministically.

## 11. Failure semantics

Intelligence failure must not corrupt canonical state.

Examples:

- model unavailable
- timeout
- malformed output
- provider rejection
- retrieval failure
- tool failure
- insufficient evidence
- conflicting sources

The caller must receive an explicit failure or uncertainty result rather than fabricated completion.

## 12. Human approval

AI-generated drafts, recommendations and proposed actions must remain distinguishable from approved actions.

For `HIGH_IMPACT` capabilities:

```text
AI proposal
    -> validation
    -> human review
    -> explicit approval
    -> authoritative action
```

No model output constitutes legal authority, government acknowledgement, filing, acceptance or decision by itself.

## 13. Evaluation

Intelligence implementations must be evaluated against the capability's defined quality and safety requirements.

Future evaluation dimensions include:

- factual/grounding accuracy
- citation accuracy
- provenance completeness
- jurisdiction accuracy
- procedure accuracy
- deadline accuracy
- contradiction detection
- hallucination rate
- tool-use safety
- approval compliance
- latency/resource characteristics

Production promotion requires explicit evaluation evidence appropriate to the risk class.

## 14. Security and adversarial resilience

Evaluation should consider:

- prompt injection
- malicious documents
- retrieval poisoning
- tool manipulation
- data exfiltration
- indirect prompt injection
- misleading or contradictory sources
- model/provider failure
- output schema attacks

The intelligence layer is untrusted computation until its output passes the applicable verification boundary.

## 15. Versioning

The contract is versioned independently from model/provider versions.

A provider/model upgrade must not silently change the meaning of a canonical capability contract. Compatibility and evaluation status must be explicit.

## 16. Non-goals

This contract does not define:

- a particular LLM
- a particular vector database
- a particular OCR engine
- a particular agent framework
- a particular cloud provider
- autonomous legal authority
- autonomous high-impact legal action

Those are implementations or integrations behind stable SIDERETH boundaries.
