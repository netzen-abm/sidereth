# The Purple Frog

**Privacy-first Legal & Regulatory Infrastructure**

**The Purple Frog** is the public product identity for a privacy-first, modular legal and regulatory infrastructure platform. The underlying ecosystem and technical architecture remain **SIDERETH** until a separate explicit platform-brand decision is made.

> **The Purple Frog — one product, shared infrastructure, many capabilities.**

The product name honors endangered and critically endangered species and uses biodiversity awareness as part of its public identity. The supplied **Gaur / Indian Bison (*Bos gaurus*)** emblem is the initial conservation symbol for the brand: a Vulnerable species used to build awareness of threatened life and support protection of threatened species, including Endangered and Critically Endangered species.

## Conservation mission

> **Awareness of a Vulnerable species. Protection of threatened species. Respect for every species.**

The Gaur is the initial ambassador for this mission. It must not be described as Critically Endangered; conservation status claims must remain sourced and current.

## What it does

- Prepare applications and compliance workflows
- Check jurisdiction, authority, procedure and documentary readiness
- Protect and record lawful government interactions
- Preserve evidence and build verifiable timelines
- Understand notices, orders and decisions
- Track deadlines and required actions
- Draft responses for user review
- Escalate unresolved matters
- Route high-stakes matters to qualified human professionals

## Architecture principle
The platform is built around shared, domain-independent capabilities. Web, mobile and messaging surfaces are independent adapters over the same contracts and engines. Domain logic must not be duplicated per surface.

## AI, agent and MCP boundary
AI is optional and user-controlled. AI/agent systems receive only the minimum authorized, policy-filtered data required for a task. Personal or sensitive case information must never be exposed directly to an AI or agent merely because it is available in the system. Redaction, data minimisation, purpose limitation, authorization, audit and provider/model policy are mandatory boundaries.

Agents automate bounded workflows, not high-impact legal judgment. MCP is an interoperability boundary for approved tools; it does not replace SIDERETH policy, authorization, case, evidence or audit infrastructure. Genkit remains a candidate AI/agent runtime and is not a SIDERETH core dependency.

## Privacy principle
Local-first by default. Sensitive case data should remain under user control. External processing is minimized, explicitly authorized, encrypted and auditable. AI-disabled operation must remain possible for core deterministic workflows.

## Conservation identity
The Purple Frog uses the Gaur as a conservation ambassador. Its mission begins with awareness of a Vulnerable species and extends to protection of threatened species, including Endangered and Critically Endangered species. Conservation-status claims are treated as sourced data, not decorative marketing. The product may later introduce structured species-awareness resources covering Vulnerable, Endangered, Critically Endangered and other conservation categories.

## Initial engineering milestone
Build the domain-independent Case and Incident Engine first, with evidence, events, source provenance and deterministic state transitions. Then add Panchayat and Municipality domain adapters without duplicating shared infrastructure.

## Repository status
The repository is undergoing a controlled foundation build. Historical prototype artifacts were audited for usefulness before disposition; obsolete material is removed from the active product path while repository history remains preserved.

## Current foundation documents
- `docs/00-DOCUMENTATION-INDEX.md`
- `docs/00-SIDERETH-MASTER-DECISIONS.md`
- `docs/01-SIDERETH-MASTER-BLUEPRINT.md`
- `docs/SIDERETH-ARCHITECTURE.md`
- `docs/ESTIMATE-WIREFRAME-PLAN.md`
- `docs/08-MCP-ARCHITECTURE.md`
- `docs/GENKIT-TECHNOLOGY-ASSESSMENT.md`
- `docs/SIDERETH-GLOSSARY.md`
- `docs/SIDERETH-MASTER-CHECKLIST.md`
- `docs/THE-PURPLE-FROG-BRAND-IDENTITY.md`
- `docs/contracts/`
- `docs/migration/`
- `docs/archive/`

## Development rule
Do not infer implementation from architecture documents. A capability becomes complete only when its contract, implementation, tests, security controls, documentation and observability are present and verified.
