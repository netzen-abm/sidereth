# SIDERETH Branch Convergence Archive — 2026-09-21

Status: canonical repository-hygiene record

## Policy

SIDERETH maintains exactly nine active branches. Non-allowlisted branches are retired after a disposition audit. Unique architectural work is reviewed before retirement; the canonical `main` branch remains authoritative.

## Nine active branches

1. `main`
2. `architecture/durable-tool-gateway-idempotency-v0-1-2026-09-19`
3. `architecture/repo-organization-privacy-runtime-licensing-2026-09-17`
4. `architecture/reticulum-inspired-cost-delay-sovereignty-research-2026-09-17`
5. `governance/ruleset-baseline-2026-09-16`
6. `docs/documentation-governance-2026-09-11`
7. `docs/semantic-audit-domain-model-2026-09-11`
8. `sidereth-ecosystem-architecture`
9. `sidereth-language-runtime-strategy`

## Retired-branch disposition

| Branch | Head SHA | Disposition |
|---|---|---|
| architecture/tool-gateway-kernel-v0-1-2026-09-17 | c4b46539eebc4840ee83418997383093c18df79a | Retire. Historical Tool Gateway implementation superseded by canonical main Tool Gateway. |
| decentralized-system | cf50619314e29ff750992c9daf1dc561c881b65e | Retire. Experimental Freenet layer; not promoted to canonical runtime. |
| feat/case-service-authorization-convergence-2026-09-14 | 55af98f7f949ec8453058e780d818d03876862bd | Retire. Divergent historical service implementation; canonical authorization/control-plane work is newer on main. |
| feat/observation-lifecycle-integrity-2026-09-12 | b5a25312444d0ceffdd8f1e8f4b0e5532724a912 | Retire after review. Observation lifecycle implementation is already present on main; branch is historically superseded. |
| refactor/canonical-authorization-binding-2026-09-14 | b3c59cedf220a9f98531938ddeee9025e4b00f07 | Retire. Authorization convergence work has been incorporated into later canonical main state. |
| sidereth-authorization-boundary | fe92ba78ee7e94cb345737fd927f13520e60dc43 | Retire. Historical authorization contract/audit work superseded by canonical contract on main. |
| sidereth-canonical-resource-links | eb37ff9e794823cc965673e31efe3f5219585370 | Retire. Resource-link and transactional persistence semantics are present in current main. |
| sidereth-capability-registry-conformance-hardening | 1de1893bf983fb9a2ff6893b6b4aa697fe1f39ce | Retire. Conformance work reviewed; no unique current implementation requiring branch retention. |
| sidereth-cas-hardening-v2 | 2dcd48797f389a4246383f3cf7f879dcdab2a594 | Retire. Historical CAS implementation superseded by current persistence/CAS semantics. |
| sidereth-evidence-persistence | 6b151334512fa461bd613eb2c303dc56d41850e1 | Retire after review. Evidence persistence implementation exists on main; branch is an older divergent implementation. |
| sidereth-license-apache-2 | 4ebfdd5ae2334f54b185b0b5fdbf52ebdd1fa37c | Retire. Apache-2.0 LICENSE is already present on main. |
| sidereth-post-pr39-semantic-audit | 992f7ea6bb09abb631513e2f4a85c356efd0256e | Retire. Historical semantic audit superseded by later canonical audits/docs. |
| sidereth-tool-gateway-implementation | 7a028da2dd200b6cdf45950d69dff43b61ffadaf | Retire. Older Tool Gateway implementation superseded by canonical main implementation. |
| sidereth-transactional-read-cas | 0ad63062b2d308081ef0512f9e19cc775e28660a | Retire. Historical transactional CAS implementation superseded by main. |
| sidereth-transactional-read-cas-current | fc269046e984b177784434f5cb0ad2575cef5486 | Retire. Historical variant superseded by main. |
| sidereth-universal-contract-hardening | b494f05def8572b22454283cfd2c403293264064 | Retire. Older universal contract hardening is materially behind current main; do not merge wholesale. |
| sidereth-universal-core-audit | 2440b096618a4ae454467f1ec1ed43dbf376b1a2 | Retire. Audit findings are historical evidence; current main architecture has incorporated the relevant control-plane direction. |

## Safety rule

Retirement means the branch ref is removed after this disposition record is merged. The canonical source of truth is `main`; historical commits are not treated as active architecture.

No retired branch may be recreated as an active branch unless it is explicitly promoted into the nine-branch allowlist through a reviewed pull request.

## Architecture consequence

Branch count is a governance property, not a substitute for code review. The repository must converge toward one canonical implementation of authorization, capability, execution, persistence, provenance, audit, lifecycle, and conformance semantics.

