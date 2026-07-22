-- ============================================================================
-- sp_auth_reset_code_has_valid
-- Check if a user has any unexpired, unused reset code.
-- Returns TRUE if exists, FALSE otherwise.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_auth_reset_code_has_valid(
    p_user_id INTEGER
)
RETURNS BOOLEAN
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN EXISTS (
        SELECT 1
        FROM password_reset_codes
        WHERE user_id = p_user_id
          AND used = FALSE
          AND expires_at > NOW()
    );
END;
$$;
