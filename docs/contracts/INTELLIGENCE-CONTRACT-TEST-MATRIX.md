# SIDERETH — Intelligence Contract Test Matrix

**Status:** CANONICAL TEST DESIGN / PRE-IMPLEMENTATION
**Contract:** `docs/contracts/INTELLIGENCE-CONTRACT.md`
**Scope:** Provider-neutral intelligence boundary only.

This matrix defines the verification required before any model runtime or provider adapter is promoted into SIDERETH. It does not implement an LLM, Ollama, OCR engine, vector database, agent framework, or cloud provider.

## 1. Test principles

1. The Intelligence Contract is tested as a boundary, not as a model benchmark.
2. Canonical legal state remains outside intelligence.
3. Provider implementations are replaceable.
4. AI output is untrusted until the applicable verification boundary accepts it.
5. Tests must prove failure and refusal semantics, not only successful generation.
6. Sensitive data must remain within the declared data boundary.
7. High-impact actions must remain behind explicit human approval.

## 2. Contract test matrix

| ID | Area | Requirement | Test | Expected result | Gate |
|---|---|---|---|---|---|
| INT-001 | Request | Stable request identity | Submit request with `request_id` | Identity is preserved end-to-end | Contract |
| INT-002 | Request | Capability binding | Submit request with capability/version | Provider cannot change capability identity | Contract |
| INT-003 | Request | Bounded context | Provide explicit context references | Only declared context is exposed | Security |
| INT-004 | Request | Data classification | Declare permitted data class | Provider rejects unsupported class deterministically | Security |
| INT-005 | Request | Risk binding | Submit HIGH_IMPACT request | Risk remains attached to request | Safety |
| INT-006 | Request | Tool permissions | Request tool access not granted by policy | Tool execution is denied | Security |
| INT-007 | Provider | Provider neutrality | Run same contract against two fake providers | Canonical request/response semantics remain stable | Contract |
| INT-008 | Provider | Model identity | Return provider/model/version metadata | Identity is explicit and auditable | Provenance |
| INT-009 | Response | Structured result | Provider returns structured result | Schema is validated before acceptance | Contract |
| INT-010 | Response | Natural language insufficiency | Provider returns prose without required structure | Result is rejected or marked non-authoritative | Safety |
| INT-011 | Epistemic | INFERRED | Provider labels a claim INFERRED | Status remains INFERRED | Trust |
| INT-012 | Epistemic | UNKNOWN | Provider cannot establish a fact | UNKNOWN is preserved; no upgrade occurs | Trust |
| INT-013 | Epistemic | CONTESTED | Conflicting evidence is supplied | CONTESTED is preserved and surfaced | Trust |
| INT-014 | Epistemic | UNVERIFIED | Unsupported assertion is returned | UNVERIFIED is preserved | Trust |
| INT-015 | Epistemic | Forbidden upgrade | Attempt to convert INFERRED to authoritative fact | Operation is rejected | Trust |
| INT-016 | Sources | Source identity | Attach source references | References survive serialization | Provenance |
| INT-017 | Sources | Source version | Attach source date/version | Version metadata is preserved | Provenance |
| INT-018 | Sources | Jurisdiction | Attach jurisdiction | Jurisdiction remains explicit | Legal correctness |
| INT-019 | Sources | Contradiction | Supply contradictory sources | Contradiction is represented, not silently resolved | Legal correctness |
| INT-020 | Evidence | Evidence reference | Attach evidence IDs | Evidence references remain typed and traceable | Provenance |
| INT-021 | Retrieval | Retrieval failure | Retrieval adapter fails | Intelligence result reports explicit failure | Reliability |
| INT-022 | Retrieval | Retrieval is non-authoritative | Retrieved index result conflicts with canonical source | Canonical source remains authoritative | Trust |
| INT-023 | Tools | Proposed tool call | Model proposes permitted tool | Proposal is emitted without automatic authority | Safety |
| INT-024 | Tools | Unauthorized tool | Model proposes disallowed tool | Gateway denies execution | Security |
| INT-025 | Tools | HIGH_IMPACT tool | Model proposes filing/submission | Human approval is required before execution | Safety |
| INT-026 | Provenance | Prompt identity | Record template/prompt identity | Identity/version is retained when policy permits | Provenance |
| INT-027 | Provenance | Context provenance | Record context references | Context provenance is retained | Provenance |
| INT-028 | Provenance | Tool provenance | Tool is invoked | Tool identity/version/result reference is retained | Provenance |
| INT-029 | Provenance | Policy decision | Policy evaluates request | Decision is traceable | Audit |
| INT-030 | Provenance | Approval decision | Approval is required and granted | Approval reference is traceable to action | Audit |
| INT-031 | Failure | Timeout | Provider times out | Explicit timeout; no fabricated completion | Reliability |
| INT-032 | Failure | Malformed output | Provider violates schema | Explicit validation failure | Reliability |
| INT-033 | Failure | Provider rejection | Provider refuses request | Explicit provider failure | Reliability |
| INT-034 | Failure | Insufficient evidence | Evidence cannot support conclusion | Uncertainty/insufficiency returned | Trust |
| INT-035 | Failure | Canonical-state isolation | Intelligence operation fails after a pending state mutation | Canonical state is unchanged | Integrity |
| INT-036 | Security | Prompt injection | Malicious context attempts to alter policy | Policy boundary remains authoritative | Security |
| INT-037 | Security | Malicious document | Document contains instructions to exfiltrate data | Instructions are treated as untrusted content | Security |
| INT-038 | Security | Retrieval poisoning | Poisoned indexed result is returned | Result cannot silently become canonical truth | Security |
| INT-039 | Security | Data exfiltration | Provider attempts undeclared external transfer | Transfer is denied by data boundary | Privacy |
| INT-040 | Security | Output schema attack | Model attempts to alter required policy/risk fields | Contract-owned fields cannot be model-authorized | Security |
| INT-041 | Privacy | Local provider | Local execution declares no network requirement | Contract records deployment/data boundary | Privacy |
| INT-042 | Privacy | Remote provider | Remote provider is selected | Data transfer is explicit and policy-evaluated | Privacy |
| INT-043 | Privacy | Unsupported data class | Sensitive data exceeds provider policy | Provider is rejected deterministically | Privacy |
| INT-044 | Versioning | Contract version | Provider receives unsupported major version | Request is rejected deterministically | Compatibility |
| INT-045 | Versioning | Model upgrade | Provider model version changes | Compatibility/evaluation status is explicit | Compatibility |
| INT-046 | Versioning | Contract/model separation | Model version changes without contract change | Contract semantics remain unchanged | Compatibility |
| INT-047 | Approval | Draft vs action | AI drafts consequential communication | Draft remains unapproved | Safety |
| INT-048 | Approval | Explicit approval | Human approves proposed high-impact action | Only approved action can cross authoritative boundary | Safety |
| INT-049 | Approval | Approval mismatch | Approved content differs from executed content | Execution is blocked | Safety |
| INT-050 | Audit | Complete execution record | Successful request completes | Required audit/provenance references exist | Audit |

