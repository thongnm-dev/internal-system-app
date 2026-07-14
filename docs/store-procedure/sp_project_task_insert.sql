-- ============================================================================
-- sp_project_task_insert
-- Insert a new project task and return the created record.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_project_task_insert(
    p_id            VARCHAR(50),
    p_project_id    INTEGER,
    p_short_name    VARCHAR(200),
    p_description   TEXT         DEFAULT '',
    p_assignee      VARCHAR(100) DEFAULT '',
    p_estimate_hour VARCHAR(20)  DEFAULT '',
    p_due_date      DATE         DEFAULT NULL,
    p_issue_key     VARCHAR(30)  DEFAULT '',
    p_is_user_added BOOLEAN      DEFAULT TRUE
)
RETURNS TABLE (
    id            VARCHAR(50),
    project_id    INTEGER,
    short_name    VARCHAR(200),
    description   TEXT,
    assignee      VARCHAR(100),
    estimate_hour VARCHAR(20),
    due_date      TEXT,
    issue_key     VARCHAR(30),
    is_user_added BOOLEAN,
    created_at    TEXT
)
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN QUERY
    INSERT INTO project_tasks (
        id, project_id, short_name, description,
        assignee, estimate_hour, due_date, issue_key, is_user_added
    )
    VALUES (
        p_id, p_project_id, p_short_name, p_description,
        p_assignee, p_estimate_hour, p_due_date, p_issue_key, p_is_user_added
    )
    RETURNING
        project_tasks.id,
        project_tasks.project_id,
        project_tasks.short_name,
        project_tasks.description,
        project_tasks.assignee,
        project_tasks.estimate_hour,
        COALESCE(project_tasks.due_date::text, ''),
        project_tasks.issue_key,
        project_tasks.is_user_added,
        project_tasks.created_at::text;
END;
$$;
