-- ============================================================================
-- sp_project_insert
-- Insert a new project and return the created record.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_project_insert(
    p_code          VARCHAR(20),
    p_name          VARCHAR(200),
    p_client        VARCHAR(200) DEFAULT '',
    p_backlog_key   VARCHAR(20)  DEFAULT '',
    p_backlog_code  TEXT         DEFAULT '0',
    p_backlog_name  VARCHAR(100) DEFAULT ''
)
RETURNS TABLE (
    id            INTEGER,
    code          VARCHAR(20),
    name          VARCHAR(200),
    client        VARCHAR(200),
    backlog_key   VARCHAR(20),
    backlog_code  TEXT,
    backlog_name  VARCHAR(100),
    is_active     BOOLEAN,
    created_at    TEXT,
    updated_at    TEXT
)
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN QUERY
    INSERT INTO projects (code, name, client, project_backlog_key, project_backlog_code, project_backlog_name)
    VALUES (p_code, p_name, p_client, p_backlog_key, p_backlog_code::numeric, p_backlog_name)
    RETURNING
        projects.id,
        projects.code,
        projects.name,
        projects.client,
        projects.project_backlog_key   AS backlog_key,
        projects.project_backlog_code::text  AS backlog_code,
        projects.project_backlog_name  AS backlog_name,
        projects.is_active,
        projects.created_at::text,
        projects.updated_at::text;
END;
$$;
