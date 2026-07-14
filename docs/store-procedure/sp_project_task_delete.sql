-- ============================================================================
-- sp_project_task_delete
-- Delete a project task by id. Categories removed via ON DELETE CASCADE.
-- Returns deleted count.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_project_task_delete(
    p_task_id VARCHAR(50)
)
RETURNS INTEGER
LANGUAGE plpgsql
AS $$
DECLARE
    v_deleted INTEGER;
BEGIN
    DELETE FROM project_tasks WHERE id = p_task_id;
    GET DIAGNOSTICS v_deleted = ROW_COUNT;
    RETURN v_deleted;
END;
$$;
