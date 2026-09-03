# SIDERETH — Core v0.8 Deadline Infrastructure

Status: DRAFT / CORE V0.8
Version: 1.0

## 1. Purpose

Core v0.8 establishes a deterministic deadline model after procedure
infrastructure and before reminders, notifications, or execution services.

## 2. Canonical chain

**Procedure → Deadline → Obligation → Due Date → Status → Escalation**

A deadline records a time-bound requirement or procedural window. It does
not itself establish that the requirement applies to a person or case.

## 3. Deadline model

A `Deadline` contains:

- stable identity
- procedure reference
- name
- deadline type
- anchor date/time
- duration in whole days
- due date/time
- status
- legal-source references

The core uses explicit calendar dates supplied by an upstream verified layer.
It does not infer holidays, business days, extensions, limitation periods,
or jurisdiction-specific computation rules in v0.8.

## 4. Obligation boundary

An `Obligation` links a deadline to a bounded obligation description and
records whether applicability is verified, uncertain, or requires review.

The model distinguishes:

- defined obligation
- applicable obligation
- applicability uncertain
- satisfied
- overdue
- waived or extended when evidenced

Creating an obligation is not a legal conclusion about the user.

## 5. Date integrity

A deadline must not have an invalid interval. Duration must be non-negative.
The supplied due date must equal the anchor date plus the declared duration
under the core's explicit calendar-day rule.

Timezone and jurisdiction-specific calendar computation remain outside the
core until a dedicated contract is approved.

## 6. Status transitions

Status changes are deterministic and constrained by the domain model.
The core does not automatically mark an obligation overdue based on wall
clock time. An external scheduler may propose a status transition, but the
transition must still pass domain validation.

## 7. Provenance

Every deadline and obligation definition requires legal-source references.
References identify provenance; they do not by themselves prove current
applicability or validity.

## 8. Safety

The system must never convert a missing deadline calculation into a false
assurance that a filing, response, appeal, or limitation period is safe.
Where computation depends on holidays, service date, receipt date, extension,
statutory interpretation, or other unresolved facts, the state must remain
uncertain or require professional review.

## 9. Scope exclusions

Core v0.8 does not implement:

- notifications or reminders
- background schedulers
- holiday calendars
- business-day calculations
- limitation-period law
- automatic extension calculations
- live legal-source ingestion
- legal advice
- autonomous filing or government communication
- AI-generated legal conclusions
