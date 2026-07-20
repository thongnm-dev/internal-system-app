-- ============================================================================
-- sp_role_delete
-- Delete a role by ID. Returns the count of deleted rows.
-- Deleting a role cascades to user_roles; the service layer guards against
-- removing a role that is still assigned to users.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_role_delete(
    p_id INTEGER
)
RETURNS INTEGER
LANGUAGE plpgsql
AS $$
DECLARE
    v_count INTEGER;
BEGIN
    DELETE FROM roles WHERE id = p_id;
    GET DIAGNOSTICS v_count = ROW_COUNT;
    RETURN v_count;
END;
$$;
