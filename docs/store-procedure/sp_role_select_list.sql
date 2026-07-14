-- ============================================================================
-- sp_role_select_list
-- List all available roles.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_role_select_list()
RETURNS TABLE (
    id          INTEGER,
    name        VARCHAR(50),
    description TEXT
)
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN QUERY
    SELECT r.id, r.name, r.description
    FROM roles r
    ORDER BY r.id;
END;
$$;
