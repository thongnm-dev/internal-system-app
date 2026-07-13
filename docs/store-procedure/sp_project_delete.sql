-- ============================================================================
-- sp_project_delete
-- Delete a project by ID. Returns the number of rows deleted.
-- Members are removed automatically via ON DELETE CASCADE.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_project_delete(
    p_id INTEGER
)
RETURNS INTEGER
LANGUAGE plpgsql
AS $$
DECLARE
    v_deleted INTEGER;
BEGIN
    DELETE FROM projects WHERE id = p_id;
    GET DIAGNOSTICS v_deleted = ROW_COUNT;
    RETURN v_deleted;
END;
$$;
