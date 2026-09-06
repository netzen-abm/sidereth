-- SIDERETH provider-neutral resource persistence boundary.
--
-- This migration intentionally keeps the canonical payload opaque to PostgreSQL:
-- Rust owns domain semantics; PostgreSQL owns durability and transactionality.

CREATE TABLE IF NOT EXISTS sidereth_resource_records (
    resource_type TEXT NOT NULL,
    resource_id TEXT NOT NULL,
    schema_version INTEGER NOT NULL CHECK (schema_version > 0),
    revision BIGINT NOT NULL DEFAULT 0 CHECK (revision >= 0),
    payload JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (resource_type, resource_id)
);

ALTER TABLE sidereth_resource_records
    ADD COLUMN IF NOT EXISTS revision BIGINT NOT NULL DEFAULT 0;

ALTER TABLE sidereth_resource_records
    DROP CONSTRAINT IF EXISTS sidereth_resource_records_revision_check;

ALTER TABLE sidereth_resource_records
    ADD CONSTRAINT sidereth_resource_records_revision_check CHECK (revision >= 0);

CREATE TABLE IF NOT EXISTS sidereth_resource_links (
    source_type TEXT NOT NULL,
    source_id TEXT NOT NULL,
    relation TEXT NOT NULL CHECK (btrim(relation) <> ''),
    target_type TEXT NOT NULL,
    target_id TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (
        source_type,
        source_id,
        relation,
        target_type,
        target_id
    )
);

CREATE INDEX IF NOT EXISTS idx_sidereth_resource_links_target
    ON sidereth_resource_links (target_type, target_id);
