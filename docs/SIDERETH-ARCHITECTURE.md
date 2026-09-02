# SIDERETH — Target Architecture V1

**Status:** CANONICAL / PRE-IMPLEMENTATION

## 1. Product boundary
SIDERETH is a Legal & Regulatory Infrastructure system. It helps people and businesses navigate lawful procedures, prepare correctly, preserve evidence, meet deadlines, understand notices and decisions, respond, escalate and reach qualified human assistance.

It is not an autonomous lawyer and must not obstruct lawful government action.

## 2. System principle
Build shared capabilities once and expose them through independent adapters. Web, mobile, messaging and future surfaces are replaceable interfaces; legal/regulatory semantics live in the shared infrastructure.

## 3. Lifecycle
**Discover → Classify → Verify → Prepare → Act → Record → Monitor → Respond → Escalate → Resolve → Learn**

## 4. Universal infrastructure
- Case Engine
- Incident Engine
- Event / Timeline Engine
- Evidence Vault
- Document Engine
- Legal Source Registry
- Citation / Provenance Engine
- Jurisdiction Engine
- Authority Engine
- Procedure Engine
- Compliance Engine
- Application Engine
- Deadline Engine
- Response Engine
- Escalation / Remedy Engine
- Human Assistance Router
- Identity / Policy / Authorization Gateway
- Tool Registry / Runtime
- Memory Bank
- Audit / Observability
- Privacy / Security / Model Armor

These are target capabilities. A capability is **implemented** only when executable code, tests and required operational evidence exist.

## 5. Surface architecture
```text
Web / Android / iOS / Telegram / WhatsApp / Future Surfaces
                         |
                    Adapter Layer
                         |
              Shared Capability Boundary
                         |
        +----------------+----------------+
        |                |                |
      Case           Incident        Application
        |                |                |
        +------- Evidence / Documents ----+
                         |
       Jurisdiction / Authority / Procedure
                         |
       Legal Sources / Provenance / Deadlines
                         |
       Response / Escalation / Human Help
                         |
 Identity / Policy / Permission / Data Minimisation
                         |
       Privacy / Security / Model Armor
                         |
              Audit / Observability
                         |
                    Data Layer

AI / Agents remain bounded augmentation around this boundary.
```

## 6. Canonical reasoning
**Facts → Issue → Jurisdiction → Authority → Rule → Procedure → Evidence → Deadline → Options → Risk → Escalation**

## 7. Canonical procedural model
**Authority → Power → Procedure → Document → Deadline → Decision → Appeal → Remedy**

## 8. Case aggregate
A Case contains, as applicable: parties, authority, jurisdiction, matter, incidents, events, documents, evidence, legal issues, sources, procedures, deadlines, actions, decisions, responses, appeals, escalations, remedies, human assistance and audit references.

## 9. Incident aggregate
An Incident records, as applicable: type, authority, actors, location, start/end, stated purpose, stated legal basis, actions, requests, documents, items, statements, witnesses and observations.

## 10. State and evidence principles
Material state changes are represented by validated events. Derived state must be reproducible from valid event history. Original evidence is immutable; extraction, OCR, summaries and analysis are separate derived artifacts.

## 11. Legal knowledge principles
Legal propositions must be traceable to classified sources with jurisdiction, effective date/version, retrieval context, citation and uncertainty/status information. AI output is never itself legal authority.

## 12. AI / agent boundary
AI is optional intelligence and remains user-controlled. Agents automate bounded workflows through the Tool Gateway. They cannot bypass identity, policy, permissions, data minimisation or audit. High-impact legal actions require explicit human approval and qualified professional review where required.

## 13. Trust boundary
User uploads, external documents, websites, retrieved content and tool outputs are untrusted inputs until validated. They must not directly modify system policy, permissions or agent instructions.

## 14. Privacy/security
Local-first is the default. Sensitive case data is minimized, encrypted and access-controlled. Cloud processing is optional and requires explicit authorization, minimization and audit. Sensitive content is not copied into audit records unnecessarily.

## 15. Delivery order
1. Repository/documentation foundation
2. Canonical contracts and executable verification
3. Universal Case/Incident infrastructure
4. Evidence and legal-source infrastructure
5. Security/privacy/audit foundations
6. Web MVP and shared capability adapters
7. Agent/tool infrastructure
8. Panchayat/Municipality domain adapters
9. Application Guardian and Protect Now

No downstream implementation gate is considered open merely because its architecture has been documented.
