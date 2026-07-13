-- ============================================================================
-- sp_project_member_delete_by_project
-- Remove all members for a given project (used before re-inserting on update).
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_project_member_delete_by_project(
    p_project_id INTEGER
)
RETURNS INTEGER
LANGUAGE plpgsql
AS $$
DECLARE
    v_deleted INTEGER;
BEGIN
    DELETE FROM project_members WHERE project_id = p_project_id;
    GET DIAGNOSTICS v_deleted = ROW_COUNT;
    RETURN v_deleted;
END;
$$;
