-- ============================================================================
-- sp_project_select_by_id
-- Select a single project by its ID.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_project_select_by_id(
    p_id INTEGER
)
RETURNS TABLE (
    id            INTEGER,
    code          VARCHAR(20),
    name          VARCHAR(200),
    client        VARCHAR(200),
    backlog_key   VARCHAR(20),
    backlog_url   TEXT,
    backlog_space VARCHAR(100),
    is_active     BOOLEAN,
    created_at    TEXT,
    updated_at    TEXT
)
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN QUERY
    SELECT
        p.id,
        p.code,
        p.name,
        p.client,
        p.backlog_key,
        p.backlog_url,
        p.backlog_space,
        p.is_active,
        p.created_at::text,
        p.updated_at::text
    FROM projects p
    WHERE p.id = p_id;
END;
$$;
