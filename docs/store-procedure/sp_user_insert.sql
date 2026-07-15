-- ============================================================================
-- sp_user_insert
-- Insert a new user and return the created row.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_user_insert(
    p_username      VARCHAR(100),
    p_password_hash TEXT,
    p_full_name     VARCHAR(200),
    p_email         VARCHAR(255),
    p_phone         VARCHAR(50),
    p_address       TEXT,
    p_position      VARCHAR(100)
)
RETURNS TABLE (
    id            INTEGER,
    username      VARCHAR(100),
    full_name     VARCHAR(200),
    email         VARCHAR(255),
    phone         VARCHAR(50),
    address       TEXT,
    "position"    VARCHAR(100),
    is_active     BOOLEAN,
    created_at    TEXT,
    updated_at    TEXT
)
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN QUERY
    INSERT INTO users (username, password_hash, full_name, email, phone, address, position)
    VALUES (p_username, p_password_hash, p_full_name, p_email, p_phone, p_address, p_position)
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
