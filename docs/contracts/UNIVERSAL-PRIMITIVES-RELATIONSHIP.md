# SIDERETH — Universal Primitive Relationship

**Status:** Draft for Gate 2 verification

## 1. Purpose

This document establishes how the Universal Party Model and Universal Document Model connect to the existing SIDERETH canonical domain model without creating domain-specific silos.

## 2. Core Relationship

```text
                         SIDERETH MATTER
                              |
              +---------------+---------------+
              |                               |
            Party                          Document
              |                               |
       participation                    information
              |                               |
              +---------------+---------------+
                              |
                       Case / Incident
                              |
                 +------------+------------+
                 |            |            |
              Evidence      Action      Decision
```

Party answers **who participates**. Document answers **what recorded information exists**. Case/Incident provides contextual matter boundaries. Evidence records captured material with integrity semantics.

## 3. No Ownership Duplication

- Party owns canonical party identity and participation relationships.
- Document owns logical document identity and immutable content versions.
- Evidence owns evidence-capture and integrity semantics.
- Case owns the universal matter container.
- Incident owns the official-interaction record.
- Authority and Jurisdiction remain independent canonical references.
- Action and Decision own their respective lifecycle semantics.

No primitive may copy another primitive's canonical identity into domain-specific fields except as a reference.

## 4. Examples

### Government Notice

```text
Issuer Party
     |
     +---- issuer ----> Notice Document
                           |
                           +---- recipient ----> Citizen Party
                           |
                           +---- jurisdiction ----> Jurisdiction
                           |
                           +---- authority ----> Authority
                           |
                           +---- linked ----> Case
                           |
                           +---- derives ----> Deadline
```

### Evidence Document

```text
Citizen Party
     |
     +---- captured/created ----> Evidence
                                  |
                                  +---- references ----> Document Version
                                                          |
                                                          +---- derived OCR
```

The document remains the information object; the evidence record supplies capture and integrity context.

### Legal Representative

```text
Client Party
     |
     +---- represented-by ----> Lawyer Party
                                  |
                                  +---- authorization ----> Authorization
                                  |
                                  +---- acts-in ----> Case
```

Representation does not grant authority outside its explicit authorization.

## 5. Cross-Primitive Invariants

1. References use stable canonical IDs.
2. Cross-context access is authorization-controlled.
3. Provenance is preserved across transformations.
4. Derived artifacts do not overwrite originals.
5. Party roles are contextual relationships, not party identity mutations.
6. Document versions are immutable.
7. Evidence integrity does not imply legal authenticity.
8. AI-generated interpretations remain derived artifacts or assertions and do not become canonical source facts automatically.
9. Domain packs compose these primitives rather than replacing them.

## 6. Universal Workflow Composition

The primitives support a generic lifecycle:

```text
Party identifies matter
        |
        v
Document / Incident captured
        |
        v
Evidence preserved
        |
        v
Jurisdiction + Authority resolved
        |
        v
Procedure / Rule retrieved
        |
        v
Deadline derived
        |
        v
Action prepared
        |
        v
Human approval where required
        |
        v
Action executed
        |
        v
Decision / Response recorded
        |
        v
Audit + provenance retained
```

The workflow engine composes capabilities; it does not embed party or document logic separately for each domain.

## 7. Domain-Pack Rule

A domain pack may declare:

- specialized party roles;
- specialized document types;
- specialized relationship types;
- domain-specific validation rules;
- jurisdiction-specific procedures;
- authoritative sources.

It must consume the universal Party and Document contracts and cannot redefine their canonical semantics.
