-- ============================================================================
-- sp_user_delete
-- Delete a user by ID. Returns the count of deleted rows.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_user_delete(
    p_user_id INTEGER
)
RETURNS INTEGER
LANGUAGE plpgsql
AS $$
DECLARE
    v_count INTEGER;
BEGIN
    DELETE FROM users WHERE id = p_user_id;
    GET DIAGNOSTICS v_count = ROW_COUNT;
    RETURN v_count;
END;
$$;
