# SIDERETH — Capability Model

**Status:** CANONICAL / FOUNDATION DESIGN

## Purpose

Define the reusable unit through which SIDERETH capabilities are designed, implemented, exposed and composed.

## Core model

```text
Capability
   |
   +-- Contract
   +-- Functions
   +-- Tools
   +-- Resources
   +-- Policies
   +-- Workflow participation
   +-- Implementations
   +-- Adapters
   +-- Tests
   +-- Documentation
   +-- Observability
```

## Capability identity

Each capability has a stable identifier and version. Breaking contract changes require a new major version or an explicit migration path.

Minimum metadata:

- capability_id
- version
- name
- purpose
- owner
- lifecycle status
- input schema
- output schema
- risk class
- permissions
- data classes
- jurisdiction scope
- source requirements
- approval requirements
- audit requirements
- execution modes
- implementation references

## Risk classes

### READ_ONLY
Public or otherwise authorized retrieval with no canonical mutation.

### USER_DATA
Scoped access to user-authorized case or evidence data.

### MUTATING
Changes canonical SIDERETH state.

### HIGH_IMPACT
Creates consequential external effects or legally significant actions and requires explicit human approval and professional review where required.

## Function

A function is a reusable deterministic or bounded operation within a capability. Functions should be independently testable where practical.

Examples:

- hash evidence
- validate identifier
- calculate due date
- resolve jurisdiction
- validate state transition

## Tool

A tool exposes an executable operation through a controlled interface. Tools must declare identity, permissions, data scope, jurisdiction scope, risk and audit requirements.

A tool is not trusted merely because an AI agent requested it.

## Resource

A resource is data or knowledge consumed by capabilities, such as:

- legislation
- rules
- notifications
- orders
- official procedures
- judicial decisions
- datasets
- templates
- schemas
- model artifacts
- knowledge collections

Resources require provenance and lifecycle metadata appropriate to their type.

## Workflow

A workflow composes capabilities into a controlled sequence. A workflow does not duplicate the implementation of the capabilities it uses.

Example:

```text
Notice
 -> Document
 -> Extract
 -> Jurisdiction
 -> Authority
 -> Procedure
 -> Deadline
 -> Evidence
 -> Response
 -> Approval
 -> Submit
```

## Domain composition

Domain packs provide domain-specific resources and rules while consuming universal capabilities.

```text
Domain Pack
    -> shared capabilities
    -> domain resources
    -> domain procedures
    -> domain workflows
```

## Surface composition

Surfaces translate human or transport-specific interaction into capability contracts.

```text
Web / Android / iOS / Telegram / WhatsApp / API
                    |
                    v
            Capability Contracts
```

No surface owns canonical domain logic.

## Capability lifecycle

```text
Proposed
 -> Designed
 -> Contracted
 -> Implemented
 -> Tested
 -> Security Reviewed
 -> Operationally Verified
 -> Active
 -> Deprecated
 -> Retired
```

## Completion rule

A capability is not complete because code exists. Completion requires the relevant contract, implementation, tests, security/privacy controls, documentation and observability.
