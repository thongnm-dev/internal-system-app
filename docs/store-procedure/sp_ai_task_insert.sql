-- ============================================================================
-- sp_ai_task_insert
-- Insert a new AI task. Returns the created row.
-- ============================================================================

CREATE OR REPLACE FUNCTION sp_ai_task_insert(
    p_task_cd    VARCHAR(100),
    p_task_name  VARCHAR(300),
    p_category   VARCHAR(30),
    p_created_by VARCHAR(100)
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
    INSERT INTO ai_tasks (task_cd, task_name, category, created_by, updated_by)
    VALUES (p_task_cd, p_task_name, p_category, p_created_by, p_created_by)
    RETURNING
        ai_tasks.id,
        ai_tasks.task_cd,
        ai_tasks.task_name,
        ai_tasks.category,
        ai_tasks.is_complete,
        COALESCE(ai_tasks.completed_at::text, ''),
        ai_tasks.created_at::text,
        ai_tasks.created_by,
        ai_tasks.updated_at::text,
        ai_tasks.updated_by;
END;
$$;
