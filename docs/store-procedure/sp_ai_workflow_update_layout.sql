-- ============================================================================
-- sp_ai_workflow_update_layout
-- Update only the layout (node positions) of a workflow. Returns 1 if updated.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_ai_workflow_update_layout(
    p_id         INTEGER,
    p_layout     JSONB,
    p_created_by VARCHAR(100)
)
RETURNS INTEGER
LANGUAGE plpgsql
AS $$
DECLARE
    v_count INTEGER;
BEGIN
    UPDATE ai_workflows
    SET layout = p_layout
    WHERE id = p_id AND created_by = p_created_by;

    GET DIAGNOSTICS v_count = ROW_COUNT;
    RETURN v_count;
END;
$$;
