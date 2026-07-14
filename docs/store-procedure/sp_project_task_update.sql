-- ============================================================================
-- sp_project_task_update
-- Update an existing project task and return the updated record.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_project_task_update(
    p_id            VARCHAR(50),
    p_short_name    VARCHAR(200),
    p_description   TEXT         DEFAULT '',
    p_assignee      VARCHAR(100) DEFAULT '',
    p_estimate_hour VARCHAR(20)  DEFAULT '',
    p_due_date      DATE         DEFAULT NULL,
    p_issue_key     VARCHAR(30)  DEFAULT ''
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
    UPDATE project_tasks
    SET
        short_name    = p_short_name,
        description   = p_description,
        assignee      = p_assignee,
        estimate_hour = p_estimate_hour,
        due_date      = p_due_date,
        issue_key     = p_issue_key
    WHERE project_tasks.id = p_id
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
