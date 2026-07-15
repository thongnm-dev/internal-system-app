-- ============================================================================
-- sp_daily_report_entry_upsert
-- Insert or update a daily report hour entry for a (user, task, date, category_id).
-- Returns the saved record.
-- ============================================================================

DROP FUNCTION IF EXISTS sp_daily_report_entry_upsert(
    varchar, varchar, varchar, date, text,
    double precision, boolean, double precision, double precision, varchar
);

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
    p_category_id INTEGER
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
    category_id INTEGER,
    updated_at  TEXT
)
LANGUAGE plpgsql
AS $$
#variable_conflict use_column
BEGIN
    RETURN QUERY
    INSERT INTO daily_report_entries (
        username, task_id, project_id, entry_date, comment,
        hour, is_ot, regular_ot, midnight_ot, category_id, updated_at
    )
    VALUES (
        p_username, p_task_id, p_project_id, p_entry_date, p_comment,
        p_hour, p_is_ot, p_regular_ot, p_midnight_ot, p_category_id, NOW()
    )
    ON CONFLICT (username, task_id, entry_date, category_id) DO UPDATE
        SET project_id   = EXCLUDED.project_id,
            comment      = EXCLUDED.comment,
            hour         = EXCLUDED.hour,
            is_ot        = EXCLUDED.is_ot,
            regular_ot   = EXCLUDED.regular_ot,
            midnight_ot  = EXCLUDED.midnight_ot,
            category_id  = EXCLUDED.category_id,
            updated_at   = NOW()
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
        daily_report_entries.category_id,
        daily_report_entries.updated_at::text;
END;
$$;
