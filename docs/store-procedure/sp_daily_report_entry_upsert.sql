-- ============================================================================
-- sp_daily_report_entry_upsert
-- Insert or update a daily report hour entry for a (user, task, date).
-- Returns the saved record.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_daily_report_entry_upsert(
    p_username    VARCHAR(100),
    p_task_id     VARCHAR(120),
    p_project_id  VARCHAR(120),
    p_entry_date  DATE,
    p_comment     TEXT,
    p_hour        DOUBLE PRECISION,
    p_is_ot       BOOLEAN,
    p_regular_ot  DOUBLE PRECISION,
    p_midnight_ot DOUBLE PRECISION,
    p_phase       VARCHAR(100)
)
RETURNS TABLE (
    id          INTEGER,
    username    VARCHAR(100),
    task_id     VARCHAR(120),
    project_id  VARCHAR(120),
    entry_date  TEXT,
    comment     TEXT,
    hour        DOUBLE PRECISION,
    is_ot       BOOLEAN,
    regular_ot  DOUBLE PRECISION,
    midnight_ot DOUBLE PRECISION,
    phase       VARCHAR(100),
    updated_at  TEXT
)
LANGUAGE plpgsql
AS $$
-- Cột out của RETURNS TABLE trùng tên cột bảng -> ON CONFLICT bị "ambiguous".
-- Ưu tiên hiểu định danh là CỘT bảng.
#variable_conflict use_column
BEGIN
    RETURN QUERY
    INSERT INTO daily_report_entries (
        username, task_id, project_id, entry_date, comment,
        hour, is_ot, regular_ot, midnight_ot, phase, updated_at
    )
    VALUES (
        p_username, p_task_id, p_project_id, p_entry_date, p_comment,
        p_hour, p_is_ot, p_regular_ot, p_midnight_ot, p_phase, NOW()
    )
    ON CONFLICT (username, task_id, entry_date) DO UPDATE
        SET project_id  = EXCLUDED.project_id,
            comment     = EXCLUDED.comment,
            hour        = EXCLUDED.hour,
            is_ot       = EXCLUDED.is_ot,
            regular_ot  = EXCLUDED.regular_ot,
            midnight_ot = EXCLUDED.midnight_ot,
            phase       = EXCLUDED.phase,
            updated_at  = NOW()
    RETURNING
        daily_report_entries.id,
        daily_report_entries.username,
        daily_report_entries.task_id,
        daily_report_entries.project_id,
        daily_report_entries.entry_date::text,
        daily_report_entries.comment,
        daily_report_entries.hour,
        daily_report_entries.is_ot,
        daily_report_entries.regular_ot,
        daily_report_entries.midnight_ot,
        daily_report_entries.phase,
        daily_report_entries.updated_at::text;
END;
$$;
