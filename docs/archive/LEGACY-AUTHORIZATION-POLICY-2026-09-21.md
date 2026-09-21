# Legacy Authorization Policy Archive — 2026-09-21

## Disposition

The former `AuthorizationPolicy`, `AccessRequest`, and `CaseAccessPolicy` abstraction is retired from the active Rust API.

## Reason

Repository-wide search on `main` found these symbols only in `src/authorization.rs`, `src/lib.rs`, and their unit tests. Active protected-operation paths use the canonical `AuthorizationRequest` → `AuthorizationEvaluator` → `AuthorizationResult` → `validate_authorization` boundary.

Keeping a second public authorization policy API created an avoidable semantic fork and made accidental reuse possible for future protected operations.

## Canonical replacement

1. `AuthorizationRequest` — exact subject/action/resource/purpose context.
2. `AuthorizationEvaluator` — policy evaluation.
3. `AuthorizationResult` — evaluated decision and bound context.
4. `validate_authorization` — consumer-side enforcement.

Authorization remains distinct from human approval, legal authority, and consequential execution permission.

## Preservation

The historical implementation remains preserved in Git history and this disposition record. It is not retained as active code merely for compatibility.

## Migration rule

New code MUST NOT recreate or introduce a parallel `AuthorizationPolicy` abstraction. Context-specific domains must bind their invariants to the canonical authorization boundary rather than defining another evaluator.
