-- ============================================================================
-- sp_user_username_exists
-- Check if a username exists, optionally excluding a specific user ID.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_user_username_exists(
    p_username   VARCHAR(100),
    p_exclude_id INTEGER DEFAULT NULL
)
RETURNS BOOLEAN
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN EXISTS (
        SELECT 1
        FROM users u
        WHERE LOWER(u.username) = LOWER(p_username)
          AND (p_exclude_id IS NULL OR u.id <> p_exclude_id)
    );
END;
$$;
