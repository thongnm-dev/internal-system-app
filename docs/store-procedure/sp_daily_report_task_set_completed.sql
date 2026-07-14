-- ============================================================================
-- sp_daily_report_task_set_completed
-- Mark a user-added daily report task as completed / not completed.
-- Đặt completed_at = NOW() khi hoàn thành, NULL khi bỏ hoàn thành.
-- Trả về bản ghi sau khi cập nhật.
-- ============================================================================

-- Return type có thể đổi (thêm is_user_added) -> DROP bản cũ trước.
DROP FUNCTION IF EXISTS sp_daily_report_task_set_completed(varchar, varchar, boolean);

CREATE OR REPLACE FUNCTION sp_daily_report_task_set_completed(
    p_username     VARCHAR(100),
    p_task_id      VARCHAR(120),
    p_is_completed BOOLEAN
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
BEGIN
    RETURN QUERY
    UPDATE daily_report_tasks
    SET is_completed = p_is_completed,
        completed_at = CASE WHEN p_is_completed THEN NOW() ELSE NULL END
    WHERE daily_report_tasks.username = p_username
      AND daily_report_tasks.task_id = p_task_id
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
