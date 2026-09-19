-- Add durable execution lifecycle to Tool Gateway idempotency claims.
-- Existing deployments created by 0001 receive the same semantics without
-- relying on CREATE TABLE IF NOT EXISTS to mutate an existing table.
ALTER TABLE sidereth_tool_gateway_idempotency
    ADD COLUMN IF NOT EXISTS state TEXT NOT NULL DEFAULT 'claimed';

ALTER TABLE sidereth_tool_gateway_idempotency
    ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP;

ALTER TABLE sidereth_tool_gateway_idempotency
    DROP CONSTRAINT IF EXISTS sidereth_tool_gateway_idempotency_state_check;

ALTER TABLE sidereth_tool_gateway_idempotency
    ADD CONSTRAINT sidereth_tool_gateway_idempotency_state_check
    CHECK (state IN ('claimed', 'in_progress', 'completed', 'failed', 'unknown'));
