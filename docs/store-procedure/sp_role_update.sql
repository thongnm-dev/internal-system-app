-- ============================================================================
-- sp_role_update
-- Update an existing role's name and description. Returns the updated row.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_role_update(
    p_id          INTEGER,
    p_name        VARCHAR(50),
    p_description TEXT
)
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
    UPDATE roles r
    SET name = p_name,
        description = p_description
    WHERE r.id = p_id
    RETURNING
        r.id,
        r.name,
        r.description,
        (SELECT COUNT(*) FROM user_roles ur WHERE ur.role_id = r.id),
        to_char(r.created_at, 'YYYY-MM-DD HH24:MI:SS');
END;
$$;
