-- ============================================================================
-- sp_user_select_list
-- List all users with their roles as a comma-separated string.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_user_select_list()
RETURNS TABLE (
    id            INTEGER,
    username      VARCHAR(100),
    full_name     VARCHAR(200),
    email         VARCHAR(255),
    phone         VARCHAR(50),
    "position"    VARCHAR(100),
    is_active     BOOLEAN,
    roles         TEXT,
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
        u.position,
        u.is_active,
        COALESCE(
            (SELECT string_agg(r.name, ',' ORDER BY r.name)
             FROM user_roles ur
             JOIN roles r ON r.id = ur.role_id
             WHERE ur.user_id = u.id),
            ''
        ),
        to_char(u.created_at, 'YYYY-MM-DD HH24:MI:SS'),
        to_char(u.updated_at, 'YYYY-MM-DD HH24:MI:SS')
    FROM users u
    ORDER BY u.id;
END;
$$;
