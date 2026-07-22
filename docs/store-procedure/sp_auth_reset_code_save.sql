-- ============================================================================
-- sp_auth_reset_code_save
-- Invalidate existing codes for a user, then insert a new reset code.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_auth_reset_code_save(
    p_user_id    INTEGER,
    p_code       VARCHAR(6),
    p_expires_at TIMESTAMPTZ
)
RETURNS VOID
LANGUAGE plpgsql
AS $$
BEGIN
    UPDATE password_reset_codes
    SET used = TRUE
    WHERE user_id = p_user_id AND used = FALSE;

    INSERT INTO password_reset_codes (user_id, code, expires_at)
    VALUES (p_user_id, p_code, p_expires_at);
END;
$$;
