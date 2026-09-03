# SIDERETH Core v1.1 — Test Matrix

**Status:** DRAFT

| Area | Required coverage |
| --- | --- |
| Remedy validation | required IDs, case, category, description, source references |
| Remedy applicability | explicit state validation and transitions |
| Remedy lifecycle | deterministic valid and invalid transitions |
| Resolution validation | required IDs, case, outcomes, source references |
| Outcome distinction | requested and recorded outcomes remain separate |
| Cross-links | referenced escalation/response/remedy records must exist |
| Resolution lifecycle | deterministic valid and invalid transitions |
| Registry integrity | duplicate IDs rejected |
| Determinism | stable sorted IDs and retrieval |
| Boundary safety | no legal-entitlement or lawfulness inference |

## Required Negative Tests

- empty identifiers
- missing case references
- missing descriptions/categories
- missing source references
- invalid applicability transitions
- invalid remedy transitions
- missing linked escalation
- missing linked response
- missing linked remedy
- invalid resolution transitions
- duplicate remedy IDs
- duplicate resolution IDs

## Verification Standard

All tests must pass with the repository's strict formatting, compile, test, and Clippy gates. No CI bypass is permitted.
