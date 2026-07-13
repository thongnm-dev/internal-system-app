-- ============================================================================
-- sp_project_member_upsert
-- Insert or update a member for a project.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_project_member_upsert(
    p_project_id INTEGER,
    p_username   VARCHAR(100),
    p_name       VARCHAR(300)
)
RETURNS VOID
LANGUAGE plpgsql
AS $$
BEGIN
    INSERT INTO project_members (project_id, username, name)
    VALUES (p_project_id, p_username, p_name)
    ON CONFLICT (project_id, username) DO UPDATE
        SET name = EXCLUDED.name;
END;
$$;
