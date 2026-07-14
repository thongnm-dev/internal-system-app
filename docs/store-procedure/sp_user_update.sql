-- ============================================================================
-- sp_user_update
-- Update user info (not password) and return updated row.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_user_update(
    p_user_id   INTEGER,
    p_full_name VARCHAR(200),
    p_email     VARCHAR(255),
    p_phone     VARCHAR(50),
    p_address   TEXT,
    p_position  VARCHAR(100),
    p_is_active BOOLEAN
)
RETURNS TABLE (
    id            INTEGER,
    username      VARCHAR(100),
    full_name     VARCHAR(200),
    email         VARCHAR(255),
    phone         VARCHAR(50),
    address       TEXT,
    position      VARCHAR(100),
    is_active     BOOLEAN,
    created_at    TEXT,
    updated_at    TEXT
)
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN QUERY
    UPDATE users
    SET full_name  = p_full_name,
        email      = p_email,
        phone      = p_phone,
        address    = p_address,
        position   = p_position,
        is_active  = p_is_active,
        updated_at = NOW()
    WHERE users.id = p_user_id
    RETURNING
        users.id,
        users.username,
        users.full_name,
        users.email,
        users.phone,
        users.address,
        users.position,
        users.is_active,
        to_char(users.created_at, 'YYYY-MM-DD HH24:MI:SS'),
        to_char(users.updated_at, 'YYYY-MM-DD HH24:MI:SS');
END;
$$;
