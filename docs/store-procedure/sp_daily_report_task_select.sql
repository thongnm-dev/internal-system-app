-- ============================================================================
-- sp_daily_report_task_select
-- Lấy task cho màn Daily Report của một user, gộp từ HAI nguồn:
--   (1) daily_report_tasks : task user tự thêm ngay trên Daily Report.
--   (2) project_tasks       : task thật của dự án được GIAO cho user (assignee = user),
--                             kèm category tags từ project_task_categories.
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
DECLARE
    v_month_start DATE := make_date(p_year, p_month, 1);
    v_next_month  DATE := (make_date(p_year, p_month, 1) + INTERVAL '1 month')::date;
BEGIN
    RETURN QUERY
    -- (1) Task user tự thêm trên Daily Report
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
        t.is_completed,
        t.completed_at::text,
        t.created_at::text,
        TRUE AS is_user_added
    FROM daily_report_tasks t
    WHERE t.username = p_username
      AND t.created_at < v_next_month
      AND (t.created_at >= v_month_start OR t.is_completed = FALSE)

    UNION ALL

    -- (2) Task dự án được giao cho user
    SELECT
        0::integer,
        p_username::varchar(100),
        pt.id::varchar(120),
        pt.project_id::varchar(120),
        COALESCE(
            (ARRAY_AGG(cat.process_code ORDER BY cat.process_code) FILTER (WHERE cat.process_code IS NOT NULL))[1],
            'TASK'
        )::varchar(50),
        pt.short_name::varchar(300),
        pt.description,
        COALESCE(
            ARRAY_AGG(cat.process_code ORDER BY cat.process_code) FILTER (WHERE cat.process_code IS NOT NULL),
            '{}'
        )::text[],
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
    LEFT JOIN categories cat ON cat.id = ptc.category_id
    WHERE pt.assignee = p_username
      AND pt.created_at < v_next_month
      AND (pt.created_at >= v_month_start OR pt.is_completed = FALSE)
    GROUP BY pt.id, pt.project_id, pt.short_name, pt.description, pt.assignee,
             pt.estimate_hour, pt.due_date, pt.issue_key, pt.is_completed,
             pt.completed_at, pt.created_at

    ORDER BY 4, 15;  -- project_id, created_at
END;
$$;
