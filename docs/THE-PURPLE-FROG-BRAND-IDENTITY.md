# The Purple Frog — Product Brand Identity

**Status:** CANONICAL PRODUCT-BRAND DECISION
**Decision date:** 2026-09-06

## 1. Product name

The public product/consumer-facing name is:

> **The Purple Frog**

The name is intended to honor endangered and critically endangered species and to make biodiversity awareness part of the product's public identity.

## 2. Platform / infrastructure relationship

The Purple Frog does not require renaming the underlying SIDERETH engineering ecosystem.

```text
The Purple Frog
    ↓
Product / public brand
    ↓
SIDERETH
    ↓
Shared legal & regulatory infrastructure
    ↓
Capabilities · contracts · tools · resources · workflows · intelligence · adapters
```

SIDERETH remains the infrastructure and architecture identity in the repository unless a separate explicit brand migration decision is made.

The Purple Frog is therefore a product identity, not a replacement of the underlying technical architecture by implication.

## 3. Conservation purpose

The Purple Frog has a specific conservation-awareness purpose:

> **Build awareness around a Vulnerable species and support protection of threatened species, including Endangered and Critically Endangered species.**

The initial emblem, the Gaur / Indian Bison (*Bos gaurus*), represents the first part of that purpose: bringing public attention to a real **Vulnerable** species and using that species as an ambassador for broader biodiversity awareness.

The broader protection purpose includes:

- Vulnerable species
- Endangered species
- Critically Endangered species
- Other threatened species and ecosystems requiring conservation attention

The product must not imply that every species represented by the logo or future awareness experiences has the same conservation status.

Core public principle:

> **Awareness of one Vulnerable species. Protection of threatened species. Respect for every species.**

## 4. Primary emblem

The supplied logo uses the **Gaur / Indian Bison (*Bos gaurus*)** as the visual emblem.

The emblem is a symbolic conservation ambassador for biodiversity awareness. It is not intended to mean that Gaur itself is Critically Endangered.

Current verification: *Bos gaurus* is listed as **Vulnerable (VU)**, not Critically Endangered, in the conservation sources reviewed for this decision. India's National Tiger Conservation Authority also lists Gaur as Vulnerable under the IUCN Red List and Schedule I under India's Wildlife (Protection) framework.

Therefore product copy should say that the Gaur **represents the conservation cause**, rather than describing the logo animal itself as Critically Endangered.

## 5. Logo usage

The uploaded Gaur logo is the approved visual direction for the product identity.

Preferred asset hierarchy:

1. Transparent logo — canonical master for product interfaces and documentation.
2. Dark-background logo — preferred presentation/hero variant.
3. Raster variants — compatibility assets only.
4. SVG — preferred production format where the supplied asset is validated for clean rendering.

The checkerboard transparency preview must never be treated as part of the actual logo artwork.

## 6. Existing logo text

The supplied artwork contains **THE PURPLE FROG** and, in some variants, **EST. 2024**.

`EST. 2024` must not be treated as a canonical factual claim until the founding/establishment date is explicitly verified. Until then, production brand assets should prefer the version without an unverified establishment date.

## 7. Brand architecture principle

The product brand and technical infrastructure must remain decoupled.

```text
Brand
  ≠
Domain model
  ≠
Capability contracts
  ≠
Storage provider
  ≠
AI provider
  ≠
Surface
```

This preserves the ecosystem's provider-neutral and replaceable architecture.

## 8. Conservation integrity rule

The Purple Frog must not use species status as decorative marketing language.

Any claim such as:

- Critically Endangered
- Endangered
- Vulnerable
- population declining
- population recovered
- threatened habitat

must be tied to a named source, assessment scope and date/version where applicable.

Conservation status is dynamic and should be treated as sourced data rather than permanent brand copy.

## 9. Product positioning

Working positioning:

> **The Purple Frog — technology that builds awareness around threatened life and helps people act with greater care for the living world.**

The product's initial conservation story begins with awareness of the Vulnerable Gaur and extends to protection of threatened species, including Endangered and Critically Endangered species.

This positioning is intentionally broad enough to allow the product to grow while keeping biodiversity awareness and conservation action at the center of the identity.

## 10. Future species-awareness model

The Gaur should be the initial emblem, not the only species represented by the product.

Future awareness experiences can represent different species according to verified conservation data, geography, ecosystem and educational purpose.

The system should eventually model species as structured, sourced resources rather than hard-coded marketing content.

```text
Species
  ↓
Taxonomy
  ↓
Conservation status
  ↓
Threats
  ↓
Habitat / range
  ↓
Evidence & source
  ↓
Conservation action
  ↓
Public awareness
```

## 11. Governance gate

Before public launch of conservation-status claims, verify:

- scientific name
- common name
- current conservation category
- assessment scope
- assessment date/version
- source attribution
- licensing/usage rights for external species data and imagery

This prevents the product from spreading inaccurate conservation information while attempting to raise awareness.
