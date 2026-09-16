# Capability Registry Test Diagnostic — Archived

- **Original repository path:** `tmp-capability-registry-test-error.txt`
- **Original commit:** `11b35d21d72923ac5c2f5999e432beed2360a20d`
- **Original commit message:** `chore: record temporary registry test diagnostics`
- **Captured:** 2026-09-10
- **Disposition:** Archived; the temporary root-level diagnostic is no longer part of the active repository surface.

## Diagnostic significance

The captured test run exercised the capability-registry suite and reported a failure in:

`capability_registry::tests::multi_node_cycle_is_rejected_deterministically`

The same run showed the surrounding capability-registry tests executing, including dependency validation, deterministic discovery, lifecycle promotion, provider replacement, and execution-authority separation checks.

## Preservation note

The original diagnostic text is preserved in Git history at commit `11b35d21d72923ac5c2f5999e432beed2360a20d`. This archive record preserves the diagnostic's provenance and reason for retention without keeping a transient CI/test log at the repository root.

## Current-state note

This artifact is historical diagnostic material, not a current test result or current architectural authority. Current behavior must be established from the active source, tests, and CI at the relevant `main` commit.
