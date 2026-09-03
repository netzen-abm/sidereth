# SIDERETH Core v1.2-C — Test Matrix

Status: DRAFT

| Area | Required proof |
| --- | --- |
| Typed errors | Contract operations expose stable typed persistence failures |
| Revision | Overflow fails deterministically; stale revision is rejected |
| Create | Duplicate stable IDs are rejected without replacement |
| Create publication | Complete serialized object is prepared before publication |
| Idempotency | First claim succeeds; repeated claim reports `AlreadyClaimed` |
| Idempotency restart | A claimed operation remains discoverable after reopening |
| Schema | Unsupported persisted schema versions fail closed |
| Corruption | Malformed persisted records produce integrity failure |
| Path safety | Empty, traversal and separator-containing IDs are rejected |
| Temp files | Repeated writes use distinct temporary paths |
| Replacement | Successful update publishes the new revision only after serialization succeeds |
| Concurrency scope | Reference adapter documents single-writer update semantics |
| Transactions | No transaction support is claimed without executable adapter semantics |
| Provider neutrality | No provider-specific dependency enters the core crate |

## Acceptance gate

All existing SIDERETH Foundation CI checks must pass, including formatting, compilation, tests and Clippy. No database or hosted provider may be introduced as part of this milestone.
