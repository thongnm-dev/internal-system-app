-- ============================================================================
-- sp_ai_workflow_update
-- Update name and description of a workflow. Returns updated record.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_ai_workflow_update(
    p_id          INTEGER,
    p_name        VARCHAR(200),
    p_description TEXT,
    p_created_by  VARCHAR(100)
)
RETURNS TABLE (
    id          INTEGER,
    name        VARCHAR(200),
    description TEXT,
    layout      JSONB,
    created_by  VARCHAR(100),
    created_at  TEXT,
    updated_at  TEXT
)
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN QUERY
    UPDATE ai_workflows w
    SET name = p_name, description = p_description
    WHERE w.id = p_id AND w.created_by = p_created_by
    RETURNING
        w.id,
        w.name,
        w.description,
        w.layout,
        w.created_by,
        w.created_at::text,
        w.updated_at::text;
END;
$$;
