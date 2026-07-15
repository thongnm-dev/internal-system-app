-- ============================================================================
-- sp_menu_config_select_list
-- List all menu configs ordered by display_order.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_menu_config_select_list()
RETURNS TABLE (
    key           VARCHAR(50),
    title         VARCHAR(100),
    path          VARCHAR(200),
    icon          VARCHAR(50),
    menu_group    VARCHAR(50),
    is_visible    BOOLEAN,
    display_order INTEGER
)
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN QUERY
    SELECT
        m.key,
        m.title,
        m.path,
        m.icon,
        m.menu_group,
        m.is_visible,
        m.display_order
    FROM menu_configs m
    ORDER BY m.display_order;
END;
$$;
