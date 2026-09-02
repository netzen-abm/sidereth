# SIDERETH — Data Storage Contract Foundation

Status: CANONICAL FRAMEWORK / IMPLEMENTATION PENDING
Version: 1.0

## Purpose

Define the storage boundary for SIDERETH's shared legal/regulatory infrastructure without coupling the platform to a single database technology.

## Storage domains

### Case store
Stores canonical Case state, relationships and references.

### Incident store
Stores incident chronology, participants, official interactions and references to evidence/documents.

### Evidence vault
Preserves original evidence and immutable integrity metadata. Originals are not overwritten.

### Legal source store
Stores source metadata, versions, provenance, effective periods and supersession relationships.

### Deadline store
Stores verified deadlines, source references, jurisdiction, trigger events, status and reminders.

### Audit store
Stores security and workflow metadata needed for accountability without unnecessarily duplicating sensitive payloads.

## Required properties

- stable identifiers
- schema versioning
- provenance references
- integrity protection for originals
- authorization-aware access
- retention and deletion policy
- export capability
- recovery/backup strategy
- auditability
- migration compatibility

## Privacy

Sensitive data must be minimized at collection, storage, retrieval and processing boundaries. AI/agent systems do not receive direct database credentials or unrestricted storage access.

All AI/agent access must pass through the shared Tool Gateway and applicable identity, policy, permission and data-minimisation controls.

## Implementation status

This is a storage contract, not a database implementation. Database technology, migrations, key management, backup architecture and production retention controls require separate implementation and verification evidence.
