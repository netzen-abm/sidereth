# SIDERETH Core v0.9 — Test Matrix

**Status:** DRAFT

| Area | Required coverage |
|---|---|
| Requirement validation | empty IDs, missing obligation, missing description, missing sources |
| Evidence boundary | satisfied state without evidence rejected |
| Registry | duplicate requirement IDs rejected |
| State transitions | valid transitions accepted; invalid transitions rejected |
| Retrieval | lookup returns stored requirement |
| Obligation grouping | requirements returned for the requested obligation only |
| Determinism | IDs and grouped requirements returned in stable order |
| Provenance | source references remain mandatory |
| Safety boundary | compliance state does not assert legal liability or official unlawfulness |

## Acceptance Criteria

1. `cargo fmt --all -- --check` passes.
2. `cargo check --all-targets` passes.
3. `cargo test --all-targets` passes.
4. `cargo clippy --all-targets --all-features -- -D warnings` passes.
5. No AI, network, notification, or government-submission dependency is added to core.
6. No CI bypass is introduced.
