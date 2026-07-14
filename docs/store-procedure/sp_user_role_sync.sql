-- ============================================================================
-- sp_user_role_sync
-- Delete all roles for a user, then re-insert the given role names.
-- Call once per user after create/update.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_user_role_sync(
    p_user_id    INTEGER,
    p_role_names TEXT[]
)
RETURNS VOID
LANGUAGE plpgsql
AS $$
BEGIN
    DELETE FROM user_roles WHERE user_id = p_user_id;

    INSERT INTO user_roles (user_id, role_id)
    SELECT p_user_id, r.id
    FROM roles r
    WHERE r.name = ANY(p_role_names);
END;
$$;
