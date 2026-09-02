# SIDERETH — Audit, Storage and Encryption Contract

Status: Draft for Gate 2 verification

## 1. Audit
Every security-relevant or mutating action must produce an auditable record containing:
- event/action identifier
- timestamp
- actor identity and actor type
- authorization/policy decision reference
- target aggregate and operation
- outcome
- correlation and causation identifiers where applicable
- source/provenance references when legal knowledge is involved

Audit records must be append-only to the application trust boundary. Sensitive payloads should be referenced by secure object identifiers rather than copied into logs.

## 2. Storage boundaries
Storage is separated conceptually into:
- canonical domain state
- immutable evidence objects
- derived/search/index artifacts
- legal-source metadata and snapshots
- audit records
- ephemeral workflow state

A derived artifact can be regenerated without changing the original evidence.

## 3. Encryption
Required baseline:
- TLS for network transport
- encryption at rest for persistent sensitive data
- secrets stored in a dedicated secret-management mechanism, never source control
- encryption keys separated from encrypted data where practical
- key rotation documented and auditable

## 4. Data minimisation
Only collect data required for the stated purpose. Sensitive fields must have explicit access policy and retention rules. Telemetry must avoid unnecessary case content, document contents, credentials and personal identifiers.

## 5. Retention and deletion
Each sensitive data class must have a retention policy. Legal hold, user-requested deletion, regulatory retention and evidence-integrity requirements must be represented explicitly rather than handled by ad-hoc database deletion.

## 6. Evidence integrity
Original evidence is immutable. Integrity is established through cryptographic hashing and provenance metadata. Any transformation creates a new derived artifact linked to the original.

## 7. Access
Storage access is case-scoped and least-privilege. Direct database/object-store access is not a substitute for SIDERETH authorization policy.
