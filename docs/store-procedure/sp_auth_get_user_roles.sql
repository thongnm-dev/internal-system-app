-- ============================================================================
-- sp_auth_get_user_roles
-- Get all role names assigned to a user.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_auth_get_user_roles(
    p_user_id INTEGER
)
RETURNS TABLE (
    name VARCHAR(50)
)
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN QUERY
    SELECT r.name
    FROM roles r
    INNER JOIN user_roles ur ON ur.role_id = r.id
    WHERE ur.user_id = p_user_id
    ORDER BY r.name;
END;
$$;
