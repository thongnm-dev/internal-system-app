-- ============================================================================
-- sp_daily_note_select_by_date
-- Select all notes for a user on a specific date.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_daily_note_select_by_date(
    p_username  VARCHAR(100),
    p_note_date DATE
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
BEGIN
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
      AND n.note_date = p_note_date
    ORDER BY n.created_at DESC;
END;
$$;
