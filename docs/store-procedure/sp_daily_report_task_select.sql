-- ============================================================================
-- sp_daily_report_task_select
-- Lấy task cho màn Daily Report của một user, gộp từ HAI nguồn:
--   (1) daily_report_tasks : task user tự thêm ngay trên Daily Report.
--       Dùng id (SERIAL) làm task_id.
--   (2) project_tasks       : task thật của dự án được GIAO cho user (assignee = user),
--                             kèm category từ project_task_categories.
--                             Mỗi (task, category) trả về một dòng riêng.
--
-- Cùng áp quy tắc theo tháng đang xem (p_year, p_month):
--   * Task tạo trong tháng đang xem  -> hiện tất cả (kể cả đã delivery) để xem giờ.
--   * Task tạo ở tháng trước           -> chỉ hiện nếu CHƯA delivery (carry-over).
--   * Task tạo sau tháng đang xem       -> không hiện.
--
-- Cột is_user_added phân biệt nguồn: TRUE = daily_report_tasks, FALSE = project_tasks.
-- ============================================================================
DROP FUNCTION IF EXISTS sp_daily_report_task_select(varchar, integer, integer);

CREATE OR REPLACE FUNCTION sp_daily_report_task_select(
    p_username VARCHAR(100),
    p_year     INTEGER,
    p_month    INTEGER
)
RETURNS TABLE (
    username      VARCHAR(100),
    task_id       VARCHAR(120),
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
DECLARE
    v_month_start DATE := make_date(p_year, p_month, 1);
    v_next_month  DATE := (make_date(p_year, p_month, 1) + INTERVAL '1 month')::date;
BEGIN
    RETURN QUERY
    -- (1) Task user tự thêm trên Daily Report
    SELECT
        t.username,
        t.id::varchar(120) AS task_id,
        t.project_id,
        t.name,
        t.description,
        t.category_id,
        t.assignee,
        t.estimate_hour,
        t.due_date,
        t.issue_key,
        t.is_completed,
        t.completed_at::text,
        t.created_at::text,
        TRUE AS is_user_added
    FROM daily_report_tasks t
    WHERE t.username = p_username
      AND t.created_at < v_next_month
      AND (t.created_at >= v_month_start OR t.is_completed = FALSE)

    UNION ALL

    -- (2) Task dự án được giao cho user — one row per (task, category)
    SELECT
        p_username::varchar(100),
        pt.id::varchar(120),
        pt.project_id::varchar(120),
        pt.short_name::varchar(300),
        pt.description,
        COALESCE(ptc.category_id, 0),
        pt.assignee,
        pt.estimate_hour,
        COALESCE(pt.due_date::text, '')::varchar(20),
        pt.issue_key::varchar(50),
        pt.is_completed,
        pt.completed_at::text,
        pt.created_at::text,
        FALSE AS is_user_added
    FROM project_tasks pt
    LEFT JOIN project_task_categories ptc ON ptc.task_id = pt.id
    WHERE pt.assignee = p_username
      AND pt.created_at < v_next_month
      AND (pt.created_at >= v_month_start OR pt.is_completed = FALSE)

    ORDER BY project_id, created_at;
END;
$$;
