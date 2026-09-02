# SIDERETH — Canonical Data Model Foundation

Status: CANONICAL FRAMEWORK / IMPLEMENTATION PENDING
Version: 1.0

## Purpose

Define the domain-independent data concepts shared across SIDERETH capabilities and adapters.

## Core objects

- Case — the durable legal/regulatory matter.
- Incident — a time-bounded official interaction or event that may exist independently before being linked to a Case.
- Event — an append-only domain occurrence used to reconstruct state and chronology.
- Document — an issued, received, uploaded or generated record associated with a matter.
- Evidence — preserved material supporting a fact, event, document or proposition.
- Legal Source — an authoritative or secondary source with provenance and lifecycle metadata.
- Jurisdiction — the geographic, subject-matter or institutional scope governing a matter.
- Authority — the public body, office or authorized actor relevant to a matter.
- Procedure — the applicable procedural sequence and requirements.
- Deadline — a time-bound obligation, response window or appeal period.
- Action — a proposed or approved operation in a workflow.
- Decision — an outcome issued by an authorized decision-maker.
- Assistance — a request or referral to a qualified human professional or legal-aid channel.

## Data separation rules

User-provided facts, system inferences, legal propositions and authoritative sources must remain distinguishable.

Evidence originals must never be silently overwritten. Derived representations may be regenerated from preserved originals and provenance.

Cross-case access is denied by default.

Adapters must not create parallel versions of canonical Case, Incident, Evidence or Legal Source semantics.

## Privacy boundary

Sensitive personal, business and case information is classified before processing. AI and agent runtimes receive only the minimum data authorized for the declared purpose.

The canonical data model does not grant AI access. Access is governed by identity, policy, permission, minimisation and audit controls.

## Versioning

Persisted objects and externally consumed schemas require explicit schema versions. Breaking changes require a migration strategy and compatibility assessment.

## Implementation status

This document defines the target canonical model. Production persistence, migrations, API serialization and complete validation are separate implementation work and are not claimed here.
