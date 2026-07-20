-- ============================================================================
-- sp_role_select_detail_list
-- List all roles with metadata (description, assigned user count, created_at)
-- for the governance role management screen.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_role_select_detail_list()
RETURNS TABLE (
    id          INTEGER,
    name        VARCHAR(50),
    description TEXT,
    user_count  BIGINT,
    created_at  TEXT
)
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN QUERY
    SELECT
        r.id,
        r.name,
        r.description,
        (SELECT COUNT(*) FROM user_roles ur WHERE ur.role_id = r.id),
        to_char(r.created_at, 'YYYY-MM-DD HH24:MI:SS')
    FROM roles r
    ORDER BY r.id;
END;
$$;
