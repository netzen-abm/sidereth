# ResourceLink Conformance Finding — 2026-09-21

## Finding

The PostgreSQL duplicate-write path previously allowed a legacy class-less link (`semantic_class IS NULL`) to be overwritten by a new explicit semantic class.

That violated the canonical compatibility rule: existing links must not silently acquire Strong semantics merely because their endpoints exist locally.

It also weakened RL-008 because a duplicate write could change the semantic class of an existing legacy relationship.

## Remediation

The PostgreSQL `ON CONFLICT` condition now permits:

- `NULL` → `NULL` duplicate legacy writes;
- explicit class → identical explicit class duplicates;

and rejects:

- `NULL` → Strong;
- `NULL` → Forward;
- `NULL` → External;
- Strong → another class;
- Forward → another class;
- External → another class.

This preserves historical meaning and makes semantic-class changes explicit rather than implicit.

## Gate impact

This closes the identified semantic-class escalation defect in the PostgreSQL adapter.

The full ResourceLink gate remains open until RL-001 through RL-012 have executable evidence and live PostgreSQL proof at the exact production head.
