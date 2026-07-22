-- ============================================================================
-- sp_auth_reset_code_verify
-- Verify a reset code: must match user_id, code, not used, not expired.
-- If valid, mark as used and return the row id; otherwise return 0.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_auth_reset_code_verify(
    p_user_id INTEGER,
    p_code    VARCHAR(6)
)
RETURNS INTEGER
LANGUAGE plpgsql
AS $$
DECLARE
    v_id INTEGER;
BEGIN
    SELECT id INTO v_id
    FROM password_reset_codes
    WHERE user_id = p_user_id
      AND code = p_code
      AND used = FALSE
      AND expires_at > NOW()
    LIMIT 1;

    IF v_id IS NOT NULL THEN
        UPDATE password_reset_codes SET used = TRUE WHERE id = v_id;
        RETURN v_id;
    END IF;

    RETURN 0;
END;
$$;
