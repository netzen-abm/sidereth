# SIDERETH — Core Party Persistence Contract

**Status:** CANONICAL / FOUNDATION DESIGN

## Purpose

This contract defines provider-neutral durable persistence for `Party` and `PartyRelationship` through the canonical Unit-of-Work boundary.

## Rules

1. `Party` and `PartyRelationship` are persisted as distinct resource types.
2. Persistence MUST use the canonical `UnitOfWorkContext`; no second transaction abstraction is introduced.
3. The repository validates the domain object before writing and validates decoded state after reading.
4. Insert semantics reject duplicate identifiers.
5. The adapter does not begin, commit, or roll back a transaction; the caller owns the surrounding Unit-of-Work.
6. A caller may therefore atomically persist Party, PartyRelationship, Case, Incident, Evidence, and other resources in one Unit-of-Work.
7. PostgreSQL is an implementation of the provider-neutral contract, not part of the domain contract.
8. Party relationships remain reusable across cases through explicit contextual references; the persistence boundary does not force case-specific ownership.
9. Identity, privacy, authorization, and provenance semantics remain domain contracts and are not inferred from storage mechanics.

## Completion gate

A production-ready implementation requires repository contract tests, validation-before-write tests, round-trip tests, duplicate rejection, provider error propagation, PostgreSQL adapter coverage, and an atomicity proof with another core resource.
