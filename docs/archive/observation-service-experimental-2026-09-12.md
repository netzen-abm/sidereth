# Archived Observation Service Experiment — 2026-09-12

## Status
Superseded before merge.

## Reason
An initial implementation was created as `src/observation_service.rs` while shaping the bounded Observation creation workflow. Review showed that the canonical command boundary should be named and exposed as `ObservationCommand`, not as a parallel service abstraction.

The implementation was also corrected by moving the active workflow into `src/observation_command.rs` so the repository has one explicit command boundary for Observation creation.

## Architecture decision
The active workflow reuses the existing generic `AtomicCommandPlan` and `execute_authoritative_command` infrastructure. It does not introduce an Observation-specific persistence layer, repository, lifecycle subsystem, or longitudinal aggregate.

## Historical commit
The superseded file was introduced on branch `feat/create-observation-workflow-2026-09-12` in commit `ccef5cf35076e4d2fa7207ae3f5ac65b49b317a7`.

## Replacement
`src/observation_command.rs` is the active implementation on this branch.
