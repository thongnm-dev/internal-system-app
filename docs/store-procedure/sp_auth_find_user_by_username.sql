-- ============================================================================
-- sp_auth_find_user_by_username
-- Find a user by username for login authentication.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_auth_find_user_by_username(
    p_username VARCHAR(100)
)
RETURNS TABLE (
    id            INTEGER,
    username      VARCHAR(100),
    password_hash TEXT,
    full_name     VARCHAR(200),
    email         VARCHAR(255),
    is_active     BOOLEAN
)
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN QUERY
    SELECT
        u.id,
        u.username,
        u.password_hash,
        u.full_name,
        u.email,
        u.is_active
    FROM users u
    WHERE u.username = p_username;
END;
$$;
