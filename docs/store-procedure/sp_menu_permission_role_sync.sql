-- ============================================================================
-- sp_menu_permission_role_sync
-- Replace the whole menu grant list of a role with the given menu keys.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_menu_permission_role_sync(
    p_role_id   INTEGER,
    p_menu_keys TEXT[]
)
RETURNS VOID
LANGUAGE plpgsql
AS $$
BEGIN
    DELETE FROM role_menu_permissions WHERE role_id = p_role_id;

    INSERT INTO role_menu_permissions (role_id, menu_key)
    SELECT p_role_id, m.key
    FROM menu_configs m
    WHERE m.key = ANY(p_menu_keys);
END;
$$;
