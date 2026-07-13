-- ============================================================================
-- sp_daily_note_select_by_month
-- Select all notes for a user within a given month (YYYY-MM).
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_daily_note_select_by_month(
    p_username   VARCHAR(100),
    p_year       INTEGER,
    p_month      INTEGER
)
RETURNS TABLE (
    id         INTEGER,
    username   VARCHAR(100),
    content    TEXT,
    note_date  TEXT,
    status     VARCHAR(15),
    created_at TEXT
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
        n.id,
        n.username,
        n.content,
        n.note_date::text,
        n.status,
        n.created_at::text
    FROM daily_work_notes n
    WHERE n.username = p_username
      AND n.note_date >= v_start
      AND n.note_date < v_end
    ORDER BY n.note_date, n.created_at DESC;
END;
$$;
