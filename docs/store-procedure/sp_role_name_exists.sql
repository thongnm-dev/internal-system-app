-- ============================================================================
-- sp_role_name_exists
-- Check if a role name exists, optionally excluding a specific role ID.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_role_name_exists(
    p_name       VARCHAR(50),
    p_exclude_id INTEGER DEFAULT NULL
)
RETURNS BOOLEAN
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN EXISTS (
        SELECT 1
        FROM roles r
        WHERE LOWER(r.name) = LOWER(p_name)
          AND (p_exclude_id IS NULL OR r.id <> p_exclude_id)
    );
END;
$$;
