-- ============================================================================
-- sp_daily_report_entry_delete
-- Delete a daily report entry by (user, task, date, phase). Returns deleted count.
-- ============================================================================

DROP FUNCTION IF EXISTS sp_daily_report_entry_delete(varchar, varchar, date);

CREATE OR REPLACE FUNCTION sp_daily_report_entry_delete(
    p_username   VARCHAR(100),
    p_task_id    VARCHAR(120),
    p_entry_date DATE,
    p_phase      VARCHAR(100)
)
RETURNS INTEGER
LANGUAGE plpgsql
AS $$
DECLARE
    v_deleted INTEGER;
BEGIN
    DELETE FROM daily_report_entries
    WHERE username = p_username
      AND task_id = p_task_id
      AND entry_date = p_entry_date
      AND phase = p_phase;

    GET DIAGNOSTICS v_deleted = ROW_COUNT;
    RETURN v_deleted;
END;
$$;
