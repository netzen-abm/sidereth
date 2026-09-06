# SIDERETH Authoritative Command Path

## Status

Implemented as a provider-neutral application infrastructure increment on branch `sidereth-authoritative-command-path`.

The Decision Model remains gated until this contract is integrated into every authoritative state-changing domain service and verified with provider-specific atomicity tests.

## Purpose

SIDERETH state-changing commands must not mutate a business resource first and then separately attempt idempotency, events, audit, or provenance. That ordering creates partial-success states.

The authoritative path therefore uses one `UnitOfWork` for the complete write set:

```text
Command
  |
  +--> authorization / policy evaluation
  |
  +--> AtomicCommandPlan
          |
          +--> idempotency marker
          +--> domain resource writes
          +--> resource links
          +--> event record
          +--> audit record
          +--> provenance record
          |
          +--> ONE UnitOfWork
                  |
             COMMIT or ROLLBACK
```

## Contract

`AtomicCommandPlan` is provider-neutral and resource-oriented. It supports insert/upsert operations and typed `ResourceRef` boundaries. `claim_operation()` places the operation id into the same insert set as the domain mutation and its side effects.

`execute_authoritative_command()` begins one `UnitOfWork`, applies the entire plan, runs any command completion logic inside the same context, and commits only after the operation succeeds. A provider failure or command error prevents successful completion of the UoW.

## Resource taxonomy

`Audit`, `Provenance`, and `Idempotency` are explicit `ResourceType` values rather than being hidden inside a domain-specific service. This allows the same infrastructure to be reused by Case, Incident, Evidence, Document, Compliance, and future domain packs.

## Compatibility boundary

The existing `CaseStore`, `EventStore`, `IdempotencyStore`, `AuditSink`, and repository interfaces are retained for source compatibility and existing local/legacy consumers. They are not the target architecture for new authoritative workflows.

New authoritative application services should depend on `UnitOfWorkFactory` and the canonical resource contracts. Legacy stores/repositories must not be expanded with new business workflows merely to avoid migration work.

## Important limitation

The current UoW context is a write/link boundary; it does not yet expose provider-neutral transactional reads or compare-and-set revision semantics. Therefore this increment establishes atomic coupling of the write set but does not, by itself, solve every concurrency concern in state transitions.

Before the Decision Model is unlocked, the next persistence increment must define transactional read/version semantics and then migrate authoritative Case transitions to that boundary. PostgreSQL must enforce the same semantics; local adapters may implement a compatible test adapter without weakening the contract.

## Non-goals

This contract does not:

- grant legal authority;
- replace authorization policy evaluation;
- make AI authoritative;
- introduce provider-specific semantics into the domain core;
- delete legacy interfaces;
- require a particular database or transport.
