-- ============================================================================
-- sp_project_select_list
-- List all projects with member count summary.
-- ============================================================================

-- Signature (RETURNS TABLE) đã thay đổi nên phải DROP trước; CREATE OR REPLACE
-- không cho phép đổi kiểu trả về của function đang tồn tại.
DROP FUNCTION IF EXISTS sp_project_select_list();

CREATE OR REPLACE FUNCTION sp_project_select_list()
RETURNS TABLE (
    id           INTEGER,
    code         VARCHAR(20),
    name         VARCHAR(200),
    client       VARCHAR(200),
    is_active    BOOLEAN,
    created_at   TEXT,
    member_count BIGINT,
    backlog_key  VARCHAR(20)
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
        p.is_active,
        p.created_at::text,
        COUNT(pm.id)::bigint AS member_count,
        p.project_backlog_key AS backlog_key
    FROM projects p
    LEFT JOIN project_members pm ON pm.project_id = p.id
    GROUP BY p.id
    ORDER BY p.id;
END;
$$;
