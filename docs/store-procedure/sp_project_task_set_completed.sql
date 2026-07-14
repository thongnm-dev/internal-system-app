-- ============================================================================
-- sp_project_task_set_completed
-- Đánh dấu một project_task là đã delivery / chưa delivery.
-- Dùng bởi màn Daily Report khi user bấm nút Delivery trên task được giao.
-- Trả về is_completed sau khi cập nhật; rỗng nếu không tìm thấy task.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_project_task_set_completed(
    p_id           VARCHAR(50),
    p_is_completed BOOLEAN
)
RETURNS TABLE (
    id           VARCHAR(50),
    is_completed BOOLEAN
)
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN QUERY
    UPDATE project_tasks
    SET is_completed = p_is_completed,
        completed_at = CASE WHEN p_is_completed THEN NOW() ELSE NULL END
    WHERE project_tasks.id = p_id
    RETURNING project_tasks.id, project_tasks.is_completed;
END;
$$;
