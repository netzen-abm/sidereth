# SIDERETH — Capability Contract

Status: Draft for Gate 2 verification

## 1. Purpose
SIDERETH capabilities are shared infrastructure. Web, mobile, bots, integrations and future surfaces are adapters; they do not create parallel business logic.

## 2. Capability envelope
Each capability should declare:
- `capability_id`
- `version`
- purpose
- input schema
- output schema
- required permissions
- data classes accessed
- jurisdiction scope
- risk class
- source requirements
- approval requirements
- audit requirements
- supported execution modes (sync/async/offline where applicable)

## 3. Risk classes
- `READ_ONLY`: public or user-authorized retrieval.
- `USER_DATA`: scoped access to the user's case/evidence data.
- `MUTATING`: changes canonical user state.
- `HIGH_IMPACT`: external filing, consequential communication, legal submission, or other action requiring explicit human approval.

## 4. Adapter rule
Adapters translate transport-specific input/output into canonical capability contracts. They must not bypass authorization, policy, data minimisation, audit, provenance or approval controls.

## 5. AI rule
AI may consume capability contracts but cannot acquire permissions merely by generating a request. The Tool Gateway evaluates identity, policy, scope and risk before execution.

## 6. MCP rule
MCP is an interoperability adapter for exposing selected SIDERETH capabilities to compatible AI hosts. MCP is not the source of truth for permissions, identity, case state, evidence, legal provenance or audit.

## 7. Evolution
Capability contracts are versioned. Breaking changes require a new major contract version or an explicit migration path. Consumers must be able to reject unsupported versions deterministically.
