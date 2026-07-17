-- ============================================================================
-- sp_api_key_get_value
-- Get a single API key value by name and key_label.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_api_key_get_value(
    p_name      VARCHAR(200),
    p_key_label VARCHAR(200)
)
RETURNS TABLE (
    api_key TEXT
)
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN QUERY
    SELECT a.api_key
    FROM api_keys a
    WHERE a.name = p_name
      AND a.key_label = p_key_label
    LIMIT 1;
END;
$$;
