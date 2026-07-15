-- ============================================================================
-- sp_user_select_by_id
-- Find a user by ID.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_user_select_by_id(
    p_user_id INTEGER
)
RETURNS TABLE (
    id            INTEGER,
    username      VARCHAR(100),
    full_name     VARCHAR(200),
    email         VARCHAR(255),
    phone         VARCHAR(50),
    address       TEXT,
    "position"    VARCHAR(100),
    is_active     BOOLEAN,
    created_at    TEXT,
    updated_at    TEXT
)
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN QUERY
    SELECT
        u.id,
        u.username,
        u.full_name,
        u.email,
        u.phone,
        u.address,
        u.position,
        u.is_active,
        to_char(u.created_at, 'YYYY-MM-DD HH24:MI:SS'),
        to_char(u.updated_at, 'YYYY-MM-DD HH24:MI:SS')
    FROM users u
    WHERE u.id = p_user_id;
END;
$$;
