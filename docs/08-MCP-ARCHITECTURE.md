# SIDERETH — MCP Architecture Decision

Status: Recommended / V1 architectural decision

## Decision

Adopt Model Context Protocol (MCP) as an **AI integration and interoperability layer**, not as SIDERETH's core application architecture, authorization system, workflow engine, evidence store, or legal reasoning engine.

MCP is useful because it standardizes how AI hosts/clients discover and invoke external tools and resources. The current MCP specification also supports a client-host-server model and exposes resources, tools and prompts; the July 2026 specification adds a stateless protocol core, routable requests and authorization hardening. See the official specification and release notes for the exact version implemented by SIDERETH.

## Why MCP fits SIDERETH

MCP maps well to the already-decided SIDERETH architecture:

- Tool Registry → MCP tool discovery/catalogue
- Legal Source Engine → MCP resources/read-only context
- Document/Case services → scoped MCP resources/tools
- External government/public-data connectors → MCP servers where appropriate
- Agent Runtime → MCP client capability
- Independent future integrations → MCP-compatible adapters

This can reduce bespoke AI integration work and make SIDERETH's capabilities composable with compatible AI hosts.

## What MCP must NOT become

MCP is not the SIDERETH source of truth and must not replace:

- Identity Engine
- Policy Engine
- Authorization Engine
- Case Engine
- Incident Engine
- Evidence Vault
- Legal Source provenance model
- Deadline Engine
- Audit ledger
- Human approval system
- Core workflow/state machine

MCP sits at the integration boundary.

## Required SIDERETH boundary

AI Host / Agent
        |
        v
SIDERETH MCP Client / Adapter
        |
        v
Tool Gateway
        |
        +--> Identity
        +--> Policy
        +--> Permission
        +--> Data minimisation
        +--> Risk classification
        +--> Human approval
        |
        v
Canonical SIDERETH Capabilities
        |
        +--> Case
        +--> Incident
        +--> Evidence
        +--> Legal Sources
        +--> Jurisdiction
        +--> Procedure
        +--> Deadline
        +--> Response
        +--> Escalation
        |
        v
Audit / Observability

MCP must never provide a bypass around the Tool Gateway.

## Security rules

1. Every MCP server/tool is untrusted until explicitly registered and approved.
2. Tool metadata and descriptions are not authority; they are untrusted input unless the server is trusted and verified.
3. MCP credentials must be scoped and short-lived where feasible.
4. Case data must be case-scoped and purpose-limited.
5. Sensitive data must be minimized/redacted before external MCP processing where feasible.
6. Read-only tools should be preferred for legal knowledge retrieval.
7. Write/high-impact tools require explicit policy checks and human approval.
8. No MCP tool may autonomously file, submit, appeal, represent, or communicate on behalf of a user in a high-impact legal matter without the SIDERETH approval gate.
9. MCP tool calls must be auditable with tool identity, capability, scope, actor, case, policy decision, approval and result metadata.
10. External documents/resources are untrusted content and must be protected against prompt injection and tool poisoning.

## Recommended MCP server categories

### Tier A — Read-only / low-risk
- official legal source retrieval
- government procedure/checklist retrieval
- public case-law metadata/search
- public jurisdiction/authority lookup
- public document/schema resources

### Tier B — Scoped user-data operations
- case retrieval
- evidence indexing
- document extraction
- timeline retrieval
- deadline lookup

These require authenticated, case-scoped access.

### Tier C — Mutating / high-impact
- creating submissions
- filing appeals
- sending communications
- changing case state in consequential ways
- external government transactions

These require SIDERETH policy enforcement plus explicit user confirmation and, where required, human professional review.

## MCP adoption strategy

### Phase 1 — Design only
Define the SIDERETH Tool Registry so that every tool can optionally expose an MCP adapter without changing its canonical capability contract.

### Phase 2 — Read-only MCP
Expose selected verified legal/public resources and safe retrieval capabilities.

### Phase 3 — Scoped case MCP
Expose case/evidence capabilities through strict authentication and case-scoped authorization.

### Phase 4 — Agent Runtime integration
Allow approved SIDERETH agents to discover and call approved MCP tools through the Tool Gateway.

### Phase 5 — Carefully controlled write tools
Introduce mutating tools only after security, audit, authorization and human-approval gates are production-ready.

## Architectural principle

**Build SIDERETH capabilities first. Add MCP adapters second.**

Do not design the domain model around MCP. Design stable SIDERETH capability contracts, then expose selected capabilities through MCP.

## Decision outcome

MCP is **USEFUL and RECOMMENDED**, but as a boundary protocol rather than the foundation of SIDERETH.

Priority: HIGH for the future Agent/Tool Integration layer; LOW for the immediate Case/Incident foundation implementation.

Review trigger: major MCP specification changes, security advisories, or when SIDERETH begins implementing the Agent Runtime.
