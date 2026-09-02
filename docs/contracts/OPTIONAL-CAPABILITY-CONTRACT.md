# SIDERETH Optional Capability Contract

**Status:** CANONICAL ARCHITECTURE POLICY — capability family definition; implementations are future work

## Purpose

SIDERETH may expose advanced infrastructure capabilities as **plug-and-play, independently deployable adapters**. These capabilities are not part of the mandatory legal/regulatory core and must never become hidden dependencies.

The initial optional capability family is:

1. Nostr
2. Nym
3. Reticulum
4. Zero-Knowledge Proofs (ZKP)
5. Blockchain / distributed ledgers
6. Freenet
7. WebAssembly (WASM)

The purpose is to preserve useful technology from the repository's historical decentralized work without importing its legacy identity, mock implementations, unsafe assumptions, or coupling SIDERETH to any one technology.

## User-choice principle

**Optional means user choice.**

A user may enable or disable an optional capability according to the user's needs, threat model, jurisdiction, device capability, availability, and informed preference.

SIDERETH core functionality must remain usable without any of these capabilities unless a particular future feature explicitly declares a capability as a prerequisite and communicates that prerequisite to the user before use.

Disabling an optional capability must not silently reduce the user's legal rights, remove core evidence, or prevent access to ordinary case-management functions where an equivalent core path exists.

## Architecture

```text
                         SIDERETH Core
                              |
             +----------------+----------------+
             |                |                |
       Policy/Gateway    Case/Incident     Evidence/Source
             |                |                |
             +-------- Capability Registry --+
                              |
                    Optional Capability Layer
                              |
       +------+------+------+------+------+------+------+
       | Nostr | Nym | Reticulum | ZKP | Blockchain |  |
       |       |     |            |     |            |  |
       |       |     |            |     |        Freenet
       |       |     |            |     |            |
       +-------+-----+------------+-----+------------+
                              |
                            WASM
                 (execution/isolation capability)
```

WASM is treated primarily as an execution/isolation technology and may be used to package portable capability modules. It is not automatically a network, identity, privacy, or trust layer.

## Shared capability contract

Every optional adapter MUST declare, at minimum:

- capability ID and version
- purpose and user-visible benefit
- supported jurisdictions/environments
- required permissions
- data required
- data that leaves the device
- cryptographic/security assumptions
- availability requirements
- failure/degradation behavior
- evidence/provenance implications
- audit events
- dependencies
- licensing and operational constraints
- migration/disable path
- risk classification

Adapters MUST use shared SIDERETH infrastructure for:

**Identity → Policy → Permission → Data Minimisation → Execution → Audit**

An adapter must not create a parallel authorization model or bypass the SIDERETH Tool Gateway, audit trail, or evidence provenance rules.

## Capability-specific positioning

| Capability | Potential future role | Core dependency? |
|---|---|---|
| Nostr | User-controlled/publicly verifiable event distribution, optional decentralized communication or publication | No |
| Nym | Optional privacy-enhanced network transport for supported workflows | No |
| Reticulum | Optional resilient/offline or intermittently connected networking for supported environments | No |
| ZKP | Optional privacy-preserving proofs of selected facts/attributes without revealing unnecessary underlying data | No |
| Blockchain | Optional tamper-evident anchoring, timestamping, or externally verifiable attestations | No |
| Freenet | Optional decentralized/distributed application or data capability where technically and legally appropriate | No |
| WASM | Portable sandboxed capability execution and extension mechanism | No |

These are **capability hypotheses**, not claims that the corresponding integrations currently exist.

## Non-negotiable boundaries

Optional decentralized/privacy technologies MUST NOT:

- become required to create or manage a case;
- replace SIDERETH's canonical identity and authorization controls;
- replace the legal source/provenance system;
- make legal conclusions automatically;
- permit autonomous high-impact legal actions;
- make evidence originals mutable or silently replaceable;
- weaken auditability in the name of decentralization;
- expose sensitive case data merely because a capability is enabled;
- transmit personal data without an explicit, policy-checked purpose;
- be represented as implemented before code, tests, security review, and deployment evidence exist.

## Evidence and legal provenance

Decentralized storage, ledgers, signatures, proofs, or networks may strengthen technical integrity, availability, privacy, or verifiability. They do **not**, by themselves, establish the legal truth, authenticity, admissibility, authority, or legal effect of a record.

SIDERETH must therefore preserve the distinction between:

- original user evidence;
- cryptographic integrity evidence;
- source provenance;
- legal authority;
- legal interpretation;
- user-provided facts;
- system inference; and
- professional legal opinion.

## Failure and portability

If an optional capability becomes unavailable, SIDERETH should degrade gracefully wherever possible. The core case/incident record and evidence model must remain portable and understandable without the optional provider.

A capability adapter should be replaceable without rewriting domain logic. This is the principal **plug-and-play** requirement.

## Implementation policy

Future implementation order should be driven by a demonstrated user need and a verified threat/use-case model—not by technology availability.

Before an adapter becomes production-capable, it requires:

1. threat model and privacy assessment;
2. capability contract;
3. implementation isolated from legal domain logic;
4. deterministic tests;
5. interoperability/integration tests;
6. security review;
7. provenance and audit verification;
8. failure/offline behavior tests;
9. user consent/choice UX;
10. documentation that distinguishes implemented behavior from future capability.

Until those gates are met, the capability remains **PLANNING** or **CANDIDATE**, not production functionality.
