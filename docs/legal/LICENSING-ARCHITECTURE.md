# SIDERETH — Layered Licensing Architecture

**Status:** CANONICAL LICENSING STRATEGY / IMPLEMENTATION POLICY  
**Decision date:** 2026-09-17

## 1. Principle

SIDERETH is **not intended to use one license for the entire ecosystem**.

The ecosystem uses a layered licensing model in which the license follows the nature of the artifact and its intended distribution boundary.

The initial strategy is:

```text
Open core / reusable infrastructure
        → Apache License 2.0

Selected network-facing components
        → GNU Affero General Public License v3

Commercial-only extensions / proprietary distributions
        → Proprietary Commercial License + custom EULA

Brand, trademarks and logos
        → Separate trademark / copyright policy

Third-party dependencies and data
        → Their own applicable licenses/terms
```

## 2. Apache-licensed core

The canonical SIDERETH domain/runtime and generally reusable infrastructure should remain under **Apache-2.0** unless a specific component is explicitly designated otherwise.

The current Cargo package already declares `Apache-2.0`, and the repository root `LICENSE` is Apache License 2.0. This remains the current core license baseline.

Apache-2.0 is appropriate for reusable infrastructure because it permits broad reuse while including an express patent grant. The official Apache guidance also supports file-level SPDX identification. citeturn2search2turn2search3

## 3. AGPL boundary

**AGPL-3.0-only or AGPL-3.0-or-later** may be used for a specifically designated network-facing component where source reciprocity for network use is an intentional product requirement.

AGPL must not be applied casually to the entire SIDERETH core.

The GNU project describes AGPL as GPL-based software with an additional provision for users interacting with the software over a network to receive the source for that program. citeturn2search9

An AGPL component must have:

- an explicit component boundary;
- an explicit license marker;
- dependency/license compatibility review;
- clear distribution/source obligations;
- separation from proprietary modules where required;
- release documentation identifying the applicable license.

## 4. Important compatibility rule

Apache-2.0 and AGPL are not simply interchangeable labels.

A repository may contain separately licensed components, but combining code into one derivative/linked work can create copyleft and compatibility consequences. The architecture must therefore prefer **clean component boundaries** rather than mixing licenses inside the same inseparable core module.

Every new AGPL dependency or AGPL-owned SIDERETH component requires a license compatibility review before merge.

## 5. Proprietary / commercial layer

Commercial-only functionality should not be placed into the public Apache/AGPL source tree merely to label it proprietary.

The preferred model is:

```text
Public repository
  ├── Apache-2.0 core
  └── explicitly designated AGPL components

Private/commercial distribution
  ├── proprietary extensions
  ├── commercial integrations
  ├── enterprise support features
  └── commercial EULA
```

A custom EULA may govern proprietary software, hosted services, commercial distributions and commercial support where appropriate. It does not retroactively change the license of code already released under Apache-2.0 or AGPL.

## 6. Commercial EULA requirements

A final commercial EULA should be drafted and reviewed by qualified counsel. It should address, as applicable:

- license grant;
- authorized users/devices;
- permitted environments;
- restrictions;
- ownership/IP;
- third-party/open-source components;
- updates and versioning;
- privacy/data processing;
- security responsibilities;
- AI/agent usage boundaries;
- hardware/device integrations;
- support/SLA terms;
- warranties/disclaimers;
- limitation of liability;
- indemnity where applicable;
- termination;
- data return/deletion;
- governing law/jurisdiction;
- export/sanctions requirements where applicable.

## 7. Brand and trademark separation

**The Purple Frog** and its logo are not software licenses.

Copyright licenses govern copyrightable works. Trademark rights govern use of names, marks and logos.

The combined Gaur + Purple Frog emblem must therefore have a separate brand/trademark/media-use policy. Open-source software rights must not be interpreted as permission to use the brand identity in a way that implies endorsement or affiliation.

## 8. Documentation and machine-readable licensing

Use SPDX identifiers for source and documentation files where practical, for example:

```text
SPDX-License-Identifier: Apache-2.0
SPDX-License-Identifier: AGPL-3.0-or-later
```

SPDX supports machine-readable license expressions using `AND`, `OR` and `WITH`, which should be used instead of ambiguous prose when multiple licenses apply. citeturn0search0

## 9. License inventory

The repository should maintain a machine-readable inventory covering:

- SIDERETH-owned code;
- SIDERETH-owned documentation;
- brand assets;
- third-party dependencies;
- third-party data;
- external models;
- generated artifacts where rights differ;
- proprietary/commercial modules.

## 10. Current status

**Current repository:** Apache-2.0 core baseline.  
**AGPL:** strategy approved; no blanket AGPL conversion.  
**Proprietary/EULA:** commercial layer strategy approved; final EULA remains a legal drafting gate.  
**Brand/trademark:** separate from software licensing and requires independent IP clearance.

This document is an engineering/licensing architecture policy, not legal advice.
