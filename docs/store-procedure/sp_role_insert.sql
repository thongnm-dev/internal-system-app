-- ============================================================================
-- sp_role_insert
-- Insert a new role and return the created row.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_role_insert(
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
    INSERT INTO roles (name, description)
    VALUES (p_name, p_description)
    RETURNING
        roles.id,
        roles.name,
        roles.description,
        0::BIGINT,
        to_char(roles.created_at, 'YYYY-MM-DD HH24:MI:SS');
END;
$$;