## 3. Fake-provider conformance suite

The first implementation should use deterministic fake providers. This prevents the test suite from depending on an external model runtime.

### Provider A — deterministic success

Returns:

- fixed model/provider identity;
- valid structured output;
- explicit source/evidence references;
- explicit epistemic status;
- no tool calls unless requested by the test.

### Provider B — adversarial/failure

Configurable to produce:

- malformed schema;
- timeout;
- provider rejection;
- unsupported data class;
- unauthorized tool request;
- epistemic-status upgrade attempt;
- contradictory source result;
- prompt-injection payload;
- output containing undeclared sensitive data.

The contract suite must pass or fail based on the boundary behavior, never on model quality.

## 4. Required invariant tests

### I-01 — AI cannot become legal authority

Given an intelligence response asserting a legal conclusion, the result must remain a candidate/intelligence result unless the applicable source, provenance, validation and approval boundaries establish an authoritative state.

### I-02 — AI cannot self-authorize

A generated tool call cannot create or expand its own permissions.

### I-03 — Retrieval cannot become canonical truth

A vector/index result cannot overwrite or supersede canonical legal-source state merely because retrieval ranked it highly.

### I-04 — Failure cannot mutate canonical state

A failed intelligence request cannot partially mutate Case, Party, Evidence, Provenance or other canonical resources.

### I-05 — High-impact execution requires approval

A proposed filing, submission, consequential communication or other HIGH_IMPACT operation cannot execute without the required explicit human approval.

### I-06 — Epistemic status cannot silently strengthen

`INFERRED`, `UNVERIFIED`, `CONTESTED` and `UNKNOWN` cannot be silently converted to authoritative fact by an intelligence provider.

### I-07 — Provider replacement preserves semantics

Replacing one provider adapter with another cannot require changes to canonical domain contracts.

### I-08 — Data policy is enforceable

A provider that cannot satisfy declared data-classification, jurisdiction or network policy is rejected deterministically.

## 5. Evaluation boundary

Model-quality evaluation is intentionally separate from contract conformance.

Later provider evaluation may measure:

- factual/grounding accuracy;
- citation accuracy;
- provenance completeness;
- jurisdiction accuracy;
- procedure accuracy;
- deadline accuracy;
- contradiction detection;
- hallucination rate;
- tool-use safety;
- approval compliance;
- latency and resource use.

A model can perform well on quality metrics and still fail SIDERETH contract conformance. Both gates are required for production promotion.

## 6. Promotion gates

| Stage | Minimum evidence |
|---|---|
| Contract implementation | Typed request/response + validation tests |
| Fake-provider conformance | INT-001 through INT-050 applicable tests |
| Security review | Injection, exfiltration, tool-boundary and data-policy tests |
| Provider evaluation | Capability-specific quality/safety evaluation |
| Integration verification | Persistence, provenance, audit and approval integration |
| Production readiness | Explicit risk-class approval and operational evidence |

## 7. Explicit non-goals

This matrix does **not** authorize:

- Ollama integration;
- hosted model integration;
- autonomous agents;
- OCR implementation;
- vector database selection;
- Langflow/OpenRAG adoption;
- Mojo adoption;
- external legal submission.

Those remain separate implementation decisions behind this contract.
