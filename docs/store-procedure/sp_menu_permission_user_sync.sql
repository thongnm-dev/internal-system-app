-- ============================================================================
-- sp_menu_permission_user_sync
-- Replace the per-user menu overrides. Keys in p_allow_keys are granted,
-- keys in p_deny_keys are revoked, every other menu falls back to the roles.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_menu_permission_user_sync(
    p_user_id    INTEGER,
    p_allow_keys TEXT[],
    p_deny_keys  TEXT[]
)
RETURNS VOID
LANGUAGE plpgsql
AS $$
BEGIN
    DELETE FROM user_menu_permissions WHERE user_id = p_user_id;

    INSERT INTO user_menu_permissions (user_id, menu_key, is_allowed)
    SELECT p_user_id, m.key, TRUE
    FROM menu_configs m
    WHERE m.key = ANY(p_allow_keys);

    INSERT INTO user_menu_permissions (user_id, menu_key, is_allowed)
    SELECT p_user_id, m.key, FALSE
    FROM menu_configs m
    WHERE m.key = ANY(p_deny_keys)
      AND NOT (m.key = ANY(p_allow_keys));
END;
$$;
