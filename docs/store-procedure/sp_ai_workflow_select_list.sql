-- ============================================================================
-- sp_ai_workflow_select_list
-- Select all workflows for a user, ordered by updated_at desc.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_ai_workflow_select_list(
    p_created_by VARCHAR(100)
)
RETURNS TABLE (
    id          INTEGER,
    name        VARCHAR(200),
    description TEXT,
    layout      JSONB,
    created_by  VARCHAR(100),
    step_count  BIGINT,
    created_at  TEXT,
    updated_at  TEXT
)
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN QUERY
    SELECT
        w.id,
        w.name,
        w.description,
        w.layout,
        w.created_by,
        (SELECT COUNT(*) FROM ai_workflow_steps s WHERE s.workflow_id = w.id),
        w.created_at::text,
        w.updated_at::text
    FROM ai_workflows w
    WHERE w.created_by = p_created_by
    ORDER BY w.updated_at DESC;
END;
$$;
