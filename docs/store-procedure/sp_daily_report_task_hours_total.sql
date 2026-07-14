-- ============================================================================
-- sp_daily_report_task_hours_total
-- Tổng số giờ (cột hour) đã nhập của MỌI tháng, gộp theo từng task, cho một user.
-- Dùng để hiển thị tiến độ tích luỹ (VD: 75h / 100h estimate) trên màn Daily Report.
-- Chỉ trả về task có ít nhất một bản ghi giờ công.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_daily_report_task_hours_total(
    p_username VARCHAR(100)
)
RETURNS TABLE (
    task_id    VARCHAR(120),
    total_hour DOUBLE PRECISION
)
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN QUERY
    SELECT
        e.task_id,
        COALESCE(SUM(e.hour), 0)::double precision AS total_hour
    FROM daily_report_entries e
    WHERE e.username = p_username
    GROUP BY e.task_id;
END;
$$;
