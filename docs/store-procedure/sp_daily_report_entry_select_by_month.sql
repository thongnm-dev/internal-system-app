-- ============================================================================
-- sp_daily_report_entry_select_by_month
-- Select all daily report entries for a user within a given month (YYYY-MM).
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_daily_report_entry_select_by_month(
    p_username VARCHAR(100),
    p_year     INTEGER,
    p_month    INTEGER
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
DECLARE
    v_start DATE;
    v_end   DATE;
BEGIN
    v_start := make_date(p_year, p_month, 1);
    v_end   := (v_start + INTERVAL '1 month')::date;

    RETURN QUERY
    SELECT
        e.id,
        e.username,
        e.task_id,
        e.project_id,
        e.entry_date::text,
        e.comment,
        e.hour,
        e.is_ot,
        e.regular_ot,
        e.midnight_ot,
        e.phase,
        e.updated_at::text
    FROM daily_report_entries e
    WHERE e.username = p_username
      AND e.entry_date >= v_start
      AND e.entry_date < v_end
    ORDER BY e.entry_date, e.task_id;
END;
$$;
