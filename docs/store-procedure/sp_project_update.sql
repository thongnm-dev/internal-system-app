-- ============================================================================
-- sp_project_update
-- Update an existing project and return the updated record.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_project_update(
    p_id            INTEGER,
    p_code          VARCHAR(20),
    p_name          VARCHAR(200),
    p_client        VARCHAR(200) DEFAULT '',
    p_backlog_key   VARCHAR(20)  DEFAULT '',
    p_backlog_url   TEXT         DEFAULT '',
    p_backlog_space VARCHAR(100) DEFAULT ''
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
    UPDATE projects
    SET code          = p_code,
        name          = p_name,
        client        = p_client,
        backlog_key   = p_backlog_key,
        backlog_url   = p_backlog_url,
        backlog_space = p_backlog_space
    WHERE projects.id = p_id
    RETURNING
        projects.id,
        projects.code,
        projects.name,
        projects.client,
        projects.backlog_key,
        projects.backlog_url,
        projects.backlog_space,
        projects.is_active,
        projects.created_at::text,
        projects.updated_at::text;
END;
$$;
