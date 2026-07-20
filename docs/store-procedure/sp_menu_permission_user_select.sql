-- ============================================================================
-- sp_menu_permission_user_select
-- List the per-user menu overrides (grant or revoke) of a user.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_menu_permission_user_select(
    p_user_id INTEGER
)
RETURNS TABLE (
    menu_key   VARCHAR(50),
    is_allowed BOOLEAN
)
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN QUERY
    SELECT p.menu_key, p.is_allowed
    FROM user_menu_permissions p
    JOIN menu_configs m ON m.key = p.menu_key
    WHERE p.user_id = p_user_id
    ORDER BY m.display_order;
END;
$$;
