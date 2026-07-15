-- ============================================================================
-- sp_daily_report_task_delete
-- Delete a user-added daily report task by (username, id).
-- Returns deleted count.
-- ============================================================================

DROP FUNCTION IF EXISTS sp_daily_report_task_delete(varchar, varchar);
DROP FUNCTION IF EXISTS sp_daily_report_task_delete(varchar, integer);

CREATE OR REPLACE FUNCTION sp_daily_report_task_delete(
    p_username VARCHAR(100),
    p_id       INTEGER
)
RETURNS INTEGER
LANGUAGE plpgsql
AS $$
DECLARE
    v_deleted INTEGER;
BEGIN
    DELETE FROM daily_report_tasks
    WHERE username = p_username
      AND id = p_id;

    GET DIAGNOSTICS v_deleted = ROW_COUNT;
    RETURN v_deleted;
END;
$$;
