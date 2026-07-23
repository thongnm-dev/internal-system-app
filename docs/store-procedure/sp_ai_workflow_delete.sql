-- ============================================================================
-- sp_ai_workflow_delete
-- Delete a workflow by ID and created_by. Returns deleted count.
-- Steps are cascade-deleted by FK constraint.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_ai_workflow_delete(
    p_id         INTEGER,
    p_created_by VARCHAR(100)
)
RETURNS INTEGER
LANGUAGE plpgsql
AS $$
DECLARE
    v_deleted INTEGER;
BEGIN
    DELETE FROM ai_workflows
    WHERE id = p_id AND created_by = p_created_by;

    GET DIAGNOSTICS v_deleted = ROW_COUNT;
    RETURN v_deleted;
END;
$$;
