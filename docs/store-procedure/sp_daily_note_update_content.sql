-- ============================================================================
-- sp_daily_note_update_content
-- Update the content of a daily work note. Returns the updated record.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_daily_note_update_content(
    p_id       INTEGER,
    p_username VARCHAR(100),
    p_content  TEXT
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
    UPDATE daily_work_notes
    SET content = p_content
    WHERE daily_work_notes.id = p_id
      AND daily_work_notes.username = p_username
    RETURNING
        daily_work_notes.id,
        daily_work_notes.username,
        daily_work_notes.content,
        daily_work_notes.note_date::text,
        daily_work_notes.status,
        daily_work_notes.created_at::text;
END;
$$;
