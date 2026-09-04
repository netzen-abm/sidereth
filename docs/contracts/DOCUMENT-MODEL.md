# SIDERETH — Universal Document Model

**Status:** Draft for Gate 2 verification  
**Scope:** Domain-independent document identity, versions, provenance and derived artifacts

## 1. Purpose

The Document Model defines the universal representation of a document used anywhere in SIDERETH: notice, application, order, licence, contract, affidavit, submission, correspondence, evidence attachment, decision, or other structured or unstructured record.

A Document is a reusable domain primitive. Domain packs may add document types and fields through namespaced extensions, but must not redefine canonical identity, versioning, provenance, integrity, authorization, or audit semantics.

## 2. Document Identity vs Content

A Document represents the logical record. Its content is represented through one or more immutable versions.

This separation is required because a document may:

- receive corrected or superseding versions;
- have multiple representations or formats;
- be translated or OCR-processed;
- produce derived artifacts;
- be linked to evidence without changing the original.

The original content must never be overwritten.

## 3. Canonical Document Record

Minimum concepts:

- `document_id`
- `schema_version`
- `document_type`
- `status`
- `title`
- `issuer_party_id` (nullable)
- `recipient_party_refs`
- `case_refs`
- `incident_refs`
- `jurisdiction_refs`
- `authority_ref` (nullable)
- `current_version_id`
- `provenance_ref`
- `privacy_classification`
- `retention_policy_ref`
- `created_at`
- `updated_at`
- `extension`

## 4. Document Version

Every material content state is represented as an immutable version.

Minimum concepts:

- `document_version_id`
- `document_id`
- `schema_version`
- `version_number`
- `media_type`
- `content_ref`
- `content_hash`
- `byte_length` where available
- `captured_at` or `created_at`
- `created_by`
- `source_ref`
- `provenance_ref`
- `integrity_status`
- `supersedes_version_id` (nullable)
- `language` (nullable)
- `created_at`

A new version does not mutate an earlier version.

## 5. Document Types

`document_type` is a controlled canonical category, extensible through domain namespaces.

Examples:

- `notice`
- `order`
- `decision`
- `application`
- `response`
- `appeal`
- `complaint`
- `affidavit`
- `contract`
- `licence`
- `certificate`
- `correspondence`
- `report`
- `invoice`
- `receipt`
- `submission`
- `legal_source_copy`
- `evidence_attachment`
- `other`

A document type describes the kind of record, not its legal validity.

## 6. Content and Representations

A single logical document may have multiple representations:

```text
Original Document
      |
      +---- PDF
      +---- image
      +---- scan
      +---- text extraction
      +---- OCR output
      +---- translation
      +---- structured extraction
```

Derived representations must reference the source version and must never be presented as the original.

## 7. Derived Artifacts

Derived artifacts are separately addressable outputs generated from a document version.

Examples:

- OCR text
- page images
- extracted fields
- entities
- tables
- translation
- summary
- classification
- citation mapping
- embeddings
- redacted copy

Minimum concepts:

- `artifact_id`
- `source_document_version_id`
- `artifact_type`
- `content_ref`
- `content_hash` where applicable
- `created_at`
- `created_by`
- `processing_provenance_ref`
- `model_ref` where AI/ML was used
- `confidence` where applicable

An AI-generated artifact is never automatically authoritative merely because it was generated from an authoritative document.

## 8. Provenance

A document and each material version/artifact must preserve provenance appropriate to its purpose.

Provenance may identify:

- user upload
- official source
- government portal
- email/message source
- camera capture
- evidence capture
- system transformation
- OCR engine
- translation system
- AI/ML model
- human review

The provenance chain must distinguish source material from transformations and interpretations.

## 9. Integrity

Document versions should carry a cryptographic content hash when technically available.

Integrity states should distinguish at minimum:

- `verified`
- `unverified`
- `modified`
- `unavailable`

Hash verification detects content change; it must not be described as proof of legal authenticity or truth.

## 10. Legal and Evidentiary Status

The Document Model must not infer legal validity solely from document structure.

Separate metadata may identify states such as:

- user-provided
- source-verified
- authenticity-unverified
- superseded
- revoked
- disputed
- under-review

Such states require provenance or explicit human/system assertions governed by policy.

## 11. Parties and Roles

Document participation is represented through Party relationships rather than embedding personal identity into the Document.

Examples:

```text
Party A --issuer------> Document
Party B --recipient---> Document
Party C --signatory---> Document
Party D --witness------> Document
```

A document can therefore be reused across cases and workflows without duplicating Party identity.

## 12. Jurisdiction and Authority

A document may reference one or more jurisdictions and authorities. These references are contextual and do not by themselves establish legal competence or validity.

Where a document is used for legal reasoning, the applicable jurisdiction and authority should be explicit or marked unknown.

## 13. Access and Privacy

Document existence does not grant document access.

Access is governed by:

- actor identity
- context/case authorization
- purpose
- privacy classification
- policy
- retention rules
- audit requirements

Sensitive documents must not be exposed to AI, agents, integrations or surfaces merely because they exist in storage.

## 14. Retention and Deletion

Documents and versions are subject to explicit retention policy.

Deletion, legal hold, archival, redaction and de-identification must be policy-governed and auditable.

An immutable original means the application must not overwrite it; it does not mean every document must be retained forever.

## 15. Lifecycle

Canonical logical-document statuses:

- `draft`
- `active`
- `superseded`
- `revoked`
- `archived`
- `deleted`
- `unknown`

Version status is separate from logical document status.

Lifecycle transitions must be attributable and policy-validated.

## 16. Invariants

1. `document_id` is stable within the SIDERETH deployment boundary.
2. Every Document carries a schema version.
3. Document identity is separate from content versions.
4. Document versions are immutable.
5. Derived artifacts never replace or masquerade as originals.
6. Every material version has provenance appropriate to its use.
7. Content hashes provide integrity verification, not legal authenticity by themselves.
8. User facts, source facts and system inferences remain distinct.
9. Access is denied by default across unauthorized contexts.
10. AI/agent processing requires explicit policy authorization and data minimisation.
11. Domain adapters cannot redefine canonical document identity, versioning, provenance, integrity or access semantics.
12. Every mutating operation is attributable to an authorized actor.

## 17. Extension Rule

Domain packs may add document types, fields and processing metadata under namespaced extensions. They must not alter canonical semantics of identity, versioning, provenance, integrity, lifecycle, authorization, privacy or audit.

## 18. Relationship to Other Universal Primitives

```text
Party
  |
  +---- Document <----> Document Version
  |                         |
  |                         +---- Derived Artifact
  |
  +---- Case
  +---- Incident
  +---- Evidence
  +---- Authority
  +---- Jurisdiction
  +---- Action
  +---- Decision
  +---- Deadline
  +---- Response
  +---- Audit
```

Document is therefore a reusable information primitive, while Evidence remains the integrity-oriented record of captured material. A document may be evidence, but not every document is evidence.

## 19. Non-goals

This contract does not define:

- a universal legal-authenticity determination
- document OCR implementation
- a specific storage provider
- a specific AI model
- electronic-signature protocol
- document-management UI
- jurisdiction-specific document rules
- legal advice or legal conclusions
