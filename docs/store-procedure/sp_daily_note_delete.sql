-- ============================================================================
-- sp_daily_note_delete
-- Delete a daily work note by ID and username. Returns deleted count.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_daily_note_delete(
    p_id       INTEGER,
    p_username VARCHAR(100)
)
RETURNS INTEGER
LANGUAGE plpgsql
AS $$
DECLARE
    v_deleted INTEGER;
BEGIN
    DELETE FROM daily_work_notes
    WHERE id = p_id AND username = p_username;

    GET DIAGNOSTICS v_deleted = ROW_COUNT;
    RETURN v_deleted;
END;
$$;
