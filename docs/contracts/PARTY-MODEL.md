# SIDERETH — Universal Party Model

**Status:** Draft for Gate 2 verification  
**Scope:** Domain-independent identity and participation primitive

## 1. Purpose

The Party Model defines the universal representation of an actor that participates in a SIDERETH matter, workflow, document, action, decision, communication, or authorization context.

A Party is a reusable domain primitive. Domain adapters may specialize a party through namespaced extensions, but must not redefine canonical identity, provenance, authorization, audit, or privacy semantics.

The model deliberately separates **party identity** from **party role**. A single party may occupy different roles in different relationships or matters.

## 2. What is a Party?

A Party is an identifiable actor or actor-like entity relevant to a SIDERETH process.

Supported canonical party kinds:

- `person` — a natural person
- `organization` — a company, association, NGO, institution, etc.
- `government_entity` — a ministry, department, authority, local body, office, or other public entity
- `role_actor` — a non-person actor represented by a stable institutional or procedural identity where an individual identity is not required
- `system_actor` — an authorized SIDERETH service or automation identity

A Party is **not** synonymous with a user account, login identity, contact, or legal representative.

## 3. Canonical Party Record

Minimum concepts:

- `party_id`
- `schema_version`
- `party_kind`
- `status`
- `display_name`
- `identity_refs`
- `jurisdiction_refs`
- `organization_ref` (nullable)
- `provenance_ref`
- `privacy_classification`
- `created_at`
- `updated_at`
- `extension`

### 3.1 Identity separation

`party_id` is the stable SIDERETH identifier. External identifiers must be stored as separately governed identity references and must not replace the canonical ID.

Examples include:

- government-issued identifier references
- organization registration references
- professional registration references
- external system identifiers

Sensitive identifiers should be minimized, encrypted or tokenized where appropriate, and disclosed only under an explicit authorization and purpose.

## 4. Party Roles

Roles are contextual relationships, not intrinsic party types.

Examples:

- `applicant`
- `respondent`
- `complainant`
- `witness`
- `authority`
- `officer`
- `lawyer`
- `legal_representative`
- `service_provider`
- `beneficiary`
- `counterparty`
- `issuer`
- `recipient`
- `reviewer`
- `approver`
- `decision_maker`
- `owner`
- `assignee`

A domain pack may define additional roles under a namespaced extension, subject to the canonical relationship rules.

## 5. Party Relationships

Party participation is represented explicitly through a relationship object rather than embedding domain-specific role fields into Party.

Minimum concepts:

- `relationship_id`
- `schema_version`
- `from_party_id`
- `to_party_id`
- `relationship_type`
- `context_ref` (case, incident, document, action, decision, or other authorized context)
- `role`
- `valid_from`
- `valid_to` (nullable)
- `provenance_ref`
- `authorization_ref` where required
- `created_at`

This permits the same Party to be, for example, an `applicant` in one case and a `witness` in another without mutating the party's intrinsic identity.

## 6. Organization and Representation

Representation is explicit.

A lawyer, authorized representative, guardian, employee, officer, or other representative must not automatically inherit the represented party's authority.

Where one party acts for another, SIDERETH should record:

```text
Principal Party
      |
      | representation relationship
      v
Representative Party
      |
      v
Authorization / mandate
```

Authorization scope, validity and revocation remain governed by the authorization subsystem.

## 7. Party Status

Canonical status values:

- `active`
- `inactive`
- `suspended`
- `unknown`
- `superseded`

Status changes must be attributable and auditable. A historical party record must not be silently rewritten to reflect a later status.

## 8. Privacy and Data Minimisation

SIDERETH must collect the minimum party data necessary for the declared purpose.

The canonical Party model must not require:

- home address
- family information
- religion
- political affiliation
- unrelated personal attributes
- unnecessary biometric data
- sensitive identifiers when a less sensitive reference is sufficient

Public-office and professional information may be stored when necessary for the matter and supported by provenance, but this does not make unrelated personal information part of the canonical model.

## 9. Provenance

Identity claims and material party attributes must be traceable where their correctness matters.

Examples:

- user-provided fact
- official registry
- government source
- uploaded document
- verified professional source
- system-derived assertion

A system inference must never be represented as a user-provided fact or authoritative identity claim.

## 10. Authorization Boundary

Party existence does not grant access.

Party records must be evaluated against:

- actor identity
- case/context authorization
- purpose
- data classification
- policy
- applicable consent or legal basis where required
- audit requirements

Cross-case access is denied by default, consistent with the Canonical Domain Model.

## 11. Lifecycle

A Party record may be created, enriched, linked, corrected, superseded, deactivated, or deleted subject to retention and legal requirements.

Corrections to material identity assertions should preserve the audit trail and provenance of the previous assertion.

Deletion must respect retention obligations and should not destroy required audit evidence. Where deletion of the underlying subject data is not legally or technically possible, SIDERETH must apply the applicable retention/de-identification policy rather than silently rewriting history.

## 12. Invariants

1. `party_id` is stable within the SIDERETH deployment boundary.
2. Every Party carries a schema version.
3. Party identity and contextual role are separate concepts.
4. External identifiers never become canonical SIDERETH identifiers.
5. Representation does not imply authorization.
6. Party data is access-controlled by default.
7. Sensitive attributes are purpose-limited and minimized.
8. Material identity assertions have provenance where verification is required.
9. User facts, verified source facts and system inferences are distinct.
10. Party history cannot be silently rewritten.
11. Domain adapters cannot redefine canonical party identity or authorization semantics.
12. Every mutating operation is attributable to an authorized actor.

## 13. Extension Rule

Domain adapters may add party attributes and specialized relationship types under a namespaced extension object. They must not alter the semantics of `party_id`, party kind, contextual role, provenance, authorization, privacy classification, lifecycle or audit.

## 14. Relationship to Other Universal Primitives

Party is designed to connect to the existing SIDERETH universal model:

```text
Party
  |
  +---- Case
  +---- Incident
  +---- Document
  +---- Evidence
  +---- Authority
  +---- Jurisdiction
  +---- Action
  +---- Decision
  +---- Deadline
  +---- Response
  +---- Audit
```

Party therefore supplies participation and identity context without owning the lifecycle rules of those other primitives.

## 15. Non-goals

This contract does not define:

- authentication protocol
- account/session management
- full identity-proofing/KYC procedure
- legal determination of personhood or capacity
- domain-specific representative registries
- a universal global identity standard
- social profiles
- political scoring

Those concerns belong to separate contracts or domain packs.
