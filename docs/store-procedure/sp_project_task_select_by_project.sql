-- ============================================================================
-- sp_project_task_select_by_project
-- Select all tasks for a project, aggregating categories into an array.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_project_task_select_by_project(
    p_project_id INTEGER
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
    categories    TEXT[],
    created_at    TEXT
)
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN QUERY
    SELECT
        t.id,
        t.project_id,
        t.short_name,
        t.description,
        t.assignee,
        t.estimate_hour,
        COALESCE(t.due_date::text, ''),
        t.issue_key,
        t.is_user_added,
        COALESCE(
            ARRAY_AGG(c.category ORDER BY c.category)
                FILTER (WHERE c.category IS NOT NULL),
            '{}'
        )::text[],
        t.created_at::text
    FROM project_tasks t
    LEFT JOIN project_task_categories c ON c.task_id = t.id
    WHERE t.project_id = p_project_id
    GROUP BY t.id, t.project_id, t.short_name, t.description, t.assignee,
             t.estimate_hour, t.due_date, t.issue_key, t.is_user_added, t.created_at
    ORDER BY t.created_at;
END;
$$;
