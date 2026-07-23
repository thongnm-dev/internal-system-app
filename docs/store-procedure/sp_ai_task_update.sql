-- ============================================================================
-- sp_ai_task_update
-- Update an AI task. Auto-sets completed_at when is_complete changes to TRUE.
-- Returns the updated row.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_ai_task_update(
    p_id          INTEGER,
    p_task_cd     VARCHAR(100),
    p_task_name   VARCHAR(300),
    p_category    VARCHAR(30),
    p_is_complete BOOLEAN,
    p_updated_by  VARCHAR(100)
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
    UPDATE ai_tasks t
    SET task_cd      = p_task_cd,
        task_name    = p_task_name,
        category     = p_category,
        is_complete  = p_is_complete,
        completed_at = CASE
                         WHEN p_is_complete AND NOT t.is_complete THEN NOW()
                         WHEN NOT p_is_complete THEN NULL
                         ELSE t.completed_at
                       END,
        updated_by   = p_updated_by
    WHERE t.id = p_id
    RETURNING
        t.id,
        t.task_cd,
        t.task_name,
        t.category,
        t.is_complete,
        COALESCE(t.completed_at::text, ''),
        t.created_at::text,
        t.created_by,
        t.updated_at::text,
        t.updated_by;
END;
$$;
