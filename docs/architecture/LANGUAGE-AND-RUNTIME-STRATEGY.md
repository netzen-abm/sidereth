# SIDERETH Language and Runtime Strategy

**Status:** Accepted architectural direction

## Decision

SIDERETH is not defined by a programming language. It is defined by its canonical domain model, contracts, invariants, security policies, and observable behavior.

**Canonical Domain Runtime: Rust.**

Rust is the canonical implementation runtime for the deterministic domain core and shared infrastructure where strong correctness, integrity, and security guarantees are required.

**Application & Integration Layer: Polyglot.**

Application, integration, automation, AI, interface, and specialized service layers may use other languages and runtimes where a justified technical advantage exists.

## Language neutrality principle

> SIDERETH has one canonical domain and contract system. Rust is the canonical runtime for the deterministic domain core. Other languages may implement application and integration capabilities where justified. Every implementation must conform to the canonical SIDERETH contracts and must not create a competing interpretation of the domain.

## Architectural rule

**One canonical domain. One canonical contract system. Multiple implementation runtimes where justified. Independent surfaces. Shared capabilities.**
