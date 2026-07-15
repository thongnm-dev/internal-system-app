-- ============================================================================
-- sp_menu_config_upsert
-- Insert or update a single menu config item.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_menu_config_upsert(
    p_key           VARCHAR(50),
    p_title         VARCHAR(100),
    p_path          VARCHAR(200),
    p_icon          VARCHAR(50),
    p_menu_group    VARCHAR(50),
    p_is_visible    BOOLEAN,
    p_display_order INTEGER
)
RETURNS VOID
LANGUAGE plpgsql
AS $$
BEGIN
    INSERT INTO menu_configs (key, title, path, icon, menu_group, is_visible, display_order)
    VALUES (p_key, p_title, p_path, p_icon, p_menu_group, p_is_visible, p_display_order)
    ON CONFLICT (key) DO UPDATE SET
        title         = EXCLUDED.title,
        path          = EXCLUDED.path,
        icon          = EXCLUDED.icon,
        menu_group    = EXCLUDED.menu_group,
        is_visible    = EXCLUDED.is_visible,
        display_order = EXCLUDED.display_order;
END;
$$;
