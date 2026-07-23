-- ============================================================================
-- sp_ai_workflow_step_delete
-- Delete a workflow step by ID. Returns deleted count.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_ai_workflow_step_delete(
    p_id INTEGER
)
RETURNS INTEGER
LANGUAGE plpgsql
AS $$
DECLARE
    v_deleted INTEGER;
BEGIN
    DELETE FROM ai_workflow_steps WHERE id = p_id;

    GET DIAGNOSTICS v_deleted = ROW_COUNT;
    RETURN v_deleted;
END;
$$;
