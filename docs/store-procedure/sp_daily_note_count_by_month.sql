-- ============================================================================
-- sp_daily_note_count_by_month
-- Count notes per date for a user within a month (for calendar badge).
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_daily_note_count_by_month(
    p_username VARCHAR(100),
    p_year     INTEGER,
    p_month    INTEGER
)
RETURNS TABLE (
    note_date  TEXT,
    note_count BIGINT
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
        n.note_date::text,
        COUNT(*)::bigint AS note_count
    FROM daily_work_notes n
    WHERE n.username = p_username
      AND n.note_date >= v_start
      AND n.note_date < v_end
    GROUP BY n.note_date
    ORDER BY n.note_date;
END;
$$;
