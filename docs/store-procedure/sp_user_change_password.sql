-- ============================================================================
-- sp_user_change_password
-- Update the password hash for a user. Returns count of updated rows.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_user_change_password(
    p_user_id       INTEGER,
    p_password_hash TEXT
)
RETURNS INTEGER
LANGUAGE plpgsql
AS $$
DECLARE
    v_count INTEGER;
BEGIN
    UPDATE users
    SET password_hash = p_password_hash,
        updated_at    = NOW()
    WHERE id = p_user_id;
    GET DIAGNOSTICS v_count = ROW_COUNT;
    RETURN v_count;
END;
$$;
