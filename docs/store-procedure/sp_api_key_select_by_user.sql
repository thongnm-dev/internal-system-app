-- ============================================================================
-- sp_api_key_select_by_user
-- List all API keys belonging to a user, ordered by created_at.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_api_key_select_by_user(
    p_user_id INTEGER
)
RETURNS TABLE (
    id        VARCHAR(36),
    name      VARCHAR(200),
    key_label VARCHAR(200),
    api_key   TEXT
)
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN QUERY
    SELECT
        a.id,
        a.name,
        a.key_label,
        a.api_key
    FROM api_keys a
    WHERE a.user_id = p_user_id
    ORDER BY a.created_at;
END;
$$;
