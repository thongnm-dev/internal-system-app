-- ============================================================================
-- sp_daily_report_task_insert
-- Insert a user-added daily report task. Returns the saved record.
-- When p_id is provided and > 0, updates the existing record (upsert by id).
-- ============================================================================

DROP FUNCTION IF EXISTS sp_daily_report_task_insert(
    varchar, varchar, varchar, varchar, text, text[],
    varchar, varchar, varchar, varchar
);

DROP FUNCTION IF EXISTS sp_daily_report_task_insert(
    integer, varchar, varchar, varchar, varchar, text, text[],
    varchar, varchar, varchar, varchar
);

DROP FUNCTION IF EXISTS sp_daily_report_task_insert(
    integer, varchar, varchar, varchar, text, integer,
    varchar, varchar, varchar, varchar
);

CREATE OR REPLACE FUNCTION sp_daily_report_task_insert(
    p_id            INTEGER,
    p_username      VARCHAR(100),
    p_project_id    VARCHAR(120),
    p_name          VARCHAR(300),
    p_description   TEXT,
    p_category_id   INTEGER,
    p_assignee      VARCHAR(100),
    p_estimate_hour VARCHAR(20),
    p_due_date      VARCHAR(20),
    p_issue_key     VARCHAR(50)
)
RETURNS TABLE (
    id            INTEGER,
    username      VARCHAR(100),
    project_id    VARCHAR(120),
    name          VARCHAR(300),
    description   TEXT,
    category_id   INTEGER,
    assignee      VARCHAR(100),
    estimate_hour VARCHAR(20),
    due_date      VARCHAR(20),
    issue_key     VARCHAR(50),
    is_completed  BOOLEAN,
    completed_at  TEXT,
    created_at    TEXT,
    is_user_added BOOLEAN
)
LANGUAGE plpgsql
AS $$
#variable_conflict use_column
BEGIN
    IF p_id IS NOT NULL AND p_id > 0 THEN
        RETURN QUERY
        UPDATE daily_report_tasks
        SET project_id    = p_project_id,
            name          = p_name,
            description   = p_description,
            category_id   = p_category_id,
            assignee      = p_assignee,
            estimate_hour = p_estimate_hour,
            due_date      = p_due_date,
            issue_key     = p_issue_key
        WHERE daily_report_tasks.id = p_id
          AND daily_report_tasks.username = p_username
        RETURNING
            daily_report_tasks.id,
            daily_report_tasks.username,
            daily_report_tasks.project_id,
            daily_report_tasks.name,
            daily_report_tasks.description,
            daily_report_tasks.category_id,
            daily_report_tasks.assignee,
            daily_report_tasks.estimate_hour,
            daily_report_tasks.due_date,
            daily_report_tasks.issue_key,
            daily_report_tasks.is_completed,
            daily_report_tasks.completed_at::text,
            daily_report_tasks.created_at::text,
            TRUE;
    ELSE
        RETURN QUERY
        INSERT INTO daily_report_tasks (
            username, project_id, name, description,
            category_id, assignee, estimate_hour, due_date, issue_key
        )
        VALUES (
            p_username, p_project_id, p_name, p_description,
            p_category_id, p_assignee, p_estimate_hour, p_due_date, p_issue_key
        )
        RETURNING
            daily_report_tasks.id,
            daily_report_tasks.username,
            daily_report_tasks.project_id,
            daily_report_tasks.name,
            daily_report_tasks.description,
            daily_report_tasks.category_id,
            daily_report_tasks.assignee,
            daily_report_tasks.estimate_hour,
            daily_report_tasks.due_date,
            daily_report_tasks.issue_key,
            daily_report_tasks.is_completed,
            daily_report_tasks.completed_at::text,
            daily_report_tasks.created_at::text,
            TRUE;
    END IF;
END;
$$;
