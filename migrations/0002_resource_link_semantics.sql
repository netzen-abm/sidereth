-- SIDERETH ResourceLink semantic-class extension.
-- NULL preserves the legacy class-less representation.
-- New links use strong, forward, or external explicitly.

ALTER TABLE sidereth_resource_links
    ADD COLUMN IF NOT EXISTS semantic_class TEXT;

ALTER TABLE sidereth_resource_links
    DROP CONSTRAINT IF EXISTS sidereth_resource_links_semantic_class_check;

ALTER TABLE sidereth_resource_links
    ADD CONSTRAINT sidereth_resource_links_semantic_class_check
    CHECK (
        semantic_class IS NULL
        OR semantic_class IN ('strong', 'forward', 'external')
    );

CREATE INDEX IF NOT EXISTS idx_sidereth_resource_links_semantic_class
    ON sidereth_resource_links (semantic_class);
