# SIDERETH GitHub Governance Baseline

**Status:** CANONICAL / REPOSITORY CHANGE-CONTROL SPECIFICATION
**Scope:** GitHub repository governance for `main`
**Baseline reviewed:** `main` at `0ac5cdb8e8da545e1e5276af510ba4e8677558b9`
**Related issue:** #100

## 1. Purpose

This document defines the minimum GitHub change-control boundary for SIDERETH. It protects the repository's canonical architecture and evidence-based engineering process without changing runtime authorization semantics.

> Repository governance is a change-control boundary. Runtime authorization remains governed by SIDERETH's canonical authorization path.

## 2. Required `main` controls

The intended V1 ruleset for `main` is:

1. Pull request required before merge.
2. At least one approving review required.
3. Required Foundation validation.
4. Required RustSec dependency audit.
5. Required GitHub Actions security validation.
6. Force-push prohibited.
7. Branch deletion prohibited.
8. Required review conversations resolved before merge where GitHub supports the control.
9. Ordinary direct pushes to `main` prohibited.

The repository must not require a status check that is not actually produced by the active workflows.

## 3. Verified workflow/check inventory

The active Foundation workflow is `SIDERETH Foundation Validation` and its job check is `Validate Foundation`. It currently performs formatting, compilation, tests, live PostgreSQL proof, Clippy, repository/documentation integrity checks, legacy-reference checks and direct-secret-pattern checks.

The active Security workflow is `SIDERETH Security and Supply Chain` and currently produces these jobs:

- `RustSec Audit` — dependency vulnerability audit.
- `GitHub Actions Security` — zizmor workflow-security analysis.
- `OpenSSF Scorecard` — scheduled/push analysis; skipped for pull requests by design and therefore not a V1 pull-request required check.

Accordingly, the V1 required checks are the actual PR-produced checks:

- `Validate Foundation`
- `RustSec Audit`
- `GitHub Actions Security`

Do not require `OpenSSF Scorecard` as a pull-request gate until the workflow is changed to produce that check on pull requests and the change is separately audited.

## 4. Runtime boundary remains separate

GitHub branch/ruleset enforcement must never be treated as SIDERETH runtime authorization. Runtime protected operations continue to follow:

`AuthorizationRequest → AuthorizationEvaluator → AuthorizationResult → canonical consumer enforcement → contextual binding → authoritative execution/UoW → audit/provenance`

Capability leases remain a separate runtime boundary:

`Canonical Authorization/Policy → Capability Lease → Capability Adapter / OS Resource → Protected Operation → Release / Expiry → Audit`

## 5. Emergency / break-glass handling

No repository-level bypass is assumed merely because an administrator can change GitHub settings. If an emergency bypass mechanism is enabled later, it must be explicitly documented with:

- who may invoke it;
- what checks may be bypassed;
- why it was invoked;
- incident/change reference;
- duration and scope;
- post-change review;
- restoration of normal protection.

An undocumented administrator bypass is not part of the SIDERETH governance contract.

## 6. Current activation state

As of the reviewed baseline, the GitHub API reports **no repository rulesets configured**. The available integration can verify repository/ruleset state but does not have the repository-administration capability required to create or activate a GitHub ruleset or branch-protection policy.

Therefore this document records the verified target configuration, but **does not claim that `main` is protected**. Issue #100 remains open until the repository owner activates the controls and the resulting configuration is independently verified.

## 7. Change-control rules

Every change to canonical SIDERETH architecture or security boundaries should:

- use a dedicated branch;
- target `main` through a pull request;
- pass exact-head Foundation validation;
- pass exact-head security/supply-chain validation;
- receive the required review;
- preserve evidence and provenance for architectural changes;
- archive before deletion;
- avoid force-pushes to protected branches;
- avoid direct modification of `main` once governance is activated.

## 8. Verification record

The baseline immediately preceding this governance record includes:

- PR #105 merged: Purpose-Bound Capability Lease contract/model/conformance.
- PR #106 merged: root `.gitignore` repository-hygiene baseline.
- PR #107 merged: archive and removal of the historical temporary capability-registry diagnostic.

The current active workflows were inspected directly before defining the required checks. Their pull-request behavior is the authority for the check inventory, not older planning documents.

## 9. Activation acceptance test

Issue #100 may be closed only after all of the following are verified against GitHub's live repository settings:

- a ruleset exists and targets `main`;
- direct ordinary pushes are blocked;
- force-push is blocked;
- branch deletion is blocked;
- pull request requirement is active;
- at least one approval is required;
- `Validate Foundation` is required;
- `RustSec Audit` is required;
- `GitHub Actions Security` is required;
- review-conversation resolution requirement is active where supported;
- no stale/nonexistent required check is configured;
- the final ruleset configuration is recorded here with its activation date/reference.

Until then, the governance state is **DESIGNED / NOT YET ACTIVATED**.
