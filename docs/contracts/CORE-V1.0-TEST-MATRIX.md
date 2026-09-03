# SIDERETH Core v1.0 — Test Matrix

**Status:** DRAFT

| Area | Required coverage |
| --- | --- |
| Response validation | required IDs, title, content reference, source references |
| Response uniqueness | duplicate response IDs rejected |
| Response transitions | valid lifecycle transitions accepted |
| Response safety | invalid direct submission transition rejected |
| Escalation validation | required IDs, reason, target, source references |
| Escalation linkage | missing linked response rejected |
| Escalation uniqueness | duplicate escalation IDs rejected |
| Escalation transitions | valid lifecycle transitions accepted |
| Determinism | IDs returned in stable sorted order |
| Provenance | source references required on response and escalation |
| Boundary | no autonomous filing, legal advice, or lawfulness conclusion in core |

## Acceptance Criteria

- all response validation tests pass
- all response transition tests pass
- all escalation validation tests pass
- all escalation linkage tests pass
- all escalation transition tests pass
- deterministic registry tests pass
- `cargo fmt --all -- --check` passes
- `cargo check --all-targets` passes
- `cargo test --all-targets` passes
- `cargo clippy --all-targets --all-features -- -D warnings` passes
