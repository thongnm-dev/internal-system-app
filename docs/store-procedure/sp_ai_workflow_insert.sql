-- ============================================================================
-- sp_ai_workflow_insert
-- Insert a new AI workflow and return the created record.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_ai_workflow_insert(
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
    INSERT INTO ai_workflows (name, description, created_by)
    VALUES (p_name, p_description, p_created_by)
    RETURNING
        ai_workflows.id,
        ai_workflows.name,
        ai_workflows.description,
        ai_workflows.layout,
        ai_workflows.created_by,
        ai_workflows.created_at::text,
        ai_workflows.updated_at::text;
END;
$$;
