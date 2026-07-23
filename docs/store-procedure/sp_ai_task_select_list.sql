-- ============================================================================
-- sp_ai_task_select_list
-- Search/list AI tasks by keyword (matches task_code, category, description).
-- NULL/empty keyword returns the most recent tasks.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_ai_task_select_list(
    p_keyword VARCHAR(200) DEFAULT NULL
)
RETURNS TABLE (
    id          INTEGER,
    task_code   VARCHAR(100),
    category    VARCHAR(30),
    description TEXT,
    created_by  VARCHAR(100),
    created_at  TEXT
)
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN QUERY
    SELECT
        t.id,
        t.task_code,
        t.category,
        t.description,
        t.created_by,
        t.created_at::text
    FROM ai_tasks t
    WHERE p_keyword IS NULL OR p_keyword = ''
       OR t.task_code ILIKE '%' || p_keyword || '%'
       OR t.category ILIKE '%' || p_keyword || '%'
       OR t.description ILIKE '%' || p_keyword || '%'
    ORDER BY t.created_at DESC
    LIMIT 200;
END;
$$;
