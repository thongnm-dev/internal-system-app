-- ============================================================================
-- sp_daily_report_task_delete
-- Delete a user-added daily report task by (user, task_id).
-- Returns deleted count.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_daily_report_task_delete(
    p_username VARCHAR(100),
    p_task_id  VARCHAR(120)
)
RETURNS INTEGER
LANGUAGE plpgsql
AS $$
DECLARE
    v_deleted INTEGER;
BEGIN
    DELETE FROM daily_report_tasks
    WHERE username = p_username
      AND task_id = p_task_id;

    GET DIAGNOSTICS v_deleted = ROW_COUNT;
    RETURN v_deleted;
END;
$$;
