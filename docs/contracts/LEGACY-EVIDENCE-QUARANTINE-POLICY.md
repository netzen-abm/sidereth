# SIDERETH Legacy Evidence Quarantine Policy — Proposed V1

**Status:** Proposed operational policy; adoption and execution remain subject to issue #159.  
**Scope:** Historical `PersistedEvidence` records whose transformation/provenance association is absent, ambiguous, invalid, or fails current validation.

## Decision

**Fail closed for use; preserve first; never invent provenance.**

A record that contains transformations but cannot establish exactly one valid provenance binding per transformation MUST NOT be accepted by the normal Evidence Trust read/write path. Current `PersistedEvidence::validate` enforces this behavior. Missing legacy fields may decode using backward-compatible serde defaults, but defaulting is not evidence and does not establish attribution.

## Required handling

1. **Preserve source material before any migration.** Take a verified backup of the serialized record and its referenced original object. Record the original object hash, storage reference, source record identifier, schema version, export/backup identity, and time of acquisition. Do not rewrite original bytes or replace the source record in place.
2. **Classify the record.** Distinguish (a) records with no transformations and otherwise valid core metadata, (b) records with transformations and missing bindings, (c) mismatched/duplicate/orphan bindings, (d) invalid source/operation/actor relationships, and (e) unavailable or hash-mismatched original content. A record in one class must not be silently treated as another.
3. **Quarantine invalid trust state.** Mark the record unavailable to ordinary evidence retrieval, export, AI/RAG, graph traversal, and downstream legal conclusions until reviewed. Retain it for controlled forensic preservation. Quarantine metadata must identify the source record, original content hash, reason codes, detected-at time, migration tool/version, and review state. It must not overwrite the original evidence payload or manufacture a provenance binding.
4. **Keep review access narrow.** Only an explicitly authorized remediation workflow may inspect quarantined records. Access and every change to quarantine/review state must be auditable. A quarantine flag is not an authorization grant.
5. **Recover only from independent evidence.** A provenance association may be restored only where contemporaneous, independently verifiable source records support the transformation ID, input/source evidence, operation, actor identity/type, and relevant time. Preserve the source of the recovered association and label it as recovered during remediation; do not backdate it or present it as original captured provenance.
6. **Otherwise retain quarantine.** If the required attribution cannot be independently established, preserve the original object and metadata, leave the trust state unvalidated, and prevent normal trusted use. Human approval alone cannot turn unknown attribution into proven provenance.
7. **Use dry-run migration first.** A migration report must include counts by classification, collision/orphan findings, content-hash outcomes, records to quarantine, and recoverable candidates with evidence references. Review the report and backup verification before any state-changing run. Make the migration idempotent and resumable; a retry must not overwrite an earlier quarantine decision or create duplicate remediation events.

## Access semantics

- **Valid record:** normal access remains subject to the canonical authorization boundary.
- **Invalid/ambiguous provenance record:** normal read/export is denied; controlled remediation access may expose only the minimum information required for review.
- **Unavailable or hash-mismatched original object:** quarantine and investigation; never reconstruct the original bytes from derivatives.
- **No transformation history:** the absence of transformations is not itself proof of authenticity. Validate the original metadata and content hash under the existing evidence contract.

## Acceptance gates before closing issue #159

- A repository-wide inventory identifies all legacy evidence records and all read/export/RAG/graph paths that can expose them.
- The dry-run migration report and verified backup are reviewed.
- Automated tests cover missing, duplicate, orphan, mismatched, and independently recoverable provenance associations.
- Tests prove quarantined records cannot enter ordinary retrieval/export paths and that quarantine does not change the original hash or bytes.
- Migration is idempotent, rollback/restart behavior is documented, and counts reconcile before/after.
- The master checklist records exact CI evidence and the operator decision.

## Explicit non-goals

This policy does not create a second evidence repository, transaction manager, authorization service, or generic migration framework. Implement remediation only at the existing evidence persistence/authorized retrieval boundary and the smallest necessary administrative workflow. Do not run a production migration until the policy is approved and its implementation has a passing proof matrix.
