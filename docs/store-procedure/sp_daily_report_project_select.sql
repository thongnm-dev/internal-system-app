-- ============================================================================
-- sp_daily_report_project_select
-- Lấy danh sách project active kèm cờ is_member cho daily report.
-- Trả về tất cả project active, đánh dấu project nào user là thành viên.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_daily_report_project_select(
    p_username VARCHAR(100)
)
RETURNS TABLE (
    id        INTEGER,
    code      VARCHAR(20),
    name      VARCHAR(200),
    client    VARCHAR(200),
    is_member BOOLEAN
)
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN QUERY
    SELECT
        p.id,
        p.code,
        p.name,
        p.client,
        EXISTS (
            SELECT 1 FROM project_members pm
            WHERE pm.project_id = p.id AND pm.username = p_username
        ) AS is_member
    FROM projects p
    WHERE p.is_active = TRUE
    ORDER BY p.code;
END;
$$;
