# SIDERETH CLI Architecture

**Status: PLANNING — not implemented**

## Audit result

The inherited repository contained a CLI installation guide for a different project identity. That material described a hypothetical `cargo install` package and release workflow, but it did not correspond to an implemented SIDERETH CLI in the repository.

The current SIDERETH foundation contains a library crate (`sidereth-core`) and deterministic Case/Incident primitives. It does **not** currently contain a `src/bin/` CLI target, a CLI package, or a user-facing command implementation.

Therefore the repository must not advertise a SIDERETH CLI as an available product capability until executable code, tests, documentation, and release evidence exist.

## Intended role

When implemented, the CLI should be a thin independent adapter over shared SIDERETH contracts and engines. It must not contain duplicated legal/domain logic.

Potential first commands:

- `case create`
- `case show`
- `case transition`
- `incident create`
- `incident record`
- `evidence add`
- `timeline show`
- `deadline list`
- `source inspect`

These are planning examples, not implemented commands.

## Security and privacy boundary

The CLI must follow the same platform policy as every other SIDERETH surface:

- local-first operation for sensitive workflows where feasible
- explicit user authorization
- least-privilege access
- no direct exposure of personal or sensitive case data to AI/agents
- AI remains optional
- secrets must not be embedded in source or command arguments
- evidence originals must not be silently overwritten
- high-impact legal actions require explicit human approval
- all consequential mutations must be attributable and auditable

The CLI is an adapter. It must call shared policy, authorization, storage, evidence, legal-source and case/incident capabilities rather than bypassing them.

## Release rule

Do not create a CLI release workflow merely because a CLI is planned. A release workflow becomes appropriate only after a real CLI target exists and its command/API contract, tests, packaging, cross-platform behavior, and security review are verified.

Until then, CI should verify the absence of stale CLI claims rather than manufacture a binary that does not represent real SIDERETH functionality.
