-- ============================================================================
-- sp_daily_report_task_select
-- Select all user-added daily report tasks for a user.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_daily_report_task_select(
    p_username VARCHAR(100)
)
RETURNS TABLE (
    id            INTEGER,
    username      VARCHAR(100),
    task_id       VARCHAR(120),
    project_id    VARCHAR(120),
    code          VARCHAR(50),
    name          VARCHAR(300),
    description   TEXT,
    categories    TEXT[],
    assignee      VARCHAR(100),
    estimate_hour VARCHAR(20),
    due_date      VARCHAR(20),
    issue_key     VARCHAR(50),
    created_at    TEXT
)
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN QUERY
    SELECT
        t.id,
        t.username,
        t.task_id,
        t.project_id,
        t.code,
        t.name,
        t.description,
        t.categories,
        t.assignee,
        t.estimate_hour,
        t.due_date,
        t.issue_key,
        t.created_at::text
    FROM daily_report_tasks t
    WHERE t.username = p_username
    ORDER BY t.project_id, t.created_at;
END;
$$;
