-- ============================================================================
-- sp_project_member_select
-- Select all members for a given project, ordered by username.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_project_member_select(
    p_project_id INTEGER
)
RETURNS TABLE (
    username VARCHAR(100),
    name     VARCHAR(300)
)
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN QUERY
    SELECT pm.username, pm.name
    FROM project_members pm
    WHERE pm.project_id = p_project_id
    ORDER BY pm.username;
END;
$$;
