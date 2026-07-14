-- ============================================================================
-- sp_daily_report_task_insert
-- Insert (or upsert by task_id) a user-added daily report task.
-- Returns the saved record.
-- ============================================================================

-- Return type thay đổi (thêm is_completed, completed_at) -> phải DROP trước.
DROP FUNCTION IF EXISTS sp_daily_report_task_insert(
    varchar, varchar, varchar, varchar, varchar, text, text[],
    varchar, varchar, varchar, varchar
);

CREATE OR REPLACE FUNCTION sp_daily_report_task_insert(
    p_username      VARCHAR(100),
    p_task_id       VARCHAR(120),
    p_project_id    VARCHAR(120),
    p_code          VARCHAR(50),
    p_name          VARCHAR(300),
    p_description   TEXT,
    p_categories    TEXT[],
    p_assignee      VARCHAR(100),
    p_estimate_hour VARCHAR(20),
    p_due_date      VARCHAR(20),
    p_issue_key     VARCHAR(50)
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
    is_completed  BOOLEAN,
    completed_at  TEXT,
    created_at    TEXT,
    is_user_added BOOLEAN
)
LANGUAGE plpgsql
AS $$
-- RETURNS TABLE khai báo các cột out trùng tên cột bảng (username, task_id...),
-- gây "ambiguous column" ở ON CONFLICT. Ưu tiên hiểu định danh là CỘT bảng.
#variable_conflict use_column
BEGIN
    RETURN QUERY
    INSERT INTO daily_report_tasks (
        username, task_id, project_id, code, name, description,
        categories, assignee, estimate_hour, due_date, issue_key
    )
    VALUES (
        p_username, p_task_id, p_project_id, p_code, p_name, p_description,
        p_categories, p_assignee, p_estimate_hour, p_due_date, p_issue_key
    )
    ON CONFLICT (username, task_id) DO UPDATE
        SET project_id    = EXCLUDED.project_id,
            code          = EXCLUDED.code,
            name          = EXCLUDED.name,
            description   = EXCLUDED.description,
            categories    = EXCLUDED.categories,
            assignee      = EXCLUDED.assignee,
            estimate_hour = EXCLUDED.estimate_hour,
            due_date      = EXCLUDED.due_date,
            issue_key     = EXCLUDED.issue_key
    RETURNING
        daily_report_tasks.id,
        daily_report_tasks.username,
        daily_report_tasks.task_id,
        daily_report_tasks.project_id,
        daily_report_tasks.code,
        daily_report_tasks.name,
        daily_report_tasks.description,
        daily_report_tasks.categories,
        daily_report_tasks.assignee,
        daily_report_tasks.estimate_hour,
        daily_report_tasks.due_date,
        daily_report_tasks.issue_key,
        daily_report_tasks.is_completed,
        daily_report_tasks.completed_at::text,
        daily_report_tasks.created_at::text,
        TRUE;
END;
$$;
