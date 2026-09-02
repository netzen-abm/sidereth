# Archived Legacy — `decentralized-system`

## Status
ARCHIVED / NOT APPROVED FOR MERGE

The `decentralized-system` branch contained four unique Freenet-layer files not present in the main/foundation baseline:

- `freenet-layer/package.json`
- `freenet-layer/publish.ts`
- `freenet-layer/src/lib.rs`
- `freenet-layer/tsconfig.json`

## Audit result
These files are experimental/placeholder infrastructure and are not suitable for SIDERETH production use without independent verification. In particular:

- the TypeScript publisher uses a hypothetical SDK interface;
- it targets a local WebSocket daemon;
- it contains a dummy contract identifier;
- it expects a private key environment variable;
- the Rust validation accepts UTF-8 state only;
- delta validation currently returns `Valid` without signature verification.

## Disposition
Do not merge these files into SIDERETH core. Preserve them here as historical reference until branch deletion is technically possible and independently confirmed safe.

## Future reuse rule
Any future decentralized transport must be introduced as an adapter behind SIDERETH capability contracts, identity, authorization, policy, data-minimisation and audit boundaries. It must pass a new technical and security review.
