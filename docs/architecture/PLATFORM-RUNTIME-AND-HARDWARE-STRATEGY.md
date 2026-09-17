# SIDERETH — Platform Runtime, Language & Hardware Strategy

**Status:** CANONICAL ARCHITECTURE DECISION  
**Decision date:** 2026-09-17

## 1. Objective

SIDERETH must not become dependent on one programming language, operating system, cloud provider, device family or hardware vendor.

The architecture therefore uses:

> **one canonical domain/runtime language + multiple purpose-specific implementation languages + language-neutral contracts.**

The language choice follows the function. The domain contract remains independent of the implementation language.

## 2. Canonical core language

### Rust — canonical domain/runtime language

Rust is the canonical implementation language for the security-sensitive and domain-critical SIDERETH core, including:

- canonical domain model;
- authorization and policy enforcement;
- capability and capability leases;
- evidence/provenance primitives;
- deterministic workflows and state machines;
- persistence boundaries;
- security-sensitive data handling;
- portable core libraries;
- selected edge/embedded components where justified.

This is an architectural role, not a requirement that every SIDERETH component be Rust.

## 3. Supporting and functional languages

| Function | Preferred language | Boundary rule |
|---|---|---|
| Canonical domain/security/runtime | **Rust** | Canonical semantics live here |
| Web UI / browser surfaces | **TypeScript** | Calls canonical APIs/contracts; no duplicate domain authority |
| AI/ML/science/research | **Python** | AI remains downstream of authorization and evidence boundaries |
| Android platform integration | **Kotlin** | Adapter only; no independent policy semantics |
| Apple platform integration | **Swift** | Adapter only; no independent policy semantics |
| Database migrations/query logic | **SQL** | Schema/data boundary; no competing domain authority |
| Portable sandbox/interoperability | **WebAssembly + WIT** | Interface/transport boundary, not a second domain model |
| Vendor/hardware interoperability | **C/C++ via narrow FFI when required** | Only where SDK/driver constraints justify it |
| Embedded firmware | **Rust (`no_std` where appropriate)** | Hardware-specific adapter; shared contracts remain language-neutral |
| Experimental high-performance kernels | **Mojo only if benchmark-justified** | Never a core dependency by default |

The ecosystem is therefore deliberately polyglot without becoming poly-semantic.

## 4. Language-neutral boundary

Cross-language boundaries must use canonical contracts for:

- request/response;
- errors;
- authorization context;
- capability leases;
- resource references;
- evidence;
- provenance;
- events;
- lifecycle state;
- idempotency;
- versioning.

No language-specific object, memory layout or internal type may become the ecosystem-wide contract.

Where WebAssembly components are used, WIT may define language-neutral interfaces. The WebAssembly Component Model is explicitly designed for typed, composable interfaces between components. citeturn1search0turn1search5

## 5. Hardware independence

**Hardware is optional. SIDERETH core must remain fully functional without dedicated hardware.**

Future hardware support must follow:

```text
Hardware / OS resource
        ↓
Platform or device adapter
        ↓
Capability contract
        ↓
Authorization
        ↓
Purpose-bound capability lease
        ↓
Protected operation
        ↓
Evidence / provenance
        ↓
Canonical domain
```

A hardware device must never become an implicit authorization source.

## 6. Future hardware classes

The architecture may later support, where justified:

- camera and microphone capture;
- GNSS/location sources;
- mobile sensors;
- USB/Bluetooth devices;
- wearables;
- environmental or measurement sensors;
- forensic/evidence-capture devices;
- secure edge devices;
- embedded controllers;
- specialized scientific or laboratory equipment.

No individual device, chipset, OS or vendor is a required SIDERETH dependency.

## 7. Permission and lease rule for hardware

Sensitive hardware access is always:

```text
user/system request
    ↓
canonical authorization
    ↓
purpose + scope validation
    ↓
short-lived capability lease
    ↓
OS/device permission if required
    ↓
active handle
    ↓
operation
    ↓
release / expiry
```

After the authorized purpose completes, the active capability must be released. A future use requires fresh activation under the applicable policy/lease rules.

OS permission and SIDERETH authorization remain distinct.

## 8. Embedded strategy

If embedded support becomes necessary, Rust can support `no_std` targets where the standard library is unavailable, including bare-metal firmware. citeturn5search0

Vendor C/C++ SDKs and drivers may be used through tightly scoped FFI adapters when there is no practical Rust-native implementation. The C ABI is preferred for cross-language interoperability rather than making C++ ABI semantics a core boundary. citeturn5search8turn5search9

Hardware-specific code must not leak vendor types into canonical domain contracts.

## 9. Evidence integrity for hardware

Hardware-generated evidence must preserve, where available and appropriate:

- source/device identity;
- capture timestamp and clock semantics;
- calibration metadata;
- original bytes/artifact;
- cryptographic hash;
- provenance;
- interruption boundaries;
- authorization context;
- capability lease reference;
- uncertainty and sensor limitations.

Derived analysis must never silently replace the original capture.

## 10. No vendor lock-in

A new hardware provider is acceptable only when it can be replaced without changing canonical domain semantics.

Adapters may contain:

- vendor SDKs;
- platform APIs;
- device drivers;
- transport protocols;
- native FFI.

They may not contain the canonical authorization model, canonical evidence semantics or domain truth.

## 11. Implementation gates

Before introducing a new language or hardware dependency, require:

1. concrete capability need;
2. measurable benefit;
3. security/privacy assessment;
4. supply-chain review;
5. maintenance/skill assessment;
6. replacement strategy;
7. language-neutral contract;
8. conformance tests;
9. failure/offline behavior;
10. licensing compatibility.

## 12. Non-goals

This strategy does not commit SIDERETH to:

- dedicated hardware;
- one mobile platform;
- one cloud provider;
- Mojo;
- C++ as a core language;
- a proprietary device ecosystem;
- WebAssembly as a mandatory runtime;
- any specific sensor manufacturer.
