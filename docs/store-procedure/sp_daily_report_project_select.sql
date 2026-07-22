-- ============================================================================
-- sp_daily_report_project_select
-- Lấy danh sách project active mà user là thành viên cho daily report.
-- Chỉ trả về project user được assign vào (project_members).
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
        TRUE AS is_member
    FROM projects p
    INNER JOIN project_members pm ON pm.project_id = p.id AND pm.username = p_username
    WHERE p.is_active = TRUE
    ORDER BY p.code;
END;
$$;
