# SIDERETH — System Architecture Blueprint V1

```text
                              SIDERETH
                                  |
                    EXPERIENCE / SURFACE LAYER
                                  |
        +-------------------------+-------------------------+
        |                         |                         |
       WEB                      MOBILE                  CHANNELS
        |                    Android / iOS        Telegram / WhatsApp / etc.
        +-------------------------+-------------------------+
                                  |
                           ADAPTER LAYER
                                  |
                       SHARED CAPABILITY API
                                  |
       +--------------------------+--------------------------+
       |                          |                          |
     CASE                     INCIDENT                 APPLICATION
       |                          |                          |
       +--------------------------+--------------------------+
       |       Evidence / Documents / Timeline / Deadlines   |
       +--------------------------+--------------------------+
       | Jurisdiction | Authority | Procedure | Compliance   |
       +--------------------------+--------------------------+
       | Response | Escalation/Remedy | Human Assistance      |
       +--------------------------+--------------------------+
                                  |
                     LEGAL SOURCE / KNOWLEDGE
                                  |
                  IDENTITY / POLICY / AUTHORIZATION
                                  |
                 PRIVACY / SECURITY / MODEL ARMOR
                                  |
                   AUDIT / OBSERVABILITY / EVENTS
                                  |
                         STORAGE / DATA LAYER

        AI / AGENTS are bounded augmentation around these services:
        Tool Registry -> Tool Identity -> Tool Gateway -> Policy ->
        Permission -> Data Minimisation -> Tool Runtime -> Audit
```

## Architectural rules
1. Surfaces contain presentation/channel concerns, not duplicated legal business logic.
2. Shared services are jurisdiction/domain-aware and reusable.
3. Case and Incident are universal aggregates.
4. Evidence originals are preserved independently of analysis.
5. Legal knowledge is versioned and provenance-aware.
6. AI cannot become the sole source of truth for legal propositions.
7. High-impact actions require explicit approval.
8. Agents cannot bypass the Tool Gateway or policy layer.
9. Sensitive case data is never exposed to unrelated domains or agents by default.
10. Future technologies may be plugged in through adapters without changing the universal core.

## Trust boundary
External documents, websites, retrieved content, user uploads and tool outputs are untrusted until validated. They must not directly alter system policy or agent permissions.
