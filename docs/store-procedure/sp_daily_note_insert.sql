-- ============================================================================
-- sp_daily_note_insert
-- Insert a new daily work note and return the created record.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_daily_note_insert(
    p_username  VARCHAR(100),
    p_content   TEXT,
    p_note_date DATE,
    p_status    VARCHAR(15) DEFAULT 'completed'
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
    INSERT INTO daily_work_notes (username, content, note_date, status)
    VALUES (p_username, p_content, p_note_date, p_status)
    RETURNING
        daily_work_notes.id,
        daily_work_notes.username,
        daily_work_notes.content,
        daily_work_notes.note_date::text,
        daily_work_notes.status,
        daily_work_notes.created_at::text;
END;
$$;
