-- ============================================================================
-- sp_project_insert
-- Insert a new project and return the created record.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_project_insert(
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
    INSERT INTO projects (code, name, client, backlog_key, backlog_url, backlog_space)
    VALUES (p_code, p_name, p_client, p_backlog_key, p_backlog_url, p_backlog_space)
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
