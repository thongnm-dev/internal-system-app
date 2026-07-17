-- ============================================================================
-- sp_api_key_delete_by_user
-- Delete all API keys belonging to a user.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_api_key_delete_by_user(
    p_user_id INTEGER
)
RETURNS VOID
LANGUAGE plpgsql
AS $$
BEGIN
    DELETE FROM api_keys WHERE user_id = p_user_id;
END;
$$;
