-- ============================================================================
-- sp_ai_task_select_list
-- Search/list AI tasks by keyword (matches task_cd, task_name, category).
-- NULL/empty keyword returns the most recent tasks.
-- p_is_complete: NULL = all, TRUE = completed only, FALSE = incomplete only.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_ai_task_select_list(
    p_keyword     VARCHAR(200) DEFAULT NULL,
    p_is_complete BOOLEAN      DEFAULT NULL
)
RETURNS TABLE (
    id           INTEGER,
    task_cd      VARCHAR(100),
    task_name    VARCHAR(300),
    category     VARCHAR(30),
    is_complete  BOOLEAN,
    completed_at TEXT,
    created_at   TEXT,
    created_by   VARCHAR(100),
    updated_at   TEXT,
    updated_by   VARCHAR(100)
)
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN QUERY
    SELECT
        t.id,
        t.task_cd,
        t.task_name,
        t.category,
        t.is_complete,
        COALESCE(t.completed_at::text, ''),
        t.created_at::text,
        t.created_by,
        t.updated_at::text,
        t.updated_by
    FROM ai_tasks t
    WHERE (p_keyword IS NULL OR p_keyword = ''
       OR t.task_cd ILIKE '%' || p_keyword || '%'
       OR t.task_name ILIKE '%' || p_keyword || '%'
       OR t.category ILIKE '%' || p_keyword || '%')
      AND (p_is_complete IS NULL OR t.is_complete = p_is_complete)
    ORDER BY t.created_at DESC
    LIMIT 200;
END;
$$;
