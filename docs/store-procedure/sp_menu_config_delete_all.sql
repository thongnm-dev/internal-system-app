-- ============================================================================
-- sp_menu_config_delete_all
-- Delete all menu configs (used before bulk re-insert on reset).
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_menu_config_delete_all()
RETURNS VOID
LANGUAGE plpgsql
AS $$
BEGIN
    DELETE FROM menu_configs;
END;
$$;
