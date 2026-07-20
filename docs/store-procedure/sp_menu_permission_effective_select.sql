-- ============================================================================
-- sp_menu_permission_effective_select
-- Resolve the menus a user can actually reach.
-- A per-user override wins; otherwise the union of the user's roles applies.
-- `source` tells which layer decided: 'user' or 'role'.
-- ============================================================================

-- Postgres không cho phép CREATE OR REPLACE khi cột trả về thay đổi.
DROP FUNCTION IF EXISTS sp_menu_permission_effective_select(INTEGER);

CREATE OR REPLACE FUNCTION sp_menu_permission_effective_select(
    p_user_id INTEGER
)
RETURNS TABLE (
    menu_key     VARCHAR(50),
    is_allowed   BOOLEAN,
    role_allowed BOOLEAN,
    source       VARCHAR(10)
)
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN QUERY
    SELECT
        m.key,
        COALESCE(u.is_allowed, r.granted, FALSE),
        COALESCE(r.granted, FALSE),
        (CASE WHEN u.menu_key IS NOT NULL THEN 'user' ELSE 'role' END)::VARCHAR(10)
    FROM menu_configs m
    LEFT JOIN user_menu_permissions u
        ON u.menu_key = m.key AND u.user_id = p_user_id
    LEFT JOIN LATERAL (
        SELECT TRUE AS granted
        FROM role_menu_permissions rp
        JOIN user_roles ur ON ur.role_id = rp.role_id
        WHERE ur.user_id = p_user_id AND rp.menu_key = m.key
        LIMIT 1
    ) r ON TRUE
    ORDER BY m.display_order;
END;
$$;
