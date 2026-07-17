-- ============================================================================
-- sp_api_key_insert
-- Insert a single API key record.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_api_key_insert(
    p_id        VARCHAR(36),
    p_user_id   INTEGER,
    p_name      VARCHAR(200),
    p_key_label VARCHAR(200),
    p_api_key   TEXT
)
RETURNS VOID
LANGUAGE plpgsql
AS $$
BEGIN
    INSERT INTO api_keys (id, user_id, name, key_label, api_key)
    VALUES (p_id, p_user_id, p_name, p_key_label, p_api_key);
END;
$$;
