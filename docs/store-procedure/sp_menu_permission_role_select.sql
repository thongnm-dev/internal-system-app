-- ============================================================================
-- sp_menu_permission_role_select
-- List the menu keys granted to a role.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_menu_permission_role_select(
    p_role_id INTEGER
)
RETURNS TABLE (
    menu_key VARCHAR(50)
)
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN QUERY
    SELECT p.menu_key
    FROM role_menu_permissions p
    JOIN menu_configs m ON m.key = p.menu_key
    WHERE p.role_id = p_role_id
    ORDER BY m.display_order;
END;
$$;
